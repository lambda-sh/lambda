#![allow(clippy::needless_return)]
//! Lambda is a simple, fast, and safe engine for desktop applications and rendering in Rust.
//!
//! High‑level modules
//! - `render`: windowed rendering built on top of platform abstractions with
//!   explicit command encoding and ergonomic builders.
//! - `runtimes`: application runtime helpers that create windows and drive the
//!   event/render loop.
//! - `math`: minimal vector/matrix utilities used by examples and helpers.
//!
//! See runnable demos under `demos/` and the minimal rustdoc reference example
//! under `crates/lambda-rs/examples/` for typical usage patterns. Integration
//! tests live under `crates/lambda-rs/tests/`.

pub mod component;
pub mod events;
pub mod math;
pub mod pod;
pub mod render;
pub mod runtime;
pub mod runtimes;
pub mod util;

#[cfg(any(
  feature = "audio-output-device",
  feature = "audio-sound-buffer",
  feature = "audio-sound-buffer-wav",
  feature = "audio-sound-buffer-vorbis"
))]
pub mod audio;

/// The logging module provides a simple logging interface for Lambda
/// applications.
pub use logging;
