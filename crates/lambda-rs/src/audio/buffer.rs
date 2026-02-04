#![allow(clippy::needless_return)]

use std::path::Path;

use crate::audio::AudioError;

/// Fully decoded, in-memory audio samples suitable for future mixing and
/// playback.
#[derive(Clone, Debug, PartialEq)]
pub struct SoundBuffer {
  samples: Vec<f32>,
  sample_rate: u32,
  channels: u16,
}

impl SoundBuffer {
  /// Decode a WAV file from disk into an in-memory sound buffer.
  ///
  /// This method is available when the `audio-sound-buffer-wav` feature is
  /// enabled.
  ///
  /// # Arguments
  /// - `path`: Path to a WAV file on disk.
  ///
  /// # Returns
  /// A fully decoded sound buffer with interleaved `f32` samples.
  ///
  /// # Errors
  /// Returns [`AudioError::Io`] if the file cannot be read. Returns
  /// [`AudioError::UnsupportedFormat`], [`AudioError::InvalidData`], or
  /// [`AudioError::DecodeFailed`] if decoding fails.
  #[cfg(feature = "audio-sound-buffer-wav")]
  pub fn from_wav_file(path: &Path) -> Result<Self, AudioError> {
    let bytes = std::fs::read(path).map_err(|error| {
      return AudioError::Io {
        path: Some(path.to_path_buf()),
        details: error.to_string(),
      };
    })?;

    return Self::from_wav_bytes(&bytes);
  }

  /// Decode WAV container bytes into an in-memory sound buffer.
  ///
  /// This method is available when the `audio-sound-buffer-wav` feature is
  /// enabled.
  ///
  /// # Arguments
  /// - `bytes`: Full WAV container bytes.
  ///
  /// # Returns
  /// A fully decoded sound buffer with interleaved `f32` samples.
  ///
  /// # Errors
  /// Returns [`AudioError::UnsupportedFormat`], [`AudioError::InvalidData`], or
  /// [`AudioError::DecodeFailed`] if decoding fails.
  #[cfg(feature = "audio-sound-buffer-wav")]
  pub fn from_wav_bytes(bytes: &[u8]) -> Result<Self, AudioError> {
    let decoded = lambda_platform::audio::symphonia::decode_wav_bytes(bytes)
      .map_err(map_decode_error)?;
    return Self::from_decoded(decoded);
  }

  /// Decode an OGG Vorbis file from disk into an in-memory sound buffer.
  ///
  /// This method is available when the `audio-sound-buffer-vorbis` feature is
  /// enabled.
  ///
  /// # Arguments
  /// - `path`: Path to an OGG Vorbis file on disk.
  ///
  /// # Returns
  /// A fully decoded sound buffer with interleaved `f32` samples.
  ///
  /// # Errors
  /// Returns [`AudioError::Io`] if the file cannot be read. Returns
  /// [`AudioError::UnsupportedFormat`], [`AudioError::InvalidData`], or
  /// [`AudioError::DecodeFailed`] if decoding fails.
  #[cfg(feature = "audio-sound-buffer-vorbis")]
  pub fn from_ogg_file(path: &Path) -> Result<Self, AudioError> {
    let bytes = std::fs::read(path).map_err(|error| {
      return AudioError::Io {
        path: Some(path.to_path_buf()),
        details: error.to_string(),
      };
    })?;

    return Self::from_ogg_bytes(&bytes);
  }

  /// Decode OGG Vorbis container bytes into an in-memory sound buffer.
  ///
  /// This method is available when the `audio-sound-buffer-vorbis` feature is
  /// enabled.
  ///
  /// # Arguments
  /// - `bytes`: Full OGG container bytes.
  ///
  /// # Returns
  /// A fully decoded sound buffer with interleaved `f32` samples.
  ///
  /// # Errors
  /// Returns [`AudioError::UnsupportedFormat`], [`AudioError::InvalidData`], or
  /// [`AudioError::DecodeFailed`] if decoding fails.
  #[cfg(feature = "audio-sound-buffer-vorbis")]
  pub fn from_ogg_bytes(bytes: &[u8]) -> Result<Self, AudioError> {
    let decoded =
      lambda_platform::audio::symphonia::decode_ogg_vorbis_bytes(bytes)
        .map_err(map_decode_error)?;
    return Self::from_decoded(decoded);
  }

