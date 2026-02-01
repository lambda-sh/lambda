#![allow(clippy::needless_return)]
//! Cross‑platform abstractions and utilities used by Lambda.
//!
//! This crate hosts thin wrappers around `winit` (windowing) and `wgpu`
//! (graphics) that provide consistent defaults and ergonomic builders, along
//! with shader compilation backends and small helper modules (e.g., OBJ
//! loading and random number generation).
//!
//! Stability: this is an internal support layer for `lambda-rs`. Public
//! types are exposed as a convenience to the higher‑level crate and MAY change
//! between releases to fit engine needs.
pub mod obj;
pub mod rand;
pub mod shader;
#[cfg(feature = "wgpu")]
pub mod wgpu;
pub mod winit;

#[cfg(any(
  feature = "audio",
  feature = "audio-device",
  feature = "audio-decode-wav",
  feature = "audio-decode-vorbis"
))]
pub mod audio;
