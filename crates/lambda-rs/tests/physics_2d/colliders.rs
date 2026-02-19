//! 2D collider integration tests.
//!
//! These tests validate collider attachment through the public API and ensure
//! that contact resolution influences motion for common shape combinations.

use lambda::physics::{
  Collider2DBuilder,
  PhysicsWorld2D,
  PhysicsWorld2DBuilder,
  RigidBody2DBuilder,
  RigidBodyType,
};

const DEFAULT_STEP_COUNT: u32 = 300;

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

/// Asserts that `value` is within the inclusive range `[min_value, max_value]`.
///
/// # Arguments
/// - `label`: A short label used in assertion messages.
/// - `value`: The value to validate.
/// - `min_value`: The inclusive lower bound.
/// - `max_value`: The inclusive upper bound.
///
/// # Returns
/// Returns `()` when the range check passes.
fn assert_in_range(label: &str, value: f32, min_value: f32, max_value: f32) {
  assert!(
    value >= min_value && value <= max_value,
    "{label} out of range: {value} (expected [{min_value}, {max_value}])",
  );

  return;
}

/// Ensures a dynamic circle collider collides with a static ground rectangle.
#[test]
fn physics_2d_circle_collider_collides_with_ground_rectangle() {
  let mut world = PhysicsWorld2DBuilder::new().build().unwrap();

  let ground = RigidBody2DBuilder::new(RigidBodyType::Static)
    .with_position(0.0, -1.0)
    .build(&mut world)
    .unwrap();

  Collider2DBuilder::rectangle(20.0, 0.5)
    .build(&mut world, ground)
    .unwrap();

  let ball = RigidBody2DBuilder::new(RigidBodyType::Dynamic)
    .with_position(0.0, 4.0)
    .build(&mut world)
    .unwrap();

  Collider2DBuilder::circle(0.5)
    .build(&mut world, ball)
    .unwrap();

  step_world(&mut world, DEFAULT_STEP_COUNT);

  let position = ball.position(&world).unwrap();
  let velocity = ball.velocity(&world).unwrap();

  assert_in_range("x", position[0], -0.05, 0.05);

  // Ground top is at `-1.0 + 0.5 = -0.5`, and the circle should come to rest
  // around `y = -0.5 + 0.5 = 0.0` with tolerance for solver jitter.
  assert_in_range("y", position[1], -0.10, 0.20);
  assert_in_range("vy_abs", velocity[1].abs(), 0.0, 0.50);

  return;
}

/// Ensures rectangle collider local rotation affects contact response.
///
/// This test compares a flat platform to a sloped platform created by applying
/// a local rotation to a rectangle collider. The circle's X translation MUST
/// remain close to zero for the flat platform and MUST move significantly on
/// the sloped platform.
#[test]
fn physics_2d_rectangle_collider_local_rotation_changes_motion() {
  let flat_x = simulate_circle_on_platform(0.0);
  let sloped_x = simulate_circle_on_platform(0.4);

  assert_in_range("flat_x", flat_x, -0.10, 0.10);
  assert!(
    sloped_x.abs() > 0.50,
    "sloped_x did not move enough: {sloped_x}",
  );

  return;
}

/// Simulates a circle falling onto a platform with a rotated rectangle collider.
///
/// # Arguments
/// - `platform_local_rotation`: The collider local rotation in radians.
///
/// # Returns
/// Returns the circle X position after stepping.
fn simulate_circle_on_platform(platform_local_rotation: f32) -> f32 {
  let mut world = PhysicsWorld2DBuilder::new().build().unwrap();

  let platform = RigidBody2DBuilder::new(RigidBodyType::Static)
    .with_position(0.0, 0.0)
    .build(&mut world)
    .unwrap();

  Collider2DBuilder::rectangle(8.0, 0.25)
    .with_local_rotation(platform_local_rotation)
    .with_friction(0.0)
    .build(&mut world, platform)
    .unwrap();

  let ball = RigidBody2DBuilder::new(RigidBodyType::Dynamic)
    .with_position(0.0, 5.0)
    .build(&mut world)
    .unwrap();

  Collider2DBuilder::circle(0.5)
    .with_friction(0.0)
    .build(&mut world, ball)
    .unwrap();

  step_world(&mut world, DEFAULT_STEP_COUNT);

  let position = ball.position(&world).unwrap();
  return position[0];
}
