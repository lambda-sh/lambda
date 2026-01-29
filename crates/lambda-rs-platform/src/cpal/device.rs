#![allow(clippy::needless_return)]

//! Audio output device discovery and stream initialization.
//!
//! This module defines a backend-agnostic surface that `lambda-rs` can use to
//! enumerate and initialize audio output devices. The implementation is
//! expected to be backed by a platform dependency (for example, `cpal`) behind
//! feature flags.
//!
//! This surface MUST NOT expose backend or vendor types (including `cpal`
//! types) in its public API.

use std::{
  error::Error,
  fmt,
};

/// Output sample format used by the platform stream callback.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AudioSampleFormat {
  /// 32-bit floating point samples in the nominal range `[-1.0, 1.0]`.
  F32,
  /// Signed 16-bit integer samples mapped from normalized `f32`.
  I16,
  /// Unsigned 16-bit integer samples mapped from normalized `f32`.
  U16,
}

/// Information available to audio output callbacks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AudioCallbackInfo {
  /// Audio frames per second.
  pub sample_rate: u32,
  /// Interleaved output channel count.
  pub channels: u16,
  /// The selected stream sample format.
  pub sample_format: AudioSampleFormat,
}

/// Real-time writer for audio output buffers.
///
/// This writer MUST be implemented without allocation and MUST write into the
/// underlying device output buffer for the current callback invocation.
pub trait AudioOutputWriter {
  /// Return the output channel count for the current callback invocation.
  fn channels(&self) -> u16;
  /// Return the number of frames in the output buffer for the current callback
  /// invocation.
  fn frames(&self) -> usize;
  /// Clear the entire output buffer to silence.
  fn clear(&mut self);

  /// Write a normalized sample in the range `[-1.0, 1.0]`.
  ///
  /// Implementations MUST clamp values outside `[-1.0, 1.0]`. Implementations
  /// MUST NOT panic for out-of-range indices and MUST perform no write in that
  /// case.
  fn set_sample(
    &mut self,
    frame_index: usize,
    channel_index: usize,
    sample: f32,
  );
}

/// Metadata describing an available audio output device.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AudioDeviceInfo {
  /// Human-readable device name.
  pub name: String,
  /// Whether this device is the current default output device.
  pub is_default: bool,
}

/// Actionable errors produced by the platform audio layer.
///
/// This error type is internal to `lambda-rs-platform` and MUST NOT expose
/// backend-specific types in its public API.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AudioError {
  /// The requested sample rate was invalid.
  InvalidSampleRate { requested: u32 },
  /// The requested channel count was invalid.
  InvalidChannels { requested: u16 },
  /// No audio host is available.
  HostUnavailable { details: String },
  /// No default audio output device is available.
  NoDefaultDevice,
  /// The device name could not be retrieved.
  DeviceNameUnavailable { details: String },
  /// Device enumeration failed.
  DeviceEnumerationFailed { details: String },
  /// Supported output configurations could not be retrieved.
  SupportedConfigsUnavailable { details: String },
  /// No supported output configuration satisfied the request.
  UnsupportedConfig {
    requested_sample_rate: Option<u32>,
    requested_channels: Option<u16>,
  },
  /// The selected output sample format is unsupported by this abstraction.
  UnsupportedSampleFormat { details: String },
  /// A backend-specific failure occurred.
  Platform { details: String },
  /// Building an output stream failed.
  StreamBuildFailed { details: String },
  /// Starting an output stream failed.
  StreamPlayFailed { details: String },
}

impl fmt::Display for AudioError {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::InvalidSampleRate { requested } => {
        return write!(formatter, "invalid sample rate: {requested}");
      }
      Self::InvalidChannels { requested } => {
        return write!(formatter, "invalid channel count: {requested}");
      }
      Self::HostUnavailable { details } => {
        return write!(formatter, "audio host unavailable: {details}");
      }
      Self::NoDefaultDevice => {
        return write!(formatter, "no default audio output device available");
      }
      Self::DeviceNameUnavailable { details } => {
        return write!(formatter, "device name unavailable: {details}");
      }
      Self::DeviceEnumerationFailed { details } => {
        return write!(formatter, "device enumeration failed: {details}");
      }
      Self::SupportedConfigsUnavailable { details } => {
        return write!(
          formatter,
          "supported output configs unavailable: {details}"
        );
      }
      Self::UnsupportedConfig {
        requested_sample_rate,
        requested_channels,
      } => {
        return write!(
          formatter,
          "unsupported output config: sample_rate={requested_sample_rate:?} channels={requested_channels:?}",
        );
      }
      Self::UnsupportedSampleFormat { details } => {
        return write!(
          formatter,
          "unsupported output sample format: {details}"
        );
      }
      Self::Platform { details } => {
        return write!(formatter, "platform audio error: {details}");
      }
      Self::StreamBuildFailed { details } => {
        return write!(formatter, "stream build failed: {details}");
      }
      Self::StreamPlayFailed { details } => {
        return write!(formatter, "stream play failed: {details}");
      }
    }
  }
}

