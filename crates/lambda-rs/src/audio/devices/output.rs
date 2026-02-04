#![allow(clippy::needless_return)]

//! Audio output devices.
//!
//! This module provides a backend-agnostic audio output device API for Lambda
//! applications. Platform and vendor details are implemented in
//! `lambda-rs-platform` and MUST NOT be exposed through the `lambda-rs` public
//! API.

use lambda_platform::audio::cpal as platform_audio;

use crate::audio::AudioError;

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
  /// Map a platform sample format into the public API sample format.
  ///
  /// # Arguments
  /// - `value`: The platform-provided sample format.
  ///
  /// # Returns
  /// The equivalent public API sample format.
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
  /// Map platform callback metadata into the public API callback metadata.
  ///
  /// # Arguments
  /// - `value`: The platform-provided callback metadata.
  ///
  /// # Returns
  /// The equivalent public API callback metadata.
  fn from_platform(value: platform_audio::AudioCallbackInfo) -> Self {
    return Self {
      sample_rate: value.sample_rate,
      channels: value.channels,
      sample_format: AudioSampleFormat::from_platform(value.sample_format),
    };
  }
}

/// Map platform audio errors into backend-agnostic public errors.
///
/// # Arguments
/// - `error`: The platform error.
///
/// # Returns
/// A backend-agnostic error suitable for returning from `lambda-rs`.
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
  ///
  /// # Returns
  /// The number of interleaved channels in the output buffer.
  fn channels(&self) -> u16;
  /// Return the number of frames in the output buffer for the current callback
  /// invocation.
  ///
  /// # Returns
  /// The number of frames in the output buffer.
  fn frames(&self) -> usize;
  /// Clear the entire output buffer to silence.
  ///
  /// # Returns
  /// `()` after clearing the output buffer to silence.
  fn clear(&mut self);

  /// Write a normalized sample in the range `[-1.0, 1.0]`.
  ///
  /// Implementations MUST clamp values outside `[-1.0, 1.0]`. Implementations
  /// MUST NOT panic for out-of-range indices and MUST perform no write in that
  /// case.
  ///
  /// # Arguments
  /// - `frame_index`: The target frame index within the current callback
  ///   buffer.
  /// - `channel_index`: The target channel index within the current callback
  ///   buffer.
  /// - `sample`: A normalized sample in nominal range `[-1.0, 1.0]`.
  ///
  /// # Returns
  /// `()` after attempting to write the sample.
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

  /// Request a specific sample rate (Hz).
  ///
  /// # Arguments
  /// - `rate`: Requested sample rate in Hz.
  ///
  /// # Returns
  /// The updated builder.
  pub fn with_sample_rate(mut self, rate: u32) -> Self {
    self.sample_rate = Some(rate);
    return self;
  }

  /// Request a specific channel count.
  ///
  /// # Arguments
  /// - `channels`: Requested interleaved channel count.
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

  /// Initialize the default audio output device using the requested
  /// configuration.
  ///
  /// # Returns
  /// An initialized audio output device handle. Dropping the handle stops
  /// output.
  ///
  /// # Errors
  /// Returns an error if the platform layer cannot initialize a default output
  /// device or cannot satisfy the requested configuration.
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
  ///
  /// The callback is invoked from the platform audio thread. The callback MUST
  /// avoid blocking and MUST NOT allocate.
  ///
  /// # Arguments
  /// - `callback`: A real-time callback invoked for each output buffer tick.
  ///
  /// # Returns
  /// An initialized audio output device handle. Dropping the handle stops
  /// output.
  ///
  /// # Errors
  /// Returns an error if the platform layer cannot initialize a default output
  /// device or cannot satisfy the requested configuration.
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

impl Default for AudioOutputDeviceBuilder {
  fn default() -> Self {
    return Self::new();
  }
}

