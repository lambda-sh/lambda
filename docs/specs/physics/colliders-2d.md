---
title: "2D Colliders"
document_id: "colliders-2d-2026-02-17"
status: "draft"
created: "2026-02-17T23:08:44Z"
last_updated: "2026-02-17T23:37:05Z"
version: "0.1.2"
engine_workspace_version: "2023.1.30"
wgpu_version: "26.0.1"
shader_backend_default: "naga"
winit_version: "0.29.10"
repo_commit: "43c91a76dec71326cc255ebb6fb6c6402e95735c"
owners: ["lambda-sh"]
reviewers: ["engine"]
tags: ["spec", "physics", "2d", "lambda-rs", "platform"]
---

# 2D Colliders

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

- Introduce `Collider2D` handles and a `Collider2DBuilder` for attaching one or
  more collision shapes to a `RigidBody2D`.
- Support common 2D primitive shapes (`Circle`, `Rectangle`, `Capsule`) and
  arbitrary convex polygons.
- Provide per-collider material properties (`density`, `friction`,
  `restitution`) that influence contact resolution for participating bodies.
- Support a collider-local transform (offset and rotation) to enable oriented
  shapes without requiring angular dynamics.
- Define a backend-agnostic behavior contract while allowing implementation via
  `lambda-rs-platform` (initially backed by `rapier2d`).

Rationale
- Rigid bodies without colliders cannot participate in collision detection or
  contact resolution.
- Attaching multiple colliders to a single body enables common gameplay shapes
  (compound characters, triggers-as-geometry in later work, and approximated
  concave shapes without decomposition support).

## Scope

### Goals

- Provide the following collider shapes:
  - Circle (radius)
  - Rectangle/box (half-extents)
  - Capsule (half-height, radius)
  - Convex polygon (vertex list)
- Support attaching multiple colliders to one rigid body.
- Support static, dynamic, and kinematic bodies as collider owners.
- Support collider-local offset and rotation for oriented shapes.
- Provide `density`, `friction`, and `restitution` configuration on colliders.
- Ensure friction and restitution influence collision response during world
  stepping.

### Non-Goals

- Concave shapes and automatic decomposition.
- Collision detection callbacks / event streams.
- Collision layers/masks and query filtering.
- Sensors, triggers, and overlap-only colliders.
- Body/collider destruction and mutation beyond construction-time parameters.
- Public exposure of backend/vendor types.

## Terminology

- Collider: a shape attached to a rigid body that participates in collision
  detection and contact resolution.
- Shape: the geometric primitive used by a collider.
- Material: per-collider properties (`density`, `friction`, `restitution`)
  affecting physical response.
- Local transform: the collider transform relative to the owning body origin.
- Contact: a collision manifold between two colliders produced by the backend.

## Architecture Overview

Dependencies
- This work item depends on `PhysicsWorld2D` and `RigidBody2D` as specified by:
  - `docs/specs/physics/physics-world-2d.md`
  - `docs/specs/physics/rigid-bodies-2d.md`

Crate boundaries
- Crate `lambda` (package: `lambda-rs`)
  - Public module `physics` MUST expose backend-agnostic collider APIs.
  - Public types MUST NOT expose vendor types (for example, `rapier2d`).
- Crate `lambda_platform` (package: `lambda-rs-platform`)
  - MUST own the vendor collider representation and attach it to vendor rigid
    bodies.
  - MUST translate backend behavior into the stable contract defined by this
    document.

Data flow

```
application
  └── lambda::physics::{PhysicsWorld2D, RigidBody2D, Collider2D}
        └── lambda_platform::physics::PhysicsBackend2D (internal)
              └── vendor crate (for example, rapier2d)
```

## Design

### API Surface

Module layout (new)

- `crates/lambda-rs/src/physics/mod.rs`
  - Public `Collider2D`, `Collider2DBuilder`, `ColliderShape2D`,
    and `Collider2DError`.
- `crates/lambda-rs-platform/src/physics/mod.rs`
  - Internal backend collider APIs and storage.

