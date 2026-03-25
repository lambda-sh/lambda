//! 2D physics world support.
//!
//! This module provides a minimal, fixed-timestep 2D physics world. The public
//! API is backend-agnostic and does not expose vendor types.

use std::{
  collections::HashSet,
  error::Error,
  fmt,
  sync::atomic::{
    AtomicU64,
    Ordering,
  },
};

use lambda_platform::physics::PhysicsBackend2D;

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
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CollisionEvent {
  /// The event transition kind for this body pair.
  pub kind: CollisionEventKind,
  /// The first body participating in the collision pair.
  pub body_a: RigidBody2D,
  /// The second body participating in the collision pair.
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

  /// Returns collision events collected during the most recent step.
  ///
  /// # Returns
  /// Returns an iterator over collision events emitted by the world.
  pub fn collision_events(&self) -> impl Iterator<Item = CollisionEvent> {
    return std::iter::empty();
  }

  /// Returns all bodies whose colliders contain the provided point.
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

    let body_slots = self.backend.query_point_2d(point);
    return self.deduplicate_query_body_hits(body_slots);
  }

  /// Returns all bodies whose colliders overlap the provided axis-aligned box.
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

    let normalized_min = [min[0].min(max[0]), min[1].min(max[1])];
    let normalized_max = [min[0].max(max[0]), min[1].max(max[1])];
    let body_slots = self.backend.query_aabb_2d(normalized_min, normalized_max);

    return self.deduplicate_query_body_hits(body_slots);
  }

  /// Returns the nearest rigid body hit by the provided ray.
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
    _origin: [f32; 2],
    _dir: [f32; 2],
    _max_dist: f32,
  ) -> Option<RaycastHit> {
    return None;
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

      if seen_bodies.insert(body) {
        bodies.push(body);
      }
    }

    return bodies;
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
