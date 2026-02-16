use std::sync::Arc;

use super::{
  CommandQueue,
  PlaybackCommand,
  PlaybackSharedState,
  PlaybackState,
  DEFAULT_GAIN_RAMP_FRAMES,
  MAX_PLAYBACK_CHANNELS,
};
use crate::audio::{
  AudioOutputWriter,
  SoundBuffer,
};

/// A linear gain ramp used to de-click transport transitions.
#[derive(Clone, Copy, Debug, PartialEq)]
struct GainRamp {
  current: f32,
  target: f32,
  step: f32,
  frames_remaining: usize,
}

impl GainRamp {
  /// Create a silent ramp with a target gain of `0.0`.
  ///
  /// # Returns
  /// A `GainRamp` initialized to silence.
  fn silent() -> Self {
    return Self {
      current: 0.0,
      target: 0.0,
      step: 0.0,
      frames_remaining: 0,
    };
  }

  /// Begin ramping the gain toward a target.
  ///
  /// # Arguments
  /// - `target`: Target gain in nominal range `[0.0, 1.0]`.
  /// - `frames`: Ramp duration in frames. When `0`, the gain changes
  ///   immediately.
  ///
  /// # Returns
  /// `()` after updating the ramp parameters.
  fn start(&mut self, target: f32, frames: usize) {
    let target = target.clamp(0.0, 1.0);

    if frames == 0 || (self.current - target).abs() <= f32::EPSILON {
      self.current = target;
      self.target = target;
      self.step = 0.0;
      self.frames_remaining = 0;
      return;
    }

    self.target = target;
    self.frames_remaining = frames;
    self.step = (target - self.current) / frames as f32;
    return;
  }

  /// Return whether the ramp is fully silent and stable.
  ///
  /// # Returns
  /// `true` if the current and target gain are both `0.0` with no remaining
  /// ramp frames.
  fn is_silent(&self) -> bool {
    return self.frames_remaining == 0
      && self.current.abs() <= f32::EPSILON
      && self.target.abs() <= f32::EPSILON;
  }

  /// Advance the ramp by one output frame.
  ///
  /// # Returns
  /// `()` after advancing the ramp state.
  fn advance_frame(&mut self) {
    if self.frames_remaining == 0 {
      return;
    }

    self.current += self.step;
    self.frames_remaining = self.frames_remaining.saturating_sub(1);

    if self.frames_remaining == 0 {
      self.current = self.target;
      self.step = 0.0;
    }

    return;
  }
}

/// Deterministic single-slot playback scheduler.
///
/// This scheduler is designed to run inside a real-time audio callback and
/// MUST NOT allocate or block while rendering audio.
struct PlaybackScheduler {
  state: PlaybackState,
  looping: bool,
  cursor_frames: f32,
  channels: usize,
  ramp_frames: usize,
  gain: GainRamp,
  buffer: Option<Arc<SoundBuffer>>,
  last_frame_samples: [f32; MAX_PLAYBACK_CHANNELS],
}

impl PlaybackScheduler {
  /// Create a scheduler configured for a fixed output channel count.
  ///
  /// # Arguments
  /// - `channels`: Interleaved output channel count.
  ///
  /// # Returns
  /// A scheduler initialized to `Stopped` with no buffer.
  #[allow(dead_code)]
  fn new(channels: usize) -> Self {
    return Self::new_with_ramp_frames(channels, DEFAULT_GAIN_RAMP_FRAMES);
  }

  /// Create a scheduler configured for a fixed output channel count and ramp.
  ///
  /// # Arguments
  /// - `channels`: Interleaved output channel count.
  /// - `ramp_frames`: Gain ramp length for transport de-clicking in frames.
  ///
  /// # Returns
  /// A scheduler initialized to `Stopped` with no buffer.
  fn new_with_ramp_frames(channels: usize, ramp_frames: usize) -> Self {
    return Self {
      state: PlaybackState::Stopped,
      looping: false,
      cursor_frames: 0.0,
      channels,
      ramp_frames,
      gain: GainRamp::silent(),
      buffer: None,
      last_frame_samples: [0.0; MAX_PLAYBACK_CHANNELS],
    };
  }

