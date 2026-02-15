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

#[derive(Debug, Clone, Copy)]
struct RigidBodySlot2D {
  body_type: RigidBodyType2D,
  position: [f32; 2],
  rotation: f32,
  velocity: [f32; 2],
  force_accumulator: [f32; 2],
  dynamic_mass_kg: f32,
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

    self.rigid_body_slots_2d.push(RigidBodySlot2D {
      body_type,
      position,
      rotation,
      velocity,
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
    let body = self.rigid_body_2d(slot_index, slot_generation)?;
    return Ok(body.body_type);
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
    let body = self.rigid_body_2d(slot_index, slot_generation)?;
    return Ok(body.position);
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
    let body = self.rigid_body_2d(slot_index, slot_generation)?;
    return Ok(body.rotation);
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
    let body = self.rigid_body_2d(slot_index, slot_generation)?;
    return Ok(body.velocity);
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
    let body = self.rigid_body_2d_mut(slot_index, slot_generation)?;
    body.position = position;
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
    let body = self.rigid_body_2d_mut(slot_index, slot_generation)?;
    body.rotation = rotation;
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
    let body = self.rigid_body_2d_mut(slot_index, slot_generation)?;

    match body.body_type {
      RigidBodyType2D::Static => {
        return Err(RigidBody2DBackendError::UnsupportedOperation {
          body_type: body.body_type,
        });
      }
      RigidBodyType2D::Dynamic | RigidBodyType2D::Kinematic => {
        body.velocity = velocity;
        return Ok(());
      }
    }
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
    let body = self.rigid_body_2d_mut(slot_index, slot_generation)?;

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
    let body = self.rigid_body_2d_mut(slot_index, slot_generation)?;

    if body.body_type != RigidBodyType2D::Dynamic {
      return Err(RigidBody2DBackendError::UnsupportedOperation {
        body_type: body.body_type,
      });
    }

    body.velocity[0] += impulse[0] / body.dynamic_mass_kg;
    body.velocity[1] += impulse[1] / body.dynamic_mass_kg;

    return Ok(());
  }

  /// Clears accumulated forces for all stored bodies.
  ///
  /// # Returns
  /// Returns `()` after clearing force accumulators.
  pub fn clear_rigid_body_forces_2d(&mut self) {
    for body in &mut self.rigid_body_slots_2d {
      body.force_accumulator = [0.0, 0.0];
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

    self.step_rigid_bodies_2d(timestep_seconds);

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

  fn rigid_body_2d(
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

  fn rigid_body_2d_mut(
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

  fn step_rigid_bodies_2d(&mut self, timestep_seconds: f32) {
    let gravity = [self.gravity.x, self.gravity.y];

    for body in &mut self.rigid_body_slots_2d {
      match body.body_type {
        RigidBodyType2D::Static => {}
        RigidBodyType2D::Kinematic => {
          body.position[0] += body.velocity[0] * timestep_seconds;
          body.position[1] += body.velocity[1] * timestep_seconds;
        }
        RigidBodyType2D::Dynamic => {
          let acceleration_x =
            gravity[0] + (body.force_accumulator[0] / body.dynamic_mass_kg);
          let acceleration_y =
            gravity[1] + (body.force_accumulator[1] / body.dynamic_mass_kg);

          body.velocity[0] += acceleration_x * timestep_seconds;
          body.velocity[1] += acceleration_y * timestep_seconds;

          body.position[0] += body.velocity[0] * timestep_seconds;
          body.position[1] += body.velocity[1] * timestep_seconds;
        }
      }
    }

    return;
  }
}

fn validate_position(x: f32, y: f32) -> Result<(), RigidBody2DBackendError> {
  if !x.is_finite() || !y.is_finite() {
    return Err(RigidBody2DBackendError::InvalidPosition { x, y });
  }

  return Ok(());
}

fn validate_rotation(radians: f32) -> Result<(), RigidBody2DBackendError> {
  if !radians.is_finite() {
    return Err(RigidBody2DBackendError::InvalidRotation { radians });
  }

  return Ok(());
}

fn validate_velocity(x: f32, y: f32) -> Result<(), RigidBody2DBackendError> {
  if !x.is_finite() || !y.is_finite() {
    return Err(RigidBody2DBackendError::InvalidVelocity { x, y });
  }

  return Ok(());
}

fn validate_force(x: f32, y: f32) -> Result<(), RigidBody2DBackendError> {
  if !x.is_finite() || !y.is_finite() {
    return Err(RigidBody2DBackendError::InvalidForce { x, y });
  }

  return Ok(());
}

fn validate_impulse(x: f32, y: f32) -> Result<(), RigidBody2DBackendError> {
  if !x.is_finite() || !y.is_finite() {
    return Err(RigidBody2DBackendError::InvalidImpulse { x, y });
  }

  return Ok(());
}

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