### lambda-rs Public API

Public entry points (draft)

```rust
/// Maximum supported vertices for `ColliderShape2D::ConvexPolygon`.
pub const MAX_CONVEX_POLYGON_VERTICES: usize = 64;

/// An opaque handle to a collider stored in a `PhysicsWorld2D`.
///
/// This handle is world-scoped. Operations MUST validate that the handle
/// belongs to the provided world.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Collider2D {
  world_id: u64,
  slot_index: u32,
  slot_generation: u32,
}

/// Supported 2D collider shapes.
#[derive(Debug, Clone, PartialEq)]
pub enum ColliderShape2D {
  Circle { radius: f32 },
  Rectangle { half_width: f32, half_height: f32 },
  Capsule { half_height: f32, radius: f32 },
  ConvexPolygon { vertices: Vec<[f32; 2]> },
}

/// Material parameters for a `Collider2D`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColliderMaterial2D {
  density: f32,
  friction: f32,
  restitution: f32,
}

/// Builder for `Collider2D`.
#[derive(Debug, Clone)]
pub struct Collider2DBuilder {
  shape: ColliderShape2D,
  local_offset: [f32; 2],
  local_rotation: f32,
  material: ColliderMaterial2D,
}

impl Collider2DBuilder {
  /// Creates a circle collider builder with stable defaults.
  pub fn circle(radius: f32) -> Self;

  /// Creates a rectangle collider builder with stable defaults.
  pub fn rectangle(half_width: f32, half_height: f32) -> Self;

  /// Creates a capsule collider builder with stable defaults.
  pub fn capsule(half_height: f32, radius: f32) -> Self;

  /// Creates a convex polygon collider builder with stable defaults.
  pub fn polygon(vertices: Vec<[f32; 2]>) -> Self;

  /// Creates a convex polygon collider builder by copying from a slice.
  pub fn polygon_from_slice(vertices: &[[f32; 2]]) -> Self;

  /// Sets the collider local offset relative to the owning body origin, in
  /// meters.
  pub fn with_offset(self, x: f32, y: f32) -> Self;

  /// Sets the collider local rotation relative to the owning body, in radians.
  pub fn with_local_rotation(self, radians: f32) -> Self;

  /// Sets the collider material parameters.
  pub fn with_material(self, material: ColliderMaterial2D) -> Self;

  /// Sets the density, in kg/m², for mass property computation.
  pub fn with_density(self, density: f32) -> Self;

  /// Sets the friction coefficient (unitless, `>= 0.0`).
  pub fn with_friction(self, friction: f32) -> Self;

  /// Sets the restitution coefficient in `[0.0, 1.0]`.
  pub fn with_restitution(self, restitution: f32) -> Self;

  /// Attaches the collider to a body and returns a world-scoped handle.
  pub fn build(
    self,
    world: &mut PhysicsWorld2D,
    body: RigidBody2D,
  ) -> Result<Collider2D, Collider2DError>;
}
```

Notes
- `Collider2D` is a handle and MUST NOT own simulation state.
- The `build()` signature includes `PhysicsWorld2D` to match handle validation
  patterns established by `RigidBody2D`.

### Behavior

Coordinate system and units
- Shape dimensions, local offsets, and positions MUST be interpreted as meters.
- Local rotations MUST be interpreted as radians around the 2D Z axis, with
  positive values rotating counterclockwise.

Local transform (normative)
- Each collider MUST have a local transform relative to the owning body:
  - `local_offset`: translation in meters.
  - `local_rotation`: rotation in radians.
- The collider pose in world space MUST be the composition of:
  - body pose (position and rotation), then
  - collider local transform (offset and rotation).

Attachment and ownership
- `Collider2DBuilder::build()` MUST attach the collider to the given `body` in
  the given `world`.
- Multiple colliders MAY be attached to a single body. Each collider MUST
  participate independently in collision detection and contact generation.
- Colliders MUST be supported on `Static`, `Dynamic`, and `Kinematic` bodies.

