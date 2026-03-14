//! Rapier-backed 2D physics backend.
//!
//! This module provides a minimal wrapper around `rapier2d` to support the
//! higher-level `lambda-rs` physics APIs without exposing vendor types outside
//! of the platform layer.

use std::{
  error::Error,
  fmt,
};

use rapier2d::prelude::*;

/// The rigid body integration mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RigidBodyType2D {
  /// A body that does not move under simulation.
  Static,
  /// A body affected by gravity and forces.
  Dynamic,
  /// A body integrated only by user-provided motion.
  Kinematic,
}

/// Backend errors for 2D rigid body operations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RigidBody2DBackendError {
  /// The referenced rigid body was not found.
  BodyNotFound,
  /// The provided position is invalid.
  InvalidPosition { x: f32, y: f32 },
  /// The provided rotation is invalid.
  InvalidRotation { radians: f32 },
  /// The provided linear velocity is invalid.
  InvalidVelocity { x: f32, y: f32 },
  /// The provided force is invalid.
  InvalidForce { x: f32, y: f32 },
  /// The provided impulse is invalid.
  InvalidImpulse { x: f32, y: f32 },
  /// The provided dynamic mass is invalid.
  InvalidMassKg { mass_kg: f32 },
  /// The requested operation is unsupported for the body type.
  UnsupportedOperation { body_type: RigidBodyType2D },
}

impl fmt::Display for RigidBody2DBackendError {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::BodyNotFound => {
        return write!(formatter, "rigid body not found");
      }
      Self::InvalidPosition { x, y } => {
        return write!(formatter, "invalid position: ({x}, {y})");
      }
      Self::InvalidRotation { radians } => {
        return write!(formatter, "invalid rotation: {radians}");
      }
      Self::InvalidVelocity { x, y } => {
        return write!(formatter, "invalid velocity: ({x}, {y})");
      }
      Self::InvalidForce { x, y } => {
        return write!(formatter, "invalid force: ({x}, {y})");
      }
      Self::InvalidImpulse { x, y } => {
        return write!(formatter, "invalid impulse: ({x}, {y})");
      }
      Self::InvalidMassKg { mass_kg } => {
        return write!(formatter, "invalid mass_kg: {mass_kg}");
      }
      Self::UnsupportedOperation { body_type } => {
        return write!(
          formatter,
          "unsupported operation for body_type: {body_type:?}"
        );
      }
    }
  }
}

impl Error for RigidBody2DBackendError {}

/// Backend errors for 2D collider operations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Collider2DBackendError {
  /// The referenced rigid body was not found.
  BodyNotFound,
  /// The provided polygon could not be represented as a convex hull.
  InvalidPolygonDegenerate,
}

impl fmt::Display for Collider2DBackendError {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::BodyNotFound => {
        return write!(formatter, "rigid body not found");
      }
      Self::InvalidPolygonDegenerate => {
        return write!(formatter, "invalid polygon: degenerate");
      }
    }
  }
}

impl Error for Collider2DBackendError {}

/// The fallback mass applied to dynamic bodies before density colliders exist.
const DYNAMIC_BODY_FALLBACK_MASS_KG: f32 = 1.0;

/// Stores per-body state that `lambda-rs` tracks alongside Rapier.
///
/// This slot exists because `lambda-rs` defines integration semantics that are
/// stricter than the vendor backend:
/// - Forces are accumulated and cleared explicitly by the public API.
/// - Impulses update velocity immediately.
///
/// # Invariants
/// - `rapier_handle` MUST reference a body in `PhysicsBackend2D::bodies`.
/// - `explicit_dynamic_mass_kg` MUST be `Some` only for dynamic bodies.
/// - `generation` MUST be non-zero and is used to validate handles.
#[derive(Debug, Clone, Copy)]
struct RigidBodySlot2D {
  /// The rigid body's integration mode.
  body_type: RigidBodyType2D,
  /// The handle to the Rapier rigid body stored in the `RigidBodySet`.
  rapier_handle: RigidBodyHandle,
  /// Accumulated forces applied by the public API, in Newtons.
  force_accumulator: [f32; 2],
  /// The explicitly configured body mass in kilograms, if set.
  ///
  /// When this value is `Some`, collider density MUST NOT affect body mass
  /// properties. The backend enforces this by creating attached colliders with
  /// zero density and using the configured value as the body's additional mass.
  explicit_dynamic_mass_kg: Option<f32>,
  /// Tracks whether the body has at least one positive-density collider.
  ///
  /// This flag supports the spec requirement that bodies with no positive
  /// density colliders default to `1.0` kg, while bodies with at least one
  /// positive-density collider compute mass from collider density alone.
  has_positive_density_colliders: bool,
  /// A monotonically increasing counter used to validate stale handles.
  generation: u32,
}

/// Stores per-collider state that `lambda-rs` tracks alongside Rapier.
///
/// # Invariants
/// - `rapier_handle` MUST reference a collider in `PhysicsBackend2D::colliders`.
/// - `generation` MUST be non-zero and is used to validate stale handles.
#[derive(Debug, Clone, Copy)]
struct ColliderSlot2D {
  /// The handle to the Rapier collider stored in the `ColliderSet`.
  rapier_handle: ColliderHandle,
  /// A monotonically increasing counter used to validate stale handles.
  generation: u32,
}

/// Describes how collider attachment should affect dynamic-body mass semantics.
///
/// This helper isolates `lambda-rs` mass rules from the Rapier attachment flow
/// so body creation and collider attachment share one backend policy source.
#[derive(Debug, Clone, Copy, PartialEq)]
struct ColliderAttachmentMassPlan2D {
  /// The density value that MUST be passed to the Rapier collider builder.
  rapier_density: f32,
  /// Whether attaching this collider transitions the body to density-driven
  /// mass computation.
  should_mark_has_positive_density_colliders: bool,
  /// Whether the initial fallback mass MUST be removed before insertion.
  should_remove_fallback_mass: bool,
}

/// A 2D physics backend powered by `rapier2d`.
///
/// This type is an internal implementation detail used by `lambda-rs`.
pub struct PhysicsBackend2D {
  gravity: Vector,
  integration_parameters: IntegrationParameters,
  islands: IslandManager,
  broad_phase: BroadPhaseBvh,
  narrow_phase: NarrowPhase,
  bodies: RigidBodySet,
  colliders: ColliderSet,
  impulse_joints: ImpulseJointSet,
  multibody_joints: MultibodyJointSet,
  ccd_solver: CCDSolver,
  pipeline: PhysicsPipeline,
  rigid_body_slots_2d: Vec<RigidBodySlot2D>,
  collider_slots_2d: Vec<ColliderSlot2D>,
}

impl PhysicsBackend2D {
  /// Creates a new empty 2D physics backend.
  ///
  /// # Arguments
  /// - `gravity`: The gravity vector in meters per second squared.
  /// - `timestep_seconds`: The fixed integration timestep in seconds.
  ///
  /// # Returns
  /// Returns a new `PhysicsBackend2D` with no bodies, colliders, or joints.
  pub fn new(gravity: [f32; 2], timestep_seconds: f32) -> Self {
    let gravity_vector = Vector::new(gravity[0], gravity[1]);

    let integration_parameters = IntegrationParameters {
      dt: timestep_seconds,
      ..Default::default()
    };

    return Self {
      gravity: gravity_vector,
      integration_parameters,
      islands: IslandManager::new(),
      broad_phase: BroadPhaseBvh::new(),
      narrow_phase: NarrowPhase::new(),
      bodies: RigidBodySet::new(),
      colliders: ColliderSet::new(),
      impulse_joints: ImpulseJointSet::new(),
      multibody_joints: MultibodyJointSet::new(),
      ccd_solver: CCDSolver::new(),
      pipeline: PhysicsPipeline::new(),
      rigid_body_slots_2d: Vec::new(),
      collider_slots_2d: Vec::new(),
    };
  }

