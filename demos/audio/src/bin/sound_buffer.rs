#![allow(clippy::needless_return)]
//! Sound buffer loading demo that decodes a WAV or OGG Vorbis file.
//!
//! This demo assumes `lambda-demos-audio` is built with its default features.

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
    .unwrap_or_else(|| "sound_buffer".to_string());

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
    .unwrap_or_default();

  match extension.as_str() {
    "wav" => {
      return SoundBuffer::from_wav_file(path);
    }
    "ogg" | "oga" => {
      return SoundBuffer::from_ogg_file(path);
    }
    _ => {
      return Err(AudioError::UnsupportedFormat {
        details: format!("unsupported file extension: {extension:?}"),
      });
    }
  }
}
