#![allow(clippy::needless_return)]

//! Internal audio dependency wrappers used by `lambda-rs`.
//!
//! This module is internal to the engine. Applications MUST NOT depend on
//! `lambda-rs-platform` directly.
//!
//! Each dependency wrapper is gated by a granular Cargo feature in this crate.
//! Feature checks are scoped to the smallest possible surface so that
//! `lambda-rs` can compose audio features without exposing vendor details.

#[cfg(feature = "audio-device")]
pub mod cpal;

#[cfg(any(feature = "audio-decode-wav", feature = "audio-decode-vorbis"))]
pub mod symphonia;