Builder defaults (normative)
- The builder MUST provide stable defaults:
  - `local_offset`: `(0.0, 0.0)`
  - `local_rotation`: `0.0`
  - `material.density`: `1.0`
  - `material.friction`: `0.5`
  - `material.restitution`: `0.0`

Shape semantics
- `Circle { radius }` represents a circle centered at the collider origin.
- `Rectangle { half_width, half_height }` represents an axis-aligned rectangle
  in collider local space before applying the collider local rotation.
- `Capsule { half_height, radius }` represents a vertical capsule aligned with
  the collider local Y axis before applying the collider local rotation, with:
  - a central segment of length `2.0 * half_height`
  - semicircular end caps of radius `radius`
- `ConvexPolygon { vertices }` represents a convex polygon in collider local
  space before applying the collider local rotation, with the vertex order
  determining the boundary.

Collision detection
- During `PhysicsWorld2D::step()`, colliders MUST participate in broad-phase
  and narrow-phase collision detection as implemented by the backend.
- A dynamic body with at least one collider MUST respond to collisions with:
  - static bodies with colliders
  - kinematic bodies with colliders
  - other dynamic bodies with colliders

Collision response (normative)
- Contact resolution MUST update linear velocities for dynamic bodies such that
  penetrations are resolved and restitution and friction affect motion.
- Angular dynamics are out of scope for the initial 2D physics surface.
  Therefore, collision response MUST NOT introduce angular velocity and MUST
  NOT change `RigidBody2D` rotation during `step()`.

Material properties
- `density` MUST affect mass properties for dynamic bodies when the body mass
  is not explicitly specified by `RigidBody2DBuilder::with_dynamic_mass_kg`.
- `density` MUST NOT affect mass properties for static or kinematic bodies.
- `friction` MUST affect tangential contact response.
- `friction` values greater than `1.0` MUST be supported.
- `restitution` MUST affect normal contact response (bounce).

Material combination rules (backend-agnostic)
- When two colliders contact, the effective coefficients MUST be:
  - `combined_friction = sqrt(friction_a * friction_b)`
  - `combined_restitution = max(restitution_a, restitution_b)`

Mass properties
- When a dynamic body has no explicitly configured mass, the body mass MUST be
  computed from attached colliders with `density > 0.0` as:
  - `mass_kg = sum(shape_area_m2 * density)`
- If the computed mass is zero (no positive-density colliders), mass MUST
  default to `1.0` kg.
- When attaching a collider to a dynamic body without an explicitly configured
  mass, the body mass MUST be recomputed to include the newly attached
  collider.

Compound shapes
- With multiple colliders, collision detection and response MUST consider all
  colliders for contact generation.
- Mass computation MUST include all attached colliders that contribute mass.

### Validation and Errors

Validation principles
- Builder construction MUST accept potentially invalid parameters.
- `Collider2DBuilder::build()` MUST validate parameters and return actionable
  errors without panicking.

Input validation (normative)
- All floating-point inputs MUST be finite, including shape parameters, local
  transform parameters, and material parameters.

Shape validation (normative)
- Circle:
  - `radius` MUST be `> 0.0`.
- Rectangle:
  - `half_width` MUST be `> 0.0`.
  - `half_height` MUST be `> 0.0`.
- Capsule:
  - `radius` MUST be `> 0.0`.
  - `half_height` MUST be `>= 0.0` (zero yields a circle).
- Convex polygon:
  - `vertices.len()` MUST be `>= 3`.
  - `vertices.len()` MUST be `<= MAX_CONVEX_POLYGON_VERTICES`.
  - Polygon MUST be strictly convex.
  - Polygon MUST have non-zero signed area.
  - Vertices SHOULD be supplied in counterclockwise order. If the polygon is
    valid but clockwise, the implementation SHOULD accept it by reversing the
    vertex order.

Material validation (normative)
- `density` MUST be `>= 0.0`.
- `friction` MUST be `>= 0.0`.
- `restitution` MUST be in `[0.0, 1.0]`.

