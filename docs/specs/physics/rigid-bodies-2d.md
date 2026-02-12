---
title: "2D Rigid Bodies"
document_id: "rigid-bodies-2d-2026-02-12"
status: "draft"
created: "2026-02-12T23:03:52Z"
last_updated: "2026-02-12T23:33:50Z"
version: "0.2.0"
engine_workspace_version: "2023.1.30"
wgpu_version: "26.0.1"
shader_backend_default: "naga"
winit_version: "0.29.10"
repo_commit: "6f96052fae896a095b658f29af1eff96e5aaa348"
owners: ["lambda-sh"]
reviewers: ["engine"]
tags: ["spec", "physics", "2d", "lambda-rs", "platform"]
---

# 2D Rigid Bodies

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

- Introduce a `RigidBody2D` API for adding 2D rigid bodies to `PhysicsWorld2D`.
- Support `Static`, `Dynamic`, and `Kinematic` body types with predictable,
  backend-agnostic behavior.
- Provide configuration and mutation APIs for position, rotation, velocity,
  mass (dynamic only), forces, and impulses.
- Define a production-safe handle model (world-scoped, generational) to prevent
  cross-world misuse and stale-handle access.

Rationale
- A world stepping API without application-facing bodies cannot express common
  2D gameplay simulation (gravity, movement, and externally-applied forces).
- Rigid bodies establish the minimal entity layer required before adding
  colliders, collision response, and constraints.

## Scope

### Goals

- Support inserting rigid bodies into `PhysicsWorld2D` via a builder pattern.
- Support body types:
  - `Static`: immovable; infinite mass semantics.
  - `Dynamic`: affected by gravity and forces.
  - `Kinematic`: user-controlled; not affected by gravity or forces.
- Support querying and setting position and rotation.
- Support querying and setting linear velocity.
- Support applying forces and impulses to dynamic bodies.
- Specify integration and mutation timing rules that are stable across physics
  backend implementations.
- Support unit tests validating each body type and the force/impulse behavior.

### Non-Goals

- Collision shapes (colliders) and broad-phase collision queries.
- Collision response and contact events.
- Joints/constraints.
- Sleeping, deactivation, and other advanced solver behavior.
- Angular dynamics (angular velocity, torque) beyond explicitly setting and
  reading `rotation`.
- Body destruction/removal APIs.
- Bitwise deterministic simulation across platforms and CPU architectures.

## Terminology

- Rigid body: an entity with a transform and velocity that participates in the
  physics simulation.
- Static body: a body that is not integrated by the simulation.
- Dynamic body: a body integrated by the simulation and influenced by gravity
  and forces.
- Kinematic body: a body integrated only by user-provided motion (for example,
  explicit velocity) and not influenced by gravity or forces.
- Force: a continuous influence applied over time (units: N = kg·m/s²).
- Impulse: an instantaneous change to momentum (units: N·s = kg·m/s).
- Step: a single fixed-timestep advancement of `PhysicsWorld2D`.
- Symplectic Euler: an integration scheme that updates velocity before
  position (`v += a * dt`, then `x += v * dt`).
- Teleport: a direct transform assignment (for example, `set_position`) that
  bypasses integration for that update.

## Architecture Overview

Dependencies
- This work item depends on `PhysicsWorld2D` and its fixed-timestep stepping
  contract as specified by `docs/specs/physics/physics-world-2d.md`.

Crate boundaries
- Crate `lambda` (package: `lambda-rs`)
  - Public module `physics` MUST expose backend-agnostic rigid body APIs.
  - Public types MUST NOT expose vendor types (for example, `rapier2d`).
- Crate `lambda_platform` (package: `lambda-rs-platform`)
  - MUST contain the internal backend wrapper that owns vendor rigid body
    storage and provides body mutation/query entry points to `lambda-rs`.

Data flow

```
application
  └── lambda::physics::{PhysicsWorld2D, RigidBody2D}
        └── lambda_platform::physics::PhysicsBackend2D (internal)
              └── vendor crate (for example, rapier2d)
```

## Design

### API Surface

Module layout (new)
- `crates/lambda-rs/src/physics/rigid_body_2d.rs`
  - Public `RigidBody2D`, `RigidBody2DBuilder`, `RigidBodyType`, and error
    types.
- `crates/lambda-rs/src/physics/mod.rs`
  - Re-export `RigidBody2D` APIs as part of `lambda::physics`.
  - Provide `PhysicsWorld2D` entry points required by the handle-based API.
- `crates/lambda-rs-platform/src/physics/rapier2d.rs`
  - Add internal backend entry points for body creation, mutation, and query.

Handle model
- `RigidBody2D` MUST be a compact, copyable, opaque handle.
- Handles MUST be world-scoped and generational.
  - Rationale: Prevents use-after-free and cross-world access when bodies are
    later made removable.
