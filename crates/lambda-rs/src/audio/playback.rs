#![allow(clippy::needless_return)]

//! Single-sound playback and transport controls.
//!
//! This module provides a minimal, backend-agnostic playback facade that
//! supports one active `SoundBuffer` at a time.

use std::sync::Arc;

use crate::audio::{
  AudioError,
  AudioOutputWriter,
  SoundBuffer,
};

const DEFAULT_GAIN_RAMP_FRAMES: usize = 128;

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
#[allow(dead_code)]
struct PlaybackScheduler {
  state: PlaybackState,
  looping: bool,
  cursor_samples: usize,
  channels: usize,
  ramp_frames: usize,
  gain: GainRamp,
  buffer: Option<Arc<SoundBuffer>>,
  last_frame_samples: Vec<f32>,
}

#[allow(dead_code)]
impl PlaybackScheduler {
  /// Create a scheduler configured for a fixed output channel count.
  ///
  /// # Arguments
  /// - `channels`: Interleaved output channel count.
  ///
  /// # Returns
  /// A scheduler initialized to `Stopped` with no buffer.
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
      last_frame_samples: vec![0.0; channels],
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

  /// Begin playback, or resume if paused.
  ///
  /// # Returns
  /// `()` after updating the transport state.
  fn play(&mut self) {
    self.state = PlaybackState::Playing;
    self.gain.start(1.0, self.ramp_frames);
    return;
  }

  /// Pause playback, preserving the current cursor.
  ///
  /// # Returns
  /// `()` after updating the transport state.
  fn pause(&mut self) {
    self.state = PlaybackState::Paused;
    self.gain.start(0.0, self.ramp_frames);
    return;
  }

  /// Stop playback and reset the cursor to the start.
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

    if writer_channels != self.channels {
      writer.clear();
      return;
    }

    if self.state != PlaybackState::Playing && self.gain.is_silent() {
      writer.clear();
      return;
    }

    for frame_index in 0..frames {
      let frame_gain = self.gain.current;

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
        let sample = self
          .last_frame_samples
          .get(channel_index)
          .copied()
          .unwrap_or(0.0);
        writer.set_sample(frame_index, channel_index, sample * frame_gain);
      }

      self.gain.advance_frame();
    }

    return;
  }
}

/// A queryable playback state for a `SoundInstance`.
///
/// This state is observable from the application thread and is intended to
/// provide basic transport visibility.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlaybackState {
  /// The sound is currently playing.
  Playing,
  /// The sound is currently paused.
  Paused,
  /// The sound is stopped and positioned at the start.
  Stopped,
}

/// A lightweight handle controlling the active sound playback slot.
///
/// This type is a placeholder API surface used while sound playback is under
/// active development. It is expected to become fully functional in a
/// subsequent change set.
pub struct SoundInstance {
  state: PlaybackState,
  looping: bool,
}

impl SoundInstance {
  /// Begin playback, or resume if paused.
  ///
  /// # Returns
  /// `()` after updating the requested transport state.
  pub fn play(&mut self) {
    self.state = PlaybackState::Playing;
    return;
  }

  /// Pause playback, preserving playback position.
  ///
  /// # Returns
  /// `()` after updating the requested transport state.
  pub fn pause(&mut self) {
    self.state = PlaybackState::Paused;
    return;
  }

  /// Stop playback and reset position to the start of the buffer.
  ///
  /// # Returns
  /// `()` after updating the requested transport state.
  pub fn stop(&mut self) {
    self.state = PlaybackState::Stopped;
    return;
  }

  /// Enable or disable looping playback.
  ///
  /// # Arguments
  /// - `looping`: Whether the sound should loop on completion.
  ///
  /// # Returns
  /// `()` after updating the looping flag.
  pub fn set_looping(&mut self, looping: bool) {
    self.looping = looping;
    return;
  }

  /// Query the current state of this instance.
  ///
  /// # Returns
  /// The current transport state.
  pub fn state(&self) -> PlaybackState {
    return self.state;
  }