/// Enumerate available audio output devices via the platform layer.
///
/// # Returns
/// A list of available output devices with stable metadata.
///
/// # Errors
/// Returns an error if the platform layer cannot enumerate devices.
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

  /// Error mapping MUST remain backend-agnostic.
  #[test]
  fn errors_map_without_leaking_platform_types() {
    let result = AudioOutputDeviceBuilder::new().with_sample_rate(0).build();
    assert!(matches!(
      result,
      Err(AudioError::InvalidSampleRate { requested: 0 })
    ));

    let _result = enumerate_output_devices();
    return;
  }

  /// Sample format mapping MUST be stable.
  #[test]
  fn sample_format_maps_from_platform() {
    assert_eq!(
      AudioSampleFormat::from_platform(platform_audio::AudioSampleFormat::F32),
      AudioSampleFormat::F32
    );
    assert_eq!(
      AudioSampleFormat::from_platform(platform_audio::AudioSampleFormat::I16),
      AudioSampleFormat::I16
    );
    assert_eq!(
      AudioSampleFormat::from_platform(platform_audio::AudioSampleFormat::U16),
      AudioSampleFormat::U16
    );
    return;
  }

  /// Callback metadata mapping MUST preserve stable fields.
  #[test]
  fn callback_info_maps_from_platform() {
    let platform_info = platform_audio::AudioCallbackInfo {
      sample_rate: 48_000,
      channels: 2,
      sample_format: platform_audio::AudioSampleFormat::F32,
    };

    let info = AudioCallbackInfo::from_platform(platform_info);
    assert_eq!(info.sample_rate, 48_000);
    assert_eq!(info.channels, 2);
    assert_eq!(info.sample_format, AudioSampleFormat::F32);
    return;
  }

  /// Platform error mapping MUST cover each supported public error variant.
  #[test]
  fn platform_errors_map_to_public_errors() {
    assert!(matches!(
      map_platform_error(platform_audio::AudioError::InvalidSampleRate {
        requested: 1
      }),
      AudioError::InvalidSampleRate { requested: 1 }
    ));

    assert!(matches!(
      map_platform_error(platform_audio::AudioError::InvalidChannels {
        requested: 2
      }),
      AudioError::InvalidChannels { requested: 2 }
    ));

    assert!(matches!(
      map_platform_error(platform_audio::AudioError::NoDefaultDevice),
      AudioError::NoDefaultDevice
    ));

    assert!(matches!(
      map_platform_error(platform_audio::AudioError::UnsupportedConfig {
        requested_sample_rate: Some(44_100),
        requested_channels: Some(1),
      }),
      AudioError::UnsupportedConfig {
        requested_sample_rate: Some(44_100),
        requested_channels: Some(1),
      }
    ));

    assert!(matches!(
      map_platform_error(platform_audio::AudioError::UnsupportedSampleFormat {
        details: "format".to_string(),
      }),
      AudioError::UnsupportedSampleFormat { .. }
    ));

    let mapped =
      map_platform_error(platform_audio::AudioError::StreamBuildFailed {
        details: "boom".to_string(),
      });
    assert!(matches!(mapped, AudioError::Platform { .. }));
    return;
  }

  /// Output writer adapter MUST forward calls to the platform writer.
  #[test]
  fn output_writer_adapter_forwards_calls() {
    #[derive(Default)]
    struct StubPlatformWriter {
      channels: u16,
      frames: usize,
      cleared: bool,
      last_sample: Option<(usize, usize, f32)>,
    }

    impl platform_audio::AudioOutputWriter for StubPlatformWriter {
      fn channels(&self) -> u16 {
        return self.channels;
      }

      fn frames(&self) -> usize {
        return self.frames;
      }

      fn clear(&mut self) {
        self.cleared = true;
        return;
      }

      fn set_sample(
        &mut self,
        frame_index: usize,
        channel_index: usize,
        sample: f32,
      ) {
        self.last_sample = Some((frame_index, channel_index, sample));
        return;
      }
    }

    let mut writer = StubPlatformWriter {
      channels: 2,
      frames: 3,
      ..Default::default()
    };

    {
      let mut adapter = OutputWriterAdapter {
        writer: &mut writer,
      };
      assert_eq!(adapter.channels(), 2);
      assert_eq!(adapter.frames(), 3);
      adapter.clear();
      adapter.set_sample(1, 0, 0.5);
    }

    assert!(writer.cleared);
    assert_eq!(writer.last_sample, Some((1, 0, 0.5)));
    return;
  }
}
