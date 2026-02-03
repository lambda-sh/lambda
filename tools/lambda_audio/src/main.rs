#![allow(clippy::needless_return)]
//! A small CLI for inspecting and playing sound files using the `lambda` audio
//! APIs.
//!
//! This tool is intended for quick manual validation of decoding, device
//! enumeration, and basic output playback behavior.
//!
//! # Commands
//! - `info <path>`: Decode a sound file and print basic metadata.
//! - `view <path>`: Decode a sound file, print metadata, and render an ASCII
//!   waveform preview.
//! - `play <path>`: Decode a sound file and play it through the default output
//!   device.
//! - `list-devices`: List output devices (platform-dependent).

use std::{
  path::Path,
  sync::{
    atomic::{
      AtomicUsize,
      Ordering,
    },
    Arc,
  },
  time::Duration,
};

use lambda::audio::{
  enumerate_output_devices,
  AudioError,
  AudioOutputDeviceBuilder,
  SoundBuffer,
};

/// Runs the `lambda-audio` CLI.
fn main() {
  let mut args = std::env::args();
  let raw_program_name =
    args.next().unwrap_or_else(|| "lambda-audio".to_string());
  let program_name = Path::new(&raw_program_name)
    .file_name()
    .and_then(|value| value.to_str())
    .unwrap_or(raw_program_name.as_str())
    .to_string();

  let command = args.next().unwrap_or_else(|| "help".to_string());

  let result = match command.as_str() {
    "help" | "--help" | "-h" => {
      print_usage(&program_name);
      Ok(())
    }
    "info" => cmd_info(&program_name, args.next()),
    "view" => cmd_view(&program_name, args.next()),
    "play" => cmd_play(&program_name, args.next()),
    "list-devices" => cmd_list_devices(),
    other => {
      eprintln!("unknown command: {other}");
      print_usage(&program_name);
      Err(ExitError::Usage)
    }
  };

  match result {
    Ok(()) => {
      return;
    }
    Err(ExitError::Usage) => {
      std::process::exit(2);
    }
    Err(ExitError::Runtime(error)) => {
      eprintln!("{error}");
      std::process::exit(1);
    }
  }
}

#[derive(Debug)]
/// A CLI error type that separates usage errors from runtime failures.
enum ExitError {
  /// The user provided invalid arguments.
  Usage,
  /// The command failed due to an audio/runtime error.
  Runtime(AudioError),
}

/// Prints metadata about a decoded sound file.
///
/// # Arguments
/// - `program_name`: The CLI program name used for usage output.
/// - `path`: A file path argument, if provided.
///
/// # Returns
/// Returns `Ok(())` when the command completes successfully.
///
/// # Errors
/// Returns `ExitError::Usage` when the path argument is missing.
/// Returns `ExitError::Runtime` when decoding fails.
fn cmd_info(program_name: &str, path: Option<String>) -> Result<(), ExitError> {
  let path = require_path(program_name, "info", path)?;
  let buffer = load_sound_buffer(&path).map_err(ExitError::Runtime)?;
  print_info(&path, &buffer);
  return Ok(());
}

/// Prints metadata and renders an ASCII waveform preview.
///
/// # Arguments
/// - `program_name`: The CLI program name used for usage output.
/// - `path`: A file path argument, if provided.
///
/// # Returns
/// Returns `Ok(())` when the command completes successfully.
///
/// # Errors
/// Returns `ExitError::Usage` when the path argument is missing.
/// Returns `ExitError::Runtime` when decoding fails.
fn cmd_view(program_name: &str, path: Option<String>) -> Result<(), ExitError> {
  let path = require_path(program_name, "view", path)?;
  let buffer = load_sound_buffer(&path).map_err(ExitError::Runtime)?;
  print_info(&path, &buffer);
  print_waveform(&buffer);
  return Ok(());
}