  /// Creates and stores a new 2D rigid body without colliders.
  ///
  /// # Arguments
  /// - `body_type`: The integration mode for the rigid body.
  /// - `position`: The initial position in meters.
  /// - `rotation`: The initial rotation in radians.
  /// - `velocity`: The initial linear velocity in meters per second.
  /// - `dynamic_mass_kg`: The mass in kilograms for dynamic bodies.
  ///
  /// # Returns
  /// Returns a `(slot_index, slot_generation)` pair for the created body.
  ///
  /// # Errors
  /// Returns `RigidBody2DBackendError` if any input is invalid or unsupported.
  pub fn create_rigid_body_2d(
    &mut self,
    body_type: RigidBodyType2D,
    position: [f32; 2],
    rotation: f32,
    velocity: [f32; 2],
    dynamic_mass_kg: Option<f32>,
  ) -> Result<(u32, u32), RigidBody2DBackendError> {
    validate_position(position[0], position[1])?;
    validate_rotation(rotation)?;
    validate_velocity(velocity[0], velocity[1])?;

    let explicit_dynamic_mass_kg =
      resolve_explicit_dynamic_mass_kg(body_type, dynamic_mass_kg)?;
    let additional_mass_kg =
      resolve_additional_mass_kg(body_type, explicit_dynamic_mass_kg)?;

    let slot_index = self.rigid_body_slots_2d.len() as u32;
    let slot_generation = 1;

    let rapier_body = build_rapier_rigid_body(
      body_type,
      position,
      rotation,
      velocity,
      additional_mass_kg,
    );
    let rapier_handle = self.bodies.insert(rapier_body);

    if body_type == RigidBodyType2D::Dynamic {
      let Some(rapier_body) = self.bodies.get_mut(rapier_handle) else {
        return Err(RigidBody2DBackendError::BodyNotFound);
      };
      rapier_body.recompute_mass_properties_from_colliders(&self.colliders);
    }

    self.rigid_body_slots_2d.push(RigidBodySlot2D {
      body_type,
      rapier_handle,
      force_accumulator: [0.0, 0.0],
      explicit_dynamic_mass_kg,
      has_positive_density_colliders: false,
      generation: slot_generation,
    });

    return Ok((slot_index, slot_generation));
  }

  /// Creates and attaches a circle collider to a rigid body.
  ///
  /// The caller MUST validate all collider inputs before reaching this backend.
  /// `lambda-rs` performs that validation in `Collider2DBuilder::build()`.
  ///
  /// # Arguments
  /// - `parent_slot_index`: The rigid body slot index.
  /// - `parent_slot_generation`: The rigid body slot generation counter.
  /// - `radius`: The circle radius in meters.
  /// - `local_offset`: The collider local translation in meters.
  /// - `local_rotation`: The collider local rotation in radians.
  /// - `density`: The density in kg/m².
  /// - `friction`: The friction coefficient (unitless).
  /// - `restitution`: The restitution coefficient in `[0.0, 1.0]`.
  ///
  /// # Returns
  /// Returns a `(slot_index, slot_generation)` pair for the created collider.
  ///
  /// # Errors
  /// Returns `Collider2DBackendError::BodyNotFound` if the parent body does
  /// not exist.
  #[allow(clippy::too_many_arguments)]
  pub fn create_circle_collider_2d(
    &mut self,
    parent_slot_index: u32,
    parent_slot_generation: u32,
    radius: f32,
    local_offset: [f32; 2],
    local_rotation: f32,
    density: f32,
    friction: f32,
    restitution: f32,
  ) -> Result<(u32, u32), Collider2DBackendError> {
    return self.attach_collider_2d(
      parent_slot_index,
      parent_slot_generation,
      ColliderBuilder::ball(radius),
      local_offset,
      local_rotation,
      density,
      friction,
      restitution,
    );
  }

  /// Creates and attaches a rectangle collider to a rigid body.
  ///
  /// The caller MUST validate all collider inputs before reaching this backend.
  /// `lambda-rs` performs that validation in `Collider2DBuilder::build()`.
  ///
  /// # Arguments
  /// - `parent_slot_index`: The rigid body slot index.
  /// - `parent_slot_generation`: The rigid body slot generation counter.
  /// - `half_width`: The rectangle half-width in meters.
  /// - `half_height`: The rectangle half-height in meters.
  /// - `local_offset`: The collider local translation in meters.
  /// - `local_rotation`: The collider local rotation in radians.
  /// - `density`: The density in kg/m².
  /// - `friction`: The friction coefficient (unitless).
  /// - `restitution`: The restitution coefficient in `[0.0, 1.0]`.
  ///
  /// # Returns
  /// Returns a `(slot_index, slot_generation)` pair for the created collider.
  ///
  /// # Errors
  /// Returns `Collider2DBackendError::BodyNotFound` if the parent body does
  /// not exist.
  #[allow(clippy::too_many_arguments)]
  pub fn create_rectangle_collider_2d(
    &mut self,
    parent_slot_index: u32,
    parent_slot_generation: u32,
    half_width: f32,
    half_height: f32,
    local_offset: [f32; 2],
    local_rotation: f32,
    density: f32,
    friction: f32,
    restitution: f32,
  ) -> Result<(u32, u32), Collider2DBackendError> {
    return self.attach_collider_2d(
      parent_slot_index,
      parent_slot_generation,
      ColliderBuilder::cuboid(half_width, half_height),
      local_offset,
      local_rotation,
      density,
      friction,
      restitution,
    );
  }

  /// Creates and attaches a capsule collider to a rigid body.
  ///
  /// The capsule is aligned with the collider local Y axis.
  /// The caller MUST validate all collider inputs before reaching this backend.
  /// `lambda-rs` performs that validation in `Collider2DBuilder::build()`.
  ///
  /// # Arguments
  /// - `parent_slot_index`: The rigid body slot index.
  /// - `parent_slot_generation`: The rigid body slot generation counter.
  /// - `half_height`: The half-height of the capsule segment in meters.
  /// - `radius`: The capsule radius in meters.
  /// - `local_offset`: The collider local translation in meters.
  /// - `local_rotation`: The collider local rotation in radians.
  /// - `density`: The density in kg/m².
  /// - `friction`: The friction coefficient (unitless).
  /// - `restitution`: The restitution coefficient in `[0.0, 1.0]`.
  ///
  /// # Returns
  /// Returns a `(slot_index, slot_generation)` pair for the created collider.
  ///
  /// # Errors
  /// Returns `Collider2DBackendError::BodyNotFound` if the parent body does
  /// not exist.
  #[allow(clippy::too_many_arguments)]
  pub fn create_capsule_collider_2d(
    &mut self,
    parent_slot_index: u32,
    parent_slot_generation: u32,
    half_height: f32,
    radius: f32,
    local_offset: [f32; 2],
    local_rotation: f32,
    density: f32,
    friction: f32,
    restitution: f32,
  ) -> Result<(u32, u32), Collider2DBackendError> {
    let rapier_builder = if half_height == 0.0 {
      ColliderBuilder::ball(radius)
    } else {
      ColliderBuilder::capsule_y(half_height, radius)
    };

    return self.attach_collider_2d(
      parent_slot_index,
      parent_slot_generation,
      rapier_builder,
      local_offset,
      local_rotation,
      density,
      friction,
      restitution,
    );
  }