  /// Convenience query for `state() == PlaybackState::Playing`.
  ///
  /// # Returns
  /// `true` if the instance state is `Playing`.
  pub fn is_playing(&self) -> bool {
    return self.state == PlaybackState::Playing;
  }

  /// Convenience query for `state() == PlaybackState::Paused`.
  ///
  /// # Returns
  /// `true` if the instance state is `Paused`.
  pub fn is_paused(&self) -> bool {
    return self.state == PlaybackState::Paused;
  }

  /// Convenience query for `state() == PlaybackState::Stopped`.
  ///
  /// # Returns
  /// `true` if the instance state is `Stopped`.
  pub fn is_stopped(&self) -> bool {
    return self.state == PlaybackState::Stopped;
  }
}

/// A playback context owning an output device and one active playback slot.
///
/// This type is a placeholder API surface used while sound playback is under
/// active development. It is expected to become fully functional in a
/// subsequent change set.
pub struct AudioContext {
  _requested_sample_rate: Option<u32>,
  _requested_channels: Option<u16>,
  _label: Option<String>,
}

/// Builder for creating an `AudioContext`.
#[derive(Debug, Clone)]
pub struct AudioContextBuilder {
  sample_rate: Option<u32>,
  channels: Option<u16>,
  label: Option<String>,
}

impl AudioContextBuilder {
  /// Create a builder with engine defaults.
  ///
  /// # Returns
  /// A builder with no explicit configuration requests.
  pub fn new() -> Self {
    return Self {
      sample_rate: None,
      channels: None,
      label: None,
    };
  }

  /// Request an output sample rate.
  ///
  /// # Arguments
  /// - `rate`: Requested output sample rate in frames per second.
  ///
  /// # Returns
  /// The updated builder.
  pub fn with_sample_rate(mut self, rate: u32) -> Self {
    self.sample_rate = Some(rate);
    return self;
  }

  /// Request an output channel count.
  ///
  /// # Arguments
  /// - `channels`: Requested interleaved output channel count.
  ///
  /// # Returns
  /// The updated builder.
  pub fn with_channels(mut self, channels: u16) -> Self {
    self.channels = Some(channels);
    return self;
  }

  /// Attach a label for diagnostics.
  ///
  /// # Arguments
  /// - `label`: A human-readable label used for diagnostics.
  ///
  /// # Returns
  /// The updated builder.
  pub fn with_label(mut self, label: &str) -> Self {
    self.label = Some(label.to_string());
    return self;
  }

  /// Build an `AudioContext` using the requested configuration.
  ///
  /// # Returns
  /// An initialized audio context handle.
  ///
  /// # Errors
  /// Returns an error until sound playback is integrated with an audio output
  /// device callback.
  pub fn build(self) -> Result<AudioContext, AudioError> {
    return Err(AudioError::InvalidData {
      details: format!(
        "audio context playback is not implemented in this build (requested_sample_rate={:?}, requested_channels={:?}, label={:?})",
        self.sample_rate,
        self.channels,
        self.label
      ),
    });
  }
}

impl Default for AudioContextBuilder {
  fn default() -> Self {
    return Self::new();
  }
}

impl AudioContext {
  /// Play a decoded `SoundBuffer` through this context.
  ///
  /// # Arguments
  /// - `buffer`: The decoded sound buffer to schedule for playback.
  ///
  /// # Returns
  /// A lightweight `SoundInstance` handle for controlling playback.
  ///
  /// # Errors
  /// Returns an error until the playback implementation is integrated with an
  /// audio output device callback.
  pub fn play_sound(
    &mut self,
    buffer: &SoundBuffer,
  ) -> Result<SoundInstance, AudioError> {
    return Err(AudioError::InvalidData {
      details: format!(
        "sound playback is not implemented in this build (buffer_sample_rate={}, buffer_channels={})",
        buffer.sample_rate(),
        buffer.channels()
      ),
    });
  }
}

#[cfg(test)]
mod tests {
  use super::*;

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
}