Handle validation (normative)
- All collider operations MUST validate:
  - The `Collider2D` world identifier matches the provided `PhysicsWorld2D`.
  - The slot exists and the generation matches (stale-handle detection).

Errors (draft)
- `Collider2DError` MUST include actionable variants for:
  - world mismatch / invalid handle
  - body not found
  - invalid shape parameters (including per-shape details)
  - polygon too many vertices
  - invalid material parameters

### Cargo Features

- All public APIs in this work item MUST be gated under the existing umbrella
  feature `physics-2d` (crate: `lambda-rs`).
- The platform backend support MUST be enabled via
  `lambda-rs-platform/physics-2d`.
- No additional feature flags are introduced by this specification.

## Constraints and Rules

- Public APIs MUST remain backend-agnostic and MUST NOT expose vendor types.
- Public APIs MUST NOT panic on invalid user-provided parameters.
- The collider attachment API MUST be world-scoped and MUST reject cross-world
  handles.
- This work item MUST NOT add collision callbacks, layers/masks, or concave
  shapes.

## Performance Considerations

- Collider insertion SHOULD avoid allocations beyond what is required for
  storing polygon vertices.
- Convex polygon validation SHOULD be linear or near-linear in vertex count
  and MUST avoid quadratic algorithms for common sizes where feasible.
- The polygon vertex cap MUST prevent backend-dependent allocation blowups and
  MUST fail with a deterministic error if exceeded.
- Attaching multiple colliders to a body MAY increase contact pairs; tests MUST
  include a compound-body scenario to detect regressions.

## Requirements Checklist

- [ ] Circle colliders detect collisions correctly.
- [ ] Rectangle colliders detect collisions correctly.
- [ ] Collider local rotation produces oriented shapes correctly.
- [ ] Capsule colliders support character-like shapes.
- [ ] Polygon colliders support arbitrary convex shapes.
- [ ] Multiple colliders attached to one body work together.
- [ ] Friction and restitution affect collision response.
- [ ] Density affects dynamic body mass when not explicitly set.
- [ ] No public vendor types are exposed from `lambda-rs`.

## Verification and Testing

Unit tests (crate: `lambda-rs`)
- Validate `Collider2DBuilder::build()` rejects invalid parameters.
- Validate world mismatch and stale-handle behavior for `Collider2D`.

Integration tests (crate: `lambda-rs`, `crates/lambda-rs/tests/runnables.rs`)
- Circle vs. circle:
  - Two dynamic bodies with circle colliders converge, collide, and separate.
- Rectangle vs. rectangle:
  - A dynamic box falls onto a static box and comes to rest.
- Oriented box:
  - A rotated rectangle collider behaves as expected when colliding with a
    static ground rectangle.
- Capsule character:
  - A dynamic capsule falls onto a static ground rectangle and does not snag.
- Convex polygon:
  - A dynamic convex polygon collides with a static rectangle.
- Compound shape:
  - A dynamic body with two colliders collides as expected (both colliders
    generate contacts).
- Friction and restitution:
  - Restitution `0.0` yields minimal bounce; restitution `1.0` yields maximal
    bounce in a controlled scenario.
  - A slope test demonstrates friction affecting sliding behavior.

Manual verification (optional)
- A minimal example MAY be added under `crates/lambda-rs/examples/` to render a
  simple debug visualization and visually verify compound shapes and capsule
  behavior. This example MUST remain optional and gated behind `physics-2d`.

## Compatibility and Migration

- This work item adds new APIs under `physics-2d` and is additive for existing
  users.
- Mass computation for dynamic bodies MAY be affected for bodies that do not
  explicitly set mass and then attach positive-density colliders. This behavior
  is required for density-based mass, and must be documented in the public API
  docs for `Collider2DBuilder::with_density` and `build()`.

## Changelog

- 2026-02-17 0.1.0: Define 2D collider shapes and attachment APIs.
- 2026-02-17 0.1.1: Specify defaults and mass recomputation rules.
- 2026-02-17 0.1.2: Add local rotation, material struct, and polygon limits.
