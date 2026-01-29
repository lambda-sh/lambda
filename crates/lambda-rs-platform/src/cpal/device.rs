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

  #[allow(dead_code)]
  pub fn sample_format(&self) -> AudioSampleFormat {
    return self.buffer.sample_format();
  }
}

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
  }

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
  }

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
  }
}
