#![allow(clippy::needless_return)]
//! Audio demo exercising `SoundInstance` gain and pitch controls via a CLI.
//!
//! This demo validates that:
//! - `SoundInstance::set_volume` affects playback loudness
//! - `SoundInstance::set_pitch` affects playback speed and frequency
//! - `AudioContext::set_master_volume` affects all playback output
//!
//! Note: this demo intentionally uses discrete jumps (no fades) to match the
//! current feature scope.

use std::{
  io::{
    BufRead,
    Write,
  },
  time::Duration,
};

use args::{
  ArgsError,
  Argument,
  ArgumentParser,
  ArgumentType,
};
use lambda::audio::{
  AudioContextBuilder,
  SoundBuffer,
};

fn main() {
  let parser = build_cli();
  let argv: Vec<String> = std::env::args().collect();
  let usage = parser.usage();

  let parsed = match parser.parse(&argv) {
    Ok(parsed) => parsed,
    Err(ArgsError::HelpRequested(help)) => {
      print!("{}", help);
      return;
    }
    Err(error) => {
      eprintln!("{}\n{}", error, usage);
      return;
    }
  };

  let master_volume = parsed.get_f32("--master-volume").unwrap_or(0.25);

  const SLASH_VORBIS_STEREO_48000_OGG: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../crates/lambda-rs-platform/assets/audio/slash_vorbis_stereo_48000.ogg"
  ));

  let buffer =
    SoundBuffer::from_ogg_bytes(SLASH_VORBIS_STEREO_48000_OGG).unwrap();

  let mut context = AudioContextBuilder::new()
    .with_label("sound-instance-gain-pitch")
    .with_sample_rate(buffer.sample_rate())
    .with_channels(buffer.channels())
    .build()
    .unwrap();

  // Start with a conservative master volume to avoid unexpectedly loud output.
  context.set_master_volume(master_volume);

  let mut instance = context.play_sound(&buffer).unwrap();

  match parsed.subcommand() {
    Some(("repl", sub)) => {
      let initial_volume = sub.get_f32("--volume").unwrap_or(1.0);
      let initial_pitch = sub.get_f32("--pitch").unwrap_or(1.0);
      let looping = sub.get_bool("--looping").unwrap_or(false);

      instance.set_volume(initial_volume);
      instance.set_pitch(initial_pitch);
      instance.set_looping(looping);

      run_repl(&mut context, &mut instance, looping);
    }
    Some(("play", sub)) => {
      let volume = sub.get_f32("--volume").unwrap_or(1.0);
      let pitch = sub.get_f32("--pitch").unwrap_or(1.0);
      let looping = sub.get_bool("--looping").unwrap_or(false);
      let duration_ms = sub.get_i64("--duration-ms").unwrap_or(2_000);

      instance.set_volume(volume);
      instance.set_pitch(pitch);
      instance.set_looping(looping);

      if duration_ms > 0 {
        std::thread::sleep(Duration::from_millis(duration_ms as u64));
      }
    }
    Some(("script", _)) | None => {
      run_script(&mut context, &mut instance);
    }
    Some((name, _sub)) => {
      eprintln!("Unknown subcommand: {}\n{}", name, usage);
      return;
    }
  }

  return;
}

fn build_cli() -> ArgumentParser {
  let root = ArgumentParser::new("sound_instance_gain_pitch")
    .with_description(
      "Play a built-in sound and interact with transport/gain/pitch controls.",
    )
    .with_argument(
      Argument::new("--master-volume").with_type(ArgumentType::Float),
    )
    .with_subcommand(
      ArgumentParser::new("script").with_description("Run a scripted sequence"),
    )
    .with_subcommand(
      ArgumentParser::new("play")
        .with_description("Play with fixed settings for a duration")
        .with_argument(Argument::new("--volume").with_type(ArgumentType::Float))
        .with_argument(Argument::new("--pitch").with_type(ArgumentType::Float))
        .with_argument(
          Argument::new("--looping").with_type(ArgumentType::Boolean),
        )
        .with_argument(
          Argument::new("--duration-ms").with_type(ArgumentType::Integer),
        ),
    )
    .with_subcommand(
      ArgumentParser::new("repl")
        .with_description("Interactive mode (commands on stdin)")
        .with_argument(Argument::new("--volume").with_type(ArgumentType::Float))
        .with_argument(Argument::new("--pitch").with_type(ArgumentType::Float))
        .with_argument(
          Argument::new("--looping").with_type(ArgumentType::Boolean),
        ),
    );

  return root;
}

