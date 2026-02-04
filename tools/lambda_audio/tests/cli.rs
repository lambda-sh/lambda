#![allow(clippy::needless_return)]

use std::process::Command;

/// `lambda-audio --help` MUST succeed and print usage.
#[test]
fn lambda_audio_help_prints_usage() {
  let exe = env!("CARGO_BIN_EXE_lambda-audio");

  let output = Command::new(exe)
    .arg("--help")
    .output()
    .expect("failed to run lambda-audio");

  assert!(output.status.success());

  let stdout = String::from_utf8_lossy(&output.stdout);
  assert!(stdout.contains("usage:"));
  return;
}
