#![allow(clippy::needless_return)]

//! Internal audio backend abstractions used by `lambda-rs` (cpal backend).
//!
//! This module provides a stable, backend-agnostic surface that is implemented
//! using `cpal` when the `audio-device` feature is enabled.

/// `cpal`-backed output device discovery and stream initialization.
pub mod device;

/// Re-export the backend-agnostic surface types for consumption by `lambda-rs`.
pub use device::{
  enumerate_devices,
  AudioCallbackInfo,
  AudioDevice,
  AudioDeviceBuilder,
  AudioDeviceInfo,
  AudioError,
  AudioOutputWriter,
  AudioSampleFormat,
};
