#![allow(clippy::needless_return)]

//! Internal audio dependency wrappers used by `lambda-rs`.
//!
//! This module is internal to the engine. Applications MUST NOT depend on
//! `lambda-rs-platform` directly.

#[cfg(feature = "audio-device")]
pub mod cpal;

#[cfg(any(feature = "audio-decode-wav", feature = "audio-decode-vorbis"))]
pub mod symphonia;
