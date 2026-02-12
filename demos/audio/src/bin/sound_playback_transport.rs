#![allow(clippy::needless_return)]
//! Audio demo exercising `AudioContext` transport controls.
//!
//! This demo validates that `AudioContext` can play a decoded `SoundBuffer`
//! through the output device and that `SoundInstance` transport operations
//! (play/pause/stop/looping) behave as expected.

use std::time::Duration;

use lambda::audio::{
  AudioContextBuilder,
  SoundBuffer,
};

fn main() {
  const SLASH_VORBIS_STEREO_48000_OGG: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../crates/lambda-rs-platform/assets/audio/slash_vorbis_stereo_48000.ogg"
  ));

  let buffer =
    SoundBuffer::from_ogg_bytes(SLASH_VORBIS_STEREO_48000_OGG).unwrap();

  let mut context = AudioContextBuilder::new()
    .with_label("sound-playback-transport")
    .with_sample_rate(buffer.sample_rate())
    .with_channels(buffer.channels())
    .build()
    .unwrap();

  let mut instance = context.play_sound(&buffer).unwrap();
  std::thread::sleep(Duration::from_millis(250));

  instance.pause();
  std::thread::sleep(Duration::from_millis(250));

  instance.play();
  std::thread::sleep(Duration::from_millis(250));

  instance.stop();
  std::thread::sleep(Duration::from_millis(250));

  instance.play();
  std::thread::sleep(Duration::from_millis(300));

  instance.set_looping(true);
  std::thread::sleep(Duration::from_secs(2));

  instance.set_looping(false);
  std::thread::sleep(Duration::from_millis(300));
  return;
}
