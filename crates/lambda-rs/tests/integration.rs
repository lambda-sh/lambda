//! Integration tests for `lambda-rs`.
//!
//! This file is the primary integration-test entrypoint. Feature-specific
//! integration tests live in dedicated modules under `tests/`.

#![allow(clippy::needless_return)]
// Tests in this repository use explicit `return` statements for consistency
// with the engine's style guidelines.

#[cfg(feature = "physics-2d")]
mod physics_2d;
