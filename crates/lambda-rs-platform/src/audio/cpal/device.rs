#![allow(clippy::needless_return)]

//! Audio output device discovery and stream initialization (cpal backend).
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

use ::cpal as cpal_backend;
use cpal_backend::traits::{
  DeviceTrait,
  HostTrait,
  StreamTrait,
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

/// A typed view of an interleaved output buffer for a single callback.
///
/// This type is internal and exists to support backend callback adapters.
#[allow(dead_code)]
pub(crate) enum AudioOutputBuffer<'buffer> {
  /// Interleaved `f32` samples.
  F32(&'buffer mut [f32]),
  /// Interleaved `i16` samples.
  I16(&'buffer mut [i16]),
  /// Interleaved `u16` samples.
  U16(&'buffer mut [u16]),
}

impl<'buffer> AudioOutputBuffer<'buffer> {
  /// Return the number of interleaved samples in the underlying buffer.
  #[allow(dead_code)]
  fn len(&self) -> usize {
    match self {
      Self::F32(buffer) => {
        return buffer.len();
      }
      Self::I16(buffer) => {
        return buffer.len();
      }
      Self::U16(buffer) => {
        return buffer.len();
      }
    }
  }

  /// Return the sample format of the underlying typed buffer.
  fn sample_format(&self) -> AudioSampleFormat {
    match self {
      Self::F32(_) => {
        return AudioSampleFormat::F32;
      }
      Self::I16(_) => {
        return AudioSampleFormat::I16;
      }
      Self::U16(_) => {
        return AudioSampleFormat::U16;
      }
    }
  }
}

/// An [`AudioOutputWriter`] implementation for interleaved buffers.
///
/// This type is internal and exists to support backend callback adapters.
#[allow(dead_code)]
pub(crate) struct InterleavedAudioOutputWriter<'buffer> {
  channels: u16,
  frames: usize,
  buffer: AudioOutputBuffer<'buffer>,
}

impl<'buffer> InterleavedAudioOutputWriter<'buffer> {
  /// Create a writer for an interleaved output buffer.
  ///
  /// `channels` MUST match the channel count encoded in the output stream
  /// configuration. The frame count is derived from the buffer length and
  /// channel count.
  ///
  /// # Arguments
  /// - `channels`: Interleaved output channel count.
  /// - `buffer`: A typed interleaved output buffer view for the current audio
  ///   callback.
  ///
  /// # Returns
  /// A writer that can clear and write normalized samples into `buffer`.
  #[allow(dead_code)]
  pub fn new(channels: u16, buffer: AudioOutputBuffer<'buffer>) -> Self {
    let channels_usize = channels as usize;
    let frames = if channels_usize == 0 {
      0
    } else {
      buffer.len() / channels_usize
    };

    return Self {
      channels,
      frames,
      buffer,
    };
  }

  /// Return the sample format of the current callback buffer.
  ///
  /// # Returns
  /// The typed sample format for the current callback buffer.
  #[allow(dead_code)]
  pub fn sample_format(&self) -> AudioSampleFormat {
    return self.buffer.sample_format();
  }
}

/// Clamp a normalized audio sample to the nominal output range `[-1.0, 1.0]`.
///
/// # Arguments
/// - `sample`: A potentially out-of-range normalized sample.
///
/// # Returns
/// The clamped sample in range `[-1.0, 1.0]`.
#[allow(dead_code)]
fn clamp_normalized_sample(sample: f32) -> f32 {
  if sample > 1.0 {
    return 1.0;
  }

  if sample < -1.0 {
    return -1.0;
  }

  return sample;
}

impl<'buffer> AudioOutputWriter for InterleavedAudioOutputWriter<'buffer> {
  fn channels(&self) -> u16 {
    return self.channels;
  }

  fn frames(&self) -> usize {
    return self.frames;
  }

  fn clear(&mut self) {
    match &mut self.buffer {
      AudioOutputBuffer::F32(buffer) => {
        buffer.fill(0.0);
        return;
      }
      AudioOutputBuffer::I16(buffer) => {
        buffer.fill(0);
        return;
      }
      AudioOutputBuffer::U16(buffer) => {
        buffer.fill(32768);
        return;
      }
    }
  }

  fn set_sample(
    &mut self,
    frame_index: usize,
    channel_index: usize,
    sample: f32,
  ) {
    let channels = self.channels as usize;
    if channels == 0 {
      return;
    }

    if channel_index >= channels {
      if cfg!(all(debug_assertions, not(test))) {
        eprintln!(
          "audio: set_sample channel_index out of range (channel_index={channel_index} channels={channels})"
        );
      }
      return;
    }

    if frame_index >= self.frames {
      if cfg!(all(debug_assertions, not(test))) {
        eprintln!(
          "audio: set_sample frame_index out of range (frame_index={frame_index} frames={})",
          self.frames
        );
      }
      return;
    }

    let sample_index = frame_index * channels + channel_index;
    if sample_index >= self.buffer.len() {
      if cfg!(all(debug_assertions, not(test))) {
        eprintln!(
          "audio: set_sample buffer index out of range (sample_index={sample_index} len={})",
          self.buffer.len()
        );
      }
      return;
    }

    let sample = clamp_normalized_sample(sample);

    match &mut self.buffer {
      AudioOutputBuffer::F32(buffer) => {
        buffer[sample_index] = sample;
        return;
      }
      AudioOutputBuffer::I16(buffer) => {
        let scaled = (sample * 32767.0).round();
        buffer[sample_index] = scaled as i16;
        return;
      }
      AudioOutputBuffer::U16(buffer) => {
        let scaled = ((sample + 1.0) * 0.5 * 65535.0).round();
        buffer[sample_index] = scaled as u16;
        return;
      }
    }
  }
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
  _stream: cpal_backend::Stream,
  #[allow(dead_code)]
  sample_rate: u32,
  #[allow(dead_code)]
  channels: u16,
  #[allow(dead_code)]
  sample_format: AudioSampleFormat,
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
  /// This method selects a supported output configuration from the default
  /// output device and starts an output stream that plays silence.
  ///
  /// # Returns
  /// An initialized audio output device handle. Dropping the handle stops
  /// output.
  ///
  /// # Errors
  /// Returns an error when the host, device, or stream cannot be initialized,
  /// or when no supported output configuration satisfies the request.
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

    let host = cpal_backend::default_host();

    let device = host
      .default_output_device()
      .ok_or(AudioError::NoDefaultDevice)?;

    let supported_configs =
      device.supported_output_configs().map_err(|error| {
        AudioError::SupportedConfigsUnavailable {
          details: error.to_string(),
        }
      })?;

    let supported_configs: Vec<cpal_backend::SupportedStreamConfigRange> =
      supported_configs.collect();

    let selected_config = select_output_stream_config(
      &supported_configs,
      self.sample_rate,
      self.channels,
    )?;

    let stream_config = selected_config.config();
    let sample_rate = stream_config.sample_rate;
    let channels = stream_config.channels;

    let sample_format = match selected_config.sample_format() {
      cpal_backend::SampleFormat::F32 => AudioSampleFormat::F32,
      cpal_backend::SampleFormat::I16 => AudioSampleFormat::I16,
      cpal_backend::SampleFormat::U16 => AudioSampleFormat::U16,
      other => {
        return Err(AudioError::UnsupportedSampleFormat {
          details: format!("{other:?}"),
        });
      }
    };

    let stream = match selected_config.sample_format() {
      cpal_backend::SampleFormat::F32 => device
        .build_output_stream(
          &stream_config,
          |data: &mut [f32], _info| {
            data.fill(0.0);
            return;
          },
          |_error| {
            return;
          },
          None,
        )
        .map_err(|error| AudioError::StreamBuildFailed {
          details: error.to_string(),
        })?,
      cpal_backend::SampleFormat::I16 => device
        .build_output_stream(
          &stream_config,
          |data: &mut [i16], _info| {
            data.fill(0);
            return;
          },
          |_error| {
            return;
          },
          None,
        )
        .map_err(|error| AudioError::StreamBuildFailed {
          details: error.to_string(),
        })?,
      cpal_backend::SampleFormat::U16 => device
        .build_output_stream(
          &stream_config,
          |data: &mut [u16], _info| {
            data.fill(32768);
            return;
          },
          |_error| {
            return;
          },
          None,
        )
        .map_err(|error| AudioError::StreamBuildFailed {
          details: error.to_string(),
        })?,
      other => {
        return Err(AudioError::UnsupportedSampleFormat {
          details: format!("{other:?}"),
        });
      }
    };

    stream
      .play()
      .map_err(|error| AudioError::StreamPlayFailed {
        details: error.to_string(),
      })?;

    return Ok(AudioDevice {
      _stream: stream,
      sample_rate,
      channels,
      sample_format,
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
  /// Returns an error when the host, device, or stream cannot be initialized,
  /// or when no supported output configuration satisfies the request.
  pub fn build_with_output_callback<Callback>(
    self,
    callback: Callback,
  ) -> Result<AudioDevice, AudioError>
  where
    Callback:
      'static + Send + FnMut(&mut dyn AudioOutputWriter, AudioCallbackInfo),
  {
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

    let host = cpal_backend::default_host();

    let device = host
      .default_output_device()
      .ok_or(AudioError::NoDefaultDevice)?;

    let supported_configs =
      device.supported_output_configs().map_err(|error| {
        AudioError::SupportedConfigsUnavailable {
          details: error.to_string(),
        }
      })?;

    let supported_configs: Vec<cpal_backend::SupportedStreamConfigRange> =
      supported_configs.collect();

    let selected_config = select_output_stream_config(
      &supported_configs,
      self.sample_rate,
      self.channels,
    )?;

    let stream_config = selected_config.config();
    let sample_rate = stream_config.sample_rate;
    let channels = stream_config.channels;

    let sample_format = match selected_config.sample_format() {
      cpal_backend::SampleFormat::F32 => AudioSampleFormat::F32,
      cpal_backend::SampleFormat::I16 => AudioSampleFormat::I16,
      cpal_backend::SampleFormat::U16 => AudioSampleFormat::U16,
      other => {
        return Err(AudioError::UnsupportedSampleFormat {
          details: format!("{other:?}"),
        });
      }
    };

    let callback_info = AudioCallbackInfo {
      sample_rate,
      channels,
      sample_format,
    };

    let mut callback = callback;

    let stream = match selected_config.sample_format() {
      cpal_backend::SampleFormat::F32 => device
        .build_output_stream(
          &stream_config,
          move |data: &mut [f32], _info| {
            invoke_output_callback_on_buffer(
              channels,
              AudioOutputBuffer::F32(data),
              callback_info,
              &mut callback,
            );
            return;
          },
          |_error| {
            return;
          },
          None,
        )
        .map_err(|error| AudioError::StreamBuildFailed {
          details: error.to_string(),
        })?,
      cpal_backend::SampleFormat::I16 => device
        .build_output_stream(
          &stream_config,
          move |data: &mut [i16], _info| {
            invoke_output_callback_on_buffer(
              channels,
              AudioOutputBuffer::I16(data),
              callback_info,
              &mut callback,
            );
            return;
          },
          |_error| {
            return;
          },
          None,
        )
        .map_err(|error| AudioError::StreamBuildFailed {
          details: error.to_string(),
        })?,
      cpal_backend::SampleFormat::U16 => device
        .build_output_stream(
          &stream_config,
          move |data: &mut [u16], _info| {
            invoke_output_callback_on_buffer(
              channels,
              AudioOutputBuffer::U16(data),
              callback_info,
              &mut callback,
            );
            return;
          },
          |_error| {
            return;
          },
          None,
        )
        .map_err(|error| AudioError::StreamBuildFailed {
          details: error.to_string(),
        })?,
      other => {
        return Err(AudioError::UnsupportedSampleFormat {
          details: format!("{other:?}"),
        });
      }
    };

    stream
      .play()
      .map_err(|error| AudioError::StreamPlayFailed {
        details: error.to_string(),
      })?;

    return Ok(AudioDevice {
      _stream: stream,
      sample_rate,
      channels,
      sample_format,
    });
  }
}

impl Default for AudioDeviceBuilder {
  fn default() -> Self {
    return Self::new();
  }
}

/// Invoke an output callback using a typed platform buffer.
///
/// This adapter:
/// - Wraps the typed `cpal` output slice in an [`AudioOutputWriter`].
/// - Clears the buffer to silence before invoking the callback.
/// - Guarantees a single callback invocation per platform callback tick.
///
/// # Arguments
/// - `channels`: Interleaved channel count for the output buffer.
/// - `buffer`: A typed interleaved output buffer view.
/// - `callback_info`: Stream metadata for the current callback invocation.
/// - `callback`: The engine callback to invoke.
///
/// # Returns
/// `()` after clearing the buffer and invoking the callback.
fn invoke_output_callback_on_buffer<Callback>(
  channels: u16,
  buffer: AudioOutputBuffer<'_>,
  callback_info: AudioCallbackInfo,
  callback: &mut Callback,
) where
  Callback: FnMut(&mut dyn AudioOutputWriter, AudioCallbackInfo),
{
  let mut writer = InterleavedAudioOutputWriter::new(channels, buffer);
  writer.clear();
  callback(&mut writer, callback_info);
  return;
}

/// Convert a `cpal` sample format into a stable preference ordering.
///
/// The current backend prefers `f32`, then `i16`, then `u16`. Any other format
/// is treated as unsupported by this abstraction.
///
/// # Arguments
/// - `sample_format`: The `cpal` sample format for a supported stream config.
///
/// # Returns
/// A priority value where higher values are preferred.
fn sample_format_priority(sample_format: cpal_backend::SampleFormat) -> u8 {
  match sample_format {
    cpal_backend::SampleFormat::F32 => {
      return 3;
    }
    cpal_backend::SampleFormat::I16 => {
      return 2;
    }
    cpal_backend::SampleFormat::U16 => {
      return 1;
    }
    _ => {
      return 0;
    }
  }
}

/// Select a supported output stream configuration for the default device.
///
/// Selection rules:
/// - If `requested_channels` is set, only exact channel matches are considered.
/// - If `requested_sample_rate` is set, only ranges that contain the rate are
///   considered.
/// - If no rate is requested, the selection targets `48_000` Hz and chooses the
///   closest available rate within each range.
/// - Higher-quality sample formats are preferred (`f32` > `i16` > `u16`).
/// - When priorities tie, the configuration with sample rate closest to
///   `48_000` Hz is preferred.
///
/// # Arguments
/// - `supported_configs`: The device-provided supported config ranges.
/// - `requested_sample_rate`: Optional exact sample rate request.
/// - `requested_channels`: Optional exact channel count request.
///
/// # Returns
/// A supported stream configuration selected from `supported_configs`.
///
/// # Errors
/// Returns [`AudioError::UnsupportedConfig`] when no configuration satisfies
/// the request. Returns [`AudioError::UnsupportedSampleFormat`] when the device
/// does not expose any supported sample format among `f32`, `i16`, and `u16`.
fn select_output_stream_config(
  supported_configs: &[cpal_backend::SupportedStreamConfigRange],
  requested_sample_rate: Option<u32>,
  requested_channels: Option<u16>,
) -> Result<cpal_backend::SupportedStreamConfig, AudioError> {
  let mut best_config: Option<cpal_backend::SupportedStreamConfig> = None;
  let mut best_priority = 0u8;
  let mut best_sample_rate_distance = u32::MAX;

  for range in supported_configs.iter().copied() {
    if let Some(channels) = requested_channels {
      if range.channels() != channels {
        continue;
      }
    }

    if sample_format_priority(range.sample_format()) == 0 {
      continue;
    }

    let min_sample_rate = range.min_sample_rate();
    let max_sample_rate = range.max_sample_rate();

    let sample_rate = if let Some(requested_sample_rate) = requested_sample_rate
    {
      if requested_sample_rate < min_sample_rate
        || max_sample_rate < requested_sample_rate
      {
        continue;
      }

      requested_sample_rate
    } else {
      let target_sample_rate = 48_000;
      if target_sample_rate < min_sample_rate {
        min_sample_rate
      } else if max_sample_rate < target_sample_rate {
        max_sample_rate
      } else {
        target_sample_rate
      }
    };

    let config = match range.try_with_sample_rate(sample_rate) {
      Some(config) => config,
      None => {
        continue;
      }
    };

    let priority = sample_format_priority(config.sample_format());
    let sample_rate_distance = if config.sample_rate() < 48_000 {
      48_000 - config.sample_rate()
    } else {
      config.sample_rate() - 48_000
    };

    if priority < best_priority {
      continue;
    }

    if priority == best_priority
      && best_config.is_some()
      && best_sample_rate_distance < sample_rate_distance
    {
      continue;
    }

    best_priority = priority;
    best_sample_rate_distance = sample_rate_distance;
    best_config = Some(config);
  }

  if let Some(config) = best_config {
    return Ok(config);
  }

  if supported_configs
    .iter()
    .all(|config| sample_format_priority(config.sample_format()) == 0)
  {
    return Err(AudioError::UnsupportedSampleFormat {
      details: "no supported sample format among f32/i16/u16".to_string(),
    });
  }

  return Err(AudioError::UnsupportedConfig {
    requested_sample_rate,
    requested_channels,
  });
}

/// Enumerate available audio output devices.
///
/// # Returns
/// A list of available output devices with stable metadata.
///
/// # Errors
/// Returns an error if the platform host cannot enumerate output devices or if
/// device metadata cannot be retrieved.
pub fn enumerate_devices() -> Result<Vec<AudioDeviceInfo>, AudioError> {
  let host = cpal_backend::default_host();

  let default_device_id = host
    .default_output_device()
    .and_then(|device| device.id().ok());

  let devices = host.output_devices().map_err(|error| {
    return AudioError::DeviceEnumerationFailed {
      details: error.to_string(),
    };
  })?;

  let mut output_devices = Vec::new();
  for device in devices {
    let name = device
      .description()
      .map(|description| description.name().to_string())
      .map_err(|error| {
        return AudioError::DeviceNameUnavailable {
          details: error.to_string(),
        };
      })?;

    let is_default = default_device_id
      .as_ref()
      .is_some_and(|default_id| device.id().ok().as_ref() == Some(default_id));

    output_devices.push(AudioDeviceInfo { name, is_default });
  }

  return Ok(output_devices);
}

#[cfg(test)]
mod tests {
  use cpal_backend::SupportedBufferSize;

  use super::*;

  /// Normalized sample clamping MUST clamp to the nominal output range.
  #[test]
  fn clamp_normalized_sample_clamps_to_nominal_range() {
    assert_eq!(clamp_normalized_sample(2.0), 1.0);
    assert_eq!(clamp_normalized_sample(-2.0), -1.0);
    assert_eq!(clamp_normalized_sample(0.25), 0.25);
    return;
  }

  /// Sample format priority MUST prefer `f32`, then `i16`, then `u16`.
  #[test]
  fn sample_format_priority_orders_supported_formats() {
    assert!(
      sample_format_priority(cpal_backend::SampleFormat::F32)
        > sample_format_priority(cpal_backend::SampleFormat::I16)
    );
    assert!(
      sample_format_priority(cpal_backend::SampleFormat::I16)
        > sample_format_priority(cpal_backend::SampleFormat::U16)
    );
    assert_eq!(sample_format_priority(cpal_backend::SampleFormat::I8), 0);
    return;
  }

  /// Platform audio error display MUST cover each error variant.
  #[test]
  fn platform_audio_error_display_covers_each_variant() {
    let cases = [
      AudioError::InvalidSampleRate { requested: 0 }.to_string(),
      AudioError::InvalidChannels { requested: 0 }.to_string(),
      AudioError::HostUnavailable {
        details: "missing".to_string(),
      }
      .to_string(),
      AudioError::NoDefaultDevice.to_string(),
      AudioError::DeviceNameUnavailable {
        details: "name".to_string(),
      }
      .to_string(),
      AudioError::DeviceEnumerationFailed {
        details: "enum".to_string(),
      }
      .to_string(),
      AudioError::SupportedConfigsUnavailable {
        details: "configs".to_string(),
      }
      .to_string(),
      AudioError::UnsupportedConfig {
        requested_sample_rate: Some(44_100),
        requested_channels: Some(2),
      }
      .to_string(),
      AudioError::UnsupportedSampleFormat {
        details: "fmt".to_string(),
      }
      .to_string(),
      AudioError::Platform {
        details: "backend".to_string(),
      }
      .to_string(),
      AudioError::StreamBuildFailed {
        details: "build".to_string(),
      }
      .to_string(),
      AudioError::StreamPlayFailed {
        details: "play".to_string(),
      }
      .to_string(),
    ];

    assert!(cases.iter().all(|value| !value.trim().is_empty()));
    return;
  }

  /// Output buffer helpers MUST report length and format.
  #[test]
  fn output_buffer_reports_length_and_format() {
    let mut samples_f32 = [0.0f32, 0.0];
    let buffer = AudioOutputBuffer::F32(&mut samples_f32);
    assert_eq!(buffer.len(), 2);
    assert_eq!(buffer.sample_format(), AudioSampleFormat::F32);

    let mut samples_i16 = [0i16, 0];
    let buffer = AudioOutputBuffer::I16(&mut samples_i16);
    assert_eq!(buffer.len(), 2);
    assert_eq!(buffer.sample_format(), AudioSampleFormat::I16);

    let mut samples_u16 = [0u16, 0];
    let buffer = AudioOutputBuffer::U16(&mut samples_u16);
    assert_eq!(buffer.len(), 2);
    assert_eq!(buffer.sample_format(), AudioSampleFormat::U16);
    return;
  }

  /// Writer construction MUST handle invalid channel counts without panicking.
  #[test]
  fn writer_new_handles_zero_channels() {
    let mut samples_f32 = [0.0f32, 0.0];
    let writer = InterleavedAudioOutputWriter::new(
      0,
      AudioOutputBuffer::F32(&mut samples_f32),
    );
    assert_eq!(writer.frames(), 0);
    assert_eq!(writer.channels(), 0);
    assert_eq!(writer.sample_format(), AudioSampleFormat::F32);
    return;
  }

  /// Config selection MUST reject devices that only expose unsupported formats.
  #[test]
  fn select_output_stream_config_rejects_all_unsupported_formats() {
    let supported_configs = [cpal_backend::SupportedStreamConfigRange::new(
      2,
      44_100,
      48_000,
      SupportedBufferSize::Unknown,
      cpal_backend::SampleFormat::I8,
    )];

    let result = select_output_stream_config(&supported_configs, None, None);
    assert!(matches!(
      result,
      Err(AudioError::UnsupportedSampleFormat { .. })
    ));
    return;
  }

  /// Config selection MUST pick the sample rate closest to 48_000 when
  /// priorities tie.
  #[test]
  fn select_output_stream_config_prefers_closest_sample_rate_when_tied() {
    let supported_configs = [
      cpal_backend::SupportedStreamConfigRange::new(
        2,
        44_100,
        44_100,
        SupportedBufferSize::Unknown,
        cpal_backend::SampleFormat::F32,
      ),
      cpal_backend::SupportedStreamConfigRange::new(
        2,
        48_000,
        48_000,
        SupportedBufferSize::Unknown,
        cpal_backend::SampleFormat::F32,
      ),
    ];

    let selected =
      select_output_stream_config(&supported_configs, None, None).unwrap();
    assert_eq!(selected.sample_rate(), 48_000);
    return;
  }

  /// Builder MUST reject invalid sample rates.
  #[test]
  fn build_rejects_zero_sample_rate() {
    let result = AudioDeviceBuilder::new().with_sample_rate(0).build();
    assert!(matches!(
      result,
      Err(AudioError::InvalidSampleRate { requested: 0 })
    ));
    return;
  }

  /// Builder MUST reject invalid channel counts.
  #[test]
  fn build_rejects_zero_channels() {
    let result = AudioDeviceBuilder::new().with_channels(0).build();
    assert!(matches!(
      result,
      Err(AudioError::InvalidChannels { requested: 0 })
    ));
    return;
  }

  /// Builder MUST NOT panic for typical host/device states.
  #[test]
  fn build_does_not_panic() {
    let result = AudioDeviceBuilder::new().build();
    assert!(!matches!(
      result,
      Err(AudioError::InvalidSampleRate { .. })
        | Err(AudioError::InvalidChannels { .. })
    ));
    return;
  }

  /// Device enumeration MUST NOT panic for typical host/device states.
  #[test]
  fn enumerate_devices_does_not_panic() {
    let _result = enumerate_devices();
    return;
  }

  /// Callback-based builder MUST NOT panic for typical host/device states.
  #[test]
  fn build_with_output_callback_does_not_panic() {
    let result = AudioDeviceBuilder::new().build_with_output_callback(
      |_writer, _callback_info| {
        return;
      },
    );
    assert!(!matches!(
      result,
      Err(AudioError::InvalidSampleRate { .. })
        | Err(AudioError::InvalidChannels { .. })
    ));
    return;
  }

  /// Callback adapter MUST clear and then invoke the callback for `f32`.
  #[test]
  fn invoke_output_callback_on_buffer_clears_and_invokes_callback_f32() {
    let mut buffer_f32 = [1.0, -1.0, 0.5, -0.5];
    let callback_info = AudioCallbackInfo {
      sample_rate: 48_000,
      channels: 2,
      sample_format: AudioSampleFormat::F32,
    };

    let mut callback_called = false;
    let mut callback = |writer: &mut dyn AudioOutputWriter,
                        info: AudioCallbackInfo| {
      callback_called = true;
      assert_eq!(info, callback_info);
      assert_eq!(writer.channels(), 2);
      assert_eq!(writer.frames(), 2);
      writer.set_sample(0, 0, 0.5);
      return;
    };

    invoke_output_callback_on_buffer(
      2,
      AudioOutputBuffer::F32(&mut buffer_f32),
      callback_info,
      &mut callback,
    );

    assert!(callback_called);
    assert_eq!(buffer_f32, [0.5, 0.0, 0.0, 0.0]);
    return;
  }

  /// Callback adapter MUST clear and then invoke the callback for `i16`.
  #[test]
  fn invoke_output_callback_on_buffer_clears_and_invokes_callback_i16() {
    let mut buffer_i16 = [1, -1, 200, -200];
    let callback_info = AudioCallbackInfo {
      sample_rate: 48_000,
      channels: 2,
      sample_format: AudioSampleFormat::I16,
    };

    let mut callback_called = false;
    let mut callback = |writer: &mut dyn AudioOutputWriter,
                        info: AudioCallbackInfo| {
      callback_called = true;
      assert_eq!(info, callback_info);
      writer.set_sample(0, 0, 1.0);
      return;
    };

    invoke_output_callback_on_buffer(
      2,
      AudioOutputBuffer::I16(&mut buffer_i16),
      callback_info,
      &mut callback,
    );

    assert!(callback_called);
    assert_eq!(buffer_i16, [32767, 0, 0, 0]);
    return;
  }

  /// Callback adapter MUST clear and then invoke the callback for `u16`.
  #[test]
  fn invoke_output_callback_on_buffer_clears_and_invokes_callback_u16() {
    let mut buffer_u16 = [0, 1, 65535, 12345];
    let callback_info = AudioCallbackInfo {
      sample_rate: 48_000,
      channels: 2,
      sample_format: AudioSampleFormat::U16,
    };

    let mut callback_called = false;
    let mut callback = |writer: &mut dyn AudioOutputWriter,
                        info: AudioCallbackInfo| {
      callback_called = true;
      assert_eq!(info, callback_info);
      writer.set_sample(0, 0, -1.0);
      return;
    };

    invoke_output_callback_on_buffer(
      2,
      AudioOutputBuffer::U16(&mut buffer_u16),
      callback_info,
      &mut callback,
    );

    assert!(callback_called);
    assert_eq!(buffer_u16, [0, 32768, 32768, 32768]);
    return;
  }

  /// Writer silence MUST match each sample format's conventions.
  #[test]
  fn writer_clear_sets_silence_for_all_formats() {
    let mut buffer_f32 = [1.0, -1.0, 0.5, -0.5];
    let mut writer = InterleavedAudioOutputWriter::new(
      2,
      AudioOutputBuffer::F32(&mut buffer_f32),
    );
    writer.clear();
    assert_eq!(buffer_f32, [0.0, 0.0, 0.0, 0.0]);

    let mut buffer_i16 = [1, -1, 200, -200];
    let mut writer = InterleavedAudioOutputWriter::new(
      2,
      AudioOutputBuffer::I16(&mut buffer_i16),
    );
    writer.clear();
    assert_eq!(buffer_i16, [0, 0, 0, 0]);

    let mut buffer_u16 = [0, 1, 65535, 12345];
    let mut writer = InterleavedAudioOutputWriter::new(
      2,
      AudioOutputBuffer::U16(&mut buffer_u16),
    );
    writer.clear();
    assert_eq!(buffer_u16, [32768, 32768, 32768, 32768]);
    return;
  }

  /// Writer MUST clamp normalized samples and convert to output formats.
  #[test]
  fn writer_set_sample_clamps_and_converts() {
    let mut buffer_f32 = [0.0, 0.0, 0.0, 0.0];
    let mut writer = InterleavedAudioOutputWriter::new(
      2,
      AudioOutputBuffer::F32(&mut buffer_f32),
    );
    writer.set_sample(0, 0, 2.0);
    writer.set_sample(0, 1, -2.0);
    assert_eq!(buffer_f32[0], 1.0);
    assert_eq!(buffer_f32[1], -1.0);

    let mut buffer_i16 = [0, 0, 0, 0];
    let mut writer = InterleavedAudioOutputWriter::new(
      2,
      AudioOutputBuffer::I16(&mut buffer_i16),
    );
    writer.set_sample(0, 0, 1.0);
    writer.set_sample(0, 1, -1.0);
    writer.set_sample(1, 0, 0.0);
    assert_eq!(buffer_i16[0], 32767);
    assert_eq!(buffer_i16[1], -32767);
    assert_eq!(buffer_i16[2], 0);

    let mut buffer_u16 = [0, 0, 0, 0];
    let mut writer = InterleavedAudioOutputWriter::new(
      2,
      AudioOutputBuffer::U16(&mut buffer_u16),
    );
    writer.set_sample(0, 0, -1.0);
    writer.set_sample(0, 1, 0.0);
    writer.set_sample(1, 0, 1.0);
    assert_eq!(buffer_u16[0], 0);
    assert_eq!(buffer_u16[1], 32768);
    assert_eq!(buffer_u16[2], 65535);
    return;
  }

  /// Out-of-range indices MUST be treated as no-ops.
  #[test]
  fn writer_set_sample_is_noop_for_out_of_range_indices() {
    let mut buffer_f32 = [0.25, 0.25, 0.25, 0.25];
    let mut writer = InterleavedAudioOutputWriter::new(
      2,
      AudioOutputBuffer::F32(&mut buffer_f32),
    );

    writer.set_sample(10, 0, 1.0);
    writer.set_sample(0, 10, 1.0);

    assert_eq!(buffer_f32, [0.25, 0.25, 0.25, 0.25]);
    return;
  }

  /// Config selection MUST prefer `f32` when available.
  #[test]
  fn select_output_stream_config_prefers_f32_when_available() {
    let supported_configs = [
      cpal_backend::SupportedStreamConfigRange::new(
        2,
        44_100,
        48_000,
        SupportedBufferSize::Unknown,
        cpal_backend::SampleFormat::I16,
      ),
      cpal_backend::SupportedStreamConfigRange::new(
        2,
        44_100,
        48_000,
        SupportedBufferSize::Unknown,
        cpal_backend::SampleFormat::F32,
      ),
    ];

    let selected =
      select_output_stream_config(&supported_configs, None, None).unwrap();
    assert_eq!(selected.sample_format(), cpal_backend::SampleFormat::F32);
    assert_eq!(selected.sample_rate(), 48_000);
    return;
  }

  /// Config selection MUST honor exact requested channel counts.
  #[test]
  fn select_output_stream_config_respects_requested_channels() {
    let supported_configs = [cpal_backend::SupportedStreamConfigRange::new(
      2,
      44_100,
      48_000,
      SupportedBufferSize::Unknown,
      cpal_backend::SampleFormat::F32,
    )];

    let selected =
      select_output_stream_config(&supported_configs, None, Some(2)).unwrap();
    assert_eq!(selected.channels(), 2);

    let result = select_output_stream_config(&supported_configs, None, Some(1));
    assert!(matches!(
      result,
      Err(AudioError::UnsupportedConfig {
        requested_sample_rate: None,
        requested_channels: Some(1),
      })
    ));
    return;
  }

  /// Config selection MUST honor requested sample rates when available.
  #[test]
  fn select_output_stream_config_respects_requested_sample_rate() {
    let supported_configs = [cpal_backend::SupportedStreamConfigRange::new(
      2,
      44_100,
      48_000,
      SupportedBufferSize::Unknown,
      cpal_backend::SampleFormat::F32,
    )];

    let selected =
      select_output_stream_config(&supported_configs, Some(44_100), None)
        .unwrap();
    assert_eq!(selected.sample_rate(), 44_100);

    let result =
      select_output_stream_config(&supported_configs, Some(10), None);
    assert!(matches!(
      result,
      Err(AudioError::UnsupportedConfig {
        requested_sample_rate: Some(10),
        requested_channels: None,
      })
    ));
    return;
  }
}
