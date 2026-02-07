---
title: "2D Physics World"
document_id: "physics-world-2d-2026-02-06"
status: "draft"
created: "2026-02-06T23:02:06Z"
last_updated: "2026-02-07T01:28:28Z"
version: "0.1.2"
engine_workspace_version: "2023.1.30"
wgpu_version: "26.0.1"
shader_backend_default: "naga"
winit_version: "0.29.10"
repo_commit: "d9ae52363df035954079bf2ebdc194d18281862d"
owners: ["lambda-sh"]
reviewers: ["engine"]
tags: ["spec", "physics", "2d", "lambda-rs", "platform"]
---

# 2D Physics World

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

- Introduce a `PhysicsWorld2D` type that owns 2D physics simulation state and
  exposes a fixed-timestep stepping API.
- Provide configurable gravity with a stable default of `(0.0, -9.81)`.
- Use a builder (`PhysicsWorld2DBuilder`) to construct worlds with validated
  parameters and predictable runtime behavior.

Rationale
- Lambda currently has no physics support, preventing 2D games from simulating
  physical interactions.
- The world and stepping API establishes the foundational architecture needed
  for later work items (rigid bodies, colliders, and collision queries) without
  prematurely committing to an application-facing body/shape API.

## Scope

### Goals

- Provide `PhysicsWorld2D` for managing simulation state.
- Provide configurable gravity (`(x, y)`), defaulting to `(0.0, -9.81)`.
- Provide fixed timestep integration configured at world construction time
  (default: `1.0 / 60.0` seconds).
- Provide a builder pattern (`PhysicsWorld2DBuilder`) for world creation.
- Support stepping an empty world (no bodies/colliders) without errors.
- Provide unit tests covering world construction and stepping behavior.

### Non-Goals

- 3D physics.
- Application-facing rigid body APIs (creation, mutation, querying).
- Application-facing collision shape APIs (creation, mutation, querying).
- Deterministic lockstep guarantees across platforms (floating-point bitwise
  determinism).

## Terminology

- Fixed timestep: stepping the simulation with a constant `dt` regardless of
  render frame timing.
- Sub-step: splitting one fixed timestep into multiple smaller steps for
  stability.
- Gravity: a constant acceleration applied to dynamic bodies each step.
- Backend: an internal physics engine implementation (for example, `rapier2d`)
  hidden behind `lambda-rs-platform`.

## Architecture Overview

- Crate `lambda` (package: `lambda-rs`)
  - Public module `physics` provides `PhysicsWorld2D` and
    `PhysicsWorld2DBuilder`.
  - The public API MUST be backend-agnostic and MUST NOT expose vendor types
    (for example, `rapier2d`).
- Crate `lambda_platform` (package: `lambda-rs-platform`)
  - Provides internal wrappers for the chosen 2D physics backend (initially
    `rapier2d`), owned by the platform crate to avoid leaking dependency
    surface into `lambda-rs`.

Data flow

```
application
  └── lambda::physics::PhysicsWorld2D
        └── lambda_platform::physics::<backend> (internal)
              └── vendor crate (for example, rapier2d)
```

## Design

### API Surface

Module layout (new)

- `crates/lambda-rs/src/physics/mod.rs`
  - Public `PhysicsWorld2D` and `PhysicsWorld2DBuilder`.
  - Public error type(s) for builder validation.
- `crates/lambda-rs-platform/src/physics/mod.rs`
  - Internal backend wrapper module(s).
  - Applications MUST NOT depend on these types directly.

### lambda-rs Public API

Public entry points (draft)

```rust
// crates/lambda-rs/src/physics/mod.rs

/// A 2D physics simulation world.
pub struct PhysicsWorld2D {
  // Internal backend state (for example, rapier2d pipeline and sets).
}

/// Builder for `PhysicsWorld2D`.
pub struct PhysicsWorld2DBuilder {
  gravity: [f32; 2],
  timestep_seconds: f32,
  substeps: u32,
}

impl PhysicsWorld2DBuilder {
  /// Creates a builder with defaults:
  /// - gravity: (0.0, -9.81)
  /// - timestep: 1.0 / 60.0
  /// - substeps: 1
  pub fn new() -> Self;

  /// Sets gravity, in meters per second squared.
  pub fn with_gravity(self, x: f32, y: f32) -> Self;

  /// Sets the fixed timestep in seconds.
  pub fn with_timestep_seconds(self, timestep_seconds: f32) -> Self;

  /// Sets the number of sub-steps per fixed timestep.
  pub fn with_substeps(self, substeps: u32) -> Self;

  /// Builds a validated `PhysicsWorld2D`.
  pub fn build(self) -> Result<PhysicsWorld2D, PhysicsWorld2DError>;
}

impl PhysicsWorld2D {
  /// Advances the world by one configured fixed timestep.
  pub fn step(&mut self);

  /// Returns the configured gravity.
  pub fn gravity(&self) -> [f32; 2];

  /// Returns the configured fixed timestep in seconds.
  pub fn timestep_seconds(&self) -> f32;
}

/// Construction-time configuration errors for `PhysicsWorld2D`.
pub enum PhysicsWorld2DError {
  InvalidTimestepSeconds { timestep_seconds: f32 },
  InvalidSubsteps { substeps: u32 },
  InvalidGravity { x: f32, y: f32 },
}
```