  /// Replace the active buffer and reset playback to the start.
  ///
  /// # Arguments
  /// - `buffer`: The decoded buffer to schedule.
  ///
  /// # Returns
  /// `()` after updating the active buffer.
  fn set_buffer(&mut self, buffer: Arc<SoundBuffer>) {
    self.buffer = Some(buffer);
    self.cursor_frames = 0.0;
    return;
  }

  /// Enable or disable looping.
  ///
  /// # Arguments
  /// - `looping`: Whether playback should loop on completion.
  ///
  /// # Returns
  /// `()` after updating the looping flag.
  fn set_looping(&mut self, looping: bool) {
    self.looping = looping;
    return;
  }

  /// Transition the scheduler to playing.
  ///
  /// # Returns
  /// `()` after updating the transport state.
  fn play(&mut self) {
    if self.state == PlaybackState::Playing {
      return;
    }

    self.state = PlaybackState::Playing;
    self.gain.start(1.0, self.ramp_frames);
    return;
  }

  /// Transition the scheduler to paused without resetting position.
  ///
  /// # Returns
  /// `()` after updating the transport state.
  fn pause(&mut self) {
    if self.state != PlaybackState::Playing {
      return;
    }

    self.state = PlaybackState::Paused;
    self.gain.start(0.0, self.ramp_frames);
    return;
  }

  /// Stop playback and reset position to the start.
  ///
  /// # Returns
  /// `()` after updating the transport state.
  fn stop(&mut self) {
    self.state = PlaybackState::Stopped;
    self.cursor_frames = 0.0;
    self.gain.start(0.0, self.ramp_frames);
    return;
  }

  /// Return the current transport state.
  ///
  /// # Returns
  /// The current `PlaybackState`.
  fn state(&self) -> PlaybackState {
    return self.state;
  }

  /// Return the current interleaved cursor position in samples.
  ///
  /// # Returns
  /// The cursor position as an interleaved sample index.
  #[allow(dead_code)]
  fn cursor_samples(&self) -> usize {
    let frames = self.cursor_frames.floor().max(0.0) as usize;
    return frames.saturating_mul(self.channels);
  }

  /// Render audio for a callback tick into an output writer.
  ///
  /// # Arguments
  /// - `writer`: Real-time writer for the current callback output buffer.
  /// - `master_volume`: Global/master volume scalar applied to all output.
  /// - `instance_volume`: Per-instance volume scalar for the active slot.
  /// - `instance_pitch`: Per-instance pitch/playback speed scalar.
  ///
  /// # Returns
  /// `()` after writing the output buffer.
  fn render(
    &mut self,
    writer: &mut dyn AudioOutputWriter,
    master_volume: f32,
    instance_volume: f32,
    instance_pitch: f32,
  ) {
    let writer_channels = writer.channels() as usize;
    let frames = writer.frames();
    let soft_clip_enabled = master_volume * instance_volume > 1.0;

    if writer_channels == 0 || frames == 0 {
      return;
    }

    if writer_channels > MAX_PLAYBACK_CHANNELS {
      writer.clear();
      return;
    }

    if writer_channels != self.channels {
      writer.clear();
      return;
    }

    for frame_index in 0..frames {
      let frame_gain = self.gain.current;
      let output_gain = frame_gain * master_volume * instance_volume;

      if self.state != PlaybackState::Playing && self.gain.is_silent() {
        writer.clear();
        return;
      }

      if self.state == PlaybackState::Playing {
        let Some(buffer) = self.buffer.as_ref() else {
          for channel_index in 0..writer_channels {
            writer.set_sample(frame_index, channel_index, 0.0);
          }
          self.gain.advance_frame();
          continue;
        };

        let samples = buffer.samples();
        let total_frames = buffer.frames();

        let mut should_stop = false;

        if total_frames == 0 {
          should_stop = true;
        } else {
          let total_frames_f32 = total_frames as f32;

          if self.cursor_frames >= total_frames_f32 {
            if self.looping {
              self.cursor_frames =
                self.cursor_frames.rem_euclid(total_frames_f32);
            } else {
              should_stop = true;
            }
          }

          if !should_stop {
            // When playback continues, we resample the buffer at a fractional
            // frame cursor:
            //
            // - `cursor_frame_position` is the current playback position in
            //   frames (not interleaved samples).
            // - `frame_index0` is the base frame index: floor(cursor).
            // - `frame_lerp_t` is the fractional part in `[0.0, 1.0)`.
            // - `frame_index1` is the next frame used for interpolation. If
            //   the cursor is on the last frame:
            //   - when looping, wrap to frame `0`
            //   - otherwise, reuse `frame_index0` (hold the last sample)
            //
            // Each output frame is produced by linear interpolation between
            // the two source frames, then applying gain and clipping.
            let cursor_frame_position = self.cursor_frames.max(0.0);
            let frame_index0 = cursor_frame_position.floor() as usize;
            let frame_lerp_t = cursor_frame_position - frame_index0 as f32;
            let frame_index1 = if frame_index0.saturating_add(1) < total_frames
            {
              frame_index0 + 1
            } else if self.looping {
              0
            } else {
              frame_index0
            };

            for channel_index in 0..writer_channels {
              let s0_index = frame_index0
                .saturating_mul(writer_channels)
                .saturating_add(channel_index);
              let s1_index = frame_index1
                .saturating_mul(writer_channels)
                .saturating_add(channel_index);
              let s0 = samples.get(s0_index).copied().unwrap_or(0.0);
              let s1 = samples.get(s1_index).copied().unwrap_or(s0);
              let sample = s0 + (s1 - s0) * frame_lerp_t;

              self.last_frame_samples[channel_index] = sample;
              writer.set_sample(
                frame_index,
                channel_index,
                clip_sample(sample * output_gain, soft_clip_enabled),
              );
            }

            self.cursor_frames = cursor_frame_position + instance_pitch;
            self.gain.advance_frame();
            continue;
          }
        }

        if should_stop {
          self.state = PlaybackState::Stopped;
          self.cursor_frames = 0.0;
          self.gain.start(0.0, self.ramp_frames);
        }
      }

      for channel_index in 0..writer_channels {
        let sample = self.last_frame_samples[channel_index];
        writer.set_sample(
          frame_index,
          channel_index,
          clip_sample(sample * output_gain, soft_clip_enabled),
        );
      }

      self.gain.advance_frame();
    }

    return;
  }
}

