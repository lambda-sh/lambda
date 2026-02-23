//! 2D physics integration tests.
//!
//! These tests validate `lambda-rs` 2D physics behavior through the public API
//! surface, including cross-crate wiring through `lambda-rs-platform`.

mod colliders;
mod compound_colliders;
mod materials;

use lambda::physics::{
  PhysicsWorld2DBuilder,
  RigidBody2DBuilder,
  RigidBodyType,
};

/// Ensures an empty 2D world can be stepped without panicking.
#[test]
fn physics_2d_world_smoke_steps_empty_world() {
  let mut world = PhysicsWorld2DBuilder::new().build().unwrap();

  world.step();
  world.step();

  return;
}

/// Ensures a dynamic rigid body can be created and advances under gravity.
#[test]
fn physics_2d_rigid_body_smoke_builds_and_steps() {
  let mut world = PhysicsWorld2DBuilder::new().build().unwrap();

  let body = RigidBody2DBuilder::new(RigidBodyType::Dynamic)
    .with_position(0.0, 10.0)
    .build(&mut world)
    .unwrap();

  let initial_position = body.position(&world).unwrap();
  let initial_velocity = body.velocity(&world).unwrap();

  world.step();

  let stepped_position = body.position(&world).unwrap();
  let stepped_velocity = body.velocity(&world).unwrap();

  assert!(stepped_position[1] < initial_position[1]);
  assert!(stepped_velocity[1] < initial_velocity[1]);

  return;
}