/// Plays a decoded sound file through the default output device.
///
/// # Arguments
/// - `program_name`: The CLI program name used for usage output.
/// - `path`: A file path argument, if provided.
///
/// # Returns
/// Returns `Ok(())` when the command completes successfully.
///
/// # Errors
/// Returns `ExitError::Usage` when the path argument is missing.
/// Returns `ExitError::Runtime` when decoding or playback fails.
fn cmd_play(program_name: &str, path: Option<String>) -> Result<(), ExitError> {
  let path = require_path(program_name, "play", path)?;
  let buffer = load_sound_buffer(&path).map_err(ExitError::Runtime)?;
  print_info(&path, &buffer);
  play_buffer(&buffer).map_err(ExitError::Runtime)?;
  return Ok(());
}

/// Lists available output devices.
///
/// # Returns
/// Returns `Ok(())` when the command completes successfully.
///
/// # Errors
/// Returns `ExitError::Runtime` when device enumeration fails.
fn cmd_list_devices() -> Result<(), ExitError> {
  let devices = enumerate_output_devices().map_err(ExitError::Runtime)?;

  if devices.is_empty() {
    println!("no output devices found");
    return Ok(());
  }

  for device in devices {
    let default_marker = if device.is_default { "*" } else { " " };
    println!("{default_marker} {}", device.name);
  }

  return Ok(());
}

/// Requires a file path argument for a command.
///
/// # Arguments
/// - `program_name`: The CLI program name used for usage output.
/// - `command`: The command name used for usage output.
/// - `path`: A file path argument, if provided.
///
/// # Returns
/// Returns the provided path string.
///
/// # Errors
/// Returns `ExitError::Usage` when `path` is `None`.
fn require_path(
  program_name: &str,
  command: &str,
  path: Option<String>,
) -> Result<String, ExitError> {
  let Some(path) = path else {
    eprintln!("usage: {program_name} {command} <path-to-wav-or-ogg>");
    return Err(ExitError::Usage);
  };
  return Ok(path);
}

/// Loads a sound file into a decoded `SoundBuffer` based on its file extension.
///
/// # Arguments
/// - `path`: A file path to a supported sound file.
///
/// # Returns
/// Returns a decoded `SoundBuffer`.
///
/// # Errors
/// Returns an `AudioError` if decoding fails or the file extension is not
/// supported.
fn load_sound_buffer(path: &str) -> Result<SoundBuffer, AudioError> {
  let path_value = Path::new(path);
  let extension = path_value
    .extension()
    .and_then(|value| value.to_str())
    .map(|value| value.to_ascii_lowercase())
    .unwrap_or_default();

  match extension.as_str() {
    "wav" => {
      return SoundBuffer::from_wav_file(path_value);
    }
    "ogg" | "oga" => {
      return SoundBuffer::from_ogg_file(path_value);
    }
    _ => {
      return Err(AudioError::UnsupportedFormat {
        details: format!("unsupported file extension: {extension:?}"),
      });
    }
  }
}

/// Prints basic decoded metadata for a `SoundBuffer`.
///
/// # Arguments
/// - `path`: The decoded source path for display purposes.
/// - `buffer`: The decoded sound buffer.
///
/// # Returns
/// Returns `()` after printing metadata to stdout.
fn print_info(path: &str, buffer: &SoundBuffer) {
  println!("path: {path}");
  println!("sample_rate: {}", buffer.sample_rate());
  println!("channels: {}", buffer.channels());
  println!("frames: {}", buffer.frames());
  println!("samples: {}", buffer.samples().len());
  println!("duration_seconds: {:.3}", buffer.duration_seconds());
  return;
}