/// Clip a sample into the nominal output range.
///
/// This function enforces “clipping awareness” for amplified output:
/// - values are bounded to `[-1.0, 1.0]`
/// - non-finite values map to `0.0` to avoid propagating NaNs/Infs downstream
/// - when soft clipping is enabled, values in the knee region are gently
///   compressed before saturating
///
/// # Arguments
/// - `sample`: Candidate sample value.
/// - `soft_clip`: Whether to use a soft knee before saturation.
///
/// # Returns
/// A finite, bounded sample suitable for `AudioOutputWriter`.
fn clip_sample(sample: f32, soft_clip: bool) -> f32 {
  if !sample.is_finite() {
    return 0.0;
  }

  if !soft_clip {
    return sample.clamp(-1.0, 1.0);
  }

  // Soft knee limiter:
  // - linear in [-KNEE_START, KNEE_START]
  // - smoothstep curve from KNEE_START..1.0
  // - saturates to [-1.0, 1.0] beyond unity
  const KNEE_START: f32 = 0.95;

  let sign = sample.signum();
  let x = sample.abs();

  if x <= KNEE_START {
    return sample;
  }

  if x >= 1.0 {
    return sign;
  }

  let t = (x - KNEE_START) / (1.0 - KNEE_START);
  let smoothstep = t * t * (3.0 - 2.0 * t);
  let y = KNEE_START + (1.0 - KNEE_START) * smoothstep;
  return sign * y;
}

/// A callback-safe controller that drains transport commands and renders audio.
///
/// This type is intended to be owned by the platform audio callback closure.
pub(super) struct PlaybackController<const COMMAND_CAPACITY: usize> {
  command_queue: Arc<CommandQueue<PlaybackCommand, COMMAND_CAPACITY>>,
  shared_state: Arc<PlaybackSharedState>,
  scheduler: PlaybackScheduler,
}

impl<const COMMAND_CAPACITY: usize> PlaybackController<COMMAND_CAPACITY> {
  /// Create a controller configured for a fixed output channel count.
  ///
  /// # Arguments
  /// - `channels`: Interleaved output channel count.
  /// - `command_queue`: Shared producer/consumer command queue.
  ///
  /// # Returns
  /// A controller initialized to `Stopped` with no active buffer.
  #[allow(dead_code)]
  pub(super) fn new(
    channels: usize,
    command_queue: Arc<CommandQueue<PlaybackCommand, COMMAND_CAPACITY>>,
    shared_state: Arc<PlaybackSharedState>,
  ) -> Self {
    return Self::new_with_ramp_frames(
      channels,
      DEFAULT_GAIN_RAMP_FRAMES,
      command_queue,
      shared_state,
    );
  }