  /// Creates and attaches a convex polygon collider to a rigid body.
  ///
  /// The polygon vertices are interpreted as points in collider local space.
  /// The caller MUST validate and normalize polygon inputs before reaching this
  /// backend. `lambda-rs` performs that validation in
  /// `Collider2DBuilder::build()`.
  ///
  /// # Arguments
  /// - `parent_slot_index`: The rigid body slot index.
  /// - `parent_slot_generation`: The rigid body slot generation counter.
  /// - `vertices`: The polygon vertices in meters.
  /// - `local_offset`: The collider local translation in meters.
  /// - `local_rotation`: The collider local rotation in radians.
  /// - `density`: The density in kg/m².
  /// - `friction`: The friction coefficient (unitless).
  /// - `restitution`: The restitution coefficient in `[0.0, 1.0]`.
  ///
  /// # Returns
  /// Returns a `(slot_index, slot_generation)` pair for the created collider.
  ///
  /// # Errors
  /// Returns `Collider2DBackendError::BodyNotFound` if the parent body does
  /// not exist. Returns `Collider2DBackendError::InvalidPolygonDegenerate` if
  /// the validated polygon still cannot be represented as a Rapier convex
  /// hull.
  #[allow(clippy::too_many_arguments)]
  pub fn create_convex_polygon_collider_2d(
    &mut self,
    parent_slot_index: u32,
    parent_slot_generation: u32,
    vertices: Vec<[f32; 2]>,
    local_offset: [f32; 2],
    local_rotation: f32,
    density: f32,
    friction: f32,
    restitution: f32,
  ) -> Result<(u32, u32), Collider2DBackendError> {
    let rapier_vertices: Vec<Vector> = vertices
      .iter()
      .map(|vertex| Vector::new(vertex[0], vertex[1]))
      .collect();

    let Some(rapier_builder) =
      ColliderBuilder::convex_hull(rapier_vertices.as_slice())
    else {
      return Err(Collider2DBackendError::InvalidPolygonDegenerate);
    };

    return self.attach_collider_2d(
      parent_slot_index,
      parent_slot_generation,
      rapier_builder,
      local_offset,
      local_rotation,
      density,
      friction,
      restitution,
    );
  }

  /// Returns the rigid body type for the referenced body.
  ///
  /// # Arguments
  /// - `slot_index`: The body slot index.
  /// - `slot_generation`: The slot generation counter.
  ///
  /// # Returns
  /// Returns the rigid body type.
  ///
  /// # Errors
  /// Returns `RigidBody2DBackendError` if the referenced body does not exist.
  pub fn rigid_body_type_2d(
    &self,
    slot_index: u32,
    slot_generation: u32,
  ) -> Result<RigidBodyType2D, RigidBody2DBackendError> {
    let body_slot = self.rigid_body_slot_2d(slot_index, slot_generation)?;
    return Ok(body_slot.body_type);
  }

  /// Returns the current position for the referenced body.
  ///
  /// # Arguments
  /// - `slot_index`: The body slot index.
  /// - `slot_generation`: The slot generation counter.
  ///
  /// # Returns
  /// Returns the position in meters.
  ///
  /// # Errors
  /// Returns `RigidBody2DBackendError` if the referenced body does not exist.
  pub fn rigid_body_position_2d(
    &self,
    slot_index: u32,
    slot_generation: u32,
  ) -> Result<[f32; 2], RigidBody2DBackendError> {
    let rapier_body = self.rapier_rigid_body_2d(slot_index, slot_generation)?;
    let translation = rapier_body.translation();
    return Ok([translation.x, translation.y]);
  }

  /// Returns the current rotation for the referenced body.
  ///
  /// # Arguments
  /// - `slot_index`: The body slot index.
  /// - `slot_generation`: The slot generation counter.
  ///
  /// # Returns
  /// Returns the rotation in radians.
  ///
  /// # Errors
  /// Returns `RigidBody2DBackendError` if the referenced body does not exist.
  pub fn rigid_body_rotation_2d(
    &self,
    slot_index: u32,
    slot_generation: u32,
  ) -> Result<f32, RigidBody2DBackendError> {
    let rapier_body = self.rapier_rigid_body_2d(slot_index, slot_generation)?;
    return Ok(rapier_body.rotation().angle());
  }

  /// Returns the current linear velocity for the referenced body.
  ///
  /// # Arguments
  /// - `slot_index`: The body slot index.
  /// - `slot_generation`: The slot generation counter.
  ///
  /// # Returns
  /// Returns the linear velocity in meters per second.
  ///
  /// # Errors
  /// Returns `RigidBody2DBackendError` if the referenced body does not exist.
  pub fn rigid_body_velocity_2d(
    &self,
    slot_index: u32,
    slot_generation: u32,
  ) -> Result<[f32; 2], RigidBody2DBackendError> {
    let rapier_body = self.rapier_rigid_body_2d(slot_index, slot_generation)?;
    let velocity = rapier_body.linvel();
    return Ok([velocity.x, velocity.y]);
  }

  /// Returns whether the referenced body slot resolves to a live rigid body.
  ///
  /// # Arguments
  /// - `slot_index`: The body slot index.
  /// - `slot_generation`: The slot generation counter.
  ///
  /// # Returns
  /// Returns `true` when the slot is valid and the Rapier body still exists.
  pub fn rigid_body_exists_2d(
    &self,
    slot_index: u32,
    slot_generation: u32,
  ) -> bool {
    return self
      .rapier_rigid_body_2d(slot_index, slot_generation)
      .is_ok();
  }

  /// Sets the current position for the referenced body.
  ///
  /// # Arguments
  /// - `slot_index`: The body slot index.
  /// - `slot_generation`: The slot generation counter.
  /// - `position`: The new position in meters.
  ///
  /// # Returns
  /// Returns `()` after applying the mutation.
  ///
  /// # Errors
  /// Returns `RigidBody2DBackendError` if the input is invalid or the
  /// referenced body does not exist.
  pub fn rigid_body_set_position_2d(
    &mut self,
    slot_index: u32,
    slot_generation: u32,
    position: [f32; 2],
  ) -> Result<(), RigidBody2DBackendError> {
    validate_position(position[0], position[1])?;
    let rapier_body =
      self.rapier_rigid_body_2d_mut(slot_index, slot_generation)?;
    rapier_body.set_translation(Vector::new(position[0], position[1]), true);
    return Ok(());
  }

