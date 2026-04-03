---
title: "2D Collision Queries and Events"
document_id: "collision-queries-events-2d-2026-03-25"
status: "draft"
created: "2026-03-25T16:39:52Z"
last_updated: "2026-03-25T16:39:52Z"
version: "0.1.0"
engine_workspace_version: "2023.1.30"
wgpu_version: "26.0.1"
shader_backend_default: "naga"
winit_version: "0.29.10"
repo_commit: "f3c56aaa0985993cc7e751865913e7a2ef27040e"
owners: ["lambda-sh"]
reviewers: ["engine"]
tags: ["spec", "physics", "2d", "lambda-rs", "platform"]
---

# 2D Collision Queries and Events

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

- Introduce collision event delivery for `PhysicsWorld2D` so applications can
  respond when two bodies begin or end contact.
- Introduce read-only spatial queries for point overlap, axis-aligned bounding
  box (AABB) overlap, and raycasts without advancing the simulation.
- Introduce per-collider collision filtering through `CollisionFilter`
  group/mask bitmasks.
- Keep the public API backend-agnostic while allowing
  `lambda-rs-platform` to translate backend contact and query results into the
  stable `lambda-rs` contract.

Rationale
- 2D rigid bodies and colliders exist, but gameplay code still lacks a stable
  way to detect contacts and inspect the world outside of solver stepping.
- Collision events and read-only queries provide the minimum interaction layer
  required for common gameplay systems such as pickups, hit scans, grounded
  checks, and scripted physics responses.

## Scope

### Goals

- Provide collision start and end notifications after `PhysicsWorld2D::step()`.
- Provide representative contact information for collision start events.
- Provide per-collider collision filtering via layers and masks.
- Provide point overlap queries that return owning `RigidBody2D` handles.
- Provide AABB overlap queries that return owning `RigidBody2D` handles.
- Provide raycasts that return the nearest hit body and hit information.
- Support collision and query inspection without requiring a simulation step in
  the same frame.

### Non-Goals

- Trigger volumes, sensors, and overlap-only colliders.
- Arbitrary user callback registration or invocation during stepping.
- Query-side filtering rules beyond the built-in geometry tests.
- Complex filtering expressions, tag systems, or scriptable predicates.
- Public exposure of backend/vendor contact, query, or shape types.

## Terminology

- Collision event: a notification emitted when a pair of bodies transitions
  into or out of contact.
- Contact point: a representative world-space point on the collision manifold
  for a collision start event.
- Normal: a unit-length world-space vector pointing from `body_a` toward
  `body_b` for the representative collision contact.
- Penetration: the representative overlap depth in meters for a collision
  start event.
- Collision filter: a pair of bitmasks (`group`, `mask`) used to determine
  whether two colliders may generate contacts.
- Overlap query: a read-only test that returns all bodies whose colliders
  intersect a geometric region.
- Raycast: a read-only intersection test along a finite ray segment.
- AABB: axis-aligned bounding box.

## Architecture Overview

Dependencies
- This work item depends on the following specifications:
  - `docs/specs/physics/physics-world-2d.md`
  - `docs/specs/physics/rigid-bodies-2d.md`
  - `docs/specs/physics/colliders-2d.md`

Crate boundaries
- Crate `lambda` (package: `lambda-rs`)
  - MUST expose collision events, filters, and query APIs through the public
    `physics` module.
  - MUST NOT expose backend/vendor types.
- Crate `lambda_platform` (package: `lambda-rs-platform`)
  - MUST own backend event collection, contact extraction, filter mapping, and
    spatial query execution.
  - MUST translate backend-specific results into the public types defined by
    this specification.

Data flow

```text
application
  └── lambda::physics::PhysicsWorld2D
        ├── post-step collision event queue
        ├── query_point / query_aabb / raycast
        └── lambda_platform::physics::PhysicsBackend2D (internal)
              └── vendor crate (for example, rapier2d)
```

## Design

### API Surface

Module layout (new or extended)

