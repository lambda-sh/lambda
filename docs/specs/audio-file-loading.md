---
title: "Audio File Loading (SoundBuffer)"
document_id: "audio-file-loading-2026-01-31"
status: "draft"
created: "2026-01-31T22:07:49Z"
last_updated: "2026-01-31T23:03:17Z"
version: "0.2.0"
engine_workspace_version: "2023.1.30"
wgpu_version: "26.0.1"
shader_backend_default: "naga"
winit_version: "0.29.10"
repo_commit: "7cf8891f861a625b989f3751fd61674d072a53fe"
owners: ["lambda-sh"]
reviewers: ["engine", "rendering"]
tags: ["spec", "audio", "lambda-rs", "platform", "assets"]
---

# Audio File Loading (SoundBuffer)

## Table of Contents

- [Summary](#summary)
- [Scope](#scope)
- [Terminology](#terminology)
- [Architecture Overview](#architecture-overview)
- [Design](#design)
  - [API Surface](#api-surface)
  - [lambda-rs Public API](#lambda-rs-public-api)
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

- Add the ability to load audio files from disk or in-memory bytes into a
  decoded `SoundBuffer` suitable for future playback and mixing.
- Implement application-facing APIs in `lambda-rs` while placing codec
  dependencies behind `lambda-rs-platform` wrappers to avoid leaking vendor
  types into the public API.
- Support common formats starting with WAV and OGG Vorbis.

## Scope

### Goals

- Load WAV files into decoded audio buffers.
- Load OGG Vorbis files into decoded audio buffers.
- Provide a `SoundBuffer` type holding decoded audio data (`f32` samples).
- Support loading from file path and from memory bytes.
- Provide actionable, backend-agnostic error reporting for unsupported formats
  and decoding failures.

### Non-Goals

- MP3 support.
- Streaming large files (incremental decode, disk-backed buffers).
- Audio playback.

## Terminology

- SoundBuffer: a fully decoded, in-memory buffer of audio samples suitable for
  immediate use by a playback or mixing system.
- Sample: a single channel value in nominal range `[-1.0, 1.0]`.
- Frame: one sample per channel at a given time (for example, stereo has 2
  samples per frame).
- Interleaved: sample order is per-frame with channels adjacent (for example,
  `L0, R0, L1, R1, ...`).
- WAV: Waveform Audio File Format, typically PCM or IEEE float samples.
- OGG Vorbis: Ogg container format carrying Vorbis-compressed audio.

## Architecture Overview

- Crate `lambda` (package: `lambda-rs`)
  - `audio` module provides the application-facing `SoundBuffer` API.
  - The public API MUST remain backend-agnostic and MUST NOT expose `symphonia`
    or `lambda-rs-platform` types.
- Crate `lambda_platform` (package: `lambda-rs-platform`)
  - `audio::symphonia` module provides a WAV and OGG Vorbis decoding wrapper,
    backed by `symphonia` 0.5.5.
  - These modules are internal dependency wrappers and MAY change between
    releases.

Data flow

```
application
  └── lambda::audio::SoundBuffer
        ├── from_wav_file / from_wav_bytes
        │     └── lambda_platform::audio::symphonia (internal)
        └── from_ogg_file / from_ogg_bytes
              └── lambda_platform::audio::symphonia (internal)
```

## Design

### API Surface

This section describes the platform layer surface used by `lambda-rs`
implementations. Applications MUST NOT depend on `lambda-rs-platform` or use
its decoding APIs directly.

Module layout (new)

- `crates/lambda-rs-platform/src/audio/symphonia/mod.rs`
  - Provides WAV and OGG Vorbis decode wrappers used by `lambda-rs`.
  - The wrapper MUST use `symphonia` 0.5.5 and MUST disable non-required
    decoders and demuxers via dependency feature configuration.

Platform data model

The platform layer MUST return decoded audio in a backend-agnostic shape that
can be converted into `lambda::audio::SoundBuffer` without exposing codec
types.

```rust
// crates/lambda-rs-platform/src/audio_decode.rs (module name selected in
// implementation)

#[derive(Clone, Debug, PartialEq)]
pub struct DecodedAudio {
  pub samples: Vec<f32>,
  pub sample_rate: u32,
  pub channels: u16,
}

#[derive(Clone, Debug)]
pub enum AudioDecodeError {
  UnsupportedFormat { details: String },
  InvalidData { details: String },
  DecodeFailed { details: String },
}
```

Notes

- The implementation MAY avoid adding a shared `DecodedAudio` module and MAY
  instead implement format-specific decode functions returning an equivalent
  internal struct.
- The platform error type MUST implement `Display` and MUST NOT include vendor
  error types in variants.

### lambda-rs Public API

The `SoundBuffer` API MUST be implemented in `lambda-rs`.

Module layout (new)

- `crates/lambda-rs/src/audio/buffer.rs` (new)
  - Defines `SoundBuffer` and its file/byte loading entry points.
- `crates/lambda-rs/src/audio/mod.rs` (existing module; file layout MAY be
  converted from `audio.rs` to `audio/mod.rs` to host submodules).

Public API

```rust
// crates/lambda-rs/src/audio/buffer.rs

pub struct SoundBuffer {
  samples: Vec<f32>,
  sample_rate: u32,
  channels: u16,
}

impl SoundBuffer {
  pub fn from_wav_file(path: &std::path::Path) -> Result<Self, AudioError>;
  pub fn from_wav_bytes(bytes: &[u8]) -> Result<Self, AudioError>;
  pub fn from_ogg_file(path: &std::path::Path) -> Result<Self, AudioError>;
  pub fn from_ogg_bytes(bytes: &[u8]) -> Result<Self, AudioError>;

  pub fn sample_rate(&self) -> u32;
  pub fn channels(&self) -> u16;
  pub fn duration_seconds(&self) -> f32;
}
```

### Behavior

- `SoundBuffer` samples MUST be interleaved `f32` samples in nominal range
  `[-1.0, 1.0]`.
- `from_*_file` MUST read the entire file into memory and decode it.
  - Rationale: streaming is an explicit non-goal for this work item.
- `from_*_bytes` MUST decode from the provided byte slice without attempting
  any filesystem access.
- `duration_seconds` MUST be computed as:
  - `frames = samples.len() / channels`
  - `duration = frames as f32 / sample_rate as f32`
- WAV decoding MUST support:
  - mono and stereo
  - 16-bit PCM, 24-bit PCM, and 32-bit float sample representations
- OGG decoding MUST support:
  - OGG Vorbis in mono and stereo

### Validation and Errors

The public API MUST return actionable, backend-agnostic errors.

The existing `lambda::audio::AudioError` MUST be extended to represent decode
and I/O errors produced by sound buffer loading.

Error behavior

- Unsupported formats MUST return an explicit error variant indicating that
  the input format is not supported (for example, a WAV with an unsupported
  bit depth or a non-Vorbis Ogg stream).
- Invalid or corrupted input MUST return an explicit error variant indicating
  invalid data.
- Filesystem read failures MUST return an explicit error variant indicating an
  I/O failure and SHOULD include the input path in the error details.
- Errors MUST NOT panic.
- Errors MUST NOT expose vendor types.

### Cargo Features

Audio behavior in this workspace is opt-in and controlled via Cargo features.
This specification introduces new granular features that MUST be wired into
existing umbrella features.

Crate `lambda-rs` (package: `lambda-rs`)

- New granular features (disabled by default)
  - `audio-sound-buffer-wav`: enables `SoundBuffer::from_wav_*`.
  - `audio-sound-buffer-vorbis`: enables `SoundBuffer::from_ogg_*`.
- New umbrella feature (disabled by default)
  - `audio-sound-buffer`: composes `audio-sound-buffer-wav` and
    `audio-sound-buffer-vorbis`.
- Existing umbrella feature (disabled by default)
  - `audio`: MUST compose `audio-output-device` and `audio-sound-buffer`.

Crate `lambda-rs-platform` (package: `lambda-rs-platform`)

- New granular features (disabled by default)
  - `audio-decode-wav`: enables WAV decode support via the `symphonia` wrapper.
  - `audio-decode-vorbis`: enables OGG Vorbis decode support via the `symphonia`
    wrapper.
- Existing umbrella feature (disabled by default)
  - `audio`: MUST compose `audio-device`, `audio-decode-wav`, and
    `audio-decode-vorbis`.

Feature gating rules

- The `lambda::audio` module MUST be compiled when either `audio-output-device`
  or `audio-sound-buffer` is enabled.
- Format-specific entry points SHOULD be gated behind the corresponding
  granular features and MUST return a deterministic error if called when the
  required feature is disabled (if the symbol remains available).
- `docs/features.md` MUST be updated in the implementation change that adds
  these features.

## Constraints and Rules

- `SoundBuffer` MUST store decoded samples as `f32` to support future mixing
  and processing without requiring format-specific sample conversions.
- `SoundBuffer` MUST store sample rate and channel count from the decoded
  source.
- Loading functions MUST reject inputs with `channels == 0` or
  `sample_rate == 0` with a validation error.
- Audio decode dependencies MUST only be referenced from `lambda-rs-platform`
  modules located under `crates/lambda-rs-platform/src/audio/symphonia/`.

## Performance Considerations

Recommendations

- Decode paths SHOULD decode directly into the destination `Vec<f32>` without
  additional intermediate allocations where feasible.
  - Rationale: file loading already requires full-buffer allocation; extra
    copies scale linearly with file size.
- Loading functions SHOULD use `Vec::try_reserve` (or equivalent) to surface
  allocation errors as `AudioError` rather than panicking.
  - Rationale: buffer sizes depend on file contents and may exceed memory
    availability.

## Requirements Checklist

- Functionality
  - [ ] WAV decode implemented (16-bit PCM, 24-bit PCM, 32-bit float)
  - [ ] OGG Vorbis decode implemented
  - [ ] Load-from-file and load-from-bytes supported
- API Surface
  - [ ] `SoundBuffer` public API implemented in `lambda-rs`
  - [ ] `lambda-rs` does not expose vendor/platform decode types
  - [ ] `lambda::audio` module is available when sound-buffer features enabled
- Validation and Errors
  - [ ] Unsupported formats return actionable errors
  - [ ] Corrupt data returns actionable errors
  - [ ] File I/O errors return actionable errors
- Documentation and Examples
  - [ ] `docs/features.md` updated with new features and defaults
  - [ ] Minimal example loads a sound file and prints metadata
- Tests
  - [ ] Unit tests cover WAV mono and stereo
  - [ ] Unit tests cover OGG Vorbis mono and stereo
  - [ ] Test assets are stored under `crates/lambda-rs/assets/`

For each checked item, include a reference to a commit, pull request, or file
path that demonstrates the implementation.

## Verification and Testing

### Unit Tests

Coverage targets

- WAV
  - mono 16-bit PCM
  - stereo 16-bit PCM
  - mono 24-bit PCM
  - stereo 32-bit float
- OGG Vorbis
  - mono
  - stereo

Commands

- `cargo test -p lambda-rs -- --nocapture`
- `cargo test --workspace`

### Example

- Add `crates/lambda-rs/examples/sound_buffer_load.rs`.
- The example SHOULD load a file path provided via CLI args and print:
  - channels
  - sample rate
  - duration

## Compatibility and Migration

- Adding decoding variants to `lambda::audio::AudioError` is a source-breaking
  change for applications that match the enum exhaustively.
  - Migration: add a wildcard match arm or handle the new variants explicitly.
- No other user-visible behavior changes are required.

## Changelog

- 2026-01-31 (v0.2.0) — Center decoding on `symphonia` 0.5.5.
- 2026-01-31 (v0.1.1) — Align spec with platform audio module layout.
- 2026-01-31 (v0.1.0) — Initial draft.
