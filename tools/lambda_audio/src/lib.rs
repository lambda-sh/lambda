#![allow(clippy::needless_return)]
//! A small CLI library for inspecting and playing sound files using the
//! `lambda` audio APIs.
//!
//! This crate is intended for quick manual validation of decoding, device
//! enumeration, and basic output playback behavior. The `lambda-audio` binary
//! delegates to this library so command behavior can be tested.

use std::{
  io::Write,
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
  AudioOutputWriter,
  SoundBuffer,
};

#[derive(Debug)]
/// A CLI error type that separates usage errors from runtime failures.
enum ExitError {
  /// The user provided invalid arguments.
  Usage,
  /// The command failed due to an audio runtime error.
  Runtime(AudioError),
}

/// Run the `lambda-audio` CLI with explicit output writers.
///
/// # Arguments
/// - `args`: The full CLI argument iterator including the program name.
/// - `stdout`: A writer that receives standard output.
/// - `stderr`: A writer that receives usage and runtime errors.
///
/// # Returns
/// A conventional process exit code (`0` success, `1` runtime error, `2` usage
/// error).
pub fn run<I>(args: I, stdout: &mut dyn Write, stderr: &mut dyn Write) -> i32
where
  I: IntoIterator<Item = String>,
{
  let mut args = args.into_iter();
  let raw_program_name =
    args.next().unwrap_or_else(|| "lambda-audio".to_string());
  let program_name = program_name_from_argv0(&raw_program_name);

  let command = args.next().unwrap_or_else(|| "help".to_string());

  let result = match command.as_str() {
    "help" | "--help" | "-h" => {
      write_usage(stdout, &program_name);
      Ok(())
    }
    "info" => cmd_info(stdout, stderr, &program_name, args.next()),
    "view" => cmd_view(stdout, stderr, &program_name, args.next()),
    "play" => cmd_play(stdout, stderr, &program_name, args.next()),
    "list-devices" => cmd_list_devices(stdout).map_err(ExitError::Runtime),
    other => {
      let _ = writeln!(stderr, "unknown command: {other}");
      write_usage(stderr, &program_name);
      Err(ExitError::Usage)
    }
  };

  match result {
    Ok(()) => {
      return 0;
    }
    Err(ExitError::Usage) => {
      return 2;
    }
    Err(ExitError::Runtime(error)) => {
      let _ = writeln!(stderr, "{error}");
      return 1;
    }
  }
}

/// Derive a stable program name from the argv[0] path.
///
/// # Arguments
/// - `argv0`: The raw argv[0] string.
///
/// # Returns
/// A best-effort program name for usage output.
fn program_name_from_argv0(argv0: &str) -> String {
  let path = Path::new(argv0);
  let name = path
    .file_name()
    .and_then(|value| value.to_str())
    .unwrap_or(argv0);
  return name.to_string();
}

/// Print metadata about a decoded sound file.
///
/// # Arguments
/// - `stdout`: The output writer for metadata.
/// - `stderr`: The output writer for usage errors.
/// - `program_name`: The CLI program name used for usage output.
/// - `path`: A file path argument, if provided.
///
/// # Returns
/// Returns `Ok(())` when the command completes successfully.
///
/// # Errors
/// Returns `ExitError::Usage` when the path argument is missing.
/// Returns `ExitError::Runtime` when decoding fails.
fn cmd_info(
  stdout: &mut dyn Write,
  stderr: &mut dyn Write,
  program_name: &str,
  path: Option<String>,
) -> Result<(), ExitError> {
  let path = require_path(stderr, program_name, "info", path)?;
  let buffer = load_sound_buffer(&path).map_err(ExitError::Runtime)?;
  write_info(stdout, &path, &buffer);
  return Ok(());
}

/// Print metadata and render an ASCII waveform preview.
///
/// # Arguments
/// - `stdout`: The output writer for metadata and waveform output.
/// - `stderr`: The output writer for usage errors.
/// - `program_name`: The CLI program name used for usage output.
/// - `path`: A file path argument, if provided.
///
/// # Returns
/// Returns `Ok(())` when the command completes successfully.
///
/// # Errors
/// Returns `ExitError::Usage` when the path argument is missing.
/// Returns `ExitError::Runtime` when decoding fails.
fn cmd_view(
  stdout: &mut dyn Write,
  stderr: &mut dyn Write,
  program_name: &str,
  path: Option<String>,
) -> Result<(), ExitError> {
  let path = require_path(stderr, program_name, "view", path)?;
  let buffer = load_sound_buffer(&path).map_err(ExitError::Runtime)?;
  write_info(stdout, &path, &buffer);
  write_waveform(stdout, &buffer);
  return Ok(());
}

