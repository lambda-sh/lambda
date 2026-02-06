//! Rapier-backed 2D physics backend.
//!
//! This module provides a minimal wrapper around `rapier2d` to support the
//! higher-level `lambda-rs` physics APIs without exposing vendor types outside
//! of the platform layer.

use rapier2d::prelude::*;

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

    let mut integration_parameters = IntegrationParameters::default();
    integration_parameters.dt = timestep_seconds;

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
    };
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
  }
}
