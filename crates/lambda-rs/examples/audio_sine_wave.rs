#![allow(clippy::needless_return)]
//! Audio output example that plays a short sine wave tone.
//!
//! This example is application-facing and uses only the `lambda-rs` API surface.

#[cfg(feature = "audio-output-device")]
use std::{
  thread,
  time::Duration,
};

#[cfg(feature = "audio-output-device")]
use lambda::audio::{
  enumerate_output_devices,
  AudioOutputDeviceBuilder,
};

#[cfg(not(feature = "audio-output-device"))]
fn main() {
  eprintln!(
    "This example requires `lambda-rs` feature `audio-output-device`.\n\n\
Run:\n  cargo run -p lambda-rs --example audio_sine_wave --features audio-output-device"
  );
}

#[cfg(feature = "audio-output-device")]
fn main() {
  let devices = match enumerate_output_devices() {
    Ok(devices) => devices,
    Err(error) => {
      eprintln!("Failed to enumerate audio output devices: {error:?}");
      return;
    }
  };

  println!("Available output devices:");
  for device in devices {
    let default_marker = if device.is_default { " (default)" } else { "" };
    println!("  - {}{default_marker}", device.name);
  }

  let frequency_hz = 440.0f32;
  let amplitude = 0.10f32;
  let mut phase_radians = 0.0f32;

  let device = AudioOutputDeviceBuilder::new()
    .with_label("audio_sine_wave")
    .build_with_output_callback(move |writer, callback_info| {
      let phase_delta = std::f32::consts::TAU * frequency_hz
        / (callback_info.sample_rate as f32);

      for frame_index in 0..writer.frames() {
        let sample = (phase_radians.sin()) * amplitude;
        for channel_index in 0..(writer.channels() as usize) {
          writer.set_sample(frame_index, channel_index, sample);
        }

        phase_radians += phase_delta;
        if phase_radians > std::f32::consts::TAU {
          phase_radians -= std::f32::consts::TAU;
        }
      }

      return;
    });

  let _device = match device {
    Ok(device) => device,
    Err(error) => {
      eprintln!("Failed to initialize audio output device: {error:?}");
      return;
    }
  };

  println!("Playing 440 Hz sine wave for 2 seconds...");
  thread::sleep(Duration::from_secs(2));
  println!("Done.");
}
