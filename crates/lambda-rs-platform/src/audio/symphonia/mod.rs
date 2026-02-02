#![allow(clippy::needless_return)]

//! `symphonia` dependency wrapper.
//!
//! This module will provide WAV and OGG Vorbis decode helpers for `lambda-rs`.
//! It is intentionally internal and MAY change between releases.

use std::{
  fmt,
  io::Cursor,
};

use symphonia::core::{
  errors::Error,
  formats::FormatOptions,
  io::MediaSourceStream,
  meta::MetadataOptions,
  probe::Hint,
};

/// Fully decoded, interleaved `f32` samples with associated metadata.
#[derive(Clone, Debug, PartialEq)]
pub struct DecodedAudio {
  pub samples: Vec<f32>,
  pub sample_rate: u32,
  pub channels: u16,
}

/// Vendor-free errors produced by audio decoding helpers.
#[derive(Clone, Debug)]
pub enum AudioDecodeError {
  UnsupportedFormat { details: String },
  InvalidData { details: String },
  DecodeFailed { details: String },
}

impl fmt::Display for AudioDecodeError {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::UnsupportedFormat { details } => {
        return write!(formatter, "unsupported audio format: {details}");
      }
      Self::InvalidData { details } => {
        return write!(formatter, "invalid audio data: {details}");
      }
      Self::DecodeFailed { details } => {
        return write!(formatter, "audio decode failed: {details}");
      }
    }
  }
}

impl std::error::Error for AudioDecodeError {}

fn hint_for_decode(extensions: &[&str]) -> Hint {
  let mut hint_value = Hint::new();
  for extension in extensions {
    hint_value.with_extension(extension);
  }
  return hint_value;
}

fn map_probe_error(source_description: &str, error: Error) -> AudioDecodeError {
  match error {
    Error::Unsupported(_) => {
      return AudioDecodeError::UnsupportedFormat {
        details: format!("unsupported or unrecognized {source_description}"),
      };
    }
    Error::IoError(_) => {
      return AudioDecodeError::InvalidData {
        details: format!("failed to read {source_description} bytes"),
      };
    }
    other => {
      return AudioDecodeError::DecodeFailed {
        details: format!("{source_description} probe error: {other}"),
      };
    }
  }
}

fn probe_bytes(
  bytes: &[u8],
  source_description: &str,
  extensions: &[&str],
) -> Result<(), AudioDecodeError> {
  let hint_value = hint_for_decode(extensions);

  let cursor = Cursor::new(bytes.to_vec());
  let media_source =
    MediaSourceStream::new(Box::new(cursor), Default::default());

  let probe_result = symphonia::default::get_probe()
    .format(
      &hint_value,
      media_source,
      &FormatOptions::default(),
      &MetadataOptions::default(),
    )
    .map_err(|error| map_probe_error(source_description, error))?;

  if probe_result.format.tracks().is_empty() {
    return Err(AudioDecodeError::InvalidData {
      details: "no audio tracks found".to_string(),
    });
  }

  return Ok(());
}

/// Decode WAV bytes into interleaved `f32` samples.
#[cfg(feature = "audio-decode-wav")]
pub fn decode_wav_bytes(
  bytes: &[u8],
) -> Result<DecodedAudio, AudioDecodeError> {
  probe_bytes(bytes, "WAV", &["wav"])?;
  return Err(AudioDecodeError::DecodeFailed {
    details: "WAV decoding not implemented yet".to_string(),
  });
}

/// Decode OGG Vorbis bytes into interleaved `f32` samples.
#[cfg(feature = "audio-decode-vorbis")]
pub fn decode_ogg_vorbis_bytes(
  bytes: &[u8],
) -> Result<DecodedAudio, AudioDecodeError> {
  probe_bytes(bytes, "OGG Vorbis", &["ogg", "oga"])?;
  return Err(AudioDecodeError::DecodeFailed {
    details: "OGG Vorbis decoding not implemented yet".to_string(),
  });
}
