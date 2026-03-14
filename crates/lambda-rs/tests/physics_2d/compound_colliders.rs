//! Compound collider integration tests.
//!
//! These tests validate that multiple colliders can be attached to a single
//! rigid body and that all attached shapes participate in collision response.

use lambda::physics::{
  Collider2DBuilder,
  PhysicsWorld2D,
  PhysicsWorld2DBuilder,
  RigidBody2DBuilder,
  RigidBodyType,
};

const WALL_X: f32 = 1.0;
const WALL_HALF_WIDTH: f32 = 0.1;

const CIRCLE_RADIUS: f32 = 0.25;
const COMPOUND_CIRCLE_OFFSET_X: f32 = 0.5;

const STEP_COUNT: u32 = 240;

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

/// Ensures multiple colliders attached to one body affect collision extent.
///
/// This test simulates a dynamic body moving towards a static wall. A body
/// with two offset circle colliders MUST come to rest with its center farther
/// from the wall than the equivalent body with a single circle collider.
#[test]
fn physics_2d_multiple_colliders_on_one_body_affect_collision_extent() {
  let single_circle_x = simulate_body_hitting_wall(false);
  let compound_x = simulate_body_hitting_wall(true);

  // With the wall centered at `WALL_X` and having half-width
  // `WALL_HALF_WIDTH`, the stopping distance from the wall should reflect the
  // collider extent along the X axis:
  // - Single circle extent: `CIRCLE_RADIUS`.
  // - Compound extent: `COMPOUND_CIRCLE_OFFSET_X + CIRCLE_RADIUS`.
  assert!(
    compound_x + 0.30 < single_circle_x,
    "compound_x ({compound_x}) was not far enough from single_circle_x \
({single_circle_x})",
  );

  return;
}

/// Simulates a dynamic body moving into a static wall.
///
/// # Arguments
/// - `use_compound`: When true, attaches two offset circle colliders. When
///   false, attaches a single circle collider.
///
/// # Returns
/// Returns the body's final X position after stepping.
fn simulate_body_hitting_wall(use_compound: bool) -> f32 {
  let mut world = PhysicsWorld2DBuilder::new()
    .with_gravity(0.0, 0.0)
    .build()
    .unwrap();

  let wall = RigidBody2DBuilder::new(RigidBodyType::Static)
    .with_position(WALL_X, 0.0)
    .build(&mut world)
    .unwrap();

  Collider2DBuilder::rectangle(WALL_HALF_WIDTH, 5.0)
    .with_restitution(0.0)
    .build(&mut world, wall)
    .unwrap();

  let body = RigidBody2DBuilder::new(RigidBodyType::Dynamic)
    .with_position(-2.0, 0.0)
    .with_velocity(2.0, 0.0)
    .build(&mut world)
    .unwrap();

  if use_compound {
    Collider2DBuilder::circle(CIRCLE_RADIUS)
      .with_offset(-COMPOUND_CIRCLE_OFFSET_X, 0.0)
      .with_restitution(0.0)
      .build(&mut world, body)
      .unwrap();

    Collider2DBuilder::circle(CIRCLE_RADIUS)
      .with_offset(COMPOUND_CIRCLE_OFFSET_X, 0.0)
      .with_restitution(0.0)
      .build(&mut world, body)
      .unwrap();
  } else {
    Collider2DBuilder::circle(CIRCLE_RADIUS)
      .with_restitution(0.0)
      .build(&mut world, body)
      .unwrap();
  }

  step_world(&mut world, STEP_COUNT);

  let position = body.position(&world).unwrap();
  return position[0];
}
