#![allow(clippy::needless_return)]

//! Single-sound playback and transport controls.
//!
//! This module provides a minimal, backend-agnostic playback facade that
//! supports one active `SoundBuffer` at a time.

use crate::audio::{
  AudioError,
  SoundBuffer,
};

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