impl Error for AudioError {}

/// An initialized audio output device.
///
/// This type is an opaque platform wrapper. It MUST NOT expose backend types.
pub struct AudioDevice {
  _private: (),
}

/// Builder for creating an [`AudioDevice`].
#[derive(Debug, Clone)]
pub struct AudioDeviceBuilder {
  sample_rate: Option<u32>,
  channels: Option<u16>,
  label: Option<String>,
}

impl AudioDeviceBuilder {
  /// Create a builder with engine defaults.
  pub fn new() -> Self {
    return Self {
      sample_rate: None,
      channels: None,
      label: None,
    };
  }

  /// Request a specific sample rate (Hz).
  pub fn with_sample_rate(mut self, rate: u32) -> Self {
    self.sample_rate = Some(rate);
    return self;
  }

  /// Request a specific channel count.
  pub fn with_channels(mut self, channels: u16) -> Self {
    self.channels = Some(channels);
    return self;
  }

  /// Attach a label for diagnostics.
  pub fn with_label(mut self, label: &str) -> Self {
    self.label = Some(label.to_string());
    return self;
  }

  /// Initialize the default audio output device using the requested
  /// configuration.
  pub fn build(self) -> Result<AudioDevice, AudioError> {
    if let Some(sample_rate) = self.sample_rate {
      if sample_rate == 0 {
        return Err(AudioError::InvalidSampleRate {
          requested: sample_rate,
        });
      }
    }

    if let Some(channels) = self.channels {
      if channels == 0 {
        return Err(AudioError::InvalidChannels {
          requested: channels,
        });
      }
    }

    return Err(AudioError::HostUnavailable {
      details: "audio backend not wired".to_string(),
    });
  }

  /// Initialize the default audio output device and play audio via a callback.
  pub fn build_with_output_callback<Callback>(
    self,
    callback: Callback,
  ) -> Result<AudioDevice, AudioError>
  where
    Callback:
      'static + Send + FnMut(&mut dyn AudioOutputWriter, AudioCallbackInfo),
  {
    let _ = callback;
    return self.build();
  }
}

/// Enumerate available audio output devices.
pub fn enumerate_devices() -> Result<Vec<AudioDeviceInfo>, AudioError> {
  return Err(AudioError::HostUnavailable {
    details: "audio backend not wired".to_string(),
  });
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn build_rejects_zero_sample_rate() {
    let result = AudioDeviceBuilder::new().with_sample_rate(0).build();
    assert!(matches!(
      result,
      Err(AudioError::InvalidSampleRate { requested: 0 })
    ));
  }

  #[test]
  fn build_rejects_zero_channels() {
    let result = AudioDeviceBuilder::new().with_channels(0).build();
    assert!(matches!(
      result,
      Err(AudioError::InvalidChannels { requested: 0 })
    ));
  }

  #[test]
  fn build_returns_host_unavailable_until_backend_is_wired() {
    let result = AudioDeviceBuilder::new().build();
    match result {
      Err(AudioError::HostUnavailable { details }) => {
        assert_eq!(details, "audio backend not wired");
        return;
      }
      Ok(_device) => {
        panic!("expected host unavailable error, got Ok");
      }
      Err(error) => {
        panic!("expected host unavailable error, got {error}");
      }
    }
  }

  #[test]
  fn enumerate_devices_returns_host_unavailable_until_backend_is_wired() {
    let result = enumerate_devices();
    match result {
      Err(AudioError::HostUnavailable { details }) => {
        assert_eq!(details, "audio backend not wired");
        return;
      }
      Ok(_devices) => {
        panic!("expected host unavailable error, got Ok");
      }
      Err(error) => {
        panic!("expected host unavailable error, got {error}");
      }
    }
  }

  #[test]
  fn build_with_output_callback_returns_host_unavailable_until_backend_is_wired(
  ) {
    let result = AudioDeviceBuilder::new().build_with_output_callback(
      |_writer, _callback_info| {
        return;
      },
    );
    match result {
      Err(AudioError::HostUnavailable { details }) => {
        assert_eq!(details, "audio backend not wired");
        return;
      }
      Ok(_device) => {
        panic!("expected host unavailable error, got Ok");
      }
      Err(error) => {
        panic!("expected host unavailable error, got {error}");
      }
    }
  }
}