- `crates/lambda-rs/src/physics/mod.rs`
  - Re-export `CollisionEvent`, `CollisionEventKind`, `CollisionFilter`, and
    `RaycastHit`.
  - Expose new `PhysicsWorld2D` query and event entry points.
- `crates/lambda-rs/src/physics/collider_2d.rs`
  - Extend `Collider2DBuilder` with collision filter configuration.
- `crates/lambda-rs-platform/src/physics/mod.rs`
  - Add internal query, event, and filter support for the active 2D physics
    backend.

### lambda-rs Public API

Public entry points (draft)

```rust
/// The type of collision transition represented by a `CollisionEvent`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollisionEventKind {
  Started,
  Ended,
}

/// Per-collider collision filter bitmasks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CollisionFilter {
  pub group: u32,
  pub mask: u32,
}

/// A collision transition between two bodies.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CollisionEvent {
  pub kind: CollisionEventKind,
  pub body_a: RigidBody2D,
  pub body_b: RigidBody2D,
  pub contact_point: Option<[f32; 2]>,
  pub normal: Option<[f32; 2]>,
  pub penetration: Option<f32>,
}

/// The nearest ray intersection against colliders in a `PhysicsWorld2D`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RaycastHit {
  pub body: RigidBody2D,
  pub point: [f32; 2],
  pub normal: [f32; 2],
  pub distance: f32,
}

impl PhysicsWorld2D {
  /// Returns and drains collision events collected by prior `step()` calls.
  pub fn collision_events(&self) -> impl Iterator<Item = CollisionEvent>;

  /// Returns all bodies whose colliders contain `point`.
  pub fn query_point(&self, point: [f32; 2]) -> Vec<RigidBody2D>;

  /// Returns all bodies whose colliders overlap the given AABB.
  pub fn query_aabb(
    &self,
    min: [f32; 2],
    max: [f32; 2],
  ) -> Vec<RigidBody2D>;

  /// Returns the nearest hit on the finite ray segment.
  pub fn raycast(
    &self,
    origin: [f32; 2],
    dir: [f32; 2],
    max_dist: f32,
  ) -> Option<RaycastHit>;
}

impl Collider2DBuilder {
  /// Sets the collision filter used for collider-vs-collider contact tests.
  pub fn with_collision_filter(self, filter: CollisionFilter) -> Self;
}
```

Notes
- `collision_events()` uses post-step iteration rather than callback
  registration.
- `CollisionEvent` is body-scoped rather than collider-scoped. Multiple
  colliders on the same pair of bodies MUST be coalesced into one body-pair
  event stream.

### Behavior

Collision filter defaults
- `CollisionFilter` MUST default to:
  - `group = u32::MAX`
  - `mask = u32::MAX`
- The default filter MUST preserve the pre-filter behavior where all colliders
  are eligible to collide.

Collision filter rule
- Two colliders MUST be eligible to generate contacts only when:
  - `(a.group & b.mask) != 0`
  - `(b.group & a.mask) != 0`
- If either condition fails, the pair MUST NOT generate solver contacts and
  MUST NOT contribute to collision start or end events.
- Filtering is defined per collider. A body with multiple colliders MAY collide
  with another body through one collider pair while another pair is filtered
  out.

Event collection and delivery
- `PhysicsWorld2D::step()` MUST collect collision transitions that occur during
  the step.
- `PhysicsWorld2D::collision_events()` MUST drain all currently queued events.
  Calling it again before the next queued event is added MUST yield an empty
  iterator.
- If `step()` is called multiple times before `collision_events()`, the world
  MUST retain all queued events until they are drained.
- The event queue MUST contain at most one `Started` event when a body pair
  transitions from not touching to touching.
- The event queue MUST contain at most one `Ended` event when a body pair
  transitions from touching to not touching.
- `body_a` and `body_b` identify an unordered pair. Consumers MUST NOT depend
  on their ordering.

Body-pair aggregation
- When multiple collider pairs connect the same two bodies, event generation
  MUST be body-pair scoped:
  - `Started` is emitted when the first eligible collider pair begins contact.
  - `Ended` is emitted when the last eligible collider pair stops contact.