  /// Sets the current rotation for the referenced body.
  ///
  /// # Arguments
  /// - `slot_index`: The body slot index.
  /// - `slot_generation`: The slot generation counter.
  /// - `rotation`: The new rotation in radians.
  ///
  /// # Returns
  /// Returns `()` after applying the mutation.
  ///
  /// # Errors
  /// Returns `RigidBody2DBackendError` if the input is invalid or the
  /// referenced body does not exist.
  pub fn rigid_body_set_rotation_2d(
    &mut self,
    slot_index: u32,
    slot_generation: u32,
    rotation: f32,
  ) -> Result<(), RigidBody2DBackendError> {
    validate_rotation(rotation)?;
    let rapier_body =
      self.rapier_rigid_body_2d_mut(slot_index, slot_generation)?;
    rapier_body.set_rotation(Rotation::new(rotation), true);
    return Ok(());
  }

  /// Sets the current linear velocity for the referenced body.
  ///
  /// # Arguments
  /// - `slot_index`: The body slot index.
  /// - `slot_generation`: The slot generation counter.
  /// - `velocity`: The new linear velocity in meters per second.
  ///
  /// # Returns
  /// Returns `()` after applying the mutation.
  ///
  /// # Errors
  /// Returns `RigidBody2DBackendError` if the input is invalid, if the
  /// operation is unsupported for the body type, or if the referenced body
  /// does not exist.
  pub fn rigid_body_set_velocity_2d(
    &mut self,
    slot_index: u32,
    slot_generation: u32,
    velocity: [f32; 2],
  ) -> Result<(), RigidBody2DBackendError> {
    validate_velocity(velocity[0], velocity[1])?;
    let (body_type, rapier_handle) = {
      let body_slot = self.rigid_body_slot_2d(slot_index, slot_generation)?;
      (body_slot.body_type, body_slot.rapier_handle)
    };

    if body_type == RigidBodyType2D::Static {
      return Err(RigidBody2DBackendError::UnsupportedOperation { body_type });
    }

    let Some(rapier_body) = self.bodies.get_mut(rapier_handle) else {
      return Err(RigidBody2DBackendError::BodyNotFound);
    };
    rapier_body.set_linvel(Vector::new(velocity[0], velocity[1]), true);
    return Ok(());
  }

  /// Applies a force, in Newtons, at the center of mass.
  ///
  /// # Arguments
  /// - `slot_index`: The body slot index.
  /// - `slot_generation`: The slot generation counter.
  /// - `force`: The force in Newtons.
  ///
  /// # Returns
  /// Returns `()` after accumulating the force.
  ///
  /// # Errors
  /// Returns `RigidBody2DBackendError` if the input is invalid, if the
  /// operation is unsupported for the body type, or if the referenced body
  /// does not exist.
  pub fn rigid_body_apply_force_2d(
    &mut self,
    slot_index: u32,
    slot_generation: u32,
    force: [f32; 2],
  ) -> Result<(), RigidBody2DBackendError> {
    validate_force(force[0], force[1])?;
    let body = self.rigid_body_slot_2d_mut(slot_index, slot_generation)?;

    if body.body_type != RigidBodyType2D::Dynamic {
      return Err(RigidBody2DBackendError::UnsupportedOperation {
        body_type: body.body_type,
      });
    }

    body.force_accumulator[0] += force[0];
    body.force_accumulator[1] += force[1];

    return Ok(());
  }

  /// Applies an impulse, in Newton-seconds, at the center of mass.
  ///
  /// # Arguments
  /// - `slot_index`: The body slot index.
  /// - `slot_generation`: The slot generation counter.
  /// - `impulse`: The impulse in Newton-seconds.
  ///
  /// # Returns
  /// Returns `()` after applying the impulse.
  ///
  /// # Errors
  /// Returns `RigidBody2DBackendError` if the input is invalid, if the
  /// operation is unsupported for the body type, or if the referenced body
  /// does not exist.
  pub fn rigid_body_apply_impulse_2d(
    &mut self,
    slot_index: u32,
    slot_generation: u32,
    impulse: [f32; 2],
  ) -> Result<(), RigidBody2DBackendError> {
    validate_impulse(impulse[0], impulse[1])?;
    let (body_type, rapier_handle) = {
      let body_slot = self.rigid_body_slot_2d(slot_index, slot_generation)?;
      (body_slot.body_type, body_slot.rapier_handle)
    };

    if body_type != RigidBodyType2D::Dynamic {
      return Err(RigidBody2DBackendError::UnsupportedOperation { body_type });
    }

    let Some(rapier_body) = self.bodies.get_mut(rapier_handle) else {
      return Err(RigidBody2DBackendError::BodyNotFound);
    };
    rapier_body.apply_impulse(Vector::new(impulse[0], impulse[1]), true);

    return Ok(());
  }

  /// Clears accumulated forces for all stored bodies.
  ///
  /// # Returns
  /// Returns `()` after clearing force accumulators.
  pub fn clear_rigid_body_forces_2d(&mut self) {
    for index in 0..self.rigid_body_slots_2d.len() {
      let rapier_handle = {
        let body_slot = &mut self.rigid_body_slots_2d[index];
        body_slot.force_accumulator = [0.0, 0.0];
        body_slot.rapier_handle
      };

      let Some(rapier_body) = self.bodies.get_mut(rapier_handle) else {
        continue;
      };
      rapier_body.reset_forces(true);
    }

    return;
  }

  /// Returns the gravity vector used by this backend.
  ///
  /// # Returns
  /// Returns the gravity vector in meters per second squared.
  pub fn gravity(&self) -> [f32; 2] {
    return [self.gravity.x, self.gravity.y];
  }

  /// Returns the fixed integration timestep in seconds.
  ///
  /// # Returns
  /// Returns the timestep used for each simulation step.
  pub fn timestep_seconds(&self) -> f32 {
    return self.integration_parameters.dt;
  }

  /// Advances the simulation by one fixed timestep.
  ///
  /// # Returns
  /// Returns `()` after applying integration and constraint solving for the
  /// configured timestep.
  pub fn step(&mut self) {
    return self.step_with_timestep_seconds(self.integration_parameters.dt);
  }

  /// Advances the simulation by the given timestep.
  ///
  /// # Arguments
  /// - `timestep_seconds`: The timestep used for this step.
  ///
  /// # Returns
  /// Returns `()` after applying integration and constraint solving.
  pub fn step_with_timestep_seconds(&mut self, timestep_seconds: f32) {
    self.integration_parameters.dt = timestep_seconds;

    if cfg!(debug_assertions) {
      self.debug_validate_collider_slots_2d();
    }

    // Rapier consumes user forces during each integration step, so
    // accumulated public forces must be re-synchronized before every substep.
    self.sync_force_accumulators_2d();

    self.pipeline.step(
      self.gravity,
      &self.integration_parameters,
      &mut self.islands,
      &mut self.broad_phase,
      &mut self.narrow_phase,
      &mut self.bodies,
      &mut self.colliders,
      &mut self.impulse_joints,
      &mut self.multibody_joints,
      &mut self.ccd_solver,
      &(),
      &(),
    );

    return;
  }

  /// Returns an immutable reference to a rigid body slot after validation.
  ///
  /// # Arguments
  /// - `slot_index`: The body slot index.
  /// - `slot_generation`: The slot generation counter.
  ///
  /// # Returns
  /// Returns an immutable reference to the validated `RigidBodySlot2D`.
  ///
  /// # Errors
  /// Returns `RigidBody2DBackendError::BodyNotFound` when the slot is missing
  /// or the generation does not match.
  fn rigid_body_slot_2d(
    &self,
    slot_index: u32,
    slot_generation: u32,
  ) -> Result<&RigidBodySlot2D, RigidBody2DBackendError> {
    let Some(body) = self.rigid_body_slots_2d.get(slot_index as usize) else {
      return Err(RigidBody2DBackendError::BodyNotFound);
    };

    if body.generation != slot_generation {
      return Err(RigidBody2DBackendError::BodyNotFound);
    }

    return Ok(body);
  }

