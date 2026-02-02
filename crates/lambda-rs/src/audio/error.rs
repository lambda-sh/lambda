#![allow(clippy::needless_return)]

use std::fmt;

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
  /// An error occurred while reading audio bytes from disk.
  Io {
    path: Option<std::path::PathBuf>,
    details: String,
  },
  /// The input format or codec is unsupported by the configured features.
  UnsupportedFormat { details: String },
  /// The input bytes were invalid or corrupted.
  InvalidData { details: String },
  /// An unrecoverable decoding failure occurred.
  DecodeFailed { details: String },
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

impl fmt::Display for AudioError {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::InvalidSampleRate { requested } => {
        return write!(formatter, "invalid sample rate: {requested}");
      }
      Self::InvalidChannels { requested } => {
        return write!(formatter, "invalid channel count: {requested}");
      }
      Self::Io { path, details } => {
        if let Some(path) = path {
          return write!(
            formatter,
            "I/O error reading {}: {details}",
            path.display()
          );
        }
        return write!(formatter, "I/O error reading audio: {details}");
      }
      Self::UnsupportedFormat { details } => {
        return write!(formatter, "unsupported audio format: {details}");
      }
      Self::InvalidData { details } => {
        return write!(formatter, "invalid audio data: {details}");
      }
      Self::DecodeFailed { details } => {
        return write!(formatter, "audio decode failed: {details}");
      }
      Self::NoDefaultDevice => {
        return write!(
          formatter,
          "no default audio output device is available"
        );
      }
      Self::UnsupportedConfig {
        requested_sample_rate,
        requested_channels,
      } => {
        return write!(
          formatter,
          "unsupported audio output configuration (sample_rate={requested_sample_rate:?}, channels={requested_channels:?})"
        );
      }
      Self::UnsupportedSampleFormat { details } => {
        return write!(
          formatter,
          "unsupported audio output sample format: {details}"
        );
      }
      Self::Platform { details } => {
        return write!(formatter, "platform audio error: {details}");
      }
    }
  }
}

impl std::error::Error for AudioError {}
