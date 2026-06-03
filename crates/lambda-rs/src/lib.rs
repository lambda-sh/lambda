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

#[cfg(feature = "physics-2d")]
pub mod physics;

#[cfg(any(
  feature = "profiling-cpu",
  feature = "profiling-gpu",
  feature = "profiling-export-summary",
  feature = "profiling-export-trace"
))]
/// Profiling APIs and user-authored instrumentation macros.
///
/// ```no_run
/// use std::error::Error;
///
/// use lambda::profiling::{
///   profile_counter,
///   profile_scope,
///   Profiler,
///   ProfilerBuilder,
/// };
///
/// fn main() -> Result<(), Box<dyn Error>> {
///   let output_directory = std::env::temp_dir().join("lambda-profile-docs");
///   let profiler = ProfilerBuilder::new("docs-example")
///     .with_output_directory(&output_directory)
///     .with_summary_export(true)
///     .with_trace_export(true)
///     .build()?;
///
///   profiler.install_global()?;
///
///   {
///     let _scope = profile_scope!("docs.example_work");
///     profile_counter!("docs.items_processed", 4);
///   }
///
///   if let Some(profiler) = Profiler::global() {
///     let _artifacts = profiler.flush_frame()?;
///   }
///
///   return Ok(());
/// }
/// ```
pub use lambda_rs_profiling as profiling;
/// The logging module provides a simple logging interface for Lambda
/// applications.
pub use logging;