  /// Returns a mutable reference to a rigid body slot after validation.
  ///
  /// # Arguments
  /// - `slot_index`: The body slot index.
  /// - `slot_generation`: The slot generation counter.
  ///
  /// # Returns
  /// Returns a mutable reference to the validated `RigidBodySlot2D`.
  ///
  /// # Errors
  /// Returns `RigidBody2DBackendError::BodyNotFound` when the slot is missing
  /// or the generation does not match.
  fn rigid_body_slot_2d_mut(
    &mut self,
    slot_index: u32,
    slot_generation: u32,
  ) -> Result<&mut RigidBodySlot2D, RigidBody2DBackendError> {
    let Some(body) = self.rigid_body_slots_2d.get_mut(slot_index as usize)
    else {
      return Err(RigidBody2DBackendError::BodyNotFound);
    };

    if body.generation != slot_generation {
      return Err(RigidBody2DBackendError::BodyNotFound);
    }

    return Ok(body);
  }

  /// Returns an immutable reference to the Rapier rigid body for a slot.
  ///
  /// # Arguments
  /// - `slot_index`: The body slot index.
  /// - `slot_generation`: The slot generation counter.
  ///
  /// # Returns
  /// Returns an immutable reference to the underlying Rapier `RigidBody`.
  ///
  /// # Errors
  /// Returns `RigidBody2DBackendError::BodyNotFound` when the slot is invalid
  /// or the Rapier body has been removed.
  fn rapier_rigid_body_2d(
    &self,
    slot_index: u32,
    slot_generation: u32,
  ) -> Result<&RigidBody, RigidBody2DBackendError> {
    let body_slot = self.rigid_body_slot_2d(slot_index, slot_generation)?;
    let Some(rapier_body) = self.bodies.get(body_slot.rapier_handle) else {
      return Err(RigidBody2DBackendError::BodyNotFound);
    };

    return Ok(rapier_body);
  }

  /// Returns a mutable reference to the Rapier rigid body for a slot.
  ///
  /// # Arguments
  /// - `slot_index`: The body slot index.
  /// - `slot_generation`: The slot generation counter.
  ///
  /// # Returns
  /// Returns a mutable reference to the underlying Rapier `RigidBody`.
  ///
  /// # Errors
  /// Returns `RigidBody2DBackendError::BodyNotFound` when the slot is invalid
  /// or the Rapier body has been removed.
  fn rapier_rigid_body_2d_mut(
    &mut self,
    slot_index: u32,
    slot_generation: u32,
  ) -> Result<&mut RigidBody, RigidBody2DBackendError> {
    let rapier_handle = {
      let body_slot = self.rigid_body_slot_2d(slot_index, slot_generation)?;
      body_slot.rapier_handle
    };

    let Some(rapier_body) = self.bodies.get_mut(rapier_handle) else {
      return Err(RigidBody2DBackendError::BodyNotFound);
    };

    return Ok(rapier_body);
  }

  /// Syncs accumulated forces into Rapier prior to stepping.
  ///
  /// `lambda-rs` exposes a force accumulation API that persists forces until
  /// explicitly cleared. Rapier stores forces on each rigid body. This function
  /// overwrites Rapier's stored force with the value tracked by `lambda-rs` so
  /// Rapier can integrate forces and gravity consistently during stepping.
  /// Bodies with zero accumulated force are skipped because `clear_*` methods
  /// and Rapier step completion already leave them with no user force to
  /// reapply.
  ///
  /// # Returns
  /// Returns `()` after updating Rapier force state for all dynamic bodies.
  fn sync_force_accumulators_2d(&mut self) {
    for index in 0..self.rigid_body_slots_2d.len() {
      let (body_type, rapier_handle, force_accumulator) = {
        let body_slot = &self.rigid_body_slots_2d[index];
        (
          body_slot.body_type,
          body_slot.rapier_handle,
          body_slot.force_accumulator,
        )
      };

      if body_type != RigidBodyType2D::Dynamic {
        continue;
      }

      if force_accumulator[0] == 0.0 && force_accumulator[1] == 0.0 {
        continue;
      }

      let Some(rapier_body) = self.bodies.get_mut(rapier_handle) else {
        continue;
      };

      let should_wake =
        force_accumulator[0] != 0.0 || force_accumulator[1] != 0.0;
      rapier_body.reset_forces(false);
      rapier_body.add_force(
        Vector::new(force_accumulator[0], force_accumulator[1]),
        should_wake,
      );
    }

    return;
  }

  /// Attaches a prepared Rapier collider builder to a parent rigid body.
  ///
  /// This helper applies the shared local transform and material properties,
  /// inserts the built collider into Rapier, recomputes parent mass
  /// properties, and allocates the public collider slot.
  ///
  /// Lambda material semantics are encoded using Rapier's built-in combine
  /// rules instead of a custom contact hook:
  /// - friction stores `sqrt(requested_friction)` and uses `Multiply`
  /// - restitution stores the requested value and uses `Max`
  ///
  /// # Arguments
  /// - `parent_slot_index`: The parent rigid body slot index.
  /// - `parent_slot_generation`: The parent slot generation counter.
  /// - `rapier_builder`: The shape-specific Rapier collider builder.
  /// - `local_offset`: The collider local translation in meters.
  /// - `local_rotation`: The collider local rotation in radians.
  /// - `density`: The requested density in kg/m².
  /// - `friction`: The friction coefficient (unitless).
  /// - `restitution`: The restitution coefficient in `[0.0, 1.0]`.
  ///
  /// # Returns
  /// Returns a `(slot_index, slot_generation)` pair for the created collider.
  ///
  /// # Errors
  /// Returns `Collider2DBackendError` if the parent body does not exist.
  #[allow(clippy::too_many_arguments)]
  fn attach_collider_2d(
    &mut self,
    parent_slot_index: u32,
    parent_slot_generation: u32,
    rapier_builder: ColliderBuilder,
    local_offset: [f32; 2],
    local_rotation: f32,
    density: f32,
    friction: f32,
    restitution: f32,
  ) -> Result<(u32, u32), Collider2DBackendError> {
    let (rapier_parent_handle, rapier_density) = self
      .prepare_parent_body_for_collider_attachment_2d(
        parent_slot_index,
        parent_slot_generation,
        density,
      )?;

    let rapier_collider = rapier_builder
      .translation(Vector::new(local_offset[0], local_offset[1]))
      .rotation(local_rotation)
      .density(rapier_density)
      .friction(encode_rapier_friction_coefficient(friction))
      .friction_combine_rule(CoefficientCombineRule::Multiply)
      .restitution(restitution)
      .restitution_combine_rule(CoefficientCombineRule::Max)
      .build();

    let rapier_handle = self.colliders.insert_with_parent(
      rapier_collider,
      rapier_parent_handle,
      &mut self.bodies,
    );

    self.recompute_parent_mass_after_collider_attachment_2d(
      parent_slot_index,
      parent_slot_generation,
      rapier_parent_handle,
    )?;

    let slot_index = self.collider_slots_2d.len() as u32;
    let slot_generation = 1;
    self.collider_slots_2d.push(ColliderSlot2D {
      rapier_handle,
      generation: slot_generation,
    });

    return Ok((slot_index, slot_generation));
  }

