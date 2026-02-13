//! Internal physics backends.
//!
//! This module contains physics backend implementations that support the
//! higher-level `lambda-rs` physics APIs without exposing vendor types from
//! `lambda-rs` itself.

pub mod rapier2d;

pub use rapier2d::{
  PhysicsBackend2D,
  RigidBody2DBackendError,
  RigidBodyType2D,
};