fn run_script(
  _context: &mut lambda::audio::AudioContext,
  instance: &mut lambda::audio::SoundInstance,
) {
  std::thread::sleep(Duration::from_millis(300));

  // Instance volume: silence, normal, amplified.
  instance.set_volume(0.0);
  std::thread::sleep(Duration::from_millis(250));

  instance.set_volume(1.0);
  std::thread::sleep(Duration::from_millis(250));

  instance.set_volume(2.0);
  std::thread::sleep(Duration::from_millis(300));

  instance.set_volume(1.0);
  std::thread::sleep(Duration::from_millis(250));

  // Pitch: half speed then double speed.
  instance.set_pitch(0.5);
  std::thread::sleep(Duration::from_millis(600));

  instance.set_pitch(2.0);
  std::thread::sleep(Duration::from_millis(600));

  instance.set_pitch(1.0);
  std::thread::sleep(Duration::from_millis(250));

  std::thread::sleep(Duration::from_millis(500));
  return;
}

fn run_repl(
  context: &mut lambda::audio::AudioContext,
  instance: &mut lambda::audio::SoundInstance,
  mut looping: bool,
) {
  print_repl_help();
  print_status(context, instance, looping);

  let stdin = std::io::stdin();
  let mut stdout = std::io::stdout();
  let mut lines = stdin.lock().lines();

  loop {
    let _ = write!(stdout, "> ");
    let _ = stdout.flush();

    let Some(Ok(line)) = lines.next() else {
      break;
    };

    let trimmed = line.trim();
    if trimmed.is_empty() {
      continue;
    }

    let mut parts = trimmed.split_whitespace();
    let Some(command) = parts.next() else {
      continue;
    };

    match command {
      "help" | "h" | "?" => {
        print_repl_help();
      }
      "status" | "s" => {
        print_status(context, instance, looping);
      }
      "play" => {
        instance.play();
      }
      "pause" => {
        instance.pause();
      }
      "stop" => {
        instance.stop();
      }
      "loop" => {
        let value = parts.next().unwrap_or("");
        let enabled = matches!(value, "1" | "true" | "on" | "yes");
        let disabled = matches!(value, "0" | "false" | "off" | "no");

        if enabled {
          looping = true;
          instance.set_looping(true);
        } else if disabled {
          looping = false;
          instance.set_looping(false);
        } else {
          eprintln!("usage: loop on|off");
        }
      }
      "volume" | "vol" => {
        let Some(value) = parts.next() else {
          eprintln!("usage: volume <f32>");
          continue;
        };

        match value.parse::<f32>() {
          Ok(volume) => instance.set_volume(volume),
          Err(_) => eprintln!("invalid volume: {}", value),
        }
      }
      "pitch" => {
        let Some(value) = parts.next() else {
          eprintln!("usage: pitch <f32>");
          continue;
        };

        match value.parse::<f32>() {
          Ok(pitch) => instance.set_pitch(pitch),
          Err(_) => eprintln!("invalid pitch: {}", value),
        }
      }
      "master" => {
        let Some(value) = parts.next() else {
          eprintln!("usage: master <f32>");
          continue;
        };

        match value.parse::<f32>() {
          Ok(volume) => context.set_master_volume(volume),
          Err(_) => eprintln!("invalid master volume: {}", value),
        }
      }
      "sleep" => {
        let Some(value) = parts.next() else {
          eprintln!("usage: sleep <ms>");
          continue;
        };

        match value.parse::<u64>() {
          Ok(ms) => std::thread::sleep(Duration::from_millis(ms)),
          Err(_) => eprintln!("invalid sleep duration: {}", value),
        }
      }
      "quit" | "exit" | "q" => {
        break;
      }
      other => {
        eprintln!("unknown command: {}", other);
        eprintln!("type `help` for commands");
      }
    }
  }

  return;
}

fn print_repl_help() {
  println!("Commands:");
  println!("  help                     Show this help");
  println!("  status                   Show current settings");
  println!("  play | pause | stop      Transport controls");
  println!("  loop on|off              Toggle looping");
  println!("  volume <f32>             Set instance volume (0.0..1.0+)");
  println!("  pitch <f32>              Set instance pitch (speed) (e.g. 0.5, 1.0, 2.0)");
  println!("  master <f32>             Set master volume (0.0..1.0+)");
  println!("  sleep <ms>               Sleep for a duration");
  println!("  quit                     Exit");
  return;
}

fn print_status(
  context: &lambda::audio::AudioContext,
  instance: &lambda::audio::SoundInstance,
  looping: bool,
) {
  println!(
    "state={:?} looping={} volume={} pitch={} master={}",
    instance.state(),
    looping,
    instance.volume(),
    instance.pitch(),
    context.master_volume()
  );
  return;
}
