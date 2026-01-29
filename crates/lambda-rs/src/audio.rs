#![allow(clippy::needless_return)]

//! Application-facing audio output devices.
//!
//! This module provides a backend-agnostic audio output device API for Lambda
//! applications. Platform and vendor details are implemented in
//! `lambda-rs-platform` and MUST NOT be exposed through the `lambda-rs` public
//! API.

use lambda_platform::cpal as platform_audio;

/// Output sample format used by an audio stream callback.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AudioSampleFormat {
  /// 32-bit floating point samples in the nominal range `[-1.0, 1.0]`.
  F32,
  /// Signed 16-bit integer samples mapped from normalized `f32`.
  I16,
  /// Unsigned 16-bit integer samples mapped from normalized `f32`.
  U16,
}

impl AudioSampleFormat {
  fn from_platform(value: platform_audio::AudioSampleFormat) -> Self {
    match value {
      platform_audio::AudioSampleFormat::F32 => {
        return Self::F32;
      }
      platform_audio::AudioSampleFormat::I16 => {
        return Self::I16;
      }
      platform_audio::AudioSampleFormat::U16 => {
        return Self::U16;
      }
    }
  }
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

impl AudioCallbackInfo {
  fn from_platform(value: platform_audio::AudioCallbackInfo) -> Self {
    return Self {
      sample_rate: value.sample_rate,
      channels: value.channels,
      sample_format: AudioSampleFormat::from_platform(value.sample_format),
    };
  }
}

/// Actionable errors produced by the `lambda-rs` audio facade.
///
/// This error type MUST remain backend-agnostic and MUST NOT expose platform or
/// vendor types.
#[derive(Clone, Debug)]
pub enum AudioError {
  /// The requested sample rate was invalid.
  InvalidSampleRate { requested: u32 },
  /// The requested channel count was invalid.
  InvalidChannels { requested: u16 },
  /// No default audio output device is available.
  NoDefaultDevice,
  /// No supported output configuration satisfied the request.
  UnsupportedConfig {
    requested_sample_rate: Option<u32>,
    requested_channels: Option<u16>,
  },
  /// The selected output sample format is unsupported by this abstraction.
  UnsupportedSampleFormat { details: String },
  /// A platform or backend specific error occurred.
  Platform { details: String },
}

fn map_platform_error(error: platform_audio::AudioError) -> AudioError {
  match error {
    platform_audio::AudioError::InvalidSampleRate { requested } => {
      return AudioError::InvalidSampleRate { requested };
    }
    platform_audio::AudioError::InvalidChannels { requested } => {
      return AudioError::InvalidChannels { requested };
    }
    platform_audio::AudioError::NoDefaultDevice => {
      return AudioError::NoDefaultDevice;
    }
    platform_audio::AudioError::UnsupportedConfig {
      requested_sample_rate,
      requested_channels,
    } => {
      return AudioError::UnsupportedConfig {
        requested_sample_rate,
        requested_channels,
      };
    }
    platform_audio::AudioError::UnsupportedSampleFormat { details } => {
      return AudioError::UnsupportedSampleFormat { details };
    }
    other => {
      return AudioError::Platform {
        details: other.to_string(),
      };
    }
  }
}

/// Metadata describing an available audio output device.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AudioOutputDeviceInfo {
  /// Human-readable device name.
  pub name: String,
  /// Whether this device is the current default output device.
  pub is_default: bool,
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

struct OutputWriterAdapter<'writer> {
  writer: &'writer mut dyn platform_audio::AudioOutputWriter,
}

impl<'writer> AudioOutputWriter for OutputWriterAdapter<'writer> {
  fn channels(&self) -> u16 {
    return self.writer.channels();
  }

  fn frames(&self) -> usize {
    return self.writer.frames();
  }

  fn clear(&mut self) {
    self.writer.clear();
    return;
  }

  fn set_sample(
    &mut self,
    frame_index: usize,
    channel_index: usize,
    sample: f32,
  ) {
    self.writer.set_sample(frame_index, channel_index, sample);
    return;
  }
}

/// An initialized audio output device.
///
/// The returned handle MUST be kept alive for as long as audio output is
/// required. Dropping the handle MUST stop output.
pub struct AudioOutputDevice {
  _platform: platform_audio::AudioDevice,
}

/// Builder for creating an [`AudioOutputDevice`].
#[derive(Debug, Clone)]
pub struct AudioOutputDeviceBuilder {
  sample_rate: Option<u32>,
  channels: Option<u16>,
  label: Option<String>,
}

impl AudioOutputDeviceBuilder {
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
  pub fn build(self) -> Result<AudioOutputDevice, AudioError> {
    let mut platform_builder = platform_audio::AudioDeviceBuilder::new();

    if let Some(sample_rate) = self.sample_rate {
      platform_builder = platform_builder.with_sample_rate(sample_rate);
    }

    if let Some(channels) = self.channels {
      platform_builder = platform_builder.with_channels(channels);
    }

    if let Some(label) = self.label {
      platform_builder = platform_builder.with_label(&label);
    }

    let platform_device =
      platform_builder.build().map_err(map_platform_error)?;

    return Ok(AudioOutputDevice {
      _platform: platform_device,
    });
  }

  /// Initialize the default audio output device and play audio via a callback.
  pub fn build_with_output_callback<Callback>(
    self,
    callback: Callback,
  ) -> Result<AudioOutputDevice, AudioError>
  where
    Callback:
      'static + Send + FnMut(&mut dyn AudioOutputWriter, AudioCallbackInfo),
  {
    let mut platform_builder = platform_audio::AudioDeviceBuilder::new();

    if let Some(sample_rate) = self.sample_rate {
      platform_builder = platform_builder.with_sample_rate(sample_rate);
    }

    if let Some(channels) = self.channels {
      platform_builder = platform_builder.with_channels(channels);
    }

    if let Some(label) = self.label {
      platform_builder = platform_builder.with_label(&label);
    }

    let mut callback = callback;
    let platform_device = platform_builder
      .build_with_output_callback(move |writer, callback_info| {
        let mut adapter = OutputWriterAdapter { writer };
        callback(
          &mut adapter,
          AudioCallbackInfo::from_platform(callback_info),
        );
        return;
      })
      .map_err(map_platform_error)?;

    return Ok(AudioOutputDevice {
      _platform: platform_device,
    });
  }
}

/// Enumerate available audio output devices via the platform layer.
pub fn enumerate_output_devices(
) -> Result<Vec<AudioOutputDeviceInfo>, AudioError> {
  let devices =
    platform_audio::enumerate_devices().map_err(map_platform_error)?;

  let devices = devices
    .into_iter()
    .map(|device| AudioOutputDeviceInfo {
      name: device.name,
      is_default: device.is_default,
    })
    .collect();

  return Ok(devices);
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn errors_map_without_leaking_platform_types() {
    let result = AudioOutputDeviceBuilder::new().with_sample_rate(0).build();
    assert!(matches!(
      result,
      Err(AudioError::InvalidSampleRate { requested: 0 })
    ));

    let result = enumerate_output_devices();
    match result {
      Err(AudioError::Platform { details }) => {
        assert_eq!(details, "audio host unavailable: audio backend not wired");
        return;
      }
      Ok(_devices) => {
        panic!("expected platform error, got Ok");
      }
      Err(error) => {
        panic!("expected platform error, got {error:?}");
      }
    }
  }
}
