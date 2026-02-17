---
title: "Sound Instance Gain and Pitch Controls"
document_id: "audio-sound-instance-gain-pitch-2026-02-15"
status: "draft"
created: "2026-02-15T00:00:00Z"
last_updated: "2026-02-15T00:00:00Z"
version: "0.1.0"
engine_workspace_version: "2023.1.30"
wgpu_version: "26.0.1"
shader_backend_default: "naga"
winit_version: "0.29.10"
repo_commit: "6f96052fae896a095b658f29af1eff96e5aaa348"
owners: ["lambda-sh"]
reviewers: ["engine"]
tags: ["spec", "audio", "lambda-rs"]
---

# Sound Instance Gain and Pitch Controls

## Table of Contents

- [Summary](#summary)
- [Scope](#scope)
- [Terminology](#terminology)
- [Architecture Overview](#architecture-overview)
- [Design](#design)
  - [API Surface](#api-surface)
  - [Behavior](#behavior)
  - [Validation and Errors](#validation-and-errors)
- [Constraints and Rules](#constraints-and-rules)
- [Performance Considerations](#performance-considerations)
- [Acceptance Criteria](#acceptance-criteria)
- [Requirements Checklist](#requirements-checklist)
- [Verification and Testing](#verification-and-testing)
- [Compatibility and Migration](#compatibility-and-migration)
- [Changelog](#changelog)

## Summary

- Add per-`SoundInstance` volume (gain) and pitch (playback speed) controls for
  basic audio manipulation.
- Add global/master volume control on `AudioContext` that affects all playback.

Rationale
- These controls are the minimum needed for comfortable iteration on demos and
  gameplay (balancing loudness, simple “slow/fast” effects) without committing
  to a full mixer/effects stack.
- For v1, pitch shifting via resampling is acceptable and is explicitly treated
  as “speed changes affect both speed and frequency”.

Prerequisites
- This specification depends on the baseline playback API and callback
  scheduler described in `docs/specs/audio/sound-playback.md`.

Affects
- Crates: `lambda-rs`, `lambda-rs-platform`

## Scope

### Goals

- Set volume/gain per `SoundInstance` (`0.0` to `1.0`+).
- Set pitch/playback speed per `SoundInstance`.
- Support global/master volume on `AudioContext`.

### Non-Goals

- Audio effects (reverb, echo).
- Per-channel volume.
- Volume fading/transitions.
- Time-stretch pitch shifting (changing pitch without changing speed).
- High-quality resampling kernels (linear interpolation is sufficient for v1).

## Terminology

- Gain/volume: a scalar multiplier applied to the sample signal.
- Master volume: a global gain applied after per-instance gain.
- Pitch / playback speed / rate: controls the rate at which the playback cursor
  advances through samples; in v1 this implies both speed and frequency change.
- Resampling: reading an audio buffer at a non-1.0 rate using a fractional
  cursor and interpolation.
- Clipping: signal values exceeding the output format range (for `f32`,
  typically `[-1.0, 1.0]`).

## Architecture Overview

This feature extends the `lambda-rs` playback scheduler to apply:

1) per-instance gain and pitch during sample generation, and
2) a master gain at the final output stage.

Data flow (single active instance, as in baseline playback)

```
application
  └── AudioContext
        ├── master_volume (scalar)
        └── play_sound(...) -> SoundInstance
              ├── instance_volume (scalar)
              └── instance_pitch (rate)
                    └── commands -> audio callback scheduler
                          └── resampler -> per-instance gain -> master gain -> clip -> output
```

## Design

### API Surface

This section describes the public API surface added to `lambda-rs`.

```rust
impl SoundInstance {
  /// Set volume where 1.0 is normal, 0.0 is silent, >1.0 amplifies.
  pub fn set_volume(&mut self, volume: f32);
  pub fn volume(&self) -> f32;

  /// Set pitch where 1.0 is normal, 0.5 is half speed, 2.0 is double.
  pub fn set_pitch(&mut self, pitch: f32);
  pub fn pitch(&self) -> f32;
}

impl AudioContext {
  pub fn set_master_volume(&mut self, volume: f32);
  pub fn master_volume(&self) -> f32;
}
```

Defaults
- New `SoundInstance` objects MUST begin with `volume == 1.0` and `pitch == 1.0`.
- New `AudioContext` objects MUST begin with `master_volume == 1.0`.

Notes
- These APIs are intentionally scalar-only for v1 (no per-channel gains).
- Naming: “volume” is treated as synonymous with “gain” and represents a
  linear multiplier.

### Behavior

#### Per-instance volume (gain)

- Output signal MUST be multiplied by `instance_volume`.
- `instance_volume == 0.0` MUST produce silence for that instance.
- `instance_volume > 1.0` MUST amplify the signal before clipping/limiting.

Clipping behavior
- The system MUST be “clipping aware”: it MUST NOT panic or produce NaNs due to
  amplification, and it MUST keep the final output within the output format’s
  representable range.
- v1 MAY implement either:
  - hard clipping (saturating clamp), or
  - soft clipping (simple saturator) for gentler distortion at >1.0 gains.
- If soft clipping is used, it MUST still guarantee bounded output.

#### Master volume

- Output signal MUST be multiplied by `master_volume` after per-instance gain.
- Master volume MUST affect all playback routed through an `AudioContext`,
  including any future multi-sound mixing within that context.
- If multiple contexts exist, master volume is per-context, not global to the
  process (unless a future design introduces a true process-wide master bus).

Effective gain
- The effective scalar applied to samples is:
  - `effective_gain = instance_volume * master_volume`

#### Pitch / playback speed

- Pitch MUST change both playback speed and perceived frequency.
- Pitch MUST be implemented by advancing the playback cursor by `pitch` samples
  per output sample frame, using a fractional cursor and interpolation.

Resampling (v1)
- v1 SHOULD use linear interpolation between adjacent samples.
- When `pitch == 1.0`, the implementation SHOULD take the fast path that reads
  samples without interpolation (when possible), but correctness is preferred
  over micro-optimizations.

Edge cases
- When pitch causes the cursor to step past the end:
  - if looping is enabled, wrap according to the looping behavior in the
    baseline playback spec;
  - otherwise, the instance MUST transition to stopped and output silence.

### Validation and Errors

The API surface does not return `Result`, so validation MUST be total and
non-panicking.

Validation rules
- `set_volume(volume)`:
  - if `volume` is not finite, treat it as `1.0`;
  - if `volume < 0.0`, clamp to `0.0`;
  - otherwise accept the value (including values > `1.0`).
- `set_master_volume(volume)` follows the same rules as `set_volume`.
- `set_pitch(pitch)`:
  - if `pitch` is not finite, treat it as `1.0`;
  - if `pitch <= 0.0`, clamp to a small positive epsilon (implementation-defined).

Rationale
- Clamping keeps behavior deterministic and avoids introducing fallible APIs for
  a v1 feature.

## Constraints and Rules

- The audio callback thread MUST NOT allocate and MUST remain lock-free or
  bounded-lock (as required by the existing playback design).
- Updates to volume/pitch/master volume MUST be safe to issue frequently (e.g.,
  every frame) without causing audible glitches from lock contention.
- The applied gain and pitch MUST be sample-accurate at the callback boundary;
  changes MAY take effect on the next callback buffer fill.

## Performance Considerations

Recommendations
- Keep resampling interpolation simple (linear) in v1.
  - Rationale: predictable cost and easy to reason about.
- Apply gain as a scalar multiply in the tight loop and clip once at the end.
  - Rationale: minimizes extra branches and redundant clamps.

## Acceptance Criteria

- [ ] Volume `0.0` produces silence
- [ ] Volume `1.0` plays at original level
- [ ] Volume `> 1.0` amplifies (with clipping awareness)
- [ ] Pitch `1.0` plays at original speed
- [ ] Pitch changes affect both speed and frequency
- [ ] Master volume affects all playing sounds

## Requirements Checklist

- Functionality
  - [ ] Per-instance volume stored and applied
  - [ ] Per-instance pitch stored and applied via resampling
  - [ ] Master volume stored and applied after per-instance gain
  - [ ] Clipping behavior defined and implemented for >1.0 gains
- API Surface
  - [ ] `SoundInstance::{set_volume,volume,set_pitch,pitch}` exposed
  - [ ] `AudioContext::{set_master_volume,master_volume}` exposed
  - [ ] Defaults are `1.0` for all new values
- Validation and Errors
  - [ ] Non-finite inputs handled deterministically
  - [ ] Negative volume clamped to `0.0`
  - [ ] Non-positive pitch clamped to epsilon
- Documentation and Examples
  - [ ] Playback example updated to adjust volume/pitch/master volume

## Verification and Testing

Unit tests (recommended)
- Validate clamping/normalization behavior for `set_volume`, `set_pitch`,
  `set_master_volume` (negative, zero, large, NaN/Inf).
- Validate pitch behavior with a small synthetic buffer:
  - `pitch == 1.0` returns the original sequence,
  - `pitch == 2.0` advances twice as fast (skips/interpolates accordingly),
  - `pitch == 0.5` repeats/interpolates accordingly.

Manual checks (recommended)
- In the playback demo/example:
  - set volume to `0.0` and confirm silence,
  - set volume to `> 1.0` and confirm audible amplification and bounded output,
  - change pitch to `0.5` and `2.0` and confirm both speed and pitch change,
  - change master volume and confirm it affects current playback.

## Compatibility and Migration

None. This is additive to the playback API.

## Changelog

- 2026-02-15 (v0.1.0) — Initial draft.