- A body MUST NOT generate collision events against itself.

Representative contact data
- `CollisionEventKind::Started` MUST populate:
  - `contact_point`
  - `normal`
  - `penetration`
- `CollisionEventKind::Ended` MUST set those fields to `None`.
- For `Started` events with multiple candidate contacts between the same body
  pair, the implementation MUST report one representative contact using the
  deepest penetration from that step.
- `normal` MUST be unit length.
- `penetration` MUST be `>= 0.0`.
- Contact positions, normals, and penetration values MUST be reported in
  world-space meters.

Event ordering
- Event order MUST be stable for a single run of a single backend when the
  same simulation inputs are replayed.
- Cross-platform or cross-backend deterministic ordering is NOT required.

Spatial query semantics
- `query_point`, `query_aabb`, and `raycast` MUST be read-only and MUST NOT
  advance the simulation.
- Spatial queries MUST operate on the current collider transforms in the
  world, including changes caused by prior `step()` calls and direct rigid body
  mutation.
- Query results MUST return bodies, not colliders.
- If more than one collider on the same body matches a query, that body MUST
  appear only once in the returned result.

Point queries
- `query_point()` MUST return every body with at least one collider containing
  the given point.
- Points on the collider boundary MUST count as hits.

AABB queries
- `query_aabb()` MUST return every body with at least one collider overlapping
  the query box.
- The implementation MUST accept `min` and `max` in any order by normalizing
  them to component-wise minimum/maximum bounds before executing the query.
- Boundary-touching overlaps MUST count as hits.

Raycasts
- `raycast()` MUST test the finite segment starting at `origin` and extending
  in direction `dir` for `max_dist` meters.
- `dir` MUST NOT be required to be pre-normalized by the application.
- The implementation MUST normalize `dir` internally before computing the hit
  distance and normal.
- `raycast()` MUST return the nearest hit only.
- `RaycastHit::distance` MUST be measured in meters from `origin` along the
  finite ray segment and MUST be in `[0.0, max_dist]`.
- `RaycastHit::normal` MUST be unit length.
- If the ray origin lies inside a collider, the query MUST return a hit with
  `distance = 0.0`.

Filtering and queries
- The collision filter defined by `CollisionFilter` applies only to
  collider-vs-collider contact generation and event generation.
- `query_point`, `query_aabb`, and `raycast` MUST ignore collision filter
  masks in this work item because the query APIs do not accept query-side
  filter inputs.

Intersection tests without simulation
- Applications MUST be able to use `query_point`, `query_aabb`, and
  `raycast` without calling `step()` in the same frame.
- This work item MUST NOT introduce a separate boolean-only intersection API.
  The spatial query set above satisfies the non-simulation intersection
  requirement.

### Validation and Errors

Validation principles
- Public APIs in this work item MUST NOT panic on invalid query inputs.
- The query methods remain infallible in the public API surface. Invalid
  inputs MUST therefore produce empty results or `None` rather than public
  error types.

Input validation (normative)
- `query_point()`:
  - If either point component is non-finite, the result MUST be empty.
- `query_aabb()`:
  - If any bound component is non-finite, the result MUST be empty.
- `raycast()`:
  - If any `origin` or `dir` component is non-finite, the result MUST be
    `None`.
  - If `dir` has zero length, the result MUST be `None`.
  - If `max_dist` is non-finite or `<= 0.0`, the result MUST be `None`.

Filter validation
- All `u32` bit patterns for `group` and `mask` MUST be accepted.
- `with_collision_filter()` MUST store the provided filter verbatim.

Backend translation
- `lambda-rs-platform` MUST convert backend contact/query results into
  backend-agnostic public types.
- Backend-specific query misses, invalid-shape rejections, or internal cache
  states MUST NOT leak into the `lambda-rs` public API.

### Cargo Features

- All public APIs in this work item MUST be gated under the existing umbrella
  feature `physics-2d` (crate: `lambda-rs`).
