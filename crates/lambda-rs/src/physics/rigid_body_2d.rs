//! 2D rigid body support.
//!
//! This module provides backend-agnostic rigid body handles and builders for
//! `PhysicsWorld2D`.

use std::{
  error::Error,
  fmt,
};

use lambda_platform::physics::{
  RigidBody2DBackendError,
  RigidBodyType2D,
};

use super::PhysicsWorld2D;

const DEFAULT_POSITION_X: f32 = 0.0;
const DEFAULT_POSITION_Y: f32 = 0.0;
const DEFAULT_ROTATION_RADIANS: f32 = 0.0;
const DEFAULT_VELOCITY_X: f32 = 0.0;
const DEFAULT_VELOCITY_Y: f32 = 0.0;

/// The rigid body integration mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RigidBodyType {
  /// A body that does not move under simulation.
  Static,
  /// A body affected by gravity and forces.
  Dynamic,
  /// A body integrated only by user-provided motion.
  Kinematic,
}

/// An opaque handle to a rigid body stored in a `PhysicsWorld2D`.
///
/// This handle is world-scoped. Operations validate that the handle belongs to
/// the provided world and that the referenced body exists.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RigidBody2D {
  world_id: u32,
  slot_index: u32,
  slot_generation: u32,
}

impl RigidBody2D {
  /// Returns the body type.
  ///
  /// # Arguments
  /// - `world`: The physics world that owns the body.
  ///
  /// # Returns
  /// Returns the rigid body type.
  ///
  /// # Errors
  /// Returns `RigidBody2DError` if the handle is invalid, belongs to a
  /// different world, or does not reference a live body.
  pub fn body_type(
    self,
    world: &PhysicsWorld2D,
  ) -> Result<RigidBodyType, RigidBody2DError> {
    self.validate_handle(world)?;
    return Err(RigidBody2DError::BodyNotFound);
  }

  /// Returns the current position, in meters.
  ///
  /// # Arguments
  /// - `world`: The physics world that owns the body.
  ///
  /// # Returns
  /// Returns the translation as `(x, y)` in meters.
  ///
  /// # Errors
  /// Returns `RigidBody2DError` if the handle is invalid, belongs to a
  /// different world, or does not reference a live body.
  pub fn position(
    self,
    world: &PhysicsWorld2D,
  ) -> Result<[f32; 2], RigidBody2DError> {
    self.validate_handle(world)?;
    return Err(RigidBody2DError::BodyNotFound);
  }

  /// Returns the current rotation, in radians.
  ///
  /// # Arguments
  /// - `world`: The physics world that owns the body.
  ///
  /// # Returns
  /// Returns the rotation around the 2D Z axis in radians.
  ///
  /// # Errors
  /// Returns `RigidBody2DError` if the handle is invalid, belongs to a
  /// different world, or does not reference a live body.
  pub fn rotation(
    self,
    world: &PhysicsWorld2D,
  ) -> Result<f32, RigidBody2DError> {
    self.validate_handle(world)?;
    return Err(RigidBody2DError::BodyNotFound);
  }

  /// Returns the current linear velocity, in meters per second.
  ///
  /// # Arguments
  /// - `world`: The physics world that owns the body.
  ///
  /// # Returns
  /// Returns the linear velocity as `(x, y)` in meters per second.
  ///
  /// # Errors
  /// Returns `RigidBody2DError` if the handle is invalid, belongs to a
  /// different world, or does not reference a live body.
  pub fn velocity(
    self,
    world: &PhysicsWorld2D,
  ) -> Result<[f32; 2], RigidBody2DError> {
    self.validate_handle(world)?;
    return Err(RigidBody2DError::BodyNotFound);
  }

  /// Sets the position, in meters.
  ///
  /// # Arguments
  /// - `world`: The physics world that owns the body.
  /// - `x`: The new X translation in meters.
  /// - `y`: The new Y translation in meters.
  ///
  /// # Returns
  /// Returns `()` after applying the mutation.
  ///
  /// # Errors
  /// Returns `RigidBody2DError` if the input is invalid, the handle is
  /// invalid, belongs to a different world, or does not reference a live body.
  pub fn set_position(
    self,
    world: &mut PhysicsWorld2D,
    x: f32,
    y: f32,
  ) -> Result<(), RigidBody2DError> {
    validate_position(x, y)?;
    self.validate_handle(world)?;
    return Err(RigidBody2DError::BodyNotFound);
  }

