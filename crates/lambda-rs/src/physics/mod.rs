//! 2D physics world support.
//!
//! This module provides a minimal, fixed-timestep 2D physics world. The public
//! API is backend-agnostic and does not expose vendor types.

use std::{
  cell::RefCell,
  collections::HashSet,
  error::Error,
  fmt,
  mem,
  sync::atomic::{
    AtomicU64,
    Ordering,
  },
};

use lambda_platform::physics::{
  CollisionEvent2DBackend,
  CollisionEventKind2DBackend,
  PhysicsBackend2D,
  RaycastHit2DBackend,
};

mod collider_2d;
mod rigid_body_2d;

pub use collider_2d::{
  Collider2D,
  Collider2DBuilder,
  Collider2DError,
  ColliderMaterial2D,
  ColliderShape2D,
  MAX_CONVEX_POLYGON_VERTICES,
};
pub use rigid_body_2d::{
  RigidBody2D,
  RigidBody2DBuilder,
  RigidBody2DError,
  RigidBodyType,
};

const DEFAULT_COLLISION_FILTER_GROUP: u32 = u32::MAX;
const DEFAULT_COLLISION_FILTER_MASK: u32 = u32::MAX;
const DEFAULT_GRAVITY_X: f32 = 0.0;
const DEFAULT_GRAVITY_Y: f32 = -9.81;
const DEFAULT_TIMESTEP_SECONDS: f32 = 1.0 / 60.0;
const DEFAULT_SUBSTEPS: u32 = 1;

static NEXT_WORLD_ID: AtomicU64 = AtomicU64::new(1);

/// Indicates whether a collision pair started or ended contact this step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollisionEventKind {
  /// The two bodies started touching during the most recent simulation step.
  Started,
  /// The two bodies stopped touching during the most recent simulation step.
  Ended,
}

/// Describes a body pair collision observed during simulation stepping.
///
/// The body pair is unordered. `body_a` and `body_b` identify the two bodies
/// involved in the event, but their positions are not stable or semantically
/// meaningful across runs, backends, or separate events. Callers MUST treat
/// the pair as unordered and MUST NOT rely on one body always appearing in the
/// same field.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CollisionEvent {
  /// The event transition kind for this body pair.
  pub kind: CollisionEventKind,
  /// One body participating in the unordered collision pair.
  pub body_a: RigidBody2D,
  /// The other body participating in the unordered collision pair.
  pub body_b: RigidBody2D,
  /// The representative contact point, when available.
  pub contact_point: Option<[f32; 2]>,
  /// The representative contact normal, when available.
  pub normal: Option<[f32; 2]>,
  /// The representative penetration depth, when available.
  pub penetration: Option<f32>,
}

/// Configures collider collision group and mask bitfields.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CollisionFilter {
  /// The membership bitfield for this collider.
  pub group: u32,
  /// The bitfield describing which groups this collider can collide with.
  pub mask: u32,
}

impl Default for CollisionFilter {
  fn default() -> Self {
    return Self {
      group: DEFAULT_COLLISION_FILTER_GROUP,
      mask: DEFAULT_COLLISION_FILTER_MASK,
    };
  }
}

/// Describes the nearest body hit by a ray query.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RaycastHit {
  /// The rigid body hit by the ray.
  pub body: RigidBody2D,
  /// The world-space hit position.
  pub point: [f32; 2],
  /// The world-space hit normal.
  pub normal: [f32; 2],
  /// The non-negative distance from the origin to the hit point.
  pub distance: f32,
}

/// A 2D physics simulation world.
pub struct PhysicsWorld2D {
  world_id: u64,
  gravity: [f32; 2],
  timestep_seconds: f32,
  substeps: u32,
  backend: PhysicsBackend2D,
  queued_collision_events: RefCell<Vec<CollisionEvent>>,
}

impl PhysicsWorld2D {
  /// Advances the world by one configured fixed timestep.
  ///
  /// # Returns
  /// Returns `()` after stepping the simulation.
  pub fn step(&mut self) {
    let substep_timestep_seconds = self.timestep_seconds / self.substeps as f32;

    for _ in 0..self.substeps {
      self
        .backend
        .step_with_timestep_seconds(substep_timestep_seconds);
    }

    let backend_events = self.backend.drain_collision_events_2d();
    self.queue_backend_collision_events(backend_events);
    self.backend.clear_rigid_body_forces_2d();

    return;
  }

  /// Returns the configured gravity.
  ///
  /// # Returns
  /// Returns the gravity vector in meters per second squared.
  pub fn gravity(&self) -> [f32; 2] {
    return self.gravity;
  }