  /// Prepares a parent body for collider attachment and resolves the Rapier
  /// density value to apply.
  ///
  /// This function enforces spec mass semantics:
  /// - When a dynamic body's mass is explicitly configured, collider density
  ///   MUST NOT affect mass properties. The returned density is `0.0`.
  /// - When a dynamic body's mass is not explicitly configured, the backend
  ///   starts with a `1.0` kg fallback mass. When attaching the first
  ///   positive-density collider, the fallback mass is removed so the body's
  ///   mass becomes the sum of collider mass contributions.
  ///
  /// # Arguments
  /// - `parent_slot_index`: The parent rigid body slot index.
  /// - `parent_slot_generation`: The parent slot generation counter.
  /// - `requested_density`: The density requested by the public API.
  ///
  /// # Returns
  /// Returns the Rapier body handle and the density to apply to the Rapier
  /// collider.
  ///
  /// # Errors
  /// Returns `Collider2DBackendError::BodyNotFound` if the parent body is
  /// missing or the handle is stale.
  fn prepare_parent_body_for_collider_attachment_2d(
    &mut self,
    parent_slot_index: u32,
    parent_slot_generation: u32,
    requested_density: f32,
  ) -> Result<(RigidBodyHandle, f32), Collider2DBackendError> {
    let (
      body_type,
      rapier_handle,
      explicit_dynamic_mass_kg,
      has_positive_density_colliders,
    ) = {
      let body_slot = self
        .rigid_body_slot_2d(parent_slot_index, parent_slot_generation)
        .map_err(|_| Collider2DBackendError::BodyNotFound)?;
      (
        body_slot.body_type,
        body_slot.rapier_handle,
        body_slot.explicit_dynamic_mass_kg,
        body_slot.has_positive_density_colliders,
      )
    };

    if self.bodies.get(rapier_handle).is_none() {
      return Err(Collider2DBackendError::BodyNotFound);
    }

    let attachment_mass_plan = resolve_collider_attachment_mass_plan_2d(
      body_type,
      explicit_dynamic_mass_kg,
      has_positive_density_colliders,
      requested_density,
    );

    if attachment_mass_plan.should_mark_has_positive_density_colliders {
      let body_slot = self
        .rigid_body_slot_2d_mut(parent_slot_index, parent_slot_generation)
        .map_err(|_| Collider2DBackendError::BodyNotFound)?;
      body_slot.has_positive_density_colliders = true;
    }

    if attachment_mass_plan.should_remove_fallback_mass {
      let Some(rapier_body) = self.bodies.get_mut(rapier_handle) else {
        return Err(Collider2DBackendError::BodyNotFound);
      };
      rapier_body.set_additional_mass(0.0, true);
    }

    return Ok((rapier_handle, attachment_mass_plan.rapier_density));
  }

  /// Recomputes mass properties for a parent body after collider attachment.
  ///
  /// # Arguments
  /// - `parent_slot_index`: The parent rigid body slot index.
  /// - `parent_slot_generation`: The parent slot generation counter.
  /// - `rapier_parent_handle`: The Rapier body handle.
  ///
  /// # Returns
  /// Returns `()` after recomputing mass properties.
  ///
  /// # Errors
  /// Returns `Collider2DBackendError::BodyNotFound` if the parent body is
  /// missing or the handle is stale.
  fn recompute_parent_mass_after_collider_attachment_2d(
    &mut self,
    _parent_slot_index: u32,
    _parent_slot_generation: u32,
    rapier_parent_handle: RigidBodyHandle,
  ) -> Result<(), Collider2DBackendError> {
    let Some(rapier_body) = self.bodies.get_mut(rapier_parent_handle) else {
      return Err(Collider2DBackendError::BodyNotFound);
    };
    rapier_body.recompute_mass_properties_from_colliders(&self.colliders);

    return Ok(());
  }

  /// Validates that collider slots reference live Rapier colliders.
  ///
  /// This debug-only validation reads slot fields to prevent stale-handle
  /// regressions during backend refactors.
  ///
  /// # Returns
  /// Returns `()` after completing validation.
  fn debug_validate_collider_slots_2d(&self) {
    for slot in self.collider_slots_2d.iter() {
      debug_assert!(slot.generation > 0, "collider slot generation is zero");
      debug_assert!(
        self.colliders.get(slot.rapier_handle).is_some(),
        "collider slot references missing Rapier collider"
      );
    }

    return;
  }
}

/// Builds a Rapier rigid body builder with `lambda-rs` invariants applied.
///
/// Bodies created by this backend do not lock the 2D rotation axis. Dynamic
/// bodies are expected to rotate in response to collisions.
///
/// # Arguments
/// - `body_type`: The integration mode for the rigid body.
/// - `position`: The initial position in meters.
/// - `rotation`: The initial rotation in radians.
/// - `velocity`: The initial linear velocity in meters per second.
/// - `additional_mass_kg`: The additional mass in kilograms for dynamic bodies.
///
/// # Returns
/// Returns a configured Rapier `RigidBodyBuilder`.
fn build_rapier_rigid_body(
  body_type: RigidBodyType2D,
  position: [f32; 2],
  rotation: f32,
  velocity: [f32; 2],
  additional_mass_kg: f32,
) -> RigidBodyBuilder {
  let translation = Vector::new(position[0], position[1]);
  let linear_velocity = Vector::new(velocity[0], velocity[1]);

  match body_type {
    RigidBodyType2D::Static => {
      return RigidBodyBuilder::fixed()
        .translation(translation)
        .rotation(rotation)
        .linvel(linear_velocity);
    }
    RigidBodyType2D::Kinematic => {
      return RigidBodyBuilder::kinematic_velocity_based()
        .translation(translation)
        .rotation(rotation)
        .linvel(linear_velocity);
    }
    RigidBodyType2D::Dynamic => {
      return RigidBodyBuilder::dynamic()
        .translation(translation)
        .rotation(rotation)
        .linvel(linear_velocity)
        .additional_mass(additional_mass_kg);
    }
  }
}

/// Validates a 2D position.
///
/// # Arguments
/// - `x`: The X position in meters.
/// - `y`: The Y position in meters.
///
/// # Returns
/// Returns `()` when the input is finite.
///
/// # Errors
/// Returns `RigidBody2DBackendError::InvalidPosition` when any component is
/// non-finite.
fn validate_position(x: f32, y: f32) -> Result<(), RigidBody2DBackendError> {
  if !x.is_finite() || !y.is_finite() {
    return Err(RigidBody2DBackendError::InvalidPosition { x, y });
  }

  return Ok(());
}

/// Validates a rotation angle.
///
/// # Arguments
/// - `radians`: The rotation in radians.
///
/// # Returns
/// Returns `()` when the input is finite.
///
/// # Errors
/// Returns `RigidBody2DBackendError::InvalidRotation` when the angle is
/// non-finite.
fn validate_rotation(radians: f32) -> Result<(), RigidBody2DBackendError> {
  if !radians.is_finite() {
    return Err(RigidBody2DBackendError::InvalidRotation { radians });
  }

  return Ok(());
}