  /// Create a controller with an explicit gain ramp length.
  ///
  /// # Arguments
  /// - `channels`: Interleaved output channel count.
  /// - `ramp_frames`: Gain ramp length in frames.
  /// - `command_queue`: Shared producer/consumer command queue.
  ///
  /// # Returns
  /// A controller initialized to `Stopped` with no active buffer.
  pub(super) fn new_with_ramp_frames(
    channels: usize,
    ramp_frames: usize,
    command_queue: Arc<CommandQueue<PlaybackCommand, COMMAND_CAPACITY>>,
    shared_state: Arc<PlaybackSharedState>,
  ) -> Self {
    return Self {
      command_queue,
      shared_state,
      scheduler: PlaybackScheduler::new_with_ramp_frames(channels, ramp_frames),
    };
  }

  /// Drain any pending transport commands.
  ///
  /// # Returns
  /// `()` after applying all pending commands.
  fn drain_commands(&mut self) {
    while let Some(command) = self.command_queue.pop() {
      match command {
        PlaybackCommand::StopCurrent => {
          self.scheduler.stop();
          self.shared_state.set_state(PlaybackState::Stopped);
        }
        PlaybackCommand::SetBuffer {
          instance_id,
          buffer,
        } => {
          if instance_id != self.shared_state.active_instance_id() {
            continue;
          }
          self.scheduler.stop();
          self.scheduler.set_looping(false);
          self.scheduler.set_buffer(buffer);
          self.shared_state.set_state(PlaybackState::Stopped);
        }
        PlaybackCommand::SetLooping {
          instance_id,
          looping,
        } => {
          if instance_id != self.shared_state.active_instance_id() {
            continue;
          }
          self.scheduler.set_looping(looping);
        }
        PlaybackCommand::Play { instance_id } => {
          if instance_id != self.shared_state.active_instance_id() {
            continue;
          }
          self.scheduler.play();
          self.shared_state.set_state(PlaybackState::Playing);
        }
        PlaybackCommand::Pause { instance_id } => {
          if instance_id != self.shared_state.active_instance_id() {
            continue;
          }
          self.scheduler.pause();
          self.shared_state.set_state(PlaybackState::Paused);
        }
        PlaybackCommand::Stop { instance_id } => {
          if instance_id != self.shared_state.active_instance_id() {
            continue;
          }
          self.scheduler.stop();
          self.shared_state.set_state(PlaybackState::Stopped);
        }
      }
    }

    return;
  }