/// Decode and play a sound file through the default output device.
///
/// # Arguments
/// - `stdout`: The output writer for metadata.
/// - `stderr`: The output writer for usage errors.
/// - `program_name`: The CLI program name used for usage output.
/// - `path`: A file path argument, if provided.
///
/// # Returns
/// Returns `Ok(())` when the command completes successfully.
///
/// # Errors
/// Returns `ExitError::Usage` when the path argument is missing.
/// Returns `ExitError::Runtime` when decoding or playback fails.
fn cmd_play(
  stdout: &mut dyn Write,
  stderr: &mut dyn Write,
  program_name: &str,
  path: Option<String>,
) -> Result<(), ExitError> {
  let path = require_path(stderr, program_name, "play", path)?;
  let buffer = load_sound_buffer(&path).map_err(ExitError::Runtime)?;
  write_info(stdout, &path, &buffer);
  play_buffer(&buffer).map_err(ExitError::Runtime)?;
  return Ok(());
}

/// List available output devices.
///
/// # Arguments
/// - `stdout`: The output writer for device output.
///
/// # Returns
/// Returns `Ok(())` when listing succeeds.
///
/// # Errors
/// Returns an `AudioError` if device enumeration fails.
fn cmd_list_devices(stdout: &mut dyn Write) -> Result<(), AudioError> {
  let devices = enumerate_output_devices()?;

  if devices.is_empty() {
    let _ = writeln!(stdout, "no output devices found");
    return Ok(());
  }

  for device in devices {
    let default_marker = if device.is_default { "*" } else { " " };
    let _ = writeln!(stdout, "{default_marker} {}", device.name);
  }

  return Ok(());
}

/// Require a file path argument for a command.
///
/// # Arguments
/// - `stderr`: The output writer for usage output.
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
  stderr: &mut dyn Write,
  program_name: &str,
  command: &str,
  path: Option<String>,
) -> Result<String, ExitError> {
  let Some(path) = path else {
    let _ = writeln!(
      stderr,
      "usage: {program_name} {command} <path-to-wav-or-ogg>"
    );
    return Err(ExitError::Usage);
  };
  return Ok(path);
}

/// Load a sound file into a decoded `SoundBuffer` based on its file extension.
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

/// Print basic decoded metadata for a `SoundBuffer`.
///
/// # Arguments
/// - `stdout`: The output writer that receives metadata.
/// - `path`: The decoded source path for display purposes.
/// - `buffer`: The decoded sound buffer.
///
/// # Returns
/// Returns `()` after writing metadata.
fn write_info(stdout: &mut dyn Write, path: &str, buffer: &SoundBuffer) {
  let _ = writeln!(stdout, "path: {path}");
  let _ = writeln!(stdout, "sample_rate: {}", buffer.sample_rate());
  let _ = writeln!(stdout, "channels: {}", buffer.channels());
  let _ = writeln!(stdout, "frames: {}", buffer.frames());
  let _ = writeln!(stdout, "samples: {}", buffer.samples().len());
  let _ =
    writeln!(stdout, "duration_seconds: {:.3}", buffer.duration_seconds());
  return;
}

/// Render an ASCII waveform preview for a `SoundBuffer`.
///
/// The rendering uses a single channel (the first channel) and shows a
/// peak-per-column visualization, which is intended for quick human inspection
/// rather than precise analysis.
///
/// # Arguments
/// - `stdout`: The output writer that receives waveform output.
/// - `buffer`: The decoded sound buffer to visualize.
///
/// # Returns
/// Returns `()` after writing the visualization.
fn write_waveform(stdout: &mut dyn Write, buffer: &SoundBuffer) {
  write_waveform_from_parts(stdout, buffer.samples(), buffer.channels());
  return;
}

/// Render an ASCII waveform preview for decoded audio samples.
///
/// # Arguments
/// - `stdout`: The output writer that receives waveform output.
/// - `samples`: Interleaved `f32` samples.
/// - `channels`: Interleaved channel count.
///
/// # Returns
/// Returns `()` after writing the visualization.
fn write_waveform_from_parts(
  stdout: &mut dyn Write,
  samples: &[f32],
  channels: u16,
) {
  let width: usize = 64;
  let height: usize = 10;

  let channels = channels as usize;
  if samples.is_empty() || channels == 0 {
    let _ = writeln!(stdout, "<no samples>");
    return;
  }

  let frames = samples.len() / channels;
  if frames == 0 {
    let _ = writeln!(stdout, "<no frames>");
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
      let _ = write!(stdout, "{mark}");
    }
    let _ = writeln!(stdout);
  }

  return;
}