- A valid `RigidBody2D` MUST encode:
  - A stable body slot index.
  - A generation counter for the slot.
  - A world identifier.
- The handle encoding MUST be treated as an implementation detail and MUST NOT
  be relied on by applications.
- `PhysicsWorld2D` MUST assign a stable, non-zero world identifier at
  construction time, and this identifier MUST be stored in each `RigidBody2D`
  created by the world.

### lambda-rs Public API

Public entry points (draft)

```rust
/// The rigid body integration mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RigidBodyType {
  Static,
  Dynamic,
  Kinematic,
}

/// An opaque handle to a rigid body stored in a `PhysicsWorld2D`.
///
/// This handle is world-scoped. Operations MUST validate that the handle
/// belongs to the provided world.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RigidBody2D {
  // World identifier used for cross-world validation (implementation detail).
  world_id: u32,
  // Body slot index (implementation detail).
  slot_index: u32,
  // Slot generation used for stale-handle detection (implementation detail).
  slot_generation: u32,
}

/// Builder for `RigidBody2D`.
#[derive(Debug, Clone, Copy)]
pub struct RigidBody2DBuilder {
  body_type: RigidBodyType,
  position: [f32; 2],
  rotation: f32,
  velocity: [f32; 2],
  dynamic_mass_kg: Option<f32>,
}

impl RigidBody2DBuilder {
  /// Creates a builder for the given body type with stable defaults.
  pub fn new(body_type: RigidBodyType) -> Self;

  /// Sets the initial position, in meters.
  pub fn with_position(self, x: f32, y: f32) -> Self;

  /// Sets the initial rotation, in radians.
  pub fn with_rotation(self, radians: f32) -> Self;

  /// Sets the initial linear velocity, in meters per second.
  pub fn with_velocity(self, vx: f32, vy: f32) -> Self;

  /// Sets the mass, in kilograms, for dynamic bodies.
  pub fn with_dynamic_mass_kg(self, mass_kg: f32) -> Self;

  /// Inserts the body into the given world.
  pub fn build(
    self,
    world: &mut PhysicsWorld2D,
  ) -> Result<RigidBody2D, RigidBody2DError>;
}

impl RigidBody2D {
  /// Returns the body type.
  pub fn body_type(
    self,
    world: &PhysicsWorld2D,
  ) -> Result<RigidBodyType, RigidBody2DError>;

  /// Returns the current position, in meters.
  pub fn position(
    self,
    world: &PhysicsWorld2D,
  ) -> Result<[f32; 2], RigidBody2DError>;

  /// Returns the current rotation, in radians.
  pub fn rotation(
    self,
    world: &PhysicsWorld2D,
  ) -> Result<f32, RigidBody2DError>;

  /// Returns the current linear velocity, in meters per second.
  pub fn velocity(
    self,
    world: &PhysicsWorld2D,
  ) -> Result<[f32; 2], RigidBody2DError>;

  /// Sets position, in meters.
  pub fn set_position(
    self,
    world: &mut PhysicsWorld2D,
    x: f32,
    y: f32,
  ) -> Result<(), RigidBody2DError>;

  /// Sets rotation, in radians.
  pub fn set_rotation(
    self,
    world: &mut PhysicsWorld2D,
    radians: f32,
  ) -> Result<(), RigidBody2DError>;

  /// Sets linear velocity, in meters per second.
  pub fn set_velocity(
    self,
    world: &mut PhysicsWorld2D,
    vx: f32,
    vy: f32,
  ) -> Result<(), RigidBody2DError>;

  /// Applies a force, in Newtons, at the center of mass.
  pub fn apply_force(
    self,
    world: &mut PhysicsWorld2D,
    fx: f32,
    fy: f32,
  ) -> Result<(), RigidBody2DError>;

  /// Applies an impulse, in Newton-seconds, at the center of mass.
  pub fn apply_impulse(
    self,
    world: &mut PhysicsWorld2D,
    ix: f32,
    iy: f32,
  ) -> Result<(), RigidBody2DError>;
}

/// Errors for rigid body construction and operations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RigidBody2DError {
  InvalidHandle,
  WorldMismatch,
  BodyNotFound,
  InvalidPosition { x: f32, y: f32 },
  InvalidRotation { radians: f32 },
  InvalidVelocity { x: f32, y: f32 },
  InvalidForce { x: f32, y: f32 },
  InvalidImpulse { x: f32, y: f32 },
  InvalidMassKg { mass_kg: f32 },
  UnsupportedOperation { body_type: RigidBodyType },
}
```

Notes
- `RigidBody2D` is a handle and MUST NOT own simulation state.
- This work item introduces no removal API. `BodyNotFound` and `InvalidHandle`
  exist to support handle validation and future body destruction work.

