#![allow(clippy::needless_return)]

//! Application-facing audio APIs.
//!
//! This module provides backend-agnostic audio APIs for Lambda applications.
//! Platform and vendor details are implemented in `lambda-rs-platform` and MUST
//! NOT be exposed through the `lambda-rs` public API.

mod error;
pub use error::AudioError;

#[cfg(any(
  feature = "audio-sound-buffer",
  feature = "audio-sound-buffer-wav",
  feature = "audio-sound-buffer-vorbis"
))]
mod buffer;
#[cfg(any(
  feature = "audio-sound-buffer",
  feature = "audio-sound-buffer-wav",
  feature = "audio-sound-buffer-vorbis"
))]
pub use buffer::SoundBuffer;

#[cfg(feature = "audio-output-device")]
pub mod devices;

#[cfg(feature = "audio-output-device")]
pub use devices::output::*;