/// Fill an output writer from an interleaved sample buffer and cursor.
///
/// This helper clears the writer, advances the cursor, and writes samples until
/// the source buffer is exhausted.
///
/// # Arguments
/// - `writer`: The destination output writer for the current callback tick.
/// - `cursor`: A shared sample cursor tracking overall playback progress.
/// - `samples`: The source interleaved samples to play.
///
/// # Returns
/// Returns `()` after writing samples into the output buffer.
fn fill_output_writer_from_samples(
  writer: &mut dyn AudioOutputWriter,
  cursor: &AtomicUsize,
  samples: &[f32],
) {
  let writer_channels = writer.channels() as usize;
  let writer_frames = writer.frames();

  writer.clear();

  if writer_channels == 0 {
    return;
  }

  let write_samples = writer_frames.saturating_mul(writer_channels);
  let start = cursor.fetch_add(write_samples, Ordering::Relaxed);

  let source_total = samples.len();

  for frame in 0..writer_frames {
    for channel in 0..writer_channels {
      let sample_index = start
        .saturating_add(frame.saturating_mul(writer_channels))
        .saturating_add(channel);

      let value = samples.get(sample_index).copied().unwrap_or(0.0);
      if sample_index < source_total {
        writer.set_sample(frame, channel, value);
      }
    }
  }

  return;
}

/// Play a decoded `SoundBuffer` through the default output device.
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
      fill_output_writer_from_samples(
        writer,
        &cursor_for_callback,
        buffer_for_callback.samples(),
      );
      return;
    })?;

  let wait_seconds = buffer.duration_seconds() + 0.20;
  std::thread::sleep(Duration::from_secs_f32(wait_seconds));
  drop(_device);
  return Ok(());
}

/// Print usage text.
///
/// # Arguments
/// - `writer`: Output writer that receives usage text.
/// - `program_name`: The CLI program name shown in examples.
///
/// # Returns
/// Returns `()` after writing usage output.
fn write_usage(writer: &mut dyn Write, program_name: &str) {
  let _ = writeln!(writer, "usage:");
  let _ = writeln!(writer, "  {program_name} info <path>");
  let _ = writeln!(writer, "  {program_name} view <path>");
  let _ = writeln!(writer, "  {program_name} play <path>");
  let _ = writeln!(writer, "  {program_name} list-devices");
  return;
}

#[cfg(test)]
mod tests {
  use super::*;

  fn fixture_path(relative: &str) -> String {
    return Path::new(env!("CARGO_MANIFEST_DIR"))
      .join(relative)
      .to_string_lossy()
      .to_string();
  }

  /// CLI view MUST print metadata and a waveform preview.
  #[test]
  fn run_view_prints_waveform_preview_for_wav_fixture() {
    let wav_path = fixture_path(
      "../../crates/lambda-rs-platform/assets/audio/tone_s16_mono_44100.wav",
    );
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let code = run(
      vec!["lambda-audio".to_string(), "view".to_string(), wav_path],
      &mut stdout,
      &mut stderr,
    );

    assert_eq!(code, 0);
    let output = String::from_utf8(stdout).expect("stdout should be utf8");
    assert!(output.contains("sample_rate: 44100"));
    assert!(
      output.contains('#'),
      "waveform preview expected to contain at least one marker",
    );
    return;
  }

  /// CLI MUST return a runtime error for missing input files.
  #[test]
  fn run_returns_runtime_error_for_missing_files() {
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let code = run(
      vec![
        "lambda-audio".to_string(),
        "info".to_string(),
        "missing.wav".to_string(),
      ],
      &mut stdout,
      &mut stderr,
    );

    assert_eq!(code, 1);
    let error_output =
      String::from_utf8(stderr).expect("stderr should be utf8");
    assert!(error_output.contains("I/O error"), "{error_output}");
    return;
  }

  /// argv[0] parsing MUST return a stable program name.
  #[test]
  fn program_name_from_argv0_uses_file_name_when_present() {
    assert_eq!(
      program_name_from_argv0("/usr/bin/lambda-audio"),
      "lambda-audio"
    );
    assert_eq!(program_name_from_argv0("lambda-audio"), "lambda-audio");
    return;
  }

  /// CLI MUST print usage for unknown commands and return usage exit code.
  #[test]
  fn run_returns_usage_code_for_unknown_command() {
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let code = run(
      vec!["lambda-audio".to_string(), "unknown".to_string()],
      &mut stdout,
      &mut stderr,
    );

    assert_eq!(code, 2);
    assert!(String::from_utf8(stderr)
      .expect("stderr should be utf8")
      .contains("unknown command"));
    return;
  }

  /// CLI MUST print usage when invoked without a command.
  #[test]
  fn run_defaults_to_help_when_command_is_missing() {
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let code = run(vec!["lambda-audio".to_string()], &mut stdout, &mut stderr);

    assert_eq!(code, 0);
    let output = String::from_utf8(stdout).expect("stdout should be utf8");
    assert!(output.contains("usage:"));
    return;
  }