### Behavior

Coordinate system and units
- Positions and velocities MUST be interpreted as meters and meters per second.
- Rotation MUST be interpreted as radians around the 2D Z axis, with positive
  values rotating counterclockwise.
- World gravity MUST follow `PhysicsWorld2D::gravity()` (default `(0.0, -9.81)`
  meters per second squared).

Observable state
- `position()` MUST return the rigid body translation at the end of the most
  recent `step()` or after the most recent explicit teleport mutation.
- `rotation()` MUST return the rigid body rotation at the end of the most
  recent `step()` or after the most recent explicit teleport mutation.
- `velocity()` MUST return the current linear velocity used for integration.

Insertion
- `RigidBody2DBuilder::build()` MUST insert a new body into the given world and
  return a world-scoped `RigidBody2D` handle.
- The builder MUST provide stable defaults:
  - `position`: `(0.0, 0.0)`
  - `rotation`: `0.0`
  - `velocity`: `(0.0, 0.0)`
  - `dynamic_mass_kg`: unset (dynamic bodies default to `1.0`)

Mutation timing
- `set_position`, `set_rotation`, and `set_velocity` MUST take effect
  immediately and MUST be observable by subsequent queries before the next
  `PhysicsWorld2D::step()`.
- `set_position` and `set_rotation` MUST NOT modify linear velocity or the
  accumulated force state for the body.
- `apply_impulse` MUST take effect immediately by updating velocity and MUST be
  observable by `velocity()` before the next `PhysicsWorld2D::step()`.
- `apply_force` MUST affect integration starting with the next
  `PhysicsWorld2D::step()`.

Stepping
- `PhysicsWorld2D::step()` MUST integrate dynamic and kinematic rigid bodies.
- Integration MUST respect `PhysicsWorld2D` substeps (each substep applies
  gravity and applies accumulated forces for the substep duration).
- Accumulated forces MUST be applied for the full duration of the outer step
  (across all substeps) and MUST be cleared after the outer `step()` completes.

Integration model (normative)
- Dynamic and kinematic translation integration MUST be equivalent to
  symplectic Euler for linear motion:
  - Dynamic:
    - `a = gravity + (force / mass_kg)`
    - `v = v + a * dt`
    - `x = x + v * dt`
  - Kinematic:
    - `x = x + v * dt`
- `dt` MUST be the world timestep divided by `substeps` for each internal
  substep.
- `force` MUST be the accumulated per-body force vector in Newtons, computed as
  the linear sum of all `apply_force()` calls since the end of the previous
  outer `PhysicsWorld2D::step()`.
- Force and impulse APIs in this work item MUST apply at the center of mass and
  MUST NOT introduce torque.
- `apply_force()` MUST be additive. Multiple `apply_force()` calls before a
  `PhysicsWorld2D::step()` MUST sum linearly into the force accumulator.

Rotation semantics
- During `step()`, rigid body rotation MUST remain unchanged in this work item.
  - Rationale: Angular dynamics are out of scope and the public API does not
    expose angular velocity or torque.
- `set_rotation()` MUST update rotation immediately for all body types.

Static bodies
- Static bodies MUST NOT be affected by gravity, forces, or impulses.
- Static bodies MUST NOT change position or rotation during `step()` unless
  explicitly mutated via `set_position` or `set_rotation`.
- Applying forces/impulses to static bodies MUST return
  `RigidBody2DError::UnsupportedOperation { body_type: Static }`.
- Setting velocity on static bodies MUST return
  `RigidBody2DError::UnsupportedOperation { body_type: Static }`.

Dynamic bodies
- Dynamic bodies MUST be affected by gravity each step.
- `apply_force()` MUST accumulate a force that influences acceleration on the
  next `step()`.
- `apply_impulse()` MUST modify linear velocity immediately by:
  - `velocity += impulse / mass_kg`
- `set_velocity()` MUST override the current linear velocity.
- `set_position()` and `set_rotation()` MUST update the transform and MUST NOT
  panic. These operations MUST NOT implicitly clear velocity or accumulated
  forces.

Kinematic bodies
- Kinematic bodies MUST NOT be affected by gravity, forces, or impulses.
- Kinematic bodies MUST integrate using their linear velocity:
  - `position += velocity * dt`
- `set_velocity()` MUST set the kinematic integration velocity.
- Applying forces/impulses to kinematic bodies MUST return
  `RigidBody2DError::UnsupportedOperation { body_type: Kinematic }`.

Bodies without colliders
- Bodies MUST be able to move and be queried without requiring collision
  shapes.
- Bodies without colliders MUST NOT collide, generate contacts, or constrain
  motion in this work item.

### Validation and Errors

Handle validation
- All `RigidBody2D` operations MUST validate that the handle is well-formed and
  belongs to the provided world.
