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
//! See runnable examples in `crates/lambda-rs/examples/` and integration tests
//! under `crates/lambda-rs/tests/` for typical usage patterns.

pub mod component;
pub mod events;
pub mod math;
pub mod render;
pub mod runtime;
pub mod runtimes;

/// The logging module provides a simple logging interface for Lambda
/// applications.
pub use logging;