  /// Commands that require a path MUST return a usage error when missing.
  #[test]
  fn run_returns_usage_code_when_path_is_missing() {
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let code = run(
      vec!["lambda-audio".to_string(), "info".to_string()],
      &mut stdout,
      &mut stderr,
    );

    assert_eq!(code, 2);
    let error_output =
      String::from_utf8(stderr).expect("stderr should be utf8");
    assert!(error_output.contains("usage: lambda-audio info"));
    return;
  }

  /// Loading unsupported extensions MUST return an actionable error.
  #[test]
  fn load_sound_buffer_rejects_unsupported_extensions() {
    let result = load_sound_buffer("tests/fixtures/sound.xyz");
    assert!(matches!(result, Err(AudioError::UnsupportedFormat { .. })));
    return;
  }

  /// CLI MUST decode and print info for WAV fixtures.
  #[test]
  fn run_info_decodes_wav_fixture() {
    let wav_path = fixture_path(
      "../../crates/lambda-rs-platform/assets/audio/tone_s16_mono_44100.wav",
    );
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let code = run(
      vec![
        "lambda-audio".to_string(),
        "info".to_string(),
        wav_path.clone(),
      ],
      &mut stdout,
      &mut stderr,
    );

    assert_eq!(code, 0);
    let output = String::from_utf8(stdout).expect("stdout should be utf8");
    assert!(output.contains("sample_rate: 44100"));
    assert!(output.contains("channels: 1"));
    return;
  }

  /// CLI MUST decode and print info for OGG fixtures.
  #[test]
  fn run_info_decodes_ogg_fixture() {
    let ogg_path =
      fixture_path(
        "../../crates/lambda-rs-platform/assets/audio/slash_vorbis_stereo_48000.ogg",
      );
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();

    let code = run(
      vec![
        "lambda-audio".to_string(),
        "info".to_string(),
        ogg_path.clone(),
      ],
      &mut stdout,
      &mut stderr,
    );

    assert_eq!(code, 0);
    let output = String::from_utf8(stdout).expect("stdout should be utf8");
    assert!(output.contains("sample_rate: 48000"));
    assert!(output.contains("channels: 2"));
    return;
  }

  /// Waveform rendering MUST handle empty inputs.
  #[test]
  fn waveform_rendering_prints_placeholder_for_empty_buffers() {
    let mut stdout = Vec::new();
    write_waveform_from_parts(&mut stdout, &[], 1);
    assert_eq!(
      String::from_utf8(stdout)
        .expect("stdout should be utf8")
        .trim(),
      "<no samples>"
    );
    return;
  }

  /// Waveform rendering MUST handle buffers that do not contain full frames.
  #[test]
  fn waveform_rendering_prints_placeholder_for_zero_frames() {
    let mut stdout = Vec::new();
    write_waveform_from_parts(&mut stdout, &[0.25], 2);
    assert_eq!(
      String::from_utf8(stdout)
        .expect("stdout should be utf8")
        .trim(),
      "<no frames>"
    );
    return;
  }

  /// Output fill MUST advance the cursor and stop writing when samples are
  /// exhausted.
  #[test]
  fn output_fill_advances_cursor_and_stops_writing_when_exhausted() {
    #[derive(Default)]
    struct StubWriter {
      channels: u16,
      frames: usize,
      values: Vec<f32>,
    }

    impl StubWriter {
      fn new(channels: u16, frames: usize) -> Self {
        return Self {
          channels,
          frames,
          values: vec![0.0; channels as usize * frames],
        };
      }
    }

    impl AudioOutputWriter for StubWriter {
      fn channels(&self) -> u16 {
        return self.channels;
      }

      fn frames(&self) -> usize {
        return self.frames;
      }

      fn clear(&mut self) {
        self.values.fill(0.0);
        return;
      }

      fn set_sample(
        &mut self,
        frame_index: usize,
        channel_index: usize,
        sample: f32,
      ) {
        let channels = self.channels as usize;
        let index = frame_index
          .saturating_mul(channels)
          .saturating_add(channel_index);
        if index < self.values.len() {
          self.values[index] = sample;
        }
        return;
      }
    }

    let samples = [0.25, -0.25, 0.5, -0.5];
    let cursor = AtomicUsize::new(0);

    let mut writer = StubWriter::new(2, 2);
    fill_output_writer_from_samples(&mut writer, &cursor, &samples);
    assert_eq!(writer.values, samples);

    fill_output_writer_from_samples(&mut writer, &cursor, &samples);
    assert_eq!(writer.values, [0.0, 0.0, 0.0, 0.0]);
    return;
  }
}
