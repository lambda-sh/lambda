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
use symphonia::core::{
  audio::{
    AudioBufferRef,
    SampleBuffer,
  },
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
  /// Interleaved audio samples in nominal range `[-1.0, 1.0]`.
  pub samples: Vec<f32>,
  /// Sample rate in Hz.
  pub sample_rate: u32,
  /// Interleaved channel count (currently `1` or `2`).
  pub channels: u16,
}

/// Vendor-free errors produced by audio decoding helpers.
#[derive(Clone, Debug)]
pub enum AudioDecodeError {
  /// The provided bytes were not a recognized container or codec.
  UnsupportedFormat { details: String },
  /// The provided bytes were recognized but invalid or corrupted.
  InvalidData { details: String },
  /// A platform or backend error prevented decoding.
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

/// Build a `symphonia` probe hint from a list of likely filename extensions.
///
/// # Arguments
/// - `extensions`: A list of likely filename extensions (without a leading
///   period) used to guide `symphonia`'s format probe.
///
/// # Returns
/// A probe hint configured with all provided extensions.
fn hint_for_decode(extensions: &[&str]) -> Hint {
  let mut hint_value = Hint::new();
  for extension in extensions {
    hint_value.with_extension(extension);
  }
  return hint_value;
}

/// Map probe-time `symphonia` errors into backend-agnostic decode errors.
///
/// # Arguments
/// - `source_description`: A human-readable description used to contextualize
///   error messages (for example, `"WAV"` or `"OGG Vorbis"`).
/// - `error`: The `symphonia` probe error.
///
/// # Returns
/// A stable, vendor-free decode error.
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

/// Map packet read or decode-time `symphonia` errors into decode errors.
///
/// This keeps the surface area stable for `lambda-rs` and avoids leaking
/// vendor-specific error types.
///
/// # Arguments
/// - `source_description`: A human-readable description used to contextualize
///   error messages (for example, `"WAV"` or `"OGG Vorbis"`).
/// - `error`: The `symphonia` read or decode error.
///
/// # Returns
/// A stable, vendor-free decode error.
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

/// Probe the container format for an in-memory audio buffer.
///
/// `symphonia` expects a `MediaSourceStream`. This wrapper creates an owned
/// cursor backed by `bytes` so the probe can seek without borrowing the input.
///
/// # Arguments
/// - `bytes`: The complete container bytes.
/// - `source_description`: A human-readable description used to contextualize
///   error messages.
/// - `extensions`: A list of filename extensions used as a probe hint.
///
/// # Returns
/// A `FormatReader` capable of reading packets from the probed container.
///
/// # Errors
/// Returns [`AudioDecodeError::UnsupportedFormat`] when the container cannot be
/// recognized. Returns [`AudioDecodeError::InvalidData`] when the bytes are
/// recognized but no tracks can be read.
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

/// Pre-allocate a decoded sample buffer using optional codec metadata.
///
/// Failure to reserve is treated as a recoverable decode error to avoid
/// panicking on large files or constrained platforms.
///
/// # Arguments
/// - `samples`: The output sample vector to reserve capacity for.
/// - `source_description`: A human-readable description used to contextualize
///   error messages.
/// - `frames`: Optional total frame count metadata for the selected track.
/// - `channels`: Optional channel count metadata for the selected track.
///
/// # Returns
/// `Ok(())` when reservation succeeds or cannot be estimated.
///
/// # Errors
/// Returns [`AudioDecodeError::DecodeFailed`] if the allocator fails to reserve
/// the requested capacity.
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

/// Decode a single `symphonia` track into interleaved `f32` samples.
///
/// Behavior:
/// - Reads packets until end-of-stream.
/// - Ignores packets from other tracks.
/// - Handles `ResetRequired` by resetting the decoder and continuing.
/// - Validates that sample rate and channel count remain stable across packets.
/// - Restricts channel count to mono/stereo for the current engine surface.
/// - For WAV, validates the decoded sample format on first decoded packet to
///   ensure only the supported input formats are accepted.
///
/// # Arguments
/// - `format`: Container reader used to fetch packets.
/// - `track_id`: The track identifier to decode. Packets from other tracks are
///   ignored.
/// - `decoder`: Codec decoder for the selected track.
/// - `source_description`: A human-readable description used to contextualize
///   error messages.
/// - `reserve_frames`: Optional frame count metadata used to pre-reserve the
///   output buffer.
/// - `reserve_channels`: Optional channel count metadata used to pre-reserve
///   the output buffer.
///
/// # Returns
/// Fully decoded audio with interleaved `f32` samples and associated metadata.
///
/// # Errors
/// Returns:
/// - [`AudioDecodeError::UnsupportedFormat`] for unsupported channel counts or
///   unsupported WAV decoded sample formats.
/// - [`AudioDecodeError::InvalidData`] for corrupted streams or inconsistent
///   metadata during decode.
/// - [`AudioDecodeError::DecodeFailed`] for other backend failures.
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
  let mut wav_sample_format_validated = false;

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
    if frames == 0 {
      continue;
    }

