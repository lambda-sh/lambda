#![allow(clippy::needless_return)]

//! Internal audio backend abstractions used by `lambda-rs`.
//!
//! Applications MUST NOT depend on `lambda-rs-platform` directly. The types
//! exposed from this module are intended to support `lambda-rs` implementations
//! and MAY change between releases.

pub mod device;

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