  /// Returns the configured fixed timestep in seconds.
  ///
  /// # Returns
  /// Returns the timestep in seconds.
  pub fn timestep_seconds(&self) -> f32 {
    return self.timestep_seconds;
  }

  /// Drains collision events collected by prior `step()` calls.
  ///
  /// Collision events are produced while the world advances and then buffered
  /// until gameplay code asks for them. Draining here keeps the simulation step
  /// free of user callbacks and makes event consumption explicit, which is
  /// easier to integrate into fixed-update loops than re-entrant callback
  /// dispatch during contact resolution. Events remain queued across multiple
  /// `step()` calls until this method drains them.
  ///
  /// # Returns
  /// Returns an iterator over the queued collision events, draining the queue
  /// as part of iteration creation.
  pub fn collision_events(&self) -> impl Iterator<Item = CollisionEvent> {
    let queued_events: Vec<CollisionEvent> =
      mem::take(&mut *self.queued_collision_events.borrow_mut());
    return queued_events.into_iter();
  }

  /// Returns all bodies whose colliders contain the provided point.
  ///
  /// Point queries are intended for gameplay checks that can be called freely
  /// during update code. Treating invalid floating-point input as a miss keeps
  /// the public API simple for callers and avoids forcing game code to thread
  /// backend validation errors through every query site.
  ///
  /// # Arguments
  /// - `point`: The world-space point to test.
  ///
  /// # Returns
  /// Returns a vector of matching rigid body handles.
  pub fn query_point(&self, point: [f32; 2]) -> Vec<RigidBody2D> {
    if !is_valid_query_point(point) {
      return Vec::new();
    }

    // Backend queries operate in collider space, but the public API reports
    // owning bodies. Rebuild body handles and collapse duplicate hits from
    // compound colliders before returning the result.
    let body_slots = self.backend.query_point_2d(point);
    return self.deduplicate_query_body_hits(body_slots);
  }

  /// Returns all bodies whose colliders overlap the provided axis-aligned box.
  ///
  /// AABB queries are meant to be tolerant of how gameplay code produces
  /// bounds. The world normalizes `min` and `max` before delegating so callers
  /// can pass corners in either order without adding their own pre-processing.
  /// Invalid floating-point inputs are treated as a miss for the same reason as
  /// `query_point()`: query call sites stay straightforward and do not need to
  /// handle backend-specific error types.
  ///
  /// # Arguments
  /// - `min`: The minimum world-space corner of the query box.
  /// - `max`: The maximum world-space corner of the query box.
  ///
  /// # Returns
  /// Returns a vector of matching rigid body handles.
  pub fn query_aabb(&self, min: [f32; 2], max: [f32; 2]) -> Vec<RigidBody2D> {
    if !is_valid_query_point(min) || !is_valid_query_point(max) {
      return Vec::new();
    }

    // Normalize the query box so callers do not need to sort the bounds first.
    let normalized_min = [min[0].min(max[0]), min[1].min(max[1])];
    let normalized_max = [min[0].max(max[0]), min[1].max(max[1])];
    let body_slots = self.backend.query_aabb_2d(normalized_min, normalized_max);

    return self.deduplicate_query_body_hits(body_slots);
  }

  /// Returns the nearest rigid body hit by the provided ray.
  ///
  /// Raycasts are exposed as a lightweight gameplay query rather than a
  /// fallible backend operation. Inputs that cannot represent a meaningful
  /// finite segment are treated as a miss, and the backend hit is rebound to
  /// this world's public body handles before returning so the high-level API
  /// stays vendor-free.
  ///
  /// # Arguments
  /// - `origin`: The ray origin in world space.
  /// - `dir`: The ray direction in world space.
  /// - `max_dist`: The maximum query distance.
  ///
  /// # Returns
  /// Returns the nearest hit, if one exists.
  pub fn raycast(
    &self,
    origin: [f32; 2],
    dir: [f32; 2],
    max_dist: f32,
  ) -> Option<RaycastHit> {
    if !is_valid_query_point(origin)
      || !is_valid_query_direction(dir)
      || !max_dist.is_finite()
      || max_dist <= 0.0
    {
      return None;
    }

    // The backend performs the geometry query and returns backend-neutral hit
    // data, which we then bind back to this world's public rigid body handles.
    let hit = self.backend.raycast_2d(origin, dir, max_dist)?;
    return Some(self.map_backend_raycast_hit(hit));
  }

