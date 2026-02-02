#![allow(clippy::needless_return)]

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
