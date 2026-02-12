#![allow(clippy::needless_return)]

//! Single-sound playback and transport controls.
//!
//! This module provides a minimal, backend-agnostic playback facade that
//! supports one active `SoundBuffer` at a time.

mod callback;
mod context;
mod transport;

use callback::PlaybackController;
use transport::{
  CommandQueue,
  PlaybackCommand,
  PlaybackCommandQueue,
  PlaybackSharedState,
};

const DEFAULT_GAIN_RAMP_FRAMES: usize = 128;
const DEFAULT_OUTPUT_SAMPLE_RATE: u32 = 48_000;
const DEFAULT_OUTPUT_CHANNELS: u16 = 2;
const MAX_PLAYBACK_CHANNELS: usize = 8;
const PLAYBACK_COMMAND_CAPACITY: usize = 256;

/// A queryable playback state for a `SoundInstance`.
///
/// This state is observable from the application thread and is intended to
/// provide basic transport visibility.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlaybackState {
  /// The sound is currently playing.
  Playing,
  /// The sound is currently paused.
  Paused,
  /// The sound is stopped and positioned at the start.
  Stopped,
}

pub use context::{
  AudioContext,
  AudioContextBuilder,
  SoundInstance,
};
