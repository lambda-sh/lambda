#![allow(clippy::needless_return)]
//! Audio example that plays the bundled "slash" OGG Vorbis fixture.
//!
//! This example validates that `SoundBuffer` decoding and audio output playback
//! can be composed together using only the `lambda-rs` API surface.

#[cfg(all(
  feature = "audio-output-device",
  feature = "audio-sound-buffer-vorbis"
))]
use std::{
  sync::{
    atomic::{
      AtomicUsize,
      Ordering,
    },
    Arc,
  },
  time::Duration,
};

#[cfg(all(
  feature = "audio-output-device",
  feature = "audio-sound-buffer-vorbis"
))]
use lambda::audio::{
  AudioOutputDeviceBuilder,
  SoundBuffer,
};

#[cfg(all(
  feature = "audio-output-device",
  feature = "audio-sound-buffer-vorbis"
))]
fn main() {
  const SLASH_VORBIS_STEREO_48000_OGG: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../lambda-rs-platform/assets/audio/slash_vorbis_stereo_48000.ogg"
  ));

  let buffer =
    SoundBuffer::from_ogg_bytes(SLASH_VORBIS_STEREO_48000_OGG).unwrap();

  let cursor = Arc::new(AtomicUsize::new(0));
  let buffer = Arc::new(buffer);

  let cursor_for_callback = cursor.clone();
  let buffer_for_callback = buffer.clone();

  let _device = AudioOutputDeviceBuilder::new()
    .with_label("play-slash-sound")
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

      for frame in 0..writer_frames {
        for channel in 0..writer_channels {
          let sample_index = start
            .saturating_add(frame.saturating_mul(writer_channels))
            .saturating_add(channel);
          let value = source_samples.get(sample_index).copied().unwrap_or(0.0);
          writer.set_sample(frame, channel, value);
        }
      }

      return;
    })
    .unwrap();

  std::thread::sleep(Duration::from_secs_f32(buffer.duration_seconds() + 0.20));
  drop(_device);
  return;
}

#[cfg(not(all(
  feature = "audio-output-device",
  feature = "audio-sound-buffer-vorbis"
)))]
fn main() {
  eprintln!(
    "This example requires `lambda-rs` features `audio-output-device` and \
`audio-sound-buffer-vorbis`.\n\n\
Run:\n  cargo run -p lambda-rs --example play_slash_sound \\\n\
    --features audio-output-device,audio-sound-buffer-vorbis"
  );
  return;
}
