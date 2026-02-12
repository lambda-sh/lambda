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
  cursor_samples: usize,
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
      cursor_samples: 0,
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
    self.cursor_samples = 0;
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
    self.cursor_samples = 0;
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
    return self.cursor_samples;
  }

  /// Render audio for a callback tick into an output writer.
  ///
  /// # Arguments
  /// - `writer`: Real-time writer for the current callback output buffer.
  ///
  /// # Returns
  /// `()` after writing the output buffer.
  fn render(&mut self, writer: &mut dyn AudioOutputWriter) {
    let writer_channels = writer.channels() as usize;
    let frames = writer.frames();

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
        let mut frame_start = self.cursor_samples;
        let mut frame_end = frame_start.saturating_add(writer_channels);

        if frame_end > samples.len()
          && self.looping
          && samples.len() >= writer_channels
        {
          self.cursor_samples = 0;
          frame_start = 0;
          frame_end = writer_channels;
        }

        if frame_end <= samples.len() {
          for channel_index in 0..writer_channels {
            let sample = samples
              .get(frame_start.saturating_add(channel_index))
              .copied()
              .unwrap_or(0.0);
            self.last_frame_samples[channel_index] = sample;
            writer.set_sample(frame_index, channel_index, sample * frame_gain);
          }

          self.cursor_samples = frame_end;
          self.gain.advance_frame();
          continue;
        }

        self.state = PlaybackState::Stopped;
        self.cursor_samples = 0;
        self.gain.start(0.0, self.ramp_frames);
      }

      for channel_index in 0..writer_channels {
        let sample = self.last_frame_samples[channel_index];
        writer.set_sample(frame_index, channel_index, sample * frame_gain);
      }

      self.gain.advance_frame();
    }

    return;
  }
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
    self.scheduler.render(writer);
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
    scheduler.render(&mut writer);

    assert_eq!(scheduler.state(), PlaybackState::Stopped);
    assert_eq!(scheduler.cursor_samples(), 0);

    let mut writer = TestAudioOutput::new(1, 4);
    scheduler.render(&mut writer);
    assert!(writer.max_abs() <= 0.5);

    let mut writer = TestAudioOutput::new(1, 4);
    scheduler.render(&mut writer);
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
    scheduler.render(&mut writer);

    let cursor_before_pause = scheduler.cursor_samples();
    scheduler.pause();

    let mut writer = TestAudioOutput::new(1, 4);
    scheduler.render(&mut writer);
    assert_eq!(scheduler.cursor_samples(), cursor_before_pause);

    let mut writer = TestAudioOutput::new(1, 4);
    scheduler.render(&mut writer);
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
    scheduler.render(&mut writer);
    assert!(scheduler.cursor_samples() > 0);

    scheduler.stop();
    assert_eq!(scheduler.state(), PlaybackState::Stopped);
    assert_eq!(scheduler.cursor_samples(), 0);

    let mut writer = TestAudioOutput::new(1, 4);
    scheduler.render(&mut writer);
    let mut writer = TestAudioOutput::new(1, 4);
    scheduler.render(&mut writer);
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
    scheduler.render(&mut writer);

    assert_eq!(scheduler.state(), PlaybackState::Playing);
    assert!((writer.sample(0, 0) - 0.1).abs() <= 1e-6);
    assert!((writer.sample(1, 0) - 0.2).abs() <= 1e-6);
    assert!((writer.sample(2, 0) - 0.1).abs() <= 1e-6);
    assert!((writer.sample(3, 0) - 0.2).abs() <= 1e-6);
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
    scheduler.render(&mut writer);
    let last = writer.sample(7, 0);

    scheduler.pause();

    let mut writer = TestAudioOutput::new(1, 1);
    scheduler.render(&mut writer);
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
}