    if source_description == "WAV" && !wav_sample_format_validated {
      validate_wav_decoded_sample_format(&decoded)?;
      wav_sample_format_validated = true;
    }

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

/// Validate the sample format returned by `symphonia` for WAV decoding.
///
/// The engine surface currently supports WAV inputs that decode to:
/// - 16-bit signed integer (`S16`)
/// - 24-bit signed integer (`S24`)
/// - 32-bit float (`F32`)
///
/// # Arguments
/// - `decoded`: The decoded packet audio buffer view.
///
/// # Returns
/// `Ok(())` when the decoded sample format is supported.
///
/// # Errors
/// Returns [`AudioDecodeError::UnsupportedFormat`] if the decoded sample format
/// is not supported by the current engine surface.
fn validate_wav_decoded_sample_format(
  decoded: &AudioBufferRef<'_>,
) -> Result<(), AudioDecodeError> {
  match decoded {
    AudioBufferRef::S16(_)
    | AudioBufferRef::S24(_)
    | AudioBufferRef::F32(_) => {
      return Ok(());
    }
    other => {
      return Err(AudioDecodeError::UnsupportedFormat {
        details: format!(
          "unsupported WAV decoded sample format: {}",
          wav_decoded_sample_format_name(other)
        ),
      });
    }
  }
}

/// Return a stable string name for WAV decoded sample formats.
///
/// # Arguments
/// - `decoded`: The decoded packet audio buffer view.
///
/// # Returns
/// A stable name for diagnostics and error messages.
fn wav_decoded_sample_format_name(
  decoded: &AudioBufferRef<'_>,
) -> &'static str {
  match decoded {
    AudioBufferRef::U8(_) => "U8",
    AudioBufferRef::U16(_) => "U16",
    AudioBufferRef::U24(_) => "U24",
    AudioBufferRef::U32(_) => "U32",
    AudioBufferRef::S8(_) => "S8",
    AudioBufferRef::S16(_) => "S16",
    AudioBufferRef::S24(_) => "S24",
    AudioBufferRef::S32(_) => "S32",
    AudioBufferRef::F32(_) => "F32",
    AudioBufferRef::F64(_) => "F64",
  }
}

