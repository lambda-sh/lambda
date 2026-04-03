//! 2D collision filter integration tests.
//!
//! These tests validate that collider group and mask settings affect physical
//! contact generation through the public API.

use lambda::physics::{
  Collider2DBuilder,
  CollisionFilter,
  PhysicsWorld2D,
  PhysicsWorld2DBuilder,
  RigidBody2D,
  RigidBody2DBuilder,
  RigidBodyType,
};

const DEFAULT_STEP_COUNT: u32 = 240;

/// Steps a world forward for the given number of fixed timesteps.
///
/// # Arguments
/// - `world`: The world to step.
/// - `steps`: The number of steps to execute.
///
/// # Returns
/// Returns `()` after stepping the world.
fn step_world(world: &mut PhysicsWorld2D, steps: u32) {
  for _ in 0..steps {
    world.step();
  }

  return;
}

/// Creates a static ground body with the provided collision filter.
///
/// # Arguments
/// - `world`: The world that will own the body.
/// - `filter`: The collision filter to apply to the ground collider.
///
/// # Returns
/// Returns the created rigid body handle.
fn build_ground(
  world: &mut PhysicsWorld2D,
  filter: CollisionFilter,
) -> RigidBody2D {
  let ground = RigidBody2DBuilder::new(RigidBodyType::Static)
    .with_position(0.0, -1.0)
    .build(world)
    .unwrap();

  Collider2DBuilder::rectangle(20.0, 0.5)
    .with_collision_filter(filter)
    .build(world, ground)
    .unwrap();

  return ground;
}

/// Creates a dynamic ball body with the provided collision filter.
///
/// # Arguments
/// - `world`: The world that will own the body.
/// - `filter`: The collision filter to apply to the ball collider.
///
/// # Returns
/// Returns the created rigid body handle.
fn build_ball(
  world: &mut PhysicsWorld2D,
  filter: CollisionFilter,
) -> RigidBody2D {
  let ball = RigidBody2DBuilder::new(RigidBodyType::Dynamic)
    .with_position(0.0, 4.0)
    .build(world)
    .unwrap();

  Collider2DBuilder::circle(0.5)
    .with_collision_filter(filter)
    .build(world, ball)
    .unwrap();

  return ball;
}

/// Allows collisions when both colliders' group and mask settings match.
#[test]
fn physics_2d_matching_collision_filters_allow_contact() {
  let mut world = PhysicsWorld2DBuilder::new().build().unwrap();

  build_ground(
    &mut world,
    CollisionFilter {
      group: 0b0001,
      mask: 0b0010,
    },
  );
  let ball = build_ball(
    &mut world,
    CollisionFilter {
      group: 0b0010,
      mask: 0b0001,
    },
  );

  step_world(&mut world, DEFAULT_STEP_COUNT);

  let position = ball.position(&world).unwrap();

  assert!(
    position[1] > -0.25,
    "ball did not collide with the ground: y={}",
    position[1],
  );

  return;
}

/// Prevents collisions when the colliders' groups and masks do not overlap.
#[test]
fn physics_2d_mismatched_collision_filters_prevent_contact() {
  let mut world = PhysicsWorld2DBuilder::new().build().unwrap();

  build_ground(
    &mut world,
    CollisionFilter {
      group: 0b0001,
      mask: 0b0001,
    },
  );
  let ball = build_ball(
    &mut world,
    CollisionFilter {
      group: 0b0010,
      mask: 0b0010,
    },
  );

  step_world(&mut world, DEFAULT_STEP_COUNT);

  let position = ball.position(&world).unwrap();

  assert!(
    position[1] < -5.0,
    "ball unexpectedly collided with the ground: y={}",
    position[1],
  );

  return;
}