/// Validates a 2D linear velocity.
///
/// # Arguments
/// - `x`: The X velocity in meters per second.
/// - `y`: The Y velocity in meters per second.
///
/// # Returns
/// Returns `()` when the input is finite.
///
/// # Errors
/// Returns `RigidBody2DBackendError::InvalidVelocity` when any component is
/// non-finite.
fn validate_velocity(x: f32, y: f32) -> Result<(), RigidBody2DBackendError> {
  if !x.is_finite() || !y.is_finite() {
    return Err(RigidBody2DBackendError::InvalidVelocity { x, y });
  }

  return Ok(());
}

/// Validates a 2D force vector.
///
/// # Arguments
/// - `x`: The X force component in Newtons.
/// - `y`: The Y force component in Newtons.
///
/// # Returns
/// Returns `()` when the input is finite.
///
/// # Errors
/// Returns `RigidBody2DBackendError::InvalidForce` when any component is
/// non-finite.
fn validate_force(x: f32, y: f32) -> Result<(), RigidBody2DBackendError> {
  if !x.is_finite() || !y.is_finite() {
    return Err(RigidBody2DBackendError::InvalidForce { x, y });
  }

  return Ok(());
}

/// Validates a 2D impulse vector.
///
/// # Arguments
/// - `x`: The X impulse component in Newton-seconds.
/// - `y`: The Y impulse component in Newton-seconds.
///
/// # Returns
/// Returns `()` when the input is finite.
///
/// # Errors
/// Returns `RigidBody2DBackendError::InvalidImpulse` when any component is
/// non-finite.
fn validate_impulse(x: f32, y: f32) -> Result<(), RigidBody2DBackendError> {
  if !x.is_finite() || !y.is_finite() {
    return Err(RigidBody2DBackendError::InvalidImpulse { x, y });
  }

  return Ok(());
}

/// Resolves the explicitly configured mass for a rigid body.
///
/// # Arguments
/// - `body_type`: The integration mode for the rigid body.
/// - `dynamic_mass_kg`: The requested mass in kilograms for dynamic bodies.
///
/// # Returns
/// Returns the explicitly configured dynamic mass.
///
/// # Errors
/// Returns `RigidBody2DBackendError` if a mass is provided for a non-dynamic
/// body, or if the mass is non-finite or non-positive.
fn resolve_explicit_dynamic_mass_kg(
  body_type: RigidBodyType2D,
  dynamic_mass_kg: Option<f32>,
) -> Result<Option<f32>, RigidBody2DBackendError> {
  let Some(mass_kg) = dynamic_mass_kg else {
    return Ok(None);
  };

  if body_type != RigidBodyType2D::Dynamic {
    return Err(RigidBody2DBackendError::UnsupportedOperation { body_type });
  }

  if !mass_kg.is_finite() || mass_kg <= 0.0 {
    return Err(RigidBody2DBackendError::InvalidMassKg { mass_kg });
  }

  return Ok(Some(mass_kg));
}

/// Resolves the additional mass in kilograms applied when creating a body.
///
/// # Arguments
/// - `body_type`: The rigid body integration mode.
/// - `explicit_dynamic_mass_kg`: The explicitly configured mass for dynamic
///   bodies.
///
/// # Returns
/// Returns the additional mass value in kilograms.
///
/// # Errors
/// Returns `RigidBody2DBackendError::InvalidMassKg` if the fallback mass cannot
/// be represented as a positive finite value.
fn resolve_additional_mass_kg(
  body_type: RigidBodyType2D,
  explicit_dynamic_mass_kg: Option<f32>,
) -> Result<f32, RigidBody2DBackendError> {
  if body_type != RigidBodyType2D::Dynamic {
    return Ok(0.0);
  }

  if let Some(explicit_mass_kg) = explicit_dynamic_mass_kg {
    return Ok(explicit_mass_kg);
  }

  let fallback_mass_kg = DYNAMIC_BODY_FALLBACK_MASS_KG;
  if !fallback_mass_kg.is_finite() || fallback_mass_kg <= 0.0 {
    return Err(RigidBody2DBackendError::InvalidMassKg {
      mass_kg: fallback_mass_kg,
    });
  }

  return Ok(fallback_mass_kg);
}

/// Encodes a public friction coefficient for Rapier's `Multiply` rule.
///
/// Lambda specifies `sqrt(friction_a * friction_b)` as the effective contact
/// friction. Rapier cannot express that rule directly, so the backend stores
/// `sqrt(requested_friction)` on each collider and relies on
/// `CoefficientCombineRule::Multiply` to recover the public result.
///
/// # Arguments
/// - `requested_friction`: The public friction coefficient.
///
/// # Returns
/// Returns the Rapier friction coefficient to store on the collider.
fn encode_rapier_friction_coefficient(requested_friction: f32) -> f32 {
  return requested_friction.sqrt();
}

/// Resolves how attaching a collider affects a body's backend mass state.
///
/// This helper encodes the public density semantics without directly mutating
/// Rapier state or backend slots.
///
/// # Arguments
/// - `body_type`: The parent rigid body integration mode.
/// - `explicit_dynamic_mass_kg`: The explicitly configured dynamic mass, if
///   any.
/// - `has_positive_density_colliders`: Whether the body already has at least
///   one collider with `density > 0.0`.
/// - `requested_density`: The density requested for the new collider.
///
/// # Returns
/// Returns a plan describing the Rapier density and any required backend state
/// transitions.
fn resolve_collider_attachment_mass_plan_2d(
  body_type: RigidBodyType2D,
  explicit_dynamic_mass_kg: Option<f32>,
  has_positive_density_colliders: bool,
  requested_density: f32,
) -> ColliderAttachmentMassPlan2D {
  if body_type != RigidBodyType2D::Dynamic {
    return ColliderAttachmentMassPlan2D {
      rapier_density: requested_density,
      should_mark_has_positive_density_colliders: false,
      should_remove_fallback_mass: false,
    };
  }

  if explicit_dynamic_mass_kg.is_some() {
    return ColliderAttachmentMassPlan2D {
      rapier_density: 0.0,
      should_mark_has_positive_density_colliders: false,
      should_remove_fallback_mass: false,
    };
  }

  let is_first_positive_density_collider =
    requested_density > 0.0 && !has_positive_density_colliders;

  return ColliderAttachmentMassPlan2D {
    rapier_density: requested_density,
    should_mark_has_positive_density_colliders:
      is_first_positive_density_collider,
    should_remove_fallback_mass: is_first_positive_density_collider,
  };
}

#[cfg(test)]
mod tests {
  use super::*;