- `InvalidHandle` MUST be returned when the handle encoding is invalid (for
  example, a zero `world_id`).
- Using a handle from a different world MUST return
  `RigidBody2DError::WorldMismatch`.
- Using an unknown handle MUST return `RigidBody2DError::BodyNotFound`.
  Unknown includes a slot that is out of range, empty, or has a mismatched
  generation.

Input validation rules
- Position components MUST be finite.
- Rotation MUST be finite.
- Velocity components MUST be finite.
- Force components MUST be finite.
- Impulse components MUST be finite.
- For `Dynamic` bodies, mass MUST be finite and `> 0.0`.
  - If `dynamic_mass_kg` is unset, mass MUST default to `1.0`.
- For `Static` and `Kinematic` bodies, setting a dynamic mass MUST be rejected
  with `UnsupportedOperation { body_type }`.

Error reporting
- Errors MUST be backend-agnostic and MUST NOT expose vendor types.
- Errors SHOULD include offending values where doing so materially improves
  actionability (for example, `InvalidMassKg { mass_kg }`).

### Cargo Features

- This work item MUST be gated by the existing `lambda-rs/physics-2d` feature
  and MUST NOT add new physics Cargo features.
- `lambda-rs/physics-2d` MUST continue to enable
  `lambda-rs-platform/physics-2d`.
- The implementation pull request SHOULD update `docs/features.md` to include
  the rigid body APIs as part of the `physics-2d` summary.

## Constraints and Rules

- Public APIs MUST remain backend-agnostic and MUST NOT expose vendor types.
- Public APIs MUST avoid panic in library code; invalid operations MUST return
  actionable errors.
- This work item MUST NOT introduce collisions, shapes, or collision response.

## Performance Considerations

Recommendations
- Prefer using kinematic bodies for directly-authoritative gameplay movement.
  - Rationale: Avoids force tuning and solver costs when collisions are not
    required.
- Use dynamic bodies only when gravity and force-based movement is required.
  - Rationale: Dynamic integration adds per-step acceleration updates and
    force accumulation handling.

## Requirements Checklist

Functionality
- [ ] Static, dynamic, and kinematic bodies can be created in a world.
- [ ] Bodies can be queried for position, rotation, and velocity.
- [ ] Bodies can be mutated for position, rotation, and velocity.
- [ ] Gravity affects dynamic bodies and does not affect static/kinematic.
- [ ] Forces and impulses affect dynamic bodies and error on other types.

API Surface
- [ ] `RigidBody2D`, `RigidBody2DBuilder`, and `RigidBodyType` are public in
  `lambda-rs`.
- [ ] Public API is backend-agnostic and does not expose vendor types.
- [ ] Handle validation prevents cross-world misuse.
- [ ] Handles are world-scoped and generational.

Validation and Errors
- [ ] Builder validation rejects non-finite inputs and invalid dynamic mass.
- [ ] Unsupported operations return descriptive errors (no panics).

Documentation and Examples
- [ ] `docs/features.md` updated to reflect rigid body support.
- [ ] Minimal example added or updated (optional for this work item).

## Verification and Testing

Unit tests
- Construct each body type and validate default state.
- Verify static bodies do not move under gravity/forces.
- Verify dynamic bodies accelerate under gravity and respond to force/impulse
  with assertions that use explicit tolerances.
- Verify kinematic bodies move when a velocity is set.
- Verify handle validation rejects cross-world operations.
- Recommended numeric scenarios (tolerance: `1e-4`)
  - Dynamic gravity integration
    - `dt = 1.0`, `gravity = (0.0, -10.0)`, `mass_kg = 1.0`, `v0 = (0.0, 0.0)`.
    - After one step: `v1.y == -10.0`, `y1 == -10.0`.
    - After two steps: `v2.y == -20.0`, `y2 == -30.0`.
  - Dynamic impulse
    - `mass_kg = 2.0`, `apply_impulse((2.0, 0.0))` results in `vx += 1.0`.
  - Dynamic force (no gravity)
    - `dt = 1.0`, `gravity = (0.0, 0.0)`, `mass_kg = 2.0`,
      `apply_force((10.0, 0.0))` once before a step results in `vx == 5.0` and
      `x == 5.0` after the step.
  - Kinematic velocity integration
    - `dt = 1.0`, `v = (2.0, 0.0)` results in `x += 2.0` after one step.
- Commands: `cargo test -p lambda-rs --features physics-2d -- --nocapture`

Integration tests
- None required for this work item.

## Compatibility and Migration

- If the physics module remains behind `physics-2d`, no behavior changes occur
  unless the feature is enabled.
- No migration steps are required.

## Changelog

- 2026-02-12 (v0.1.0) — Initial draft.
- 2026-02-12 (v0.2.0) — Add handle rules, timing, and integration semantics.