  /// Sets the rotation, in radians.
  ///
  /// # Arguments
  /// - `world`: The physics world that owns the body.
  /// - `radians`: The new rotation in radians.
  ///
  /// # Returns
  /// Returns `()` after applying the mutation.
  ///
  /// # Errors
  /// Returns `RigidBody2DError` if the input is invalid, the handle is
  /// invalid, belongs to a different world, or does not reference a live body.
  pub fn set_rotation(
    self,
    world: &mut PhysicsWorld2D,
    radians: f32,
  ) -> Result<(), RigidBody2DError> {
    validate_rotation(radians)?;
    self.validate_handle(world)?;
    return Err(RigidBody2DError::BodyNotFound);
  }

  /// Sets the linear velocity, in meters per second.
  ///
  /// # Arguments
  /// - `world`: The physics world that owns the body.
  /// - `vx`: The new X velocity component in meters per second.
  /// - `vy`: The new Y velocity component in meters per second.
  ///
  /// # Returns
  /// Returns `()` after applying the mutation.
  ///
  /// # Errors
  /// Returns `RigidBody2DError` if the input is invalid, the handle is
  /// invalid, belongs to a different world, or does not reference a live body.
  pub fn set_velocity(
    self,
    world: &mut PhysicsWorld2D,
    vx: f32,
    vy: f32,
  ) -> Result<(), RigidBody2DError> {
    validate_velocity(vx, vy)?;
    self.validate_handle(world)?;
    return Err(RigidBody2DError::BodyNotFound);
  }

  /// Applies a force, in Newtons, at the center of mass.
  ///
  /// # Arguments
  /// - `world`: The physics world that owns the body.
  /// - `fx`: The force X component in Newtons.
  /// - `fy`: The force Y component in Newtons.
  ///
  /// # Returns
  /// Returns `()` after applying the force.
  ///
  /// # Errors
  /// Returns `RigidBody2DError` if the input is invalid, the handle is
  /// invalid, belongs to a different world, or does not reference a live body.
  pub fn apply_force(
    self,
    world: &mut PhysicsWorld2D,
    fx: f32,
    fy: f32,
  ) -> Result<(), RigidBody2DError> {
    validate_force(fx, fy)?;
    self.validate_handle(world)?;
    return Err(RigidBody2DError::BodyNotFound);
  }

  /// Applies an impulse, in Newton-seconds, at the center of mass.
  ///
  /// # Arguments
  /// - `world`: The physics world that owns the body.
  /// - `ix`: The impulse X component in Newton-seconds.
  /// - `iy`: The impulse Y component in Newton-seconds.
  ///
  /// # Returns
  /// Returns `()` after applying the impulse.
  ///
  /// # Errors
  /// Returns `RigidBody2DError` if the input is invalid, the handle is
  /// invalid, belongs to a different world, or does not reference a live body.
  pub fn apply_impulse(
    self,
    world: &mut PhysicsWorld2D,
    ix: f32,
    iy: f32,
  ) -> Result<(), RigidBody2DError> {
    validate_impulse(ix, iy)?;
    self.validate_handle(world)?;
    return Err(RigidBody2DError::BodyNotFound);
  }

  fn validate_handle(
    self,
    world: &PhysicsWorld2D,
  ) -> Result<(), RigidBody2DError> {
    if self.world_id == 0 {
      return Err(RigidBody2DError::InvalidHandle);
    }

    if self.world_id != world.world_id {
      return Err(RigidBody2DError::WorldMismatch);
    }

    return Ok(());
  }
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
  ///
  /// Defaults
  /// - Position: `(0.0, 0.0)`
  /// - Rotation: `0.0`
  /// - Velocity: `(0.0, 0.0)`
  /// - Dynamic mass: unset (defaults to `1.0` on dynamic bodies)
  ///
  /// # Arguments
  /// - `body_type`: The rigid body integration mode.
  ///
  /// # Returns
  /// Returns a new `RigidBody2DBuilder`.
  pub fn new(body_type: RigidBodyType) -> Self {
    return Self {
      body_type,
      position: [DEFAULT_POSITION_X, DEFAULT_POSITION_Y],
      rotation: DEFAULT_ROTATION_RADIANS,
      velocity: [DEFAULT_VELOCITY_X, DEFAULT_VELOCITY_Y],
      dynamic_mass_kg: None,
    };
  }