  /// Rebuilds and deduplicates rigid body handles from backend query hits.
  ///
  /// # Arguments
  /// - `body_slots`: Backend `(slot_index, slot_generation)` pairs.
  ///
  /// # Returns
  /// Returns one `RigidBody2D` handle per unique body, preserving first-hit
  /// order from the backend query.
  fn deduplicate_query_body_hits(
    &self,
    body_slots: Vec<(u32, u32)>,
  ) -> Vec<RigidBody2D> {
    let mut seen_bodies = HashSet::new();
    let mut bodies = Vec::new();

    for (slot_index, slot_generation) in body_slots {
      let body = RigidBody2D::from_backend_slot(
        self.world_id,
        slot_index,
        slot_generation,
      );

      // Spatial queries match colliders internally, but the public surface is
      // body-oriented. Compound bodies therefore collapse to one handle.
      if seen_bodies.insert(body) {
        bodies.push(body);
      }
    }

    return bodies;
  }

  /// Rebuilds a public raycast hit from backend slot and geometry data.
  ///
  /// # Arguments
  /// - `hit`: The backend hit payload.
  ///
  /// # Returns
  /// Returns a backend-agnostic `RaycastHit`.
  fn map_backend_raycast_hit(&self, hit: RaycastHit2DBackend) -> RaycastHit {
    return RaycastHit {
      body: RigidBody2D::from_backend_slot(
        self.world_id,
        hit.body_slot_index,
        hit.body_slot_generation,
      ),
      point: hit.point,
      normal: hit.normal,
      distance: hit.distance,
    };
  }

  /// Appends backend collision events to the public drain queue.
  ///
  /// The backend reports only world-local slot data. This helper rebinds those
  /// slots to world-scoped public handles and stores the results until
  /// `collision_events()` drains them.
  ///
  /// # Arguments
  /// - `backend_events`: The backend events to convert and queue.
  ///
  /// # Returns
  /// Returns `()` after queueing the converted events.
  fn queue_backend_collision_events(
    &self,
    backend_events: Vec<CollisionEvent2DBackend>,
  ) {
    let mapped_events = backend_events
      .into_iter()
      .map(|event| self.map_backend_collision_event(event));
    self
      .queued_collision_events
      .borrow_mut()
      .extend(mapped_events);

    return;
  }

  /// Rebuilds a public collision event from backend slot and contact data.
  ///
  /// # Arguments
  /// - `event`: The backend event payload.
  ///
  /// # Returns
  /// Returns a backend-agnostic public collision event.
  fn map_backend_collision_event(
    &self,
    event: CollisionEvent2DBackend,
  ) -> CollisionEvent {
    return CollisionEvent {
      kind: match event.kind {
        CollisionEventKind2DBackend::Started => CollisionEventKind::Started,
        CollisionEventKind2DBackend::Ended => CollisionEventKind::Ended,
      },
      body_a: RigidBody2D::from_backend_slot(
        self.world_id,
        event.body_a_slot_index,
        event.body_a_slot_generation,
      ),
      body_b: RigidBody2D::from_backend_slot(
        self.world_id,
        event.body_b_slot_index,
        event.body_b_slot_generation,
      ),
      contact_point: event.contact_point,
      normal: event.normal,
      penetration: event.penetration,
    };
  }
}

/// Builder for `PhysicsWorld2D`.
#[derive(Debug, Clone, Copy)]
pub struct PhysicsWorld2DBuilder {
  gravity: [f32; 2],
  timestep_seconds: f32,
  substeps: u32,
}

impl PhysicsWorld2DBuilder {
  /// Creates a builder with stable defaults.
  ///
  /// Defaults
  /// - Gravity: `(0.0, -9.81)`
  /// - Timestep: `1.0 / 60.0` seconds
  /// - Substeps: `1`
  ///
  /// # Returns
  /// Returns a new builder with default configuration.
  pub fn new() -> Self {
    return Self {
      gravity: [DEFAULT_GRAVITY_X, DEFAULT_GRAVITY_Y],
      timestep_seconds: DEFAULT_TIMESTEP_SECONDS,
      substeps: DEFAULT_SUBSTEPS,
    };
  }

  /// Sets gravity, in meters per second squared.
  ///
  /// # Arguments
  /// - `x`: The gravity acceleration on the X axis.
  /// - `y`: The gravity acceleration on the Y axis.
  ///
  /// # Returns
  /// Returns the updated builder.
  pub fn with_gravity(mut self, x: f32, y: f32) -> Self {
    self.gravity = [x, y];
    return self;
  }

  /// Sets the fixed timestep in seconds.
  ///
  /// # Arguments
  /// - `timestep_seconds`: The fixed timestep in seconds.
  ///
  /// # Returns
  /// Returns the updated builder.
  pub fn with_timestep_seconds(mut self, timestep_seconds: f32) -> Self {
    self.timestep_seconds = timestep_seconds;
    return self;
  }