  /// Convert platform decoded audio into the public sound buffer
  /// representation.
  ///
  /// # Arguments
  /// - `decoded`: Decoded audio samples and associated metadata produced by the
  ///   platform layer.
  ///
  /// # Returns
  /// A sound buffer containing the provided samples and validated metadata.
  ///
  /// # Errors
  /// Returns [`AudioError::InvalidData`] if the decoded metadata is invalid.
  fn from_decoded(
    decoded: lambda_platform::audio::symphonia::DecodedAudio,
  ) -> Result<Self, AudioError> {
    if decoded.sample_rate == 0 {
      return Err(AudioError::InvalidData {
        details: "decoded sample rate was 0".to_string(),
      });
    }

    if decoded.channels == 0 {
      return Err(AudioError::InvalidData {
        details: "decoded channel count was 0".to_string(),
      });
    }

    return Ok(Self {
      samples: decoded.samples,
      sample_rate: decoded.sample_rate,
      channels: decoded.channels,
    });
  }

  /// Return the sample rate in Hz.
  ///
  /// # Returns
  /// The sample rate in Hz.
  pub fn sample_rate(&self) -> u32 {
    return self.sample_rate;
  }

  /// Return the interleaved channel count.
  ///
  /// # Returns
  /// The channel count.
  pub fn channels(&self) -> u16 {
    return self.channels;
  }

  /// Return interleaved `f32` samples in nominal range `[-1.0, 1.0]`.
  ///
  /// # Returns
  /// A slice of interleaved samples.
  pub fn samples(&self) -> &[f32] {
    return self.samples.as_slice();
  }

  /// Return the number of frames in this buffer.
  ///
  /// # Returns
  /// The number of frames in the buffer.
  pub fn frames(&self) -> usize {
    if self.channels == 0 {
      return 0;
    }

    return self.samples.len() / self.channels as usize;
  }

  /// Return the duration of the buffer in seconds.
  ///
  /// # Returns
  /// The duration in seconds.
  pub fn duration_seconds(&self) -> f32 {
    if self.channels == 0 || self.sample_rate == 0 {
      return 0.0;
    }

    let channels = self.channels as usize;
    let frames = self.samples.len() / channels;
    return frames as f32 / self.sample_rate as f32;
  }
}