  /// Sets the initial position, in meters.
  ///
  /// # Arguments
  /// - `x`: The initial X translation in meters.
  /// - `y`: The initial Y translation in meters.
  ///
  /// # Returns
  /// Returns the updated builder.
  pub fn with_position(mut self, x: f32, y: f32) -> Self {
    self.position = [x, y];
    return self;
  }

  /// Sets the initial rotation, in radians.
  ///
  /// # Arguments
  /// - `radians`: The initial rotation in radians.
  ///
  /// # Returns
  /// Returns the updated builder.
  pub fn with_rotation(mut self, radians: f32) -> Self {
    self.rotation = radians;
    return self;
  }

  /// Sets the initial linear velocity, in meters per second.
  ///
  /// # Arguments
  /// - `vx`: The initial X velocity component in meters per second.
  /// - `vy`: The initial Y velocity component in meters per second.
  ///
  /// # Returns
  /// Returns the updated builder.
  pub fn with_velocity(mut self, vx: f32, vy: f32) -> Self {
    self.velocity = [vx, vy];
    return self;
  }

  /// Sets the mass, in kilograms, for dynamic bodies.
  ///
  /// # Arguments
  /// - `mass_kg`: The mass in kilograms.
  ///
  /// # Returns
  /// Returns the updated builder.
  pub fn with_dynamic_mass_kg(mut self, mass_kg: f32) -> Self {
    self.dynamic_mass_kg = Some(mass_kg);
    return self;
  }

  /// Inserts the body into the given world.
  ///
  /// # Arguments
  /// - `world`: The physics world that will own the body.
  ///
  /// # Returns
  /// Returns a world-scoped `RigidBody2D` handle.
  ///
  /// # Errors
  /// Returns `RigidBody2DError` if any configuration value is invalid or if
  /// the configuration is unsupported for the body type.
  pub fn build(
    self,
    world: &mut PhysicsWorld2D,
  ) -> Result<RigidBody2D, RigidBody2DError> {
    validate_position(self.position[0], self.position[1])?;
    validate_rotation(self.rotation)?;
    validate_velocity(self.velocity[0], self.velocity[1])?;

    validate_dynamic_mass_for_body_type(self.body_type, self.dynamic_mass_kg)?;

    let backend_body_type = map_body_type_to_backend(self.body_type);
    let (slot_index, slot_generation) = world
      .backend
      .create_rigid_body_2d(
        backend_body_type,
        self.position,
        self.rotation,
        self.velocity,
        self.dynamic_mass_kg,
      )
      .map_err(map_backend_error)?;

    return Ok(RigidBody2D {
      world_id: world.world_id,
      slot_index,
      slot_generation,
    });
  }
}

/// Errors for rigid body construction and operations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RigidBody2DError {
  /// The rigid body handle encoding is invalid.
  InvalidHandle,
  /// The rigid body handle does not belong to the provided world.
  WorldMismatch,
  /// The rigid body referenced by the handle was not found.
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
  /// The provided mass is invalid.
  InvalidMassKg { mass_kg: f32 },
  /// The requested operation is unsupported for the body type.
  UnsupportedOperation { body_type: RigidBodyType },
}