  /// Sets the number of sub-steps per fixed timestep.
  ///
  /// # Arguments
  /// - `substeps`: The number of sub-steps per fixed timestep.
  ///
  /// # Returns
  /// Returns the updated builder.
  pub fn with_substeps(mut self, substeps: u32) -> Self {
    self.substeps = substeps;
    return self;
  }

  /// Builds a validated `PhysicsWorld2D`.
  ///
  /// # Returns
  /// Returns a constructed `PhysicsWorld2D` on success.
  ///
  /// # Errors
  /// Returns `PhysicsWorld2DError` if any configuration value is invalid.
  pub fn build(self) -> Result<PhysicsWorld2D, PhysicsWorld2DError> {
    validate_gravity(self.gravity)?;
    validate_timestep_seconds(self.timestep_seconds)?;
    validate_substeps(self.substeps)?;

    let substep_timestep_seconds = self.timestep_seconds / self.substeps as f32;
    validate_timestep_seconds(substep_timestep_seconds)?;

    let backend = PhysicsBackend2D::new(self.gravity, substep_timestep_seconds);
    let world_id = allocate_world_id();

    return Ok(PhysicsWorld2D {
      world_id,
      gravity: self.gravity,
      timestep_seconds: self.timestep_seconds,
      substeps: self.substeps,
      backend,
      queued_collision_events: RefCell::new(Vec::new()),
    });
  }
}

impl Default for PhysicsWorld2DBuilder {
  fn default() -> Self {
    return Self::new();
  }
}

/// Construction-time configuration errors for `PhysicsWorld2D`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PhysicsWorld2DError {
  /// The configured timestep is non-finite or non-positive.
  InvalidTimestepSeconds { timestep_seconds: f32 },
  /// The configured substep count is invalid.
  InvalidSubsteps { substeps: u32 },
  /// The configured gravity is invalid.
  InvalidGravity { x: f32, y: f32 },
}

impl fmt::Display for PhysicsWorld2DError {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::InvalidTimestepSeconds { timestep_seconds } => {
        return write!(
          formatter,
          "invalid timestep_seconds: {timestep_seconds}"
        );
      }
      Self::InvalidSubsteps { substeps } => {
        return write!(formatter, "invalid substeps: {substeps}");
      }
      Self::InvalidGravity { x, y } => {
        return write!(formatter, "invalid gravity: ({x}, {y})");
      }
    }
  }
}

impl Error for PhysicsWorld2DError {}

fn validate_timestep_seconds(
  timestep_seconds: f32,
) -> Result<(), PhysicsWorld2DError> {
  if !timestep_seconds.is_finite() || timestep_seconds <= 0.0 {
    return Err(PhysicsWorld2DError::InvalidTimestepSeconds {
      timestep_seconds,
    });
  }

  return Ok(());
}

/// Validates that the configured substep count is non-zero.
fn validate_substeps(substeps: u32) -> Result<(), PhysicsWorld2DError> {
  if substeps < 1 {
    return Err(PhysicsWorld2DError::InvalidSubsteps { substeps });
  }

  return Ok(());
}

/// Validates that the configured gravity vector is finite.
fn validate_gravity(gravity: [f32; 2]) -> Result<(), PhysicsWorld2DError> {
  let x = gravity[0];
  let y = gravity[1];

  if !x.is_finite() || !y.is_finite() {
    return Err(PhysicsWorld2DError::InvalidGravity { x, y });
  }

  return Ok(());
}

/// Returns whether a query point contains only finite coordinates.
///
/// # Arguments
/// - `point`: The point to validate.
///
/// # Returns
/// Returns `true` when both coordinates are finite.
fn is_valid_query_point(point: [f32; 2]) -> bool {
  return point[0].is_finite() && point[1].is_finite();
}

/// Returns whether a ray/query direction has finite non-zero length.
///
/// # Arguments
/// - `direction`: The query direction to validate.
///
/// # Returns
/// Returns `true` when both components are finite and the vector is non-zero.
fn is_valid_query_direction(direction: [f32; 2]) -> bool {
  if !direction[0].is_finite() || !direction[1].is_finite() {
    return false;
  }

  return direction[0].hypot(direction[1]) > 0.0;
}

