#![allow(clippy::needless_return)]

//! Internal audio backend abstractions used by `lambda-rs`.

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
