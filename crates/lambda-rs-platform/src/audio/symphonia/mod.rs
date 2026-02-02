#![allow(clippy::needless_return)]

//! `symphonia` dependency wrapper.
//!
//! This module will provide WAV and OGG Vorbis decode helpers for `lambda-rs`.
//! It is intentionally internal and MAY change between releases.

use std::{
  fmt,
  io::Cursor,
};

#[cfg(feature = "audio-decode-vorbis")]
use symphonia::core::codecs::CODEC_TYPE_VORBIS;
#[cfg(feature = "audio-decode-wav")]
use symphonia::core::sample::SampleFormat;
use symphonia::core::{
  audio::SampleBuffer,
  codecs::{
    Decoder,
    DecoderOptions,
  },
  errors::Error,
  formats::{
    FormatOptions,
    FormatReader,
  },
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

fn map_read_or_decode_error(
  source_description: &str,
  error: Error,
) -> AudioDecodeError {
  match error {
    Error::Unsupported(_) => {
      return AudioDecodeError::UnsupportedFormat {
        details: format!("unsupported {source_description} audio codec"),
      };
    }
    Error::DecodeError(_) => {
      return AudioDecodeError::InvalidData {
        details: format!("{source_description} decode error: {error}"),
      };
    }
    Error::IoError(_) => {
      return AudioDecodeError::InvalidData {
        details: format!("{source_description} read error: {error}"),
      };
    }
    other => {
      return AudioDecodeError::DecodeFailed {
        details: format!("{source_description} decode failed: {other}"),
      };
    }
  }
}

fn probe_format(
  bytes: &[u8],
  source_description: &str,
  extensions: &[&str],
) -> Result<Box<dyn FormatReader>, AudioDecodeError> {
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

  return Ok(probe_result.format);
}

fn try_reserve_samples(
  samples: &mut Vec<f32>,
  source_description: &str,
  frames: Option<u64>,
  channels: Option<u16>,
) -> Result<(), AudioDecodeError> {
  let (frames, channels) = match (frames, channels) {
    (Some(frames), Some(channels)) => (frames, channels),
    _ => {
      return Ok(());
    }
  };

  let total_samples = frames.saturating_mul(channels as u64);
  if total_samples > usize::MAX as u64 {
    return Ok(());
  }

  samples.try_reserve(total_samples as usize).map_err(|_| {
    return AudioDecodeError::DecodeFailed {
      details: format!("failed to allocate {source_description} sample buffer"),
    };
  })?;
  return Ok(());
}

fn decode_track_to_interleaved_f32(
  format: &mut dyn FormatReader,
  track_id: u32,
  decoder: &mut dyn Decoder,
  source_description: &str,
  reserve_frames: Option<u64>,
  reserve_channels: Option<u16>,
) -> Result<DecodedAudio, AudioDecodeError> {
  let mut samples: Vec<f32> = Vec::new();
  try_reserve_samples(
    &mut samples,
    source_description,
    reserve_frames,
    reserve_channels,
  )?;

  let mut sample_rate: Option<u32> = None;
  let mut channel_count: Option<u16> = None;

  loop {
    let packet = match format.next_packet() {
      Ok(packet) => packet,
      Err(Error::IoError(error))
        if error.kind() == std::io::ErrorKind::UnexpectedEof =>
      {
        break;
      }
      Err(error) => {
        return Err(map_read_or_decode_error(source_description, error));
      }
    };

    if packet.track_id() != track_id {
      continue;
    }

    let decoded = match decoder.decode(&packet) {
      Ok(decoded) => decoded,
      Err(Error::ResetRequired) => {
        decoder.reset();
        continue;
      }
      Err(error) => {
        return Err(map_read_or_decode_error(source_description, error));
      }
    };

    let rate = decoded.spec().rate;
    if rate == 0 {
      return Err(AudioDecodeError::InvalidData {
        details: format!("{source_description} decoded sample rate was 0"),
      });
    }

    let channels = decoded.spec().channels.count() as u16;
    if channels == 0 {
      return Err(AudioDecodeError::InvalidData {
        details: format!("{source_description} decoded channel count was 0"),
      });
    }

    if channels != 1 && channels != 2 {
      return Err(AudioDecodeError::UnsupportedFormat {
        details: format!(
          "unsupported {source_description} channel count: {channels}"
        ),
      });
    }

    if let Some(previous_rate) = sample_rate {
      if previous_rate != rate {
        return Err(AudioDecodeError::InvalidData {
          details: format!(
            "{source_description} sample rate changed during decoding"
          ),
        });
      }
    } else {
      sample_rate = Some(rate);
    }

    if let Some(previous_channels) = channel_count {
      if previous_channels != channels {
        return Err(AudioDecodeError::InvalidData {
          details: format!(
            "{source_description} channel count changed during decoding"
          ),
        });
      }
    } else {
      channel_count = Some(channels);
    }

    let frames = decoded.frames();
    let mut sample_buffer =
      SampleBuffer::<f32>::new(frames as u64, *decoded.spec());
    sample_buffer.copy_interleaved_ref(decoded);
    samples.extend_from_slice(sample_buffer.samples());
  }

  let sample_rate = sample_rate.ok_or(AudioDecodeError::InvalidData {
    details: format!(
      "{source_description} contained no decodable audio frames"
    ),
  })?;
  let channels = channel_count.ok_or(AudioDecodeError::InvalidData {
    details: format!(
      "{source_description} contained no decodable channel configuration"
    ),
  })?;

  if samples.is_empty() {
    return Err(AudioDecodeError::InvalidData {
      details: format!("{source_description} contained no decoded samples"),
    });
  }

  return Ok(DecodedAudio {
    samples,
    sample_rate,
    channels,
  });
}

/// Decode WAV bytes into interleaved `f32` samples.
#[cfg(feature = "audio-decode-wav")]
pub fn decode_wav_bytes(
  bytes: &[u8],
) -> Result<DecodedAudio, AudioDecodeError> {
  let mut format = probe_format(bytes, "WAV", &["wav"])?;
  let (track_id, codec_params) = match format.default_track() {
    Some(track) => (track.id, track.codec_params.clone()),
    None => {
      return Err(AudioDecodeError::InvalidData {
        details: "no default audio track found".to_string(),
      });
    }
  };

  let sample_format =
    codec_params
      .sample_format
      .ok_or(AudioDecodeError::UnsupportedFormat {
        details: "WAV sample format is unspecified".to_string(),
      })?;

  match sample_format {
    SampleFormat::S16 | SampleFormat::S24 | SampleFormat::F32 => {}
    other => {
      return Err(AudioDecodeError::UnsupportedFormat {
        details: format!("unsupported WAV sample format: {other:?}"),
      });
    }
  }

  let mut decoder = symphonia::default::get_codecs()
    .make(&codec_params, &DecoderOptions::default())
    .map_err(|error| map_read_or_decode_error("WAV", error))?;

  return decode_track_to_interleaved_f32(
    &mut *format,
    track_id,
    &mut *decoder,
    "WAV",
    codec_params.n_frames,
    codec_params
      .channels
      .map(|channels| channels.count() as u16),
  );
}

/// Decode OGG Vorbis bytes into interleaved `f32` samples.
#[cfg(feature = "audio-decode-vorbis")]
pub fn decode_ogg_vorbis_bytes(
  bytes: &[u8],
) -> Result<DecodedAudio, AudioDecodeError> {
  let mut format = probe_format(bytes, "OGG Vorbis", &["ogg", "oga"])?;
  let (track_id, codec_params) = match format.default_track() {
    Some(track) => (track.id, track.codec_params.clone()),
    None => {
      return Err(AudioDecodeError::InvalidData {
        details: "no default audio track found".to_string(),
      });
    }
  };

  if codec_params.codec != CODEC_TYPE_VORBIS {
    return Err(AudioDecodeError::UnsupportedFormat {
      details: "OGG stream is not Vorbis".to_string(),
    });
  }

  let mut decoder = symphonia::default::get_codecs()
    .make(&codec_params, &DecoderOptions::default())
    .map_err(|error| map_read_or_decode_error("OGG Vorbis", error))?;

  return decode_track_to_interleaved_f32(
    &mut *format,
    track_id,
    &mut *decoder,
    "OGG Vorbis",
    codec_params.n_frames,
    codec_params
      .channels
      .map(|channels| channels.count() as u16),
  );
}

#[cfg(test)]
mod tests {
  use super::*;

  #[cfg(feature = "audio-decode-wav")]
  #[test]
  fn wav_decode_rejects_invalid_bytes() {
    let result = decode_wav_bytes(&[0u8, 1u8, 2u8, 3u8]);
    assert!(matches!(
      result,
      Err(AudioDecodeError::UnsupportedFormat { .. })
        | Err(AudioDecodeError::InvalidData { .. })
        | Err(AudioDecodeError::DecodeFailed { .. })
    ));
    return;
  }

  #[cfg(feature = "audio-decode-vorbis")]
  #[test]
  fn ogg_vorbis_decode_rejects_invalid_bytes() {
    let result = decode_ogg_vorbis_bytes(&[0u8, 1u8, 2u8, 3u8]);
    assert!(matches!(
      result,
      Err(AudioDecodeError::UnsupportedFormat { .. })
        | Err(AudioDecodeError::InvalidData { .. })
        | Err(AudioDecodeError::DecodeFailed { .. })
    ));
    return;
  }
}
