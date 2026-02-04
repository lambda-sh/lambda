#![allow(clippy::needless_return)]

/// Entry point for the `lambda-audio` tool binary.
fn main() {
  let mut stdout = std::io::stdout();
  let mut stderr = std::io::stderr();
  let exit_code =
    lambda_audio_tool::run(std::env::args(), &mut stdout, &mut stderr);
  std::process::exit(exit_code);
}