- The platform backend support MUST remain enabled through
  `lambda-rs-platform/physics-2d`.
- No additional feature flags are introduced by this specification.
- `docs/features.md` MUST describe that `physics-2d` now covers:
  - collision events
  - collision filtering
  - point/AABB/raycast queries

## Constraints and Rules

- Public APIs MUST remain backend-agnostic and MUST NOT expose vendor types.
- This work item MUST NOT add trigger volumes or sensor callbacks.
- This work item MUST NOT require applications to register callbacks into the
  world.
- Query APIs MUST be safe to call between simulation steps.
- Event delivery MUST remain post-step and pull-based for this iteration.

## Performance Considerations

Recommendations
- Applications SHOULD drain `collision_events()` once per simulation step.
  - Rationale: Avoids unnecessary event queue growth.
- Point and AABB queries SHOULD use the backend broad phase when available.
  - Rationale: Keeps query cost proportional to candidate overlap count rather
    than total collider count.
- Raycasts SHOULD stop at the first confirmed nearest hit.
  - Rationale: Preserves the expected cost model for common hit-scan gameplay.
- Body-pair event aggregation SHOULD avoid duplicate allocations for compound
  collider contacts.
  - Rationale: Compound bodies can otherwise multiply transient event cost.

## Requirements Checklist

Functionality
- [ ] Collision start and end events are exposed through `collision_events()`.
- [ ] Collision start events include representative contact data.
- [ ] Collision filters prevent masked pairs from generating contacts.
- [ ] `query_point()` returns bodies containing the point.
- [ ] `query_aabb()` returns bodies overlapping the AABB.
- [ ] `raycast()` returns the nearest hit with point, normal, and distance.

API Surface
- [ ] Public query and event APIs are exposed through `lambda::physics`.
- [ ] `Collider2DBuilder` supports `with_collision_filter()`.
- [ ] Public APIs remain backend-agnostic and vendor-free.

Validation and Errors
- [ ] Invalid query inputs return empty results or `None` without panicking.
- [ ] Collision filter inputs accept all `u32` bitmasks.
- [ ] Event payload semantics for `Started` and `Ended` are documented.

Documentation and Examples
- [ ] `docs/features.md` reflects the expanded `physics-2d` behavior.
- [ ] The spec index lists this document under Physics.

## Verification and Testing

Unit tests (crate: `lambda-rs`)
- Verify that masked collider pairs do not generate events or physical
  contacts.
- Verify that the default filter allows collisions.
- Verify that `collision_events()` drains the queue and does not duplicate
  steady-state contacts across multiple steps.
- Verify invalid query inputs return empty results or `None`.

Integration tests (crate: `lambda-rs`)
- Integration entrypoint: `crates/lambda-rs/tests/integration.rs`.
- Feature-specific physics tests: `crates/lambda-rs/tests/physics_2d/`.
- Event coverage:
  - Two bodies first touch and emit one `Started` event.
  - Two bodies separate after contact and emit one `Ended` event.
  - `Started` includes representative contact point, normal, and penetration.
- Query coverage:
  - `query_point()` hits an interior point and misses an exterior point.
  - `query_aabb()` returns all overlapping bodies and deduplicates compound
    colliders on the same body.
  - `raycast()` returns the nearest hit with correct distance ordering.
  - `raycast()` returns `distance = 0.0` when starting inside a collider.
- Commands:
  - `cargo test -p lambda-rs --features physics-2d -- --nocapture`
  - `cargo test --workspace`

Manual verification
- Optional demo coverage MAY visualize:
  - contact start/end notifications
  - point/AABB query selections
  - raycast hit normals

## Compatibility and Migration

- This work item adds APIs under the existing `physics-2d` feature and is
  additive for current users.
- Existing applications that do not enable `physics-2d` are unaffected.
- Existing collider construction code remains source-compatible because the
  default collision filter preserves current collision behavior.

## Changelog

- 2026-03-25 0.1.0: Initial draft defining collision events, filters, and
  spatial queries for 2D physics.
