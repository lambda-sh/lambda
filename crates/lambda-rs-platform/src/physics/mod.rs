//! Internal physics backends.
//!
//! This module contains physics backend implementations that support the
//! higher-level `lambda-rs` physics APIs without exposing vendor types from
//! `lambda-rs` itself.

pub mod rapier2d;

pub use rapier2d::{
  Collider2DBackendError,
  CollisionEvent2DBackend,
  CollisionEventKind2DBackend,
  PhysicsBackend2D,
  RaycastHit2DBackend,
  RigidBody2DBackendError,
  RigidBodyType2D,
};