  /// Verifies that the backend integrates gravity using symplectic Euler.
  #[test]
  fn dynamic_body_integrates_with_symplectic_euler() {
    let mut backend = PhysicsBackend2D::new([0.0, -10.0], 1.0);

    let (slot_index, slot_generation) = backend
      .create_rigid_body_2d(
        RigidBodyType2D::Dynamic,
        [0.0, 0.0],
        0.0,
        [0.0, 0.0],
        Some(1.0),
      )
      .unwrap();

    let rapier_handle =
      backend.rigid_body_slots_2d[slot_index as usize].rapier_handle;
    let rapier_body = backend.bodies.get(rapier_handle).unwrap();
    assert_eq!(rapier_body.linear_damping(), 0.0);
    assert_eq!(rapier_body.gravity_scale(), 1.0);
    assert_eq!(backend.integration_parameters.dt, 1.0);

    backend.step_with_timestep_seconds(1.0);

    assert_eq!(
      backend
        .rigid_body_velocity_2d(slot_index, slot_generation)
        .unwrap(),
      [0.0, -10.0]
    );
    assert_eq!(
      backend
        .rigid_body_position_2d(slot_index, slot_generation)
        .unwrap(),
      [0.0, -10.0]
    );

    backend.step_with_timestep_seconds(1.0);

    assert_eq!(
      backend
        .rigid_body_velocity_2d(slot_index, slot_generation)
        .unwrap(),
      [0.0, -20.0]
    );
    assert_eq!(
      backend
        .rigid_body_position_2d(slot_index, slot_generation)
        .unwrap(),
      [0.0, -30.0]
    );

    return;
  }

  /// Verifies that kinematic bodies advance via their linear velocity.
  #[test]
  fn kinematic_body_integrates_using_velocity() {
    let mut backend = PhysicsBackend2D::new([0.0, -10.0], 1.0);

    let (slot_index, slot_generation) = backend
      .create_rigid_body_2d(
        RigidBodyType2D::Kinematic,
        [0.0, 0.0],
        0.0,
        [2.0, 0.0],
        None,
      )
      .unwrap();

    backend.step_with_timestep_seconds(1.0);

    assert_eq!(
      backend
        .rigid_body_position_2d(slot_index, slot_generation)
        .unwrap(),
      [2.0, 0.0]
    );

    return;
  }

  /// Verifies that static bodies remain fixed during stepping.
  #[test]
  fn static_body_does_not_move_during_step() {
    let mut backend = PhysicsBackend2D::new([0.0, -10.0], 1.0);

    let (slot_index, slot_generation) = backend
      .create_rigid_body_2d(
        RigidBodyType2D::Static,
        [1.0, 2.0],
        0.0,
        [3.0, 4.0],
        None,
      )
      .unwrap();

    backend.step_with_timestep_seconds(1.0);

    assert_eq!(
      backend
        .rigid_body_position_2d(slot_index, slot_generation)
        .unwrap(),
      [1.0, 2.0]
    );

    return;
  }

  /// Verifies force accumulation persists until explicitly cleared.
  #[test]
  fn force_accumulates_until_cleared() {
    let mut backend = PhysicsBackend2D::new([0.0, 0.0], 1.0);

    let (slot_index, slot_generation) = backend
      .create_rigid_body_2d(
        RigidBodyType2D::Dynamic,
        [0.0, 0.0],
        0.0,
        [0.0, 0.0],
        Some(2.0),
      )
      .unwrap();

    backend
      .rigid_body_apply_force_2d(slot_index, slot_generation, [10.0, 0.0])
      .unwrap();

    backend.step_with_timestep_seconds(1.0);
    assert_eq!(
      backend
        .rigid_body_velocity_2d(slot_index, slot_generation)
        .unwrap(),
      [5.0, 0.0]
    );
    assert_eq!(
      backend
        .rigid_body_position_2d(slot_index, slot_generation)
        .unwrap(),
      [5.0, 0.0]
    );

    backend.clear_rigid_body_forces_2d();
    backend.step_with_timestep_seconds(1.0);

    assert_eq!(
      backend
        .rigid_body_velocity_2d(slot_index, slot_generation)
        .unwrap(),
      [5.0, 0.0]
    );
    assert_eq!(
      backend
        .rigid_body_position_2d(slot_index, slot_generation)
        .unwrap(),
      [10.0, 0.0]
    );

    return;
  }

  /// Reports rigid-body slot liveness without reading body state.
  #[test]
  fn rigid_body_exists_2d_reports_live_slots() {
    let mut backend = PhysicsBackend2D::new([0.0, 0.0], 1.0);

    let (slot_index, slot_generation) = backend
      .create_rigid_body_2d(
        RigidBodyType2D::Dynamic,
        [0.0, 0.0],
        0.0,
        [0.0, 0.0],
        Some(1.0),
      )
      .unwrap();

    assert!(backend.rigid_body_exists_2d(slot_index, slot_generation));
    assert!(!backend.rigid_body_exists_2d(slot_index, slot_generation + 1));
    assert!(!backend.rigid_body_exists_2d(slot_index + 1, 1));

    return;
  }

  /// Removes fallback mass only for the first positive-density collider.
  #[test]
  fn collider_attachment_mass_plan_marks_first_positive_density_collider() {
    let plan = resolve_collider_attachment_mass_plan_2d(
      RigidBodyType2D::Dynamic,
      None,
      false,
      1.0,
    );

    assert_eq!(
      plan,
      ColliderAttachmentMassPlan2D {
        rapier_density: 1.0,
        should_mark_has_positive_density_colliders: true,
        should_remove_fallback_mass: true,
      }
    );

    return;
  }

  /// Preserves density-driven mass state after the first positive collider.
  #[test]
  fn collider_attachment_mass_plan_does_not_remove_fallback_mass_twice() {
    let plan = resolve_collider_attachment_mass_plan_2d(
      RigidBodyType2D::Dynamic,
      None,
      true,
      1.0,
    );

    assert_eq!(
      plan,
      ColliderAttachmentMassPlan2D {
        rapier_density: 1.0,
        should_mark_has_positive_density_colliders: false,
        should_remove_fallback_mass: false,
      }
    );

    return;
  }

  /// Keeps explicit dynamic mass authoritative over collider density.
  #[test]
  fn collider_attachment_mass_plan_ignores_density_for_explicit_mass() {
    let plan = resolve_collider_attachment_mass_plan_2d(
      RigidBodyType2D::Dynamic,
      Some(2.0),
      false,
      5.0,
    );

    assert_eq!(
      plan,
      ColliderAttachmentMassPlan2D {
        rapier_density: 0.0,
        should_mark_has_positive_density_colliders: false,
        should_remove_fallback_mass: false,
      }
    );

    return;
  }

  /// Encodes friction so Rapier `Multiply` matches the public rule.
  #[test]
  fn rapier_friction_encoding_preserves_public_combination_semantics() {
    let encoded_friction_1 = encode_rapier_friction_coefficient(4.0);
    let encoded_friction_2 = encode_rapier_friction_coefficient(9.0);

    assert_eq!(encoded_friction_1, 2.0);
    assert_eq!(encoded_friction_2, 3.0);
    assert_eq!(encoded_friction_1 * encoded_friction_2, 6.0);

    return;
  }

  /// Verifies that applying an impulse updates velocity immediately.
  #[test]
  fn impulse_updates_velocity_immediately() {
    let mut backend = PhysicsBackend2D::new([0.0, 0.0], 1.0);

    let (slot_index, slot_generation) = backend
      .create_rigid_body_2d(
        RigidBodyType2D::Dynamic,
        [0.0, 0.0],
        0.0,
        [0.0, 0.0],
        Some(2.0),
      )
      .unwrap();

    backend
      .rigid_body_apply_impulse_2d(slot_index, slot_generation, [2.0, 0.0])
      .unwrap();

    assert_eq!(
      backend
        .rigid_body_velocity_2d(slot_index, slot_generation)
        .unwrap(),
      [1.0, 0.0]
    );

    return;
  }
}
