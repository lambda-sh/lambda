#![allow(clippy::needless_return)]

//! Application-facing audio APIs.
//!
//! This module provides backend-agnostic audio APIs for Lambda applications.
//! Platform and vendor details are implemented in `lambda-rs-platform` and MUST
//! NOT be exposed through the `lambda-rs` public API.

mod error;
pub use error::AudioError;

#[cfg(feature = "audio-output-device")]
pub mod devices;

#[cfg(feature = "audio-output-device")]
pub use devices::output::*;