  /// Render audio for a callback tick.
  ///
  /// # Arguments
  /// - `writer`: Real-time writer for the current callback output buffer.
  ///
  /// # Returns
  /// `()` after draining commands and writing the output buffer.
  pub(super) fn render(&mut self, writer: &mut dyn AudioOutputWriter) {
    self.drain_commands();
    let master_volume = self.shared_state.master_volume();
    let instance_volume = self.shared_state.instance_volume();
    let instance_pitch = self.shared_state.instance_pitch();
    self.scheduler.render(
      writer,
      master_volume,
      instance_volume,
      instance_pitch,
    );
    self.shared_state.set_state(self.scheduler.state());
    return;
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::audio::SoundBuffer;

  struct TestAudioOutput {
    channels: u16,
    frames: usize,
    samples: Vec<f32>,
  }

  impl TestAudioOutput {
    fn new(channels: u16, frames: usize) -> Self {
      return Self {
        channels,
        frames,
        samples: vec![0.0; channels as usize * frames],
      };
    }

    fn sample(&self, frame: usize, channel: usize) -> f32 {
      let index = frame
        .saturating_mul(self.channels as usize)
        .saturating_add(channel);
      return self.samples.get(index).copied().unwrap_or(0.0);
    }

    fn max_abs(&self) -> f32 {
      return self.samples.iter().fold(0.0_f32, |accumulator, value| {
        return accumulator.max(value.abs());
      });
    }
  }

  impl AudioOutputWriter for TestAudioOutput {
    fn channels(&self) -> u16 {
      return self.channels;
    }

    fn frames(&self) -> usize {
      return self.frames;
    }

    fn clear(&mut self) {
      for value in self.samples.iter_mut() {
        *value = 0.0;
      }
      return;
    }

    fn set_sample(
      &mut self,
      frame_index: usize,
      channel_index: usize,
      sample: f32,
    ) {
      let index = frame_index
        .saturating_mul(self.channels as usize)
        .saturating_add(channel_index);
      if let Some(value) = self.samples.get_mut(index) {
        *value = sample;
      }
      return;
    }
  }

  fn make_test_buffer(samples: Vec<f32>, channels: u16) -> Arc<SoundBuffer> {
    let buffer =
      SoundBuffer::from_interleaved_samples_for_test(samples, 48_000, channels)
        .expect("test buffer creation failed");
    return Arc::new(buffer);
  }

  /// Scheduler MUST stop and reset the cursor when a buffer completes.
  #[test]
  fn scheduler_stops_after_completion() {
    let buffer = make_test_buffer(vec![0.5, 0.5, 0.5, 0.5], 1);

    let mut scheduler = PlaybackScheduler::new_with_ramp_frames(1, 2);
    scheduler.set_buffer(buffer);
    scheduler.play();

    let mut writer = TestAudioOutput::new(1, 8);
    scheduler.render(&mut writer, 1.0, 1.0, 1.0);

    assert_eq!(scheduler.state(), PlaybackState::Stopped);
    assert_eq!(scheduler.cursor_samples(), 0);

    let mut writer = TestAudioOutput::new(1, 4);
    scheduler.render(&mut writer, 1.0, 1.0, 1.0);
    assert!(writer.max_abs() <= 0.5);

    let mut writer = TestAudioOutput::new(1, 4);
    scheduler.render(&mut writer, 1.0, 1.0, 1.0);
    assert!(writer.max_abs() <= f32::EPSILON);
    return;
  }

  /// Pause MUST preserve cursor and fade to silence.
  #[test]
  fn scheduler_pause_preserves_cursor() {
    let buffer =
      make_test_buffer(vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8], 1);

    let mut scheduler = PlaybackScheduler::new_with_ramp_frames(1, 2);
    scheduler.set_buffer(buffer);
    scheduler.play();

    let mut writer = TestAudioOutput::new(1, 4);
    scheduler.render(&mut writer, 1.0, 1.0, 1.0);

    let cursor_before_pause = scheduler.cursor_samples();
    scheduler.pause();

    let mut writer = TestAudioOutput::new(1, 4);
    scheduler.render(&mut writer, 1.0, 1.0, 1.0);
    assert_eq!(scheduler.cursor_samples(), cursor_before_pause);

    let mut writer = TestAudioOutput::new(1, 4);
    scheduler.render(&mut writer, 1.0, 1.0, 1.0);
    assert!(writer.max_abs() <= f32::EPSILON);
    return;
  }

  /// Stop MUST reset the cursor and fade to silence.
  #[test]
  fn scheduler_stop_resets_cursor() {
    let buffer = make_test_buffer(vec![0.25; 32], 1);

    let mut scheduler = PlaybackScheduler::new_with_ramp_frames(1, 2);
    scheduler.set_buffer(buffer);
    scheduler.play();

    let mut writer = TestAudioOutput::new(1, 8);
    scheduler.render(&mut writer, 1.0, 1.0, 1.0);
    assert!(scheduler.cursor_samples() > 0);

    scheduler.stop();
    assert_eq!(scheduler.state(), PlaybackState::Stopped);
    assert_eq!(scheduler.cursor_samples(), 0);

    let mut writer = TestAudioOutput::new(1, 4);
    scheduler.render(&mut writer, 1.0, 1.0, 1.0);
    let mut writer = TestAudioOutput::new(1, 4);
    scheduler.render(&mut writer, 1.0, 1.0, 1.0);
    assert!(writer.max_abs() <= f32::EPSILON);
    return;
  }

  /// Looping MUST wrap the cursor and continue producing samples.
  #[test]
  fn scheduler_looping_wraps() {
    let buffer = make_test_buffer(vec![0.1, 0.2], 1);

    let mut scheduler = PlaybackScheduler::new_with_ramp_frames(1, 0);
    scheduler.set_buffer(buffer);
    scheduler.set_looping(true);
    scheduler.play();

    let mut writer = TestAudioOutput::new(1, 4);
    scheduler.render(&mut writer, 1.0, 1.0, 1.0);

    assert_eq!(scheduler.state(), PlaybackState::Playing);
    assert!((writer.sample(0, 0) - 0.1).abs() <= 1e-6);
    assert!((writer.sample(1, 0) - 0.2).abs() <= 1e-6);
    assert!((writer.sample(2, 0) - 0.1).abs() <= 1e-6);
    assert!((writer.sample(3, 0) - 0.2).abs() <= 1e-6);
    return;
  }