/// Renders an ASCII waveform preview for a `SoundBuffer`.
///
/// The rendering uses a single channel (the first channel) and shows a
/// peak-per-column visualization, which is intended for quick human inspection
/// rather than precise analysis.
///
/// # Arguments
/// - `buffer`: The decoded sound buffer to visualize.
///
/// # Returns
/// Returns `()` after printing the visualization to stdout.
fn print_waveform(buffer: &SoundBuffer) {
  let width: usize = 64;
  let height: usize = 10;

  let samples = buffer.samples();
  let channels = buffer.channels() as usize;
  if samples.is_empty() || channels == 0 {
    println!("<no samples>");
    return;
  }

  let frames = buffer.frames();
  if frames == 0 {
    println!("<no frames>");
    return;
  }

  let step = (frames / width).max(1);
  let mut peaks: Vec<f32> = Vec::with_capacity(width);

  for column in 0..width {
    let start_frame = column * step;
    if start_frame >= frames {
      break;
    }
    let end_frame = ((column + 1) * step).min(frames);

    let mut peak = 0.0f32;
    for frame in start_frame..end_frame {
      let sample_index = frame.saturating_mul(channels);
      let sample = samples.get(sample_index).copied().unwrap_or(0.0);
      peak = peak.max(sample.abs());
    }

    peaks.push(peak);
  }

  for row in (0..height).rev() {
    let threshold = (row + 1) as f32 / height as f32;
    for peak in &peaks {
      let mark = if *peak >= threshold { '#' } else { ' ' };
      print!("{mark}");
    }
    println!();
  }

  return;
}

/// Plays a decoded `SoundBuffer` through the default output device.
///
/// This performs a best-effort playback by writing sequential frames into the
/// output callback. No resampling or channel remapping is performed; instead,
/// the output device is configured to match the buffer's sample rate and
/// channel count.
///
/// # Arguments
/// - `buffer`: The decoded sound buffer to play.
///
/// # Returns
/// Returns `Ok(())` after playback has completed.
///
/// # Errors
/// Returns an `AudioError` if the buffer is empty or output device creation
/// fails.
fn play_buffer(buffer: &SoundBuffer) -> Result<(), AudioError> {
  let samples = buffer.samples();
  let total_samples = samples.len();

  if total_samples == 0 {
    return Err(AudioError::InvalidData {
      details: "sound buffer had no samples".to_string(),
    });
  }

  let cursor = Arc::new(AtomicUsize::new(0));
  let buffer = Arc::new(buffer.clone());

  let cursor_for_callback = cursor.clone();
  let buffer_for_callback = buffer.clone();

  let _device = AudioOutputDeviceBuilder::new()
    .with_label("lambda-audio")
    .with_sample_rate(buffer.sample_rate())
    .with_channels(buffer.channels())
    .build_with_output_callback(move |writer, _info| {
      let writer_channels = writer.channels() as usize;
      let writer_frames = writer.frames();

      writer.clear();

      if writer_channels == 0 {
        return;
      }

      let write_samples = writer_frames.saturating_mul(writer_channels);
      let start =
        cursor_for_callback.fetch_add(write_samples, Ordering::Relaxed);

      let source_samples = buffer_for_callback.samples();
      let source_total = source_samples.len();

      for frame in 0..writer_frames {
        for channel in 0..writer_channels {
          let sample_index = start
            .saturating_add(frame.saturating_mul(writer_channels))
            .saturating_add(channel);

          let value = source_samples.get(sample_index).copied().unwrap_or(0.0);
          if sample_index < source_total {
            writer.set_sample(frame, channel, value);
          }
        }
      }

      return;
    })?;

  let wait_seconds = buffer.duration_seconds() + 0.20;
  std::thread::sleep(Duration::from_secs_f32(wait_seconds));
  drop(_device);
  return Ok(());
}

/// Prints usage text to stdout.
///
/// # Arguments
/// - `program_name`: The CLI program name shown in examples.
///
/// # Returns
/// Returns `()` after printing the usage text.
fn print_usage(program_name: &str) {
  println!("usage:");
  println!("  {program_name} info <path>");
  println!("  {program_name} view <path>");
  println!("  {program_name} play <path>");
  println!("  {program_name} list-devices");
  return;
}