/// Decode WAV bytes into interleaved `f32` samples.
///
/// # Arguments
/// - `bytes`: Complete WAV container bytes.
///
/// # Returns
/// Fully decoded audio with interleaved `f32` samples and associated metadata.
///
/// # Errors
/// Returns [`AudioDecodeError::UnsupportedFormat`] if the bytes are not a WAV
/// file or use an unsupported encoding. Returns
/// [`AudioDecodeError::InvalidData`] if the bytes are a WAV container but are
/// invalid or corrupted. Returns
/// [`AudioDecodeError::DecodeFailed`] for other backend failures.
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
///
/// # Arguments
/// - `bytes`: Complete OGG container bytes.
///
/// # Returns
/// Fully decoded audio with interleaved `f32` samples and associated metadata.
///
/// # Errors
/// Returns [`AudioDecodeError::UnsupportedFormat`] if the bytes are not an OGG
/// container, or if the OGG stream is not Vorbis. Returns
/// [`AudioDecodeError::InvalidData`] if the container is invalid or corrupted.
/// Returns [`AudioDecodeError::DecodeFailed`] for other backend failures.
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

  /// Fixture: 44100 Hz, mono, 16-bit integer PCM.
  #[cfg(feature = "audio-decode-wav")]
  const TONE_S16_MONO_44100_WAV: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/audio/tone_s16_mono_44100.wav"
  ));

  /// Fixture: 44100 Hz, stereo, 16-bit integer PCM.
  #[cfg(feature = "audio-decode-wav")]
  const TONE_S16_STEREO_44100_WAV: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/audio/tone_s16_stereo_44100.wav"
  ));

  /// Fixture: 44100 Hz, mono, 24-bit integer PCM.
  #[cfg(feature = "audio-decode-wav")]
  const TONE_S24_MONO_44100_WAV: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/audio/tone_s24_mono_44100.wav"
  ));

  /// Fixture: 44100 Hz, stereo, 32-bit float PCM.
  #[cfg(feature = "audio-decode-wav")]
  const TONE_F32_STEREO_44100_WAV: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/audio/tone_f32_stereo_44100.wav"
  ));

  /// Fixture: 48000 Hz, stereo, OGG Vorbis.
  #[cfg(feature = "audio-decode-vorbis")]
  const SLASH_VORBIS_STEREO_48000_OGG: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/audio/slash_vorbis_stereo_48000.ogg"
  ));

  /// Decoding invalid WAV bytes MUST return a structured error.
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

  /// WAV decode MUST preserve sample rate and channel metadata from the input.
  #[cfg(feature = "audio-decode-wav")]
  #[test]
  fn wav_decode_s16_mono_fixture_decodes() {
    let decoded =
      decode_wav_bytes(TONE_S16_MONO_44100_WAV).expect("decode failed");
    assert_eq!(decoded.sample_rate, 44100);
    assert_eq!(decoded.channels, 1);
    assert_eq!(decoded.samples.len(), 4410);
    return;
  }

  /// WAV decode MUST support stereo 16-bit integer PCM.
  #[cfg(feature = "audio-decode-wav")]
  #[test]
  fn wav_decode_s16_stereo_fixture_decodes() {
    let decoded =
      decode_wav_bytes(TONE_S16_STEREO_44100_WAV).expect("decode failed");
    assert_eq!(decoded.sample_rate, 44100);
    assert_eq!(decoded.channels, 2);
    assert_eq!(decoded.samples.len(), 4410 * 2);
    return;
  }

  /// WAV decode MUST support mono 24-bit integer PCM.
  #[cfg(feature = "audio-decode-wav")]
  #[test]
  fn wav_decode_s24_mono_fixture_decodes() {
    let decoded =
      decode_wav_bytes(TONE_S24_MONO_44100_WAV).expect("decode failed");
    assert_eq!(decoded.sample_rate, 44100);
    assert_eq!(decoded.channels, 1);
    assert_eq!(decoded.samples.len(), 4410);
    return;
  }

  /// WAV decode MUST support stereo 32-bit float PCM.
  #[cfg(feature = "audio-decode-wav")]
  #[test]
  fn wav_decode_f32_stereo_fixture_decodes() {
    let decoded =
      decode_wav_bytes(TONE_F32_STEREO_44100_WAV).expect("decode failed");
    assert_eq!(decoded.sample_rate, 44100);
    assert_eq!(decoded.channels, 2);
    assert_eq!(decoded.samples.len(), 4410 * 2);
    return;
  }

  /// Decoding invalid OGG bytes MUST return a structured error.
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

  /// OGG Vorbis decode MUST preserve sample rate and channel metadata.
  #[cfg(feature = "audio-decode-vorbis")]
  #[test]
  fn ogg_vorbis_decode_fixture_decodes() {
    let decoded = decode_ogg_vorbis_bytes(SLASH_VORBIS_STEREO_48000_OGG)
      .expect("decode failed");
    assert_eq!(decoded.sample_rate, 48000);
    assert_eq!(decoded.channels, 2);
    assert!(!decoded.samples.is_empty());
    return;
  }
}