  /// Pitch `1.0` MUST reproduce the original sample sequence (no resampling).
  #[test]
  fn scheduler_pitch_one_reproduces_original_sequence() {
    let buffer = make_test_buffer(vec![0.1, 0.2, 0.3, 0.4], 1);

    let mut scheduler = PlaybackScheduler::new_with_ramp_frames(1, 0);
    scheduler.set_buffer(buffer);
    scheduler.play();

    let mut writer = TestAudioOutput::new(1, 4);
    scheduler.render(&mut writer, 1.0, 1.0, 1.0);

    assert!((writer.sample(0, 0) - 0.1).abs() <= 1e-6);
    assert!((writer.sample(1, 0) - 0.2).abs() <= 1e-6);
    assert!((writer.sample(2, 0) - 0.3).abs() <= 1e-6);
    assert!((writer.sample(3, 0) - 0.4).abs() <= 1e-6);
    return;
  }

  /// Pitch `2.0` MUST advance twice as fast (every other input frame).
  #[test]
  fn scheduler_pitch_two_advances_twice_as_fast() {
    let buffer = make_test_buffer(vec![0.0, 0.2, 0.4, 0.6, 0.8, 1.0], 1);

    let mut scheduler = PlaybackScheduler::new_with_ramp_frames(1, 0);
    scheduler.set_buffer(buffer);
    scheduler.play();

    let mut writer = TestAudioOutput::new(1, 3);
    scheduler.render(&mut writer, 1.0, 1.0, 2.0);

    assert!((writer.sample(0, 0) - 0.0).abs() <= 1e-6);
    assert!((writer.sample(1, 0) - 0.4).abs() <= 1e-6);
    assert!((writer.sample(2, 0) - 0.8).abs() <= 1e-6);
    return;
  }

  /// Pitch `0.5` MUST advance half as fast using linear interpolation.
  #[test]
  fn scheduler_pitch_half_interpolates_between_frames() {
    let buffer = make_test_buffer(vec![0.0, 0.2, 0.4, 0.6], 1);

    let mut scheduler = PlaybackScheduler::new_with_ramp_frames(1, 0);
    scheduler.set_buffer(buffer);
    scheduler.play();

    let mut writer = TestAudioOutput::new(1, 4);
    scheduler.render(&mut writer, 1.0, 1.0, 0.5);

    assert!((writer.sample(0, 0) - 0.0).abs() <= 1e-6);
    assert!((writer.sample(1, 0) - 0.1).abs() <= 1e-6);
    assert!((writer.sample(2, 0) - 0.2).abs() <= 1e-6);
    assert!((writer.sample(3, 0) - 0.3).abs() <= 1e-6);
    return;
  }

  /// Transport transitions MUST avoid hard discontinuities.
  #[test]
  fn scheduler_pause_is_continuous_at_transition_boundary() {
    let buffer = make_test_buffer(vec![0.5; 64], 1);

    let mut scheduler = PlaybackScheduler::new_with_ramp_frames(1, 2);
    scheduler.set_buffer(buffer);
    scheduler.play();

    let mut writer = TestAudioOutput::new(1, 8);
    scheduler.render(&mut writer, 1.0, 1.0, 1.0);
    let last = writer.sample(7, 0);

    scheduler.pause();

    let mut writer = TestAudioOutput::new(1, 1);
    scheduler.render(&mut writer, 1.0, 1.0, 1.0);
    let first = writer.sample(0, 0);

    assert!((last - first).abs() <= 1e-6);
    return;
  }

