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

/// Stores per-body state that `lambda-rs` tracks alongside Rapier.
///
/// This slot exists because `lambda-rs` defines integration semantics that are
/// stricter than the vendor backend:
/// - Gravity and explicit force accumulation integrate via symplectic Euler.
/// - Forces are accumulated and cleared explicitly by the public API.
/// - Impulses update velocity immediately.
///
/// # Invariants
/// - `rapier_handle` MUST reference a body in `PhysicsBackend2D::bodies`.
/// - `dynamic_mass_kg` MUST be finite and positive for dynamic bodies.
/// - `generation` MUST be non-zero and is used to validate handles.
#[derive(Debug, Clone, Copy)]
struct RigidBodySlot2D {
  /// The rigid body's integration mode.
  body_type: RigidBodyType2D,
  /// The handle to the Rapier rigid body stored in the `RigidBodySet`.
  rapier_handle: RigidBodyHandle,
  /// Accumulated forces applied by the public API, in Newtons.
  force_accumulator: [f32; 2],
  /// The mass in kilograms used for manual force and impulse integration.
  dynamic_mass_kg: f32,
  /// A monotonically increasing counter used to validate stale handles.
  generation: u32,
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

    let mass_kg = resolve_dynamic_mass_kg(body_type, dynamic_mass_kg)?;

    let slot_index = self.rigid_body_slots_2d.len() as u32;
    let slot_generation = 1;

    let rapier_body =
      build_rapier_rigid_body(body_type, position, rotation, velocity, mass_kg);
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
      dynamic_mass_kg: mass_kg,
      generation: slot_generation,
    });

    return Ok((slot_index, slot_generation));
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
    let (body_type, rapier_handle, dynamic_mass_kg) = {
      let body_slot = self.rigid_body_slot_2d(slot_index, slot_generation)?;
      (
        body_slot.body_type,
        body_slot.rapier_handle,
        body_slot.dynamic_mass_kg,
      )
    };

    if body_type != RigidBodyType2D::Dynamic {
      return Err(RigidBody2DBackendError::UnsupportedOperation { body_type });
    }

    let Some(rapier_body) = self.bodies.get_mut(rapier_handle) else {
      return Err(RigidBody2DBackendError::BodyNotFound);
    };
    let current_velocity = rapier_body.linvel();
    let impulse_velocity_delta =
      Vector::new(impulse[0] / dynamic_mass_kg, impulse[1] / dynamic_mass_kg);
    rapier_body.set_linvel(current_velocity + impulse_velocity_delta, true);

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

    // `lambda-rs` defines fixed, symplectic-Euler integration semantics for
    // gravity and explicit forces across substeps. Rapier is stepped with
    // zero gravity so it only contributes constraint and collision impulses.
    self.integrate_external_forces_2d(timestep_seconds);

    self.pipeline.step(
      Vector::ZERO,
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

  /// Integrates gravity and explicit forces into rigid-body velocity.
  ///
  /// This function preserves `lambda-rs` symplectic Euler semantics by
  /// integrating external accelerations into the current velocity before
  /// allowing Rapier to solve constraints and apply collision impulses.
  ///
  /// # Arguments
  /// - `timestep_seconds`: The substep duration in seconds.
  ///
  /// # Returns
  /// Returns `()` after applying velocity updates to dynamic bodies.
  fn integrate_external_forces_2d(&mut self, timestep_seconds: f32) {
    for index in 0..self.rigid_body_slots_2d.len() {
      let (body_type, rapier_handle, force_accumulator, dynamic_mass_kg) = {
        let body_slot = &self.rigid_body_slots_2d[index];
        (
          body_slot.body_type,
          body_slot.rapier_handle,
          body_slot.force_accumulator,
          body_slot.dynamic_mass_kg,
        )
      };

      if body_type != RigidBodyType2D::Dynamic {
        continue;
      }

      let Some(rapier_body) = self.bodies.get_mut(rapier_handle) else {
        continue;
      };

      let gravity_acceleration = self.gravity;
      let acceleration_x =
        gravity_acceleration.x + (force_accumulator[0] / dynamic_mass_kg);
      let acceleration_y =
        gravity_acceleration.y + (force_accumulator[1] / dynamic_mass_kg);

      let current_velocity = rapier_body.linvel();
      let new_velocity = Vector::new(
        current_velocity.x + acceleration_x * timestep_seconds,
        current_velocity.y + acceleration_y * timestep_seconds,
      );
      rapier_body.set_linvel(new_velocity, true);
    }

    return;
  }
}

/// Builds a Rapier rigid body builder with `lambda-rs` invariants applied.
///
/// # Arguments
/// - `body_type`: The integration mode for the rigid body.
/// - `position`: The initial position in meters.
/// - `rotation`: The initial rotation in radians.
/// - `velocity`: The initial linear velocity in meters per second.
/// - `dynamic_mass_kg`: The mass in kilograms for dynamic bodies.
///
/// # Returns
/// Returns a configured Rapier `RigidBodyBuilder`.
fn build_rapier_rigid_body(
  body_type: RigidBodyType2D,
  position: [f32; 2],
  rotation: f32,
  velocity: [f32; 2],
  dynamic_mass_kg: f32,
) -> RigidBodyBuilder {
  let translation = Vector::new(position[0], position[1]);
  let linear_velocity = Vector::new(velocity[0], velocity[1]);

  match body_type {
    RigidBodyType2D::Static => {
      return RigidBodyBuilder::fixed()
        .translation(translation)
        .rotation(rotation)
        .linvel(linear_velocity)
        .lock_rotations();
    }
    RigidBodyType2D::Kinematic => {
      return RigidBodyBuilder::kinematic_velocity_based()
        .translation(translation)
        .rotation(rotation)
        .linvel(linear_velocity)
        .lock_rotations();
    }
    RigidBodyType2D::Dynamic => {
      return RigidBodyBuilder::dynamic()
        .translation(translation)
        .rotation(rotation)
        .linvel(linear_velocity)
        .additional_mass(dynamic_mass_kg)
        .lock_rotations();
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

/// Resolves a dynamic-body mass value from an optional input.
///
/// # Arguments
/// - `body_type`: The integration mode for the rigid body.
/// - `dynamic_mass_kg`: The requested mass in kilograms for dynamic bodies.
///
/// # Returns
/// Returns a mass value in kilograms.
///
/// # Errors
/// Returns `RigidBody2DBackendError` if:
/// - A mass is provided for a non-dynamic body.
/// - A dynamic mass is non-finite or non-positive.
fn resolve_dynamic_mass_kg(
  body_type: RigidBodyType2D,
  dynamic_mass_kg: Option<f32>,
) -> Result<f32, RigidBody2DBackendError> {
  let Some(mass_kg) = dynamic_mass_kg else {
    if body_type == RigidBodyType2D::Dynamic {
      return Ok(1.0);
    }

    return Ok(0.0);
  };

  if body_type != RigidBodyType2D::Dynamic {
    return Err(RigidBody2DBackendError::UnsupportedOperation { body_type });
  }

  if !mass_kg.is_finite() || mass_kg <= 0.0 {
    return Err(RigidBody2DBackendError::InvalidMassKg { mass_kg });
  }

  return Ok(mass_kg);
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
