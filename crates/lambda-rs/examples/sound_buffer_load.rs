#![allow(clippy::needless_return)]

use std::path::{
  Path,
  PathBuf,
};

use lambda::audio::{
  AudioError,
  SoundBuffer,
};

fn main() {
  let path = match parse_path_argument() {
    Ok(path) => path,
    Err(message) => {
      eprintln!("{message}");
      std::process::exit(2);
    }
  };

  let buffer = match load_sound_buffer(&path) {
    Ok(buffer) => buffer,
    Err(error) => {
      eprintln!("failed to load sound buffer: {error}");
      std::process::exit(1);
    }
  };

  println!("path: {}", path.display());
  println!("sample_rate: {}", buffer.sample_rate());
  println!("channels: {}", buffer.channels());
  println!("duration_seconds: {:.3}", buffer.duration_seconds());
  return;
}

fn parse_path_argument() -> Result<PathBuf, String> {
  let mut args = std::env::args_os();
  let program_name = args
    .next()
    .and_then(|value| value.into_string().ok())
    .unwrap_or_else(|| "sound_buffer_load".to_string());

  let path = args.next().ok_or_else(|| {
    return format!("usage: {program_name} <path-to-wav-or-ogg>");
  })?;

  return Ok(PathBuf::from(path));
}

fn load_sound_buffer(path: &Path) -> Result<SoundBuffer, AudioError> {
  let extension = path
    .extension()
    .and_then(|value| value.to_str())
    .map(|value| value.to_ascii_lowercase())
    .unwrap_or_else(|| "".to_string());

  match extension.as_str() {
    #[cfg(feature = "audio-sound-buffer-wav")]
    "wav" => {
      return SoundBuffer::from_wav_file(path);
    }
    #[cfg(not(feature = "audio-sound-buffer-wav"))]
    "wav" => {
      return Err(AudioError::UnsupportedFormat {
        details: "WAV support is disabled (enable `audio-sound-buffer-wav`)"
          .to_string(),
      });
    }
    #[cfg(feature = "audio-sound-buffer-vorbis")]
    "ogg" | "oga" => {
      return SoundBuffer::from_ogg_file(path);
    }
    #[cfg(not(feature = "audio-sound-buffer-vorbis"))]
    "ogg" | "oga" => {
      return Err(AudioError::UnsupportedFormat {
        details:
          "OGG Vorbis support is disabled (enable `audio-sound-buffer-vorbis`)"
            .to_string(),
      });
    }
    _ => {
      return Err(AudioError::UnsupportedFormat {
        details: format!("unsupported file extension: {extension:?}"),
      });
    }
  }
}
