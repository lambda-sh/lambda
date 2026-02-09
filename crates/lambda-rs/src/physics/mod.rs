//! 2D physics world support.
//!
//! This module provides a minimal, fixed-timestep 2D physics world. The public
//! API is backend-agnostic and does not expose vendor types.

use std::{
  error::Error,
  fmt,
};

use lambda_platform::physics::PhysicsBackend2D;

const DEFAULT_GRAVITY_X: f32 = 0.0;
const DEFAULT_GRAVITY_Y: f32 = -9.81;
const DEFAULT_TIMESTEP_SECONDS: f32 = 1.0 / 60.0;
const DEFAULT_SUBSTEPS: u32 = 1;

/// A 2D physics simulation world.
pub struct PhysicsWorld2D {
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

    return Ok(PhysicsWorld2D {
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

fn validate_substeps(substeps: u32) -> Result<(), PhysicsWorld2DError> {
  if substeps < 1 {
    return Err(PhysicsWorld2DError::InvalidSubsteps { substeps });
  }

  return Ok(());
}

fn validate_gravity(gravity: [f32; 2]) -> Result<(), PhysicsWorld2DError> {
  let x = gravity[0];
  let y = gravity[1];

  if !x.is_finite() || !y.is_finite() {
    return Err(PhysicsWorld2DError::InvalidGravity { x, y });
  }

  return Ok(());
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn world_builds_with_defaults() {
    let world = PhysicsWorld2DBuilder::new().build().unwrap();

    assert_eq!(world.gravity(), [0.0, -9.81]);
    assert_eq!(world.timestep_seconds(), 1.0 / 60.0);

    assert_eq!(world.backend.gravity(), [0.0, -9.81]);
    assert_eq!(world.backend.timestep_seconds(), 1.0 / 60.0);

    return;
  }

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

  #[test]
  fn step_does_not_panic_for_empty_world() {
    let mut world = PhysicsWorld2DBuilder::new().build().unwrap();
    world.step();

    return;
  }

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
}