  /// Controllers MUST drain queued commands before rendering audio.
  #[test]
  fn controller_drains_commands_before_render() {
    let command_queue: Arc<CommandQueue<PlaybackCommand, 16>> =
      Arc::new(CommandQueue::new());
    let shared_state = Arc::new(PlaybackSharedState::new());
    let buffer = make_test_buffer(vec![0.1, 0.2], 1);

    shared_state.set_active_instance_id(1);

    command_queue
      .push(PlaybackCommand::SetBuffer {
        instance_id: 1,
        buffer,
      })
      .unwrap();
    command_queue
      .push(PlaybackCommand::SetLooping {
        instance_id: 1,
        looping: true,
      })
      .unwrap();
    command_queue
      .push(PlaybackCommand::Play { instance_id: 1 })
      .unwrap();

    let mut controller = PlaybackController::new_with_ramp_frames(
      1,
      0,
      command_queue,
      shared_state.clone(),
    );

    let mut writer = TestAudioOutput::new(1, 4);
    controller.render(&mut writer);

    assert!((writer.sample(0, 0) - 0.1).abs() <= 1e-6);
    assert!((writer.sample(1, 0) - 0.2).abs() <= 1e-6);
    assert!((writer.sample(2, 0) - 0.1).abs() <= 1e-6);
    assert!((writer.sample(3, 0) - 0.2).abs() <= 1e-6);
    assert_eq!(shared_state.state(), PlaybackState::Playing);
    return;
  }

  /// Controllers MUST ignore transport commands for inactive instances.
  #[test]
  fn controller_ignores_commands_for_inactive_instance() {
    let command_queue: Arc<CommandQueue<PlaybackCommand, 16>> =
      Arc::new(CommandQueue::new());
    let shared_state = Arc::new(PlaybackSharedState::new());
    let buffer = make_test_buffer(vec![0.1, 0.2], 1);

    shared_state.set_active_instance_id(2);

    let mut controller = PlaybackController::new_with_ramp_frames(
      1,
      0,
      command_queue.clone(),
      shared_state,
    );

    command_queue
      .push(PlaybackCommand::SetBuffer {
        instance_id: 1,
        buffer,
      })
      .unwrap();
    command_queue
      .push(PlaybackCommand::Play { instance_id: 1 })
      .unwrap();

    let mut writer = TestAudioOutput::new(1, 1);
    controller.render(&mut writer);
    assert!(writer.max_abs() <= f32::EPSILON);
    return;
  }

  #[test]
  /// Master volume `0.0` MUST silence output even while playing.
  fn controller_applies_master_volume_zero_as_silence() {
    let command_queue: Arc<CommandQueue<PlaybackCommand, 16>> =
      Arc::new(CommandQueue::new());
    let shared_state = Arc::new(PlaybackSharedState::new());
    let buffer = make_test_buffer(vec![0.8, 0.8, 0.8, 0.8], 1);

    shared_state.set_active_instance_id(1);
    shared_state.set_master_volume(0.0);

    command_queue
      .push(PlaybackCommand::SetBuffer {
        instance_id: 1,
        buffer,
      })
      .unwrap();
    command_queue
      .push(PlaybackCommand::Play { instance_id: 1 })
      .unwrap();

    let mut controller = PlaybackController::new_with_ramp_frames(
      1,
      0,
      command_queue,
      shared_state,
    );

    let mut writer = TestAudioOutput::new(1, 4);
    controller.render(&mut writer);
    assert!(writer.max_abs() <= f32::EPSILON);
    return;
  }

  #[test]
  /// Master volume MUST linearly scale output samples.
  fn controller_scales_output_by_master_volume() {
    let command_queue: Arc<CommandQueue<PlaybackCommand, 16>> =
      Arc::new(CommandQueue::new());
    let shared_state = Arc::new(PlaybackSharedState::new());
    let buffer = make_test_buffer(vec![0.4, 0.4, 0.4, 0.4], 1);

    shared_state.set_active_instance_id(1);
    shared_state.set_master_volume(0.5);

    command_queue
      .push(PlaybackCommand::SetBuffer {
        instance_id: 1,
        buffer,
      })
      .unwrap();
    command_queue
      .push(PlaybackCommand::Play { instance_id: 1 })
      .unwrap();

    let mut controller = PlaybackController::new_with_ramp_frames(
      1,
      0,
      command_queue,
      shared_state,
    );

    let mut writer = TestAudioOutput::new(1, 4);
    controller.render(&mut writer);

    for frame in 0..4 {
      assert!((writer.sample(frame, 0) - 0.2).abs() <= 1e-6);
    }
    return;
  }