/// Allocates a non-zero unique world identifier.
fn allocate_world_id() -> u64 {
  loop {
    let id = NEXT_WORLD_ID.fetch_add(1, Ordering::Relaxed);
    if id != 0 {
      return id;
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  /// Builds a world using the default builder configuration.
  #[test]
  fn world_builds_with_defaults() {
    let world = PhysicsWorld2DBuilder::new().build().unwrap();

    assert_eq!(world.gravity(), [0.0, -9.81]);
    assert_eq!(world.timestep_seconds(), 1.0 / 60.0);

    assert_eq!(world.backend.gravity(), [0.0, -9.81]);
    assert_eq!(world.backend.timestep_seconds(), 1.0 / 60.0);

    return;
  }

  /// Builds a world with custom gravity, timestep, and substeps.
  #[test]
  fn world_builds_with_custom_config() {
    let world = PhysicsWorld2DBuilder::new()
      .with_gravity(1.0, 2.0)
      .with_timestep_seconds(0.5)
      .with_substeps(2)
      .build()
      .unwrap();

    assert_eq!(world.gravity(), [1.0, 2.0]);
    assert_eq!(world.timestep_seconds(), 0.5);
    assert_eq!(world.substeps, 2);

    assert_eq!(world.backend.gravity(), [1.0, 2.0]);
    assert_eq!(world.backend.timestep_seconds(), 0.25);

    return;
  }

  /// Rejects timestep values that are positive but invalid for integration.
  #[test]
  fn build_rejects_non_positive_timestep_seconds() {
    let error = match PhysicsWorld2DBuilder::new()
      .with_timestep_seconds(0.0)
      .build()
    {
      Ok(_) => {
        panic!("expected build() to fail");
      }
      Err(error) => error,
    };

    assert_eq!(
      error,
      PhysicsWorld2DError::InvalidTimestepSeconds {
        timestep_seconds: 0.0,
      }
    );

    return;
  }

  /// Rejects non-finite timestep values.
  #[test]
  fn build_rejects_non_finite_timestep_seconds() {
    let error = match PhysicsWorld2DBuilder::new()
      .with_timestep_seconds(f32::NAN)
      .build()
    {
      Ok(_) => {
        panic!("expected build() to fail");
      }
      Err(error) => error,
    };

    match error {
      PhysicsWorld2DError::InvalidTimestepSeconds { timestep_seconds } => {
        assert!(timestep_seconds.is_nan());
      }
      _ => {
        panic!("expected InvalidTimestepSeconds, got: {error:?}");
      }
    }

    return;
  }

  /// Rejects zero substeps to avoid divide-by-zero in derived substep timestep.
  #[test]
  fn build_rejects_zero_substeps() {
    let error = match PhysicsWorld2DBuilder::new().with_substeps(0).build() {
      Ok(_) => {
        panic!("expected build() to fail");
      }
      Err(error) => error,
    };

    assert_eq!(error, PhysicsWorld2DError::InvalidSubsteps { substeps: 0 });

    return;
  }

  /// Rejects gravity vectors containing non-finite components.
  #[test]
  fn build_rejects_non_finite_gravity() {
    let error = match PhysicsWorld2DBuilder::new()
      .with_gravity(f32::INFINITY, 0.0)
      .build()
    {
      Ok(_) => {
        panic!("expected build() to fail");
      }
      Err(error) => error,
    };

    assert_eq!(
      error,
      PhysicsWorld2DError::InvalidGravity {
        x: f32::INFINITY,
        y: 0.0,
      }
    );

    return;
  }

  /// Ensures stepping an empty world succeeds without panicking.
  #[test]
  fn step_does_not_panic_for_empty_world() {
    let mut world = PhysicsWorld2DBuilder::new().build().unwrap();
    world.step();

    return;
  }

  /// Ensures `step()` uses the derived substep timestep when substeps are set.
  #[test]
  fn step_uses_substep_timestep_seconds() {
    let mut world = PhysicsWorld2DBuilder::new()
      .with_timestep_seconds(1.0)
      .with_substeps(4)
      .build()
      .unwrap();

    world.step();
    assert_eq!(world.backend.timestep_seconds(), 0.25);

    return;
  }

  /// Exposes stable defaults for collider collision filtering.
  #[test]
  fn collision_filter_default_collides_with_all_groups() {
    let filter = CollisionFilter::default();

    assert_eq!(filter.group, u32::MAX);
    assert_eq!(filter.mask, u32::MAX);

    return;
  }

  /// Returns empty query and event results for an empty world.
  #[test]
  fn empty_world_collision_queries_return_no_results() {
    let world = PhysicsWorld2DBuilder::new().build().unwrap();

    assert_eq!(world.collision_events().count(), 0);
    assert!(world.query_point([0.0, 0.0]).is_empty());
    assert!(world.query_aabb([-1.0, -1.0], [1.0, 1.0]).is_empty());
    assert_eq!(world.raycast([0.0, 0.0], [1.0, 0.0], 10.0), None);

    return;
  }
}
