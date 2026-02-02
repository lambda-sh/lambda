#![allow(clippy::needless_return)]

//! Audio device APIs.
//!
//! This module hosts device surfaces for audio input and output. Output is
//! implemented first; input devices are expected to be added later.
//!
//! All public types in this module MUST remain backend-agnostic. Platform and
//! vendor details are implemented in `lambda-rs-platform` and MUST NOT be
//! exposed through this surface.

pub mod output;
