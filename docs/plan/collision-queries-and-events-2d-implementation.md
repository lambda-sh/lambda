Temporary implementation plan for
`docs/specs/physics/collision-queries-and-events-2d.md`.

Each step is intended to be independently testable and committable.
When executing this plan, implement exactly one step, run the listed
validation, and stop so the change can be reviewed and tested before moving to
the next step.

1) Add public API scaffolding and stable defaults (no backend behavior yet)
- Changes
  - Update `crates/lambda-rs/src/physics/mod.rs`:
    - Add `CollisionEventKind`.
    - Add `CollisionEvent`.
    - Add `RaycastHit`.
    - Add public `PhysicsWorld2D` methods:
      - `collision_events(&self)`
      - `query_point(&self, point: [f32; 2])`
      - `query_aabb(&self, min: [f32; 2], max: [f32; 2])`
      - `raycast(&self, origin: [f32; 2], dir: [f32; 2], max_dist: f32)`
    - Implement the new query methods as safe placeholders:
      - invalid inputs return empty results or `None`
      - valid inputs return no hits until backend support lands
  - Update `crates/lambda-rs/src/physics/collider_2d.rs`:
    - Add public `CollisionFilter`.
    - Add a `collision_filter` field to `Collider2DBuilder`.
    - Add `with_collision_filter(self, filter: CollisionFilter) -> Self`.
    - Default the filter to `group = u32::MAX`, `mask = u32::MAX`.
  - Re-export the new public types from `lambda::physics`.
- Validation
  - `cargo build -p lambda-rs --features physics-2d`
  - `cargo test -p lambda-rs --features physics-2d -- --nocapture`
- Commit message
  - `[add] physics collision query and event api scaffolding.`

2) Wire collision filters through the Rapier backend
- Changes
  - Update `crates/lambda-rs-platform/src/physics/rapier2d.rs`:
    - Add backend-side storage or translation for collider `group` and `mask`.
    - Map `CollisionFilter` to Rapier interaction groups when creating
      colliders.
    - Keep the public API vendor-free; the mapping stays inside the platform
      layer.
  - Update `crates/lambda-rs/src/physics/collider_2d.rs`:
    - Pass the configured filter through `Collider2DBuilder::build()`.
  - Add focused integration coverage in
    `crates/lambda-rs/tests/physics_2d/collision_filters.rs`:
    - default filters collide
    - masked pairs do not collide
    - one allowed pair still collides while another filtered pair does not
- Validation
  - `cargo test -p lambda-rs --features physics-2d collision_filters -- --nocapture`
- Commit message
  - `[add] physics collision filter backend support.`

3) Implement point and AABB overlap queries
- Changes
  - Update `crates/lambda-rs-platform/src/physics/rapier2d.rs`:
    - Add read-only point-query support.
    - Add read-only AABB-query support.
    - Return backend results in a form that can be mapped back to
      `RigidBody2D`.
  - Update `crates/lambda-rs/src/physics/mod.rs`:
    - Replace placeholder `query_point()` and `query_aabb()` behavior with real
      backend calls.
    - Normalize AABB bounds before querying.
    - Deduplicate bodies when multiple colliders on the same body match.
  - Add focused integration coverage in
    `crates/lambda-rs/tests/physics_2d/queries.rs`:
    - point hits interior and boundary points
    - point misses outside geometry
    - AABB hits overlapping bodies
    - AABB accepts inverted min/max input
    - compound colliders on one body produce one body handle
- Validation
  - `cargo test -p lambda-rs --features physics-2d queries -- --nocapture`
- Commit message
  - `[add] physics point and aabb queries.`

4) Implement nearest-hit raycasts
- Changes
  - Update `crates/lambda-rs-platform/src/physics/rapier2d.rs`:
    - Add finite-segment raycast support that returns the nearest hit.
    - Normalize the input direction internally.
    - Translate hit position, normal, and distance into backend-agnostic data.
  - Update `crates/lambda-rs/src/physics/mod.rs`:
    - Replace placeholder `raycast()` behavior with a backend call.
    - Preserve the infallible public contract:
      - non-finite inputs return `None`
      - zero-length direction returns `None`
      - non-positive `max_dist` returns `None`
  - Extend `crates/lambda-rs/tests/physics_2d/queries.rs`:
    - nearest hit wins
    - returned distance is measured in world meters
    - returned normal is unit length
    - origin-inside-collider returns a hit at distance `0.0`
- Validation
  - `cargo test -p lambda-rs --features physics-2d queries -- --nocapture`
- Commit message
  - `[add] physics raycast queries.`

5) Implement collision start events and queue draining
- Changes
  - Update `crates/lambda-rs-platform/src/physics/rapier2d.rs`:
    - Track body-pair contact state across steps.
    - Detect body-pair transitions from not touching to touching.
    - Extract a representative contact for the start event using the deepest
      penetration from the step.
  - Update `crates/lambda-rs/src/physics/mod.rs`:
    - Store queued `CollisionEvent` values on `PhysicsWorld2D`.
    - Drain queued events from `collision_events(&self)`.
    - Ensure the same steady-state contact does not emit repeated `Started`
      events every step.
  - Add focused integration coverage in
    `crates/lambda-rs/tests/physics_2d/collision_events.rs`:
    - first contact emits one `Started`
    - repeated steady-state contact emits no extra `Started`
    - drained queues are empty on the next read
    - `Started` includes point, normal, and penetration
- Validation
  - `cargo test -p lambda-rs --features physics-2d collision_events -- --nocapture`
- Commit message
  - `[add] physics collision start events.`

6) Implement collision end events and body-pair aggregation
- Changes
  - Update `crates/lambda-rs-platform/src/physics/rapier2d.rs`:
    - Detect body-pair transitions from touching to not touching.
    - Emit `Ended` only when the last eligible collider pair between two bodies
      separates.
    - Suppress self-events and preserve body-pair aggregation for compound
      colliders.
  - Update `crates/lambda-rs/src/physics/mod.rs`:
    - Queue `CollisionEventKind::Ended` with `None` contact fields.
    - Preserve queued events across multiple `step()` calls until drained.
  - Extend `crates/lambda-rs/tests/physics_2d/collision_events.rs`:
    - separation emits one `Ended`
    - `Ended` has no contact payload
    - compound colliders on one body still produce body-pair events, not
      collider-pair duplicates
    - multiple steps before draining preserve queued events
- Validation
  - `cargo test -p lambda-rs --features physics-2d collision_events -- --nocapture`
- Commit message
  - `[add] physics collision end events.`

7) Final verification and documentation alignment
- Changes
  - Cross-check implementation against
    `docs/specs/physics/collision-queries-and-events-2d.md`.
  - Update `docs/features.md` only if symbol names, runtime-cost wording, or
    feature coverage changed during implementation.
  - Add or adjust Rustdoc comments on all new public APIs and any non-trivial
    helpers introduced in prior steps.
  - Run formatting and lint cleanup required to leave the feature set in a
    shippable state.
- Validation
  - `cargo +nightly fmt --all`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo test -p lambda-rs --features physics-2d -- --nocapture`
  - `cargo test --workspace`
- Commit message
  - `[docs] finalize 2d collision queries and events implementation.`
