---
title: "Audio Device Abstraction"
document_id: "audio-device-abstraction-2026-01-28"
status: "draft"
created: "2026-01-28T22:59:00Z"
last_updated: "2026-02-07T00:00:00Z"
version: "0.1.18"
engine_workspace_version: "2023.1.30"
wgpu_version: "26.0.1"
shader_backend_default: "naga"
winit_version: "0.29.10"
repo_commit: "544444652b4dc3639f8b3e297e56c302183a7a0b"
owners: ["lambda-sh"]
reviewers: ["engine", "rendering"]
tags: ["spec", "audio", "lambda-rs", "platform", "cpal"]
---

# Audio Device Abstraction

## Table of Contents

- [Summary](#summary)
- [Scope](#scope)
- [Terminology](#terminology)
- [Architecture Overview](#architecture-overview)
- [Design](#design)
  - [API Surface](#api-surface)
  - [lambda-rs Public API](#lambda-rs-public-api)
  - [Application Interaction](#application-interaction)
  - [Behavior](#behavior)
  - [Validation and Errors](#validation-and-errors)
  - [Cargo Features](#cargo-features)
- [Constraints and Rules](#constraints-and-rules)
- [Performance Considerations](#performance-considerations)
- [Requirements Checklist](#requirements-checklist)
- [Verification and Testing](#verification-and-testing)
- [Compatibility and Migration](#compatibility-and-migration)
- [Changelog](#changelog)

## Summary

- Add an application-facing audio output device API in `lambda-rs` that can
  enumerate output devices and initialize the default output device.
- Implement the audio backend behind `lambda-rs-platform` so `lambda-rs` can
  remain backend-agnostic and avoid leaking platform/vendor types.
- Establish a builder-based public API (`AudioOutputDeviceBuilder`) consistent
  with existing `lambda-rs` patterns.
- Use `cpal` as the cross-platform audio backend while preventing platform and
  vendor details from leaking into the `lambda-rs` public API.

## Scope

### Goals

- Provide a `lambda-rs` audio output device handle (`AudioOutputDevice`) for
  application use.
- Provide `enumerate_output_devices` to enumerate available audio output
  devices.
- Provide `AudioOutputDeviceBuilder` to initialize a default audio output
  device with configurable sample rate and channel count.
- Provide a minimal output callback mechanism sufficient to validate audible
  playback via a deterministic example (test tone generation).
- Provide actionable error reporting for device discovery and initialization.
- Support Windows, macOS, and Linux.
- Provide a `lambda-rs-platform` implementation layer that supplies the
  backend-specific device and stream behavior required by `lambda-rs`.

### Non-Goals

- Audio input/recording.
- High-level sound playback systems (asset decoding, mixing, scheduling).
- Audio effects processing (DSP, filters, spatialization).

## Terminology

- Audio output device: an operating-system audio endpoint capable of playback.
- Stream: a running audio callback driving audio samples to an output device.
- Sample rate: audio frames per second (for example, 48_000 Hz).
- Channels: the number of interleaved output channels (for example, 2).
- Sample format: the primitive sample representation used by a stream callback
  (for example, `f32`).

## Architecture Overview

- Crate `lambda` (package: `lambda-rs`)
  - `audio` module provides the application-facing API for output device access.
  - The public API MUST remain backend-agnostic and MUST NOT expose `cpal` or
    `lambda-rs-platform` types.
- Crate `lambda_platform` (package: `lambda-rs-platform`)
  - `cpal` module provides internal implementations used by `lambda-rs`.
  - `cpal::device` wraps `cpal` device discovery and stream creation.
  - The backend dependency MUST be pinned to `cpal = "=0.17.1"`.

Data flow

```
application
  └── lambda::audio
        ├── enumerate_output_devices() -> Vec<AudioOutputDeviceInfo>
        └── AudioOutputDeviceBuilder::build() -> AudioOutputDevice
              └── lambda_platform::audio::cpal (internal)
                    ├── enumerate_devices() -> Vec<AudioDeviceInfo>
                    └── AudioDeviceBuilder::build() -> AudioDevice
                          └── cpal (host + device + stream)
                                └── OS audio backend (CoreAudio/WASAPI/ALSA/Pulse/JACK)
```

## Design

### API Surface

This section describes the platform layer surface used by `lambda-rs`
implementations. Applications MUST NOT depend on `lambda-rs-platform` or use
its audio APIs directly.

Module layout

- `crates/lambda-rs-platform/src/audio/cpal/mod.rs`
  - Re-exports `AudioDevice`, `AudioDeviceBuilder`, `AudioDeviceInfo`,
    `AudioError`, and `enumerate_devices`.
- `crates/lambda-rs-platform/src/audio/cpal/device.rs`
  - Defines `AudioDevice`, `AudioDeviceBuilder`, `AudioDeviceInfo`,
    `AudioError`, and `enumerate_devices`.

Public API

```rust
// crates/lambda-rs-platform/src/audio/cpal/device.rs

/// Output sample format used by the platform stream callback.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AudioSampleFormat {
  F32,
  I16,
  U16,
}

/// Information available to audio output callbacks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AudioCallbackInfo {
  pub sample_rate: u32,
  pub channels: u16,
  pub sample_format: AudioSampleFormat,
}

/// Real-time writer for audio output buffers.
///
/// This trait MUST be implemented without allocation and MUST write into the
/// underlying device output buffer for the current callback invocation.
pub trait AudioOutputWriter {
  fn channels(&self) -> u16;
  fn frames(&self) -> usize;
  fn clear(&mut self);

  /// Write a normalized sample in the range `[-1.0, 1.0]`.
  ///
  /// Implementations MUST clamp values outside `[-1.0, 1.0]`.
  fn set_sample(
    &mut self,
    frame_index: usize,
    channel_index: usize,
    sample: f32,
  );
}

/// Metadata describing an available audio output device.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AudioDeviceInfo {
  /// Human-readable device name.
  pub name: String,
  /// Whether this device is the current default output device.
  pub is_default: bool,
}

/// An initialized audio output device.
///
/// This type is an opaque platform wrapper. It MUST NOT expose `cpal` types.
pub struct AudioDevice {
  /* platform handle + chosen output configuration */
}

/// Builder for creating an `AudioDevice`.
#[derive(Debug, Clone)]
pub struct AudioDeviceBuilder {
  sample_rate: Option<u32>,
  channels: Option<u16>,
  label: Option<String>,
}

impl AudioDeviceBuilder {
  /// Create a builder with engine defaults.
  pub fn new() -> Self;

  /// Request a specific sample rate (Hz).
  pub fn with_sample_rate(self, rate: u32) -> Self;

  /// Request a specific channel count.
  pub fn with_channels(self, channels: u16) -> Self;

  /// Attach a label for diagnostics.
  pub fn with_label(self, label: &str) -> Self;

  /// Initialize the default audio output device using the requested
  /// configuration.
  pub fn build(self) -> Result<AudioDevice, AudioError>;

  /// Initialize the default audio output device and play audio via a callback.
  pub fn build_with_output_callback<Callback>(
    self,
    callback: Callback,
  ) -> Result<AudioDevice, AudioError>
  where
    Callback: 'static + Send + FnMut(&mut dyn AudioOutputWriter, AudioCallbackInfo);
}

/// Enumerate available audio output devices.
pub fn enumerate_devices() -> Result<Vec<AudioDeviceInfo>, AudioError>;
```

### lambda-rs Public API

`lambda-rs` provides the application-facing audio API and translates to
`lambda_platform::audio::cpal` (package: `lambda-rs-platform`) internally. The
`lambda-rs` layer MUST remain backend-agnostic and MUST NOT expose `cpal`
types.

Crate boundary

- Applications MUST use `lambda-rs` for audio and MUST NOT use
  `lambda-rs-platform` directly.
- `lambda-rs-platform` audio APIs are internal implementation details and MAY
  change without regard for application compatibility.
- `lambda::audio` MUST remain backend-agnostic and MUST NOT require direct
  use of `lambda-rs-platform` types by applications.

Application-facing API surface

```rust
// crates/lambda-rs/src/audio/devices/output.rs
// crates/lambda-rs/src/audio/error.rs

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AudioSampleFormat {
  F32,
  I16,
  U16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AudioCallbackInfo {
  pub sample_rate: u32,
  pub channels: u16,
  pub sample_format: AudioSampleFormat,
}

#[derive(Clone, Debug)]
pub enum AudioError {
  InvalidSampleRate { requested: u32 },
  InvalidChannels { requested: u16 },
  Io {
    path: Option<std::path::PathBuf>,
    details: String,
  },
  UnsupportedFormat { details: String },
  InvalidData { details: String },
  DecodeFailed { details: String },
  NoDefaultDevice,
  UnsupportedConfig {
    requested_sample_rate: Option<u32>,
    requested_channels: Option<u16>,
  },
  UnsupportedSampleFormat { details: String },
  Platform { details: String },
}

/// Metadata describing an available audio output device.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AudioOutputDeviceInfo {
  pub name: String,
  pub is_default: bool,
}

pub trait AudioOutputWriter {
  fn channels(&self) -> u16;
  fn frames(&self) -> usize;
  fn clear(&mut self);
  fn set_sample(
    &mut self,
    frame_index: usize,
    channel_index: usize,
    sample: f32,
  );
}

pub struct AudioOutputDevice {
  /* opaque platform handle + stream lifetime */
}

#[derive(Debug, Clone)]
pub struct AudioOutputDeviceBuilder {
  sample_rate: Option<u32>,
  channels: Option<u16>,
  label: Option<String>,
}

impl AudioOutputDeviceBuilder {
  pub fn new() -> Self;
  pub fn with_sample_rate(self, rate: u32) -> Self;
  pub fn with_channels(self, channels: u16) -> Self;
  pub fn with_label(self, label: &str) -> Self;
  pub fn build(self) -> Result<AudioOutputDevice, AudioError>;

  pub fn build_with_output_callback<Callback>(
    self,
    callback: Callback,
  ) -> Result<AudioOutputDevice, AudioError>
  where
    Callback: 'static + Send + FnMut(&mut dyn AudioOutputWriter, AudioCallbackInfo);
}

/// Enumerate available audio output devices via the platform layer.
pub fn enumerate_output_devices(
) -> Result<Vec<AudioOutputDeviceInfo>, AudioError>;
```

Implementation rules

- `lambda::audio` MUST translate into `lambda_platform::audio::cpal` (package:
  `lambda-rs-platform`) internally.
- `lambda::audio` MUST define its own public types and MUST NOT re-export
  `lambda-rs-platform` audio types.
- `lambda::audio::AudioError` MUST remain backend-agnostic and MUST NOT
  expose `cpal` types.

Features

- `lambda-rs` granular feature: `audio-output-device` (default: disabled)
  - Enables the `lambda::audio` output device surface.
  - Enables `lambda-rs-platform` `audio-device` internally.
- `lambda-rs` umbrella feature: `audio` (default: disabled)
  - Composes `audio-output-device` and `audio-sound-buffer`.

### Application Interaction

This section describes the intended application-facing workflow via
`lambda::audio`.

Initialization flow

- An application SHOULD enumerate devices to present names in diagnostics
  output.
- An application SHOULD create exactly one default output device during
  startup.
- The application MUST keep the returned device handle alive for as long as
  audio output is required. Dropping the device MUST stop output.

Device enumeration

```rust
let devices = lambda::audio::enumerate_output_devices()?;
for device in devices {
  println!(
    "audio: {}{}",
    device.name,
    if device.is_default { " (default)" } else { "" }
  );
}
```

Default device initialization (deterministic test tone)

```rust
let mut phase: f32 = 0.0;
let frequency_hz: f32 = 440.0;

let _audio_output = lambda::audio::AudioOutputDeviceBuilder::new()
  .with_sample_rate(48_000)
  .with_channels(2)
  .build_with_output_callback(move |writer, info| {
    let channels = info.channels as usize;
    let frames = writer.frames();
    let phase_step = 2.0 * std::f32::consts::PI * frequency_hz
      / info.sample_rate as f32;

    for frame_index in 0..frames {
      let sample = phase.sin() * 0.10;
      phase += phase_step;

      for channel_index in 0..channels {
        writer.set_sample(frame_index, channel_index, sample);
      }
    }
  })?;
```

Runtime interaction

- An application using `ApplicationRuntime` SHOULD create and store the audio
  device handle in the `main` scope before calling `start_runtime`, or store it
  in application-owned state that outlives the runtime event loop.
- `lambda-rs` components SHOULD NOT rely on blocking operations or locks in
  the audio callback.

Minimal application sketch

```rust
use lambda::runtime::start_runtime;
use lambda::runtimes::ApplicationRuntimeBuilder;

fn main() -> Result<(), lambda::audio::AudioError> {
  let _audio_output = lambda::audio::AudioOutputDeviceBuilder::new()
    .build()?;

  let runtime = ApplicationRuntimeBuilder::new("Lambda App").build();
  start_runtime(runtime);
  return Ok(());
}
```

### Behavior

Device enumeration (`lambda::audio::enumerate_output_devices`)

- `enumerate_output_devices` MUST return only output-capable devices.
- `enumerate_output_devices` MUST include the default output device when one
  exists.
- `AudioOutputDeviceInfo::is_default` MUST be `true` only for the device that
  matches the current default output device at the time of enumeration.
- `enumerate_output_devices` MUST NOT panic.

Default device initialization (`AudioOutputDeviceBuilder::build`)

- `build` MUST select the operating-system default output device.
- If no default output device exists, `build` MUST return `AudioError::NoDefaultDevice`.
- `build` MUST validate the requested configuration against the device’s
  supported output configurations.
- When `sample_rate` is not specified, `build` MUST prefer 48_000 Hz when
  supported and otherwise clamp to the nearest supported rate within the chosen
  configuration range.
- When `channels` is not specified, `build` MUST NOT filter by channel count.
- `build` MUST create an output stream that produces silence (all samples set
  to zero) and MUST keep the stream alive for the lifetime of
  `AudioOutputDevice`.

Default device initialization with output callback

- `build_with_output_callback` MUST create an output stream that invokes the
  callback to fill the output buffer.
- The output callback MUST write samples using `AudioOutputWriter`.
- `AudioOutputWriter` MUST write into an interleaved output buffer.
- `build_with_output_callback` MUST keep the stream alive for the lifetime of
  `AudioOutputDevice`.
- The callback MUST be invoked on a real-time audio thread and MUST be treated
  as real-time code.

Output writer semantics

- `AudioOutputWriter::frames` MUST return the number of frames in the output
  buffer for the current callback invocation.
- `AudioOutputWriter::clear` MUST set the entire output buffer to silence:
  - `AudioSampleFormat::F32`: `0.0`
  - `AudioSampleFormat::I16`: `0`
  - `AudioSampleFormat::U16`: `32768`
- `AudioOutputWriter::set_sample` MUST treat the provided `sample` as a
  normalized value in the range `[-1.0, 1.0]`.
- Implementations MUST clamp values outside `[-1.0, 1.0]`.
- Sample conversion MUST follow these rules:
  - `AudioSampleFormat::F32`: write the clamped value directly.
  - `AudioSampleFormat::I16`: map `[-1.0, 1.0]` to `[-32767, 32767]` and write.
  - `AudioSampleFormat::U16`: map `[-1.0, 1.0]` to `[0, 65535]` where `0.0`
    maps to `32768`.
- `set_sample` MUST NOT panic on out-of-range indices. It MUST perform no write
  for out-of-range indices and SHOULD use `debug_assertions` diagnostics.

Configuration selection rules

- When `sample_rate` is specified, `build` MUST select a supported output
  configuration whose sample-rate range contains the requested value.
- When `channels` is specified, `build` MUST select a supported output
  configuration whose channel count equals the requested value.
- When multiple configurations satisfy the request, `build` SHOULD choose the
  configuration with a sample format that is most widely supported by backends
  (for example, `f32`) and SHOULD prefer 48_000 Hz when tied.
- If no configuration satisfies the request, `build` MUST return
  `AudioError::UnsupportedConfig`.
- `build_with_output_callback` MUST select a supported stream sample format
  and expose it via `AudioCallbackInfo::sample_format`.
- If the selected stream format is not one of `AudioSampleFormat::{F32, I16, U16}`,
  `build_with_output_callback` MUST return `AudioError::UnsupportedSampleFormat`.

### Validation and Errors

Error type

- `lambda-rs` MUST define an `AudioError` error enum suitable for actionable
  diagnostics.
- `lambda::audio::AudioError` MUST remain backend-agnostic and MUST NOT
  expose `cpal` or `lambda-rs-platform` types.
- `lambda-rs-platform` MUST define an internal `AudioError` suitable for
  actionable diagnostics inside the platform layer.
- `lambda_platform::audio::cpal::AudioError` (package: `lambda-rs-platform`) MUST NOT
  expose `cpal` types in its public API.
- `lambda-rs` MUST translate `lambda_platform::audio::cpal::AudioError` (package:
  `lambda-rs-platform`) into `lambda::audio::AudioError`. Backend-specific
  failures SHOULD map to `AudioError::Platform { details }`.

Platform `AudioError` variants (internal)

- `InvalidSampleRate { requested: u32 }`
- `InvalidChannels { requested: u16 }`
- `HostUnavailable { details: String }`
- `NoDefaultDevice`
- `DeviceNameUnavailable { details: String }`
- `DeviceEnumerationFailed { details: String }`
- `SupportedConfigsUnavailable { details: String }`
- `UnsupportedConfig { requested_sample_rate: Option<u32>, requested_channels: Option<u16> }`
- `UnsupportedSampleFormat { details: String }`
- `StreamBuildFailed { details: String }`
- `StreamPlayFailed { details: String }`

Validation rules

- `AudioOutputDeviceBuilder::build` MUST return `AudioError::InvalidSampleRate`
  when `sample_rate == Some(0)`.
- `AudioOutputDeviceBuilder::build` MUST return `AudioError::InvalidChannels`
  when `channels == Some(0)`.

### Cargo Features

Features introduced by this spec

- Crate: `lambda-rs`
  - Granular feature: `audio-output-device` (default: disabled)
    - Enables `lambda::audio` output device APIs.
    - Enables `lambda-rs-platform` `audio-device` internally.
  - Umbrella feature: `audio` (default: disabled)
    - Composes `audio-output-device` and `audio-sound-buffer`.
- Crate: `lambda-rs-platform`
  - Granular feature: `audio-device` (default: disabled)
    - Enables the `cpal` module and the `AudioDevice`/`AudioDeviceBuilder`
      surface.
    - Enables the `cpal` dependency as an internal implementation detail.
  - Umbrella feature: `audio` (default: disabled)
    - Composes `audio-device`, `audio-decode-wav`, and `audio-decode-vorbis`.

Feature gating requirements

- `lambda-rs` MUST gate all application-facing audio output behavior behind
  `audio-output-device`.
- `lambda-rs-platform` MUST gate all `cpal` usage behind `audio-device`.
- The `audio` umbrella feature MUST NOT be used to gate behavior in code; it
  MUST only compose granular audio features.

## Constraints and Rules

- `AudioOutputDevice` MUST NOT expose platform backends or vendor details through
  public types, fields, or feature names.
- `AudioOutputDevice` MUST maintain ownership of the output stream such that
  dropping `AudioOutputDevice` stops the stream.
- The stream callback MUST be real-time safe:
  - It MUST NOT allocate.
  - It MUST NOT lock unbounded mutexes.
  - It MUST NOT perform I/O.
- User-provided output callbacks MUST follow the same real-time safety rules as
  the stream callback.
- Linux builds with `audio-output-device` enabled MUST provide ALSA development
  headers and `pkg-config` so the `alsa-sys` dependency can link successfully
  (for example, `libasound2-dev` on Debian/Ubuntu).

## Performance Considerations

- Recommendations
  - The silent stream callback SHOULD write zeros using a tight loop over the
    output buffer.
    - Rationale: avoids allocations and minimizes CPU overhead.
  - Device enumeration SHOULD avoid per-device expensive probing beyond device
    name and default-device matching.
    - Rationale: enumeration may be called during initialization and should not
      stall startup.

## Requirements Checklist

- Functionality
  - [x] Feature flags defined (`lambda-rs`: `audio-output-device`, `audio`)
        (`crates/lambda-rs/Cargo.toml`)
  - [x] Feature flags defined (`lambda-rs-platform`: `audio-device`, `audio`)
        (`crates/lambda-rs-platform/Cargo.toml`)
  - [x] `enumerate_output_devices` implemented and returns output devices
        (`crates/lambda-rs/src/audio/devices/output.rs`)
  - [x] `AudioOutputDeviceBuilder::build` initializes default output device
        (`crates/lambda-rs/src/audio/devices/output.rs`,
        `crates/lambda-rs-platform/src/audio/cpal/device.rs`)
  - [x] `AudioOutputDeviceBuilder::build_with_output_callback` invokes callback
        (`crates/lambda-rs/src/audio/devices/output.rs`,
        `crates/lambda-rs-platform/src/audio/cpal/device.rs`)
  - [x] Stream created and kept alive for `AudioOutputDevice` lifetime
        (`crates/lambda-rs/src/audio/devices/output.rs`,
        `crates/lambda-rs-platform/src/audio/cpal/device.rs`)
  - [x] Platform enumeration implemented (`lambda_platform::audio::cpal`)
        (`crates/lambda-rs-platform/src/audio/cpal/device.rs`)
  - [x] Platform builder implemented (`lambda_platform::audio::cpal`)
        (`crates/lambda-rs-platform/src/audio/cpal/device.rs`)
- API Surface
  - [x] Public `lambda` types implemented: `AudioOutputDevice`,
        `AudioOutputDeviceInfo`, `AudioOutputDeviceBuilder`, `AudioCallbackInfo`,
        `AudioOutputWriter`, `AudioError`
        (`crates/lambda-rs/src/audio/devices/output.rs`,
        `crates/lambda-rs/src/audio/error.rs`)
  - [x] Internal platform types implemented: `AudioDevice`, `AudioDeviceInfo`,
        `AudioDeviceBuilder`, `AudioCallbackInfo`, `AudioOutputWriter`, `AudioError`
        (`crates/lambda-rs-platform/src/audio/cpal/device.rs`)
  - [x] `lambda::audio` does not re-export `lambda-rs-platform` types
        (`crates/lambda-rs/src/audio/devices/output.rs`,
        `crates/lambda-rs/src/audio/mod.rs`)
- Validation and Errors
  - [x] Invalid builder inputs rejected (sample rate and channel count)
        (`crates/lambda-rs-platform/src/audio/cpal/device.rs`)
  - [x] Descriptive `AudioError` variants emitted on failures
        (`crates/lambda-rs/src/audio/error.rs`,
        `crates/lambda-rs-platform/src/audio/cpal/device.rs`)
  - [x] Unsupported configurations reported via `AudioError::UnsupportedConfig`
        (`crates/lambda-rs-platform/src/audio/cpal/device.rs`,
        `crates/lambda-rs/src/audio/error.rs`)
- Documentation and Examples
  - [x] `docs/features.md` updated with audio feature documentation
        (`docs/features.md`)
  - [x] Example added demonstrating audible playback (behind `audio-output-device`)
        (`demos/audio/src/bin/sine_wave.rs`)
  - [x] `lambda-rs` audio facade implemented (`crates/lambda-rs/src/audio/mod.rs`)

## Verification and Testing

Example (lambda-rs facade)

This example is the primary application-facing reference.

- Add `demos/audio/src/bin/sine_wave.rs` (crate: `lambda-demos-audio`) that:
  - Prints `lambda::audio::enumerate_output_devices()` output.
  - Builds the default output device via the facade builder and plays a
    deterministic 440 Hz tone for at least 2 seconds.

Unit tests (crate: `lambda-rs-platform`)

- Builder defaults
  - `AudioDeviceBuilder::new` sets `sample_rate` and `channels` to `None`.
  - `with_sample_rate` and `with_channels` override requested values.
  - Invalid values (`0`) are rejected.
- Enumeration
  - `enumerate_devices` returns `Result<_, _>` and does not panic.

Commands

- `cargo test -p lambda-rs -- --nocapture`
- `cargo test -p lambda-rs-platform --features audio-device -- --nocapture`

Manual checks

- Run the `lambda-rs` facade example and confirm audible playback for at least
  2 seconds.
  - `cargo run -p lambda-demos-audio --bin sine_wave`

## Compatibility and Migration

- None. No existing audio APIs exist in the workspace.

## Changelog

- 2026-02-05 (v0.1.18) — Update demo and example references for `demos/`.
- 2026-02-02 (v0.1.17) — Align specification file references with the current
  `lambda::audio` module layout and feature composition.
- 2026-01-31 (v0.1.15) — Update verification command to include
  `audio-output-device`.
- 2026-01-30 (v0.1.14) — Make `lambda-rs` audio features opt-in by default and
  update CI to test Linux audio builds explicitly.
- 2026-01-30 (v0.1.13) — Document Linux system dependencies required by the
  default audio backend.
- 2026-01-30 (v0.1.12) — Populate the requirements checklist with file
  references matching the implemented surface.
- 2026-01-30 (v0.1.11) — Align examples with the `lambda` crate name, document
  the internal `lambda_platform::audio::cpal` path and pin, and refine default
  configuration selection requirements to match the implementation.
- 2026-01-30 (v0.1.10) — Enable `lambda-rs` audio features by default.
- 2026-01-29 (v0.1.9) — Fix YAML front matter to use a single `version` field.
- 2026-01-29 (v0.1.8) — Make the `lambda-rs` facade example the primary
  reference and remove the platform example requirement.
- 2026-01-29 (v0.1.7) — Rename the platform audio implementation module to
  `lambda_platform::audio::cpal` (package: `lambda-rs-platform`) to reflect the
  internal backend.
- 2026-01-29 (v0.1.6) — Specify `lambda-rs` as the only supported
  application-facing API and treat `lambda-rs-platform` as internal.
- 2026-01-29 (v0.1.5) — Specify how `lambda-rs` applications enumerate devices
  and initialize the default output device.
- 2026-01-29 (v0.1.4) — Refine specification language and define output writer
  conversion semantics.
- 2026-01-29 (v0.1.3) — Refine callback API language and specify `AudioOutputWriter`.
- 2026-01-28 (v0.1.2) — Specify `f32` callback constraints and add a minimal
  playback sketch.
- 2026-01-28 (v0.1.1) — Add `lambda-rs` exposure and playback example sections.
- 2026-01-28 (v0.1.0) — Initial draft.