Notes
- `PhysicsWorld2D::step()` is intentionally argument-free in this work item to
  preserve a fixed timestep contract. Later work MAY add helper APIs that
  integrate variable frame delta times via accumulation.
- Public APIs MUST use explicit units in names where ambiguity exists (for
  example, `timestep_seconds`).

### Behavior

World construction
- `PhysicsWorld2DBuilder::new()` MUST set default gravity to `(0.0, -9.81)`.
- The default fixed timestep MUST be `1.0 / 60.0` seconds.
- The default sub-step count MUST be `1`.
- `build()` MUST validate configuration and return a descriptive error when
  invalid values are provided.

Stepping
- `PhysicsWorld2D::step()` MUST advance the simulation by exactly one fixed
  timestep configured in the world.
- If `substeps > 1`, `step()` MUST subdivide the timestep evenly and perform
  `substeps` internal backend steps of duration
  `timestep_seconds / substeps`.
- Stepping an empty world (no bodies/colliders) MUST be supported and MUST NOT
  panic.
- The API MUST NOT require a window, GPU device, or other rendering resources.

Backend abstraction
- The `lambda-rs` public API MUST remain backend-agnostic.
- Vendor-specific configuration (for example, CCD toggles, solver parameters)
  MUST NOT be exposed in this work item.
- The initial implementation MAY use `rapier2d`, but the platform wrapper MUST
  keep the option to swap backends without breaking public API.

### Validation and Errors

Builder validation rules
- `timestep_seconds` MUST be finite and `> 0.0`.
- `substeps` MUST be `>= 1`.
- Gravity components MUST be finite (not `NaN` or infinite).

Error reporting
- `PhysicsWorld2DError` variants MUST include the offending values to make
  errors actionable.
- Errors MUST be backend-agnostic (no vendor error types in public variants).

### Cargo Features

This work item is expected to introduce feature flags to keep optional physics
dependencies out of minimal builds.

Planned features
- Crate `lambda-rs`
  - `physics-2d` (granular, disabled by default): enables the `physics` module
    public API and its implementation.
- Crate `lambda-rs-platform`
  - `physics-2d` (granular, disabled by default): enables internal 2D physics
    backend wrappers (for example, `rapier2d`).

Feature relationships
- `lambda-rs/physics-2d` MUST enable `lambda-rs-platform/physics-2d`.
- Applications MUST NOT depend on `lambda-rs-platform` directly.

Documentation requirement
- The implementation pull request MUST update `docs/features.md` with the
  feature names, owning crates, default states, summaries, and expected runtime
  costs.

## Constraints and Rules

- Public APIs MUST avoid exposing vendor types and MUST avoid leaking
  `lambda-rs-platform` details.
- Configuration MUST be validated at build time, not at step time.
- `PhysicsWorld2D` MUST be constructible without any physics entities.

## Performance Considerations

Recommendations
- Prefer a fixed timestep of `1.0 / 60.0` seconds for typical 2D games.
  - Rationale: Stable integration with predictable costs per frame.
- Use `substeps` only when stability issues are observed.
  - Rationale: Sub-stepping increases simulation cost approximately linearly
    with the sub-step count.

## Requirements Checklist

Functionality
- [ ] `PhysicsWorld2D` and `PhysicsWorld2DBuilder` exist in `lambda-rs`.
- [ ] Gravity is configurable with default `(0.0, -9.81)`.
- [ ] Fixed timestep integration is supported with default `1.0 / 60.0`.
- [ ] `step()` advances the simulation by one fixed timestep.
- [ ] Empty world stepping is supported (no panic, no errors).

API Surface
- [ ] Builder pattern is used for world creation.
- [ ] Public API is backend-agnostic and does not expose vendor types.

Validation and Errors
- [ ] `build()` validates gravity, timestep, and sub-step counts.
- [ ] Errors are actionable and backend-agnostic.

Documentation and Examples
- [ ] `docs/features.md` updated for new feature(s) (if introduced).
- [ ] Minimal example added (optional for this work item).

## Verification and Testing

Unit tests
- Construct a default world and assert default gravity and timestep.
- Construct a world with custom gravity and timestep and assert values.
- Step a default world with no entities and assert the call does not panic.
- Validate `build()` rejects invalid values (`timestep_seconds <= 0`,
  `substeps == 0`, non-finite gravity).
- Commands: `cargo test -p lambda-rs --features physics-2d -- --nocapture`

Integration tests
- None required for this work item.

## Compatibility and Migration

- If the physics module is gated behind a feature, there are no behavior
  changes for existing applications that do not enable the feature.
- No migration steps are required.

## Changelog

- 2026-02-06 (v0.1.0) — Initial draft.