/// Map platform decode errors into backend-agnostic public errors.
///
/// # Arguments
/// - `error`: The platform decode error.
///
/// # Returns
/// The equivalent public audio error.
fn map_decode_error(
  error: lambda_platform::audio::symphonia::AudioDecodeError,
) -> AudioError {
  match error {
    lambda_platform::audio::symphonia::AudioDecodeError::UnsupportedFormat {
      details,
    } => {
      return AudioError::UnsupportedFormat { details };
    }
    lambda_platform::audio::symphonia::AudioDecodeError::InvalidData {
      details,
    } => {
      return AudioError::InvalidData { details };
    }
    lambda_platform::audio::symphonia::AudioDecodeError::DecodeFailed {
      details,
    } => {
      return AudioError::DecodeFailed { details };
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  /// Duration computation MUST match frames / sample_rate.
  #[test]
  fn duration_seconds_computes_expected_value() {
    let buffer = SoundBuffer {
      samples: vec![0.0; 48000],
      sample_rate: 48000,
      channels: 1,
    };

    assert_eq!(buffer.duration_seconds(), 1.0);
    return;
  }

  /// Frames MUST return 0 when channel metadata is invalid.
  #[test]
  fn frames_returns_zero_when_channels_is_zero() {
    let buffer = SoundBuffer {
      samples: vec![0.0, 0.0],
      sample_rate: 48_000,
      channels: 0,
    };

    assert_eq!(buffer.frames(), 0);
    assert_eq!(buffer.duration_seconds(), 0.0);
    return;
  }

  /// Decoded audio conversion MUST reject invalid metadata.
  #[test]
  fn from_decoded_rejects_invalid_metadata() {
    let result = SoundBuffer::from_decoded(
      lambda_platform::audio::symphonia::DecodedAudio {
        samples: vec![0.0],
        sample_rate: 0,
        channels: 1,
      },
    );
    assert!(matches!(result, Err(AudioError::InvalidData { .. })));

    let result = SoundBuffer::from_decoded(
      lambda_platform::audio::symphonia::DecodedAudio {
        samples: vec![0.0],
        sample_rate: 44_100,
        channels: 0,
      },
    );
    assert!(matches!(result, Err(AudioError::InvalidData { .. })));
    return;
  }

  /// Decode error mapping MUST preserve stable variants.
  #[test]
  fn map_decode_error_preserves_variants() {
    let unsupported = map_decode_error(
      lambda_platform::audio::symphonia::AudioDecodeError::UnsupportedFormat {
        details: "wav".to_string(),
      },
    );
    assert!(matches!(unsupported, AudioError::UnsupportedFormat { .. }));

    let invalid = map_decode_error(
      lambda_platform::audio::symphonia::AudioDecodeError::InvalidData {
        details: "bad".to_string(),
      },
    );
    assert!(matches!(invalid, AudioError::InvalidData { .. }));

    let failed = map_decode_error(
      lambda_platform::audio::symphonia::AudioDecodeError::DecodeFailed {
        details: "boom".to_string(),
      },
    );
    assert!(matches!(failed, AudioError::DecodeFailed { .. }));
    return;
  }

  /// WAV decode from bytes MUST succeed for the bundled fixture.
  #[cfg(feature = "audio-sound-buffer-wav")]
  #[test]
  fn from_wav_bytes_decodes_fixture() {
    let bytes = include_bytes!(concat!(
      env!("CARGO_MANIFEST_DIR"),
      "/../lambda-rs-platform/assets/audio/tone_s16_mono_44100.wav"
    ));

    let buffer = SoundBuffer::from_wav_bytes(bytes).expect("decode failed");
    assert_eq!(buffer.sample_rate(), 44100);
    assert_eq!(buffer.channels(), 1);
    assert!(buffer.duration_seconds() > 0.0);
    return;
  }

  /// WAV decode from file MUST succeed for the bundled fixture.
  #[cfg(feature = "audio-sound-buffer-wav")]
  #[test]
  fn from_wav_file_decodes_fixture() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
      .join("../lambda-rs-platform/assets/audio/tone_s16_mono_44100.wav");

    let buffer = SoundBuffer::from_wav_file(&path).expect("decode failed");
    assert_eq!(buffer.sample_rate(), 44100);
    assert_eq!(buffer.channels(), 1);
    assert!(buffer.duration_seconds() > 0.0);
    return;
  }

  /// OGG Vorbis decode from bytes MUST succeed for the bundled fixture.
  #[cfg(feature = "audio-sound-buffer-vorbis")]
  #[test]
  fn from_ogg_bytes_decodes_fixture() {
    let bytes = include_bytes!(concat!(
      env!("CARGO_MANIFEST_DIR"),
      "/../lambda-rs-platform/assets/audio/slash_vorbis_stereo_48000.ogg"
    ));

    let buffer = SoundBuffer::from_ogg_bytes(bytes).expect("decode failed");
    assert_eq!(buffer.sample_rate(), 48000);
    assert_eq!(buffer.channels(), 2);
    assert!(buffer.duration_seconds() > 0.0);
    return;
  }

  /// OGG Vorbis decode from file MUST succeed for the bundled fixture.
  #[cfg(feature = "audio-sound-buffer-vorbis")]
  #[test]
  fn from_ogg_file_decodes_fixture() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
      .join("../lambda-rs-platform/assets/audio/slash_vorbis_stereo_48000.ogg");

    let buffer = SoundBuffer::from_ogg_file(&path).expect("decode failed");
    assert_eq!(buffer.sample_rate(), 48000);
    assert_eq!(buffer.channels(), 2);
    assert!(buffer.duration_seconds() > 0.0);
    return;
  }
}
