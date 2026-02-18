//! Integration tests for `lambda-rs`.
//!
//! This file is the primary integration-test entrypoint. Feature-specific
//! integration tests live in dedicated modules under `tests/`.

#![allow(clippy::needless_return)]

#[cfg(feature = "physics-2d")]
mod physics_2d;
