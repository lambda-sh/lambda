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

  #[cfg(feature = "audio-sound-buffer-wav")]
  pub fn from_wav_bytes(bytes: &[u8]) -> Result<Self, AudioError> {
    let decoded = lambda_platform::audio::symphonia::decode_wav_bytes(bytes)
      .map_err(map_decode_error)?;
    return Self::from_decoded(decoded);
  }

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

  #[cfg(feature = "audio-sound-buffer-vorbis")]
  pub fn from_ogg_bytes(bytes: &[u8]) -> Result<Self, AudioError> {
    let decoded =
      lambda_platform::audio::symphonia::decode_ogg_vorbis_bytes(bytes)
        .map_err(map_decode_error)?;
    return Self::from_decoded(decoded);
  }

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

  pub fn sample_rate(&self) -> u32 {
    return self.sample_rate;
  }

  pub fn channels(&self) -> u16 {
    return self.channels;
  }

  pub fn duration_seconds(&self) -> f32 {
    if self.channels == 0 || self.sample_rate == 0 {
      return 0.0;
    }

    let channels = self.channels as usize;
    let frames = self.samples.len() / channels;
    return frames as f32 / self.sample_rate as f32;
  }
}

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
}