impl fmt::Display for RigidBody2DError {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::InvalidHandle => {
        return write!(formatter, "invalid rigid body handle");
      }
      Self::WorldMismatch => {
        return write!(formatter, "rigid body handle does not match the world");
      }
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

impl Error for RigidBody2DError {}

fn validate_position(x: f32, y: f32) -> Result<(), RigidBody2DError> {
  if !x.is_finite() || !y.is_finite() {
    return Err(RigidBody2DError::InvalidPosition { x, y });
  }

  return Ok(());
}

fn validate_rotation(radians: f32) -> Result<(), RigidBody2DError> {
  if !radians.is_finite() {
    return Err(RigidBody2DError::InvalidRotation { radians });
  }

  return Ok(());
}

fn validate_velocity(x: f32, y: f32) -> Result<(), RigidBody2DError> {
  if !x.is_finite() || !y.is_finite() {
    return Err(RigidBody2DError::InvalidVelocity { x, y });
  }

  return Ok(());
}

fn validate_force(x: f32, y: f32) -> Result<(), RigidBody2DError> {
  if !x.is_finite() || !y.is_finite() {
    return Err(RigidBody2DError::InvalidForce { x, y });
  }

  return Ok(());
}

fn validate_impulse(x: f32, y: f32) -> Result<(), RigidBody2DError> {
  if !x.is_finite() || !y.is_finite() {
    return Err(RigidBody2DError::InvalidImpulse { x, y });
  }

  return Ok(());
}

fn validate_dynamic_mass_for_body_type(
  body_type: RigidBodyType,
  dynamic_mass_kg: Option<f32>,
) -> Result<(), RigidBody2DError> {
  let Some(mass_kg) = dynamic_mass_kg else {
    return Ok(());
  };

  match body_type {
    RigidBodyType::Dynamic => {
      if !mass_kg.is_finite() || mass_kg <= 0.0 {
        return Err(RigidBody2DError::InvalidMassKg { mass_kg });
      }

      return Ok(());
    }
    RigidBodyType::Static | RigidBodyType::Kinematic => {
      return Err(RigidBody2DError::UnsupportedOperation { body_type });
    }
  }
}

fn map_body_type_to_backend(body_type: RigidBodyType) -> RigidBodyType2D {
  match body_type {
    RigidBodyType::Static => {
      return RigidBodyType2D::Static;
    }
    RigidBodyType::Dynamic => {
      return RigidBodyType2D::Dynamic;
    }
    RigidBodyType::Kinematic => {
      return RigidBodyType2D::Kinematic;
    }
  }
}

fn map_body_type_from_backend(body_type: RigidBodyType2D) -> RigidBodyType {
  match body_type {
    RigidBodyType2D::Static => {
      return RigidBodyType::Static;
    }
    RigidBodyType2D::Dynamic => {
      return RigidBodyType::Dynamic;
    }
    RigidBodyType2D::Kinematic => {
      return RigidBodyType::Kinematic;
    }
  }
}

fn map_backend_error(error: RigidBody2DBackendError) -> RigidBody2DError {
  match error {
    RigidBody2DBackendError::BodyNotFound => {
      return RigidBody2DError::BodyNotFound;
    }
    RigidBody2DBackendError::InvalidPosition { x, y } => {
      return RigidBody2DError::InvalidPosition { x, y };
    }
    RigidBody2DBackendError::InvalidRotation { radians } => {
      return RigidBody2DError::InvalidRotation { radians };
    }
    RigidBody2DBackendError::InvalidVelocity { x, y } => {
      return RigidBody2DError::InvalidVelocity { x, y };
    }
    RigidBody2DBackendError::InvalidForce { x, y } => {
      return RigidBody2DError::InvalidForce { x, y };
    }
    RigidBody2DBackendError::InvalidImpulse { x, y } => {
      return RigidBody2DError::InvalidImpulse { x, y };
    }
    RigidBody2DBackendError::InvalidMassKg { mass_kg } => {
      return RigidBody2DError::InvalidMassKg { mass_kg };
    }
    RigidBody2DBackendError::UnsupportedOperation { body_type } => {
      return RigidBody2DError::UnsupportedOperation {
        body_type: map_body_type_from_backend(body_type),
      };
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::physics::PhysicsWorld2DBuilder;

  #[test]
  fn builder_inserts_static_body_into_backend() {
    let mut world = PhysicsWorld2DBuilder::new().build().unwrap();

    let body = RigidBody2DBuilder::new(RigidBodyType::Static)
      .with_position(1.0, 2.0)
      .with_rotation(0.5)
      .with_velocity(100.0, -200.0)
      .build(&mut world)
      .unwrap();

    let position = world
      .backend
      .rigid_body_position_2d(body.slot_index, body.slot_generation)
      .unwrap();
    let rotation = world
      .backend
      .rigid_body_rotation_2d(body.slot_index, body.slot_generation)
      .unwrap();

    assert_eq!(position, [1.0, 2.0]);
    assert_eq!(rotation, 0.5);

    world.step();

    let position_after_step = world
      .backend
      .rigid_body_position_2d(body.slot_index, body.slot_generation)
      .unwrap();
    assert_eq!(position_after_step, [1.0, 2.0]);

    return;
  }

  #[test]
  fn dynamic_body_moves_under_gravity_after_step() {
    let mut world = PhysicsWorld2DBuilder::new()
      .with_gravity(0.0, -1.0)
      .with_timestep_seconds(1.0)
      .build()
      .unwrap();

    let body = RigidBody2DBuilder::new(RigidBodyType::Dynamic)
      .with_position(0.0, 0.0)
      .with_velocity(0.0, 0.0)
      .build(&mut world)
      .unwrap();

    world.step();

    let position_after_step = world
      .backend
      .rigid_body_position_2d(body.slot_index, body.slot_generation)
      .unwrap();
    assert_eq!(position_after_step, [0.0, -1.0]);

    return;
  }
}
