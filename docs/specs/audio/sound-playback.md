---
title: "Sound Playback and Transport Controls"
document_id: "audio-sound-playback-2026-02-09"
status: "draft"
created: "2026-02-09T00:00:00Z"
last_updated: "2026-02-09T00:10:00Z"
version: "0.1.1"
engine_workspace_version: "2023.1.30"
wgpu_version: "26.0.1"
shader_backend_default: "naga"
winit_version: "0.29.10"
repo_commit: "e1150369fb5024e47d4b8a19c116c16f8fb9abad"
owners: ["lambda-sh"]
reviewers: ["engine", "rendering"]
tags: ["spec", "audio", "lambda-rs"]
---

# Sound Playback and Transport Controls

## Table of Contents

- [Summary](#summary)
- [Scope](#scope)
- [Terminology](#terminology)
- [Architecture Overview](#architecture-overview)
- [Design](#design)
  - [API Surface](#api-surface)
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

- Add the ability to play a decoded `SoundBuffer` through an initialized audio
  output device with basic transport controls: play, pause, stop.
- Provide a lightweight `SoundInstance` handle returned from `play_sound` for
  controlling and querying playback state.
- Support looping playback for a single active sound instance.
- Maintain backend-agnostic behavior by implementing playback scheduling in
  `lambda-rs` while using `lambda-rs-platform` only for the output device and
  callback transport.

Rationale
- A minimal playback layer is required for demos and manual validation beyond
  device initialization and file decoding.
- Single-sound playback establishes the transport and thread-safety model that
  future mixing work can extend.

## Scope

### Goals

- Play a `SoundBuffer` to completion through the default audio output device.
- Pause and resume playback without audible artifacts.
- Stop playback and reset the playback position.
- Query playback state (`playing`, `paused`, `stopped`).
- Enable and disable looping playback.
- Provide a runnable example demonstrating play/pause/stop and looping.

### Non-Goals

- Volume control.
- Pitch/speed control.
- Spatial audio.
- Multiple simultaneous sounds.
- Streaming decode (disk-backed or incremental decode).
- Resampling or general channel remapping.

## Terminology

- Transport controls: operations that control playback flow (play, pause, stop).
- Sound instance: a lightweight handle for controlling one playback slot.
- Playback cursor: the current sample index within an interleaved sample buffer.
- Real-time audio thread: the platform thread that runs the audio output
  callback and MUST be treated as latency-sensitive.

## Architecture Overview

- Crate `lambda` (package: `lambda-rs`)
  - Hosts the public playback API (`AudioContext`, `SoundInstance`) and the
    playback scheduler executed inside the audio callback.
  - MUST remain backend-agnostic and MUST NOT expose platform or vendor types.
- Crate `lambda_platform` (package: `lambda-rs-platform`)
  - Provides the output device and callback transport via
    `lambda_platform::audio::cpal`.

Data flow

```
application
  └── lambda::audio
        ├── SoundBuffer (decoded samples)
        ├── AudioContextBuilder::build() -> AudioContext
        │     └── AudioOutputDeviceBuilder::build_with_output_callback(...)
        └── AudioContext::play_sound(&SoundBuffer) -> SoundInstance
              └── SoundInstance::{play,pause,stop,set_looping,...}
                    └── transport commands -> playback scheduler (audio callback)
                          └── AudioOutputWriter::set_sample(...)
                                └── lambda_platform::audio::cpal (internal)
                                      └── cpal -> OS audio backend
```

## Design

### API Surface

This section describes the public API surface added to `lambda-rs`.

Module layout (new)

- `crates/lambda-rs/src/audio/playback.rs` (or `audio/playback/mod.rs`)
  - Defines `AudioContext`, `AudioContextBuilder`, `SoundInstance`, and
    `PlaybackState`.
- `crates/lambda-rs/src/audio/mod.rs`
  - Re-exports playback types when `audio-playback` is enabled.

Public API

```rust
// crates/lambda-rs/src/audio/playback.rs

/// A queryable playback state for a `SoundInstance`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlaybackState {
  Playing,
  Paused,
  Stopped,
}

/// A lightweight handle controlling the active sound playback slot.
pub struct SoundInstance {
  /* internal handle */
}

impl SoundInstance {
  /// Begin playback, or resume if paused.
  pub fn play(&mut self);
  /// Pause playback, preserving playback position.
  pub fn pause(&mut self);
  /// Stop playback and reset position to the start of the buffer.
  pub fn stop(&mut self);
  /// Enable or disable looping playback.
  pub fn set_looping(&mut self, looping: bool);
  /// Query the current state of this instance.
  pub fn state(&self) -> PlaybackState;
  /// Convenience query for `state() == PlaybackState::Playing`.
  pub fn is_playing(&self) -> bool;
  /// Convenience query for `state() == PlaybackState::Paused`.
  pub fn is_paused(&self) -> bool;
  /// Convenience query for `state() == PlaybackState::Stopped`.
  pub fn is_stopped(&self) -> bool;
}

/// A playback context owning an output device and one active playback slot.
pub struct AudioContext {
  /* internal device + playback scheduler state */
}

/// Builder for creating an `AudioContext`.
#[derive(Debug, Clone)]
pub struct AudioContextBuilder {
  sample_rate: Option<u32>,
  channels: Option<u16>,
  label: Option<String>,
}

impl AudioContextBuilder {
  pub fn new() -> Self;
  pub fn with_sample_rate(self, rate: u32) -> Self;
  pub fn with_channels(self, channels: u16) -> Self;
  pub fn with_label(self, label: &str) -> Self;
  pub fn build(self) -> Result<AudioContext, AudioError>;
}

impl AudioContext {
  /// Play a decoded `SoundBuffer` through this context.
  pub fn play_sound(
    &mut self,
    buffer: &SoundBuffer,
  ) -> Result<SoundInstance, AudioError>;
}
```

Notes
- `AudioContext` is the only way to use `SoundInstance` playback.
- The playback system MUST support exactly one active sound at a time.
- `AudioOutputDevice` remains available for direct callback use and is not
  replaced by this API.

### Behavior

Playback lifecycle

- `AudioContext::play_sound` MUST stop any currently active sound playback,
  reset the playback cursor, and begin playing the provided buffer.
- `SoundInstance::play` MUST transition the instance to `Playing`:
  - If the instance is `Paused`, playback MUST resume from the current cursor.
  - If the instance is `Stopped`, playback MUST start from the beginning.
  - If the instance is already `Playing`, the call MUST be a no-op.
- `SoundInstance::pause` MUST transition the instance to `Paused` and MUST
  preserve the cursor.
- `SoundInstance::stop` MUST transition the instance to `Stopped` and MUST
  reset the cursor to the start of the buffer.
- When the buffer is exhausted and looping is disabled, playback MUST
  transition to `Stopped` and MUST reset the cursor to the start of the buffer.

Looping

- `SoundInstance::set_looping(true)` MUST cause playback to wrap to the start
  when the end of the buffer is reached.
- `SoundInstance::set_looping(false)` MUST cause playback to stop at the end
  of the buffer on the next exhaustion event.
- Looping changes MUST take effect without requiring a restart.

Output behavior

- When `PlaybackState` is `Stopped` or `Paused`, the audio callback MUST write
  silence to the output buffer.
- When `PlaybackState` is `Playing`, the audio callback MUST write sequential
  interleaved samples from the active `SoundBuffer` into the output buffer.
- The callback MUST NOT allocate and MUST NOT block.

Artifact avoidance (transport de-clicking)

- Transitions between audible output and silence (pause, stop, completion, and
  resume) MUST apply a short gain ramp to prevent discontinuities.
- The ramp length SHOULD be fixed and short (for example, 64–256 frames) and
  MUST be applied entirely within the audio callback without allocation.

Sound instance validity

- Only the most recently returned `SoundInstance` for an `AudioContext` is
  considered active.
- Calls on an inactive `SoundInstance` MUST be no-ops.
- Queries on an inactive `SoundInstance` MUST return `PlaybackState::Stopped`.

### Validation and Errors

General rules

- All public APIs MUST return actionable, backend-agnostic errors and MUST NOT
  panic.
- The audio callback MUST NOT panic. Failures inside the callback MUST degrade
  to silence.

`AudioContextBuilder::build`

- MUST return `AudioError::NoDefaultDevice` when no default output device
  exists.
- MUST forward configuration and platform failures using the existing
  `AudioError` variants produced by output device initialization.

`AudioContext::play_sound`

- MUST return `AudioError::InvalidData` when the provided buffer has no
  samples, `sample_rate == 0`, or `channels == 0`.
- MUST return `AudioError::InvalidData` when the provided buffer is not
  compatible with the context output configuration.

Compatibility validation

- `SoundBuffer::sample_rate()` MUST equal the `AudioContext` output sample
  rate.
- `SoundBuffer::channels()` MUST equal the `AudioContext` output channel count.
- No resampling or channel remapping is performed.

### Cargo Features

This specification introduces a new granular feature in `lambda-rs` to gate
playback behavior and dependencies.

Crate `lambda-rs` (package: `lambda-rs`)

- New granular feature (disabled by default)
  - `audio-playback`: enables the `AudioContext` and `SoundInstance` playback
    API. This feature MUST compose `audio-output-device` and
    `audio-sound-buffer` internally.
- Existing umbrella feature (disabled by default)
  - `audio`: MUST include `audio-playback` for discoverability and to provide
    a complete audio surface.

Crate `lambda-rs-platform` (package: `lambda-rs-platform`)

- No new features are required. Playback uses the existing `audio-device`
  output callback transport.

Documentation

- `docs/features.md` MUST be updated in the implementation change that adds
  `audio-playback`.

## Constraints and Rules

- The playback system MUST support exactly one active sound instance.
- The callback MUST treat `SoundBuffer` samples as interleaved `f32` in nominal
  range `[-1.0, 1.0]` and MUST clamp any out-of-range values before writing.
- Playback MUST be deterministic for a given buffer and output configuration.
- The audio callback MUST avoid blocking, locking, and allocation.

## Performance Considerations

Recommendations

- Prefer fixed-capacity, non-blocking transport mechanisms for main-thread to
  audio-thread state changes.
  - Rationale: callback jitter and contention can cause audible dropouts.
- Keep the callback inner loop branch-light and avoid per-sample atomics by
  snapshotting state once per callback tick.
  - Rationale: reduces overhead and improves callback stability.

## Requirements Checklist

Functionality
- [ ] Feature flags defined (`audio-playback`)
- [ ] `SoundBuffer` plays to completion
- [ ] Transport controls implemented (play/pause/stop)
- [ ] Looping implemented
- [ ] Playback state query implemented
- [ ] Transport de-clicking implemented

API Surface
- [ ] `AudioContext` and `AudioContextBuilder` implemented
- [ ] `SoundInstance` implemented
- [ ] `lambda::audio` re-exports wired and feature-gated

Validation and Errors
- [ ] Buffer compatibility validation implemented
- [ ] Errors are actionable and backend-agnostic

Performance
- [ ] Callback does not allocate or block
- [ ] Shared-state communication avoids locks

Documentation and Examples
- [ ] `docs/features.md` updated
- [ ] Runnable example added demonstrating transport controls

## Verification and Testing

Unit tests

- Add focused unit tests for:
  - state transitions and idempotency
  - cursor reset behavior on stop and completion
  - looping wrap behavior
  - inactive instance no-op behavior

Commands

- `cargo test -p lambda-rs --features audio-playback -- --nocapture`

Manual checks

- Add an example runnable at `demos/audio/src/bin/sound_playback_transport.rs`
  that uses the fixture
  `crates/lambda-rs-platform/assets/audio/slash_vorbis_stereo_48000.ogg` and:
  - plays briefly
  - pauses and resumes
  - stops and restarts from the beginning
  - enables looping and verifies continuous playback for at least 1 second

Command

- `cargo run -p lambda-demos-audio --features audio-playback --bin sound_playback_transport`

## Compatibility and Migration

- No existing API surface is removed.
- The `lambda-rs` `audio` feature umbrella composition changes by adding
  `audio-playback`. This is not expected to break builds because `audio` is
  disabled by default, but it MAY increase compile time when `audio` is
  enabled.

## Changelog

- 2026-02-09 (v0.1.1) — Specify a concrete transport example and fixture path.
- 2026-02-09 (v0.1.0) — Initial draft.