  #[test]
  /// Gains that amplify beyond unity MUST remain bounded by clipping.
  fn controller_hard_clips_amplified_output() {
    let command_queue: Arc<CommandQueue<PlaybackCommand, 16>> =
      Arc::new(CommandQueue::new());
    let shared_state = Arc::new(PlaybackSharedState::new());
    let buffer = make_test_buffer(vec![0.8, 0.8, 0.8, 0.8], 1);

    shared_state.set_active_instance_id(1);
    shared_state.set_master_volume(2.0);

    command_queue
      .push(PlaybackCommand::SetBuffer {
        instance_id: 1,
        buffer,
      })
      .unwrap();
    command_queue
      .push(PlaybackCommand::Play { instance_id: 1 })
      .unwrap();

    let mut controller = PlaybackController::new_with_ramp_frames(
      1,
      0,
      command_queue,
      shared_state,
    );

    let mut writer = TestAudioOutput::new(1, 4);
    controller.render(&mut writer);

    assert!(writer.max_abs() <= 1.0 + 1e-6);
    assert!(writer.samples.iter().all(|value| value.is_finite()));
    return;
  }

  #[test]
  /// Instance volume `0.0` MUST silence output even while playing.
  fn controller_applies_instance_volume_zero_as_silence() {
    let command_queue: Arc<CommandQueue<PlaybackCommand, 16>> =
      Arc::new(CommandQueue::new());
    let shared_state = Arc::new(PlaybackSharedState::new());
    let buffer = make_test_buffer(vec![0.8, 0.8, 0.8, 0.8], 1);

    shared_state.set_active_instance_id(1);
    shared_state.set_instance_volume(0.0);

    command_queue
      .push(PlaybackCommand::SetBuffer {
        instance_id: 1,
        buffer,
      })
      .unwrap();
    command_queue
      .push(PlaybackCommand::Play { instance_id: 1 })
      .unwrap();

    let mut controller = PlaybackController::new_with_ramp_frames(
      1,
      0,
      command_queue,
      shared_state,
    );

    let mut writer = TestAudioOutput::new(1, 4);
    controller.render(&mut writer);
    assert!(writer.max_abs() <= f32::EPSILON);
    return;
  }

  #[test]
  /// Instance volume MUST linearly scale output samples.
  fn controller_scales_output_by_instance_volume() {
    let command_queue: Arc<CommandQueue<PlaybackCommand, 16>> =
      Arc::new(CommandQueue::new());
    let shared_state = Arc::new(PlaybackSharedState::new());
    let buffer = make_test_buffer(vec![0.4, 0.4, 0.4, 0.4], 1);

    shared_state.set_active_instance_id(1);
    shared_state.set_instance_volume(0.5);

    command_queue
      .push(PlaybackCommand::SetBuffer {
        instance_id: 1,
        buffer,
      })
      .unwrap();
    command_queue
      .push(PlaybackCommand::Play { instance_id: 1 })
      .unwrap();

    let mut controller = PlaybackController::new_with_ramp_frames(
      1,
      0,
      command_queue,
      shared_state,
    );

    let mut writer = TestAudioOutput::new(1, 4);
    controller.render(&mut writer);

    for frame in 0..4 {
      assert!((writer.sample(frame, 0) - 0.2).abs() <= 1e-6);
    }
    return;
  }

  #[test]
  /// Amplified per-instance gain MUST remain bounded by clipping.
  fn controller_hard_clips_amplified_instance_volume() {
    let command_queue: Arc<CommandQueue<PlaybackCommand, 16>> =
      Arc::new(CommandQueue::new());
    let shared_state = Arc::new(PlaybackSharedState::new());
    let buffer = make_test_buffer(vec![0.8, 0.8, 0.8, 0.8], 1);

    shared_state.set_active_instance_id(1);
    shared_state.set_instance_volume(2.0);

    command_queue
      .push(PlaybackCommand::SetBuffer {
        instance_id: 1,
        buffer,
      })
      .unwrap();
    command_queue
      .push(PlaybackCommand::Play { instance_id: 1 })
      .unwrap();

    let mut controller = PlaybackController::new_with_ramp_frames(
      1,
      0,
      command_queue,
      shared_state,
    );

    let mut writer = TestAudioOutput::new(1, 4);
    controller.render(&mut writer);

    assert!(writer.max_abs() <= 1.0 + 1e-6);
    assert!(writer.samples.iter().all(|value| value.is_finite()));
    return;
  }
}
