//! Collider material integration tests.
//!
//! These tests validate that material properties influence collision response
//! and that density-driven mass semantics match the public specification.

use lambda::physics::{
  Collider2DBuilder,
  PhysicsWorld2D,
  PhysicsWorld2DBuilder,
  RigidBody2D,
  RigidBody2DBuilder,
  RigidBodyType,
};

const BALL_RADIUS: f32 = 0.5;
const GROUND_Y: f32 = -1.0;
const GROUND_HALF_HEIGHT: f32 = 0.5;

const DEFAULT_STEP_COUNT: u32 = 420;

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

/// Builds a static ground body with a wide rectangle collider.
///
/// # Arguments
/// - `world`: The world that owns the ground.
/// - `friction`: The ground friction coefficient.
/// - `restitution`: The ground restitution coefficient.
///
/// # Returns
/// Returns the ground rigid body handle.
fn build_ground(
  world: &mut PhysicsWorld2D,
  friction: f32,
  restitution: f32,
) -> RigidBody2D {
  let ground = RigidBody2DBuilder::new(RigidBodyType::Static)
    .with_position(0.0, GROUND_Y)
    .build(world)
    .unwrap();

  Collider2DBuilder::rectangle(40.0, GROUND_HALF_HEIGHT)
    .with_friction(friction)
    .with_restitution(restitution)
    .build(world, ground)
    .unwrap();

  return ground;
}

/// Ensures restitution influences bounce response.
#[test]
fn physics_2d_restitution_changes_bounce_height() {
  let low_restitution_peak_y = simulate_bounce_peak_y(0.0);
  let high_restitution_peak_y = simulate_bounce_peak_y(1.0);

  assert!(
    high_restitution_peak_y > low_restitution_peak_y + 1.0,
    "bounce peak did not increase enough: low={low_restitution_peak_y}, \
high={high_restitution_peak_y}",
  );

  return;
}

/// Simulates a falling ball and returns the peak height after the first hit.
///
/// # Arguments
/// - `ball_restitution`: The ball restitution coefficient.
///
/// # Returns
/// Returns the maximum Y value observed after the ball first reaches the
/// ground plane.
fn simulate_bounce_peak_y(ball_restitution: f32) -> f32 {
  let mut world = PhysicsWorld2DBuilder::new().build().unwrap();
  build_ground(&mut world, 0.0, 0.0);

  let ball = RigidBody2DBuilder::new(RigidBodyType::Dynamic)
    .with_position(0.0, 5.0)
    .build(&mut world)
    .unwrap();

  Collider2DBuilder::circle(BALL_RADIUS)
    .with_friction(0.0)
    .with_restitution(ball_restitution)
    .build(&mut world, ball)
    .unwrap();

  let mut has_reached_ground = false;
  let mut peak_y_after_hit = f32::NEG_INFINITY;

  for _ in 0..DEFAULT_STEP_COUNT {
    world.step();

    let position = ball.position(&world).unwrap();
    let y = position[1];

    if has_reached_ground {
      peak_y_after_hit = peak_y_after_hit.max(y);
      continue;
    }

    // Ground top is at `GROUND_Y + GROUND_HALF_HEIGHT = -0.5`. When the ball
    // reaches that plane, it has contacted the ground.
    if y <= (GROUND_Y + GROUND_HALF_HEIGHT) + BALL_RADIUS + 0.05 {
      has_reached_ground = true;
      peak_y_after_hit = y;
    }
  }

  return peak_y_after_hit;
}

/// Ensures friction affects tangential sliding response.
#[test]
fn physics_2d_friction_changes_sliding_velocity_decay() {
  let low_friction_velocity_x = simulate_slide_velocity_x(0.0);
  let high_friction_velocity_x = simulate_slide_velocity_x(4.0);

  assert!(
    low_friction_velocity_x > high_friction_velocity_x + 1.0,
    "friction did not reduce sliding enough: low={low_friction_velocity_x}, \
high={high_friction_velocity_x}",
  );

  return;
}

/// Simulates a ball sliding along the ground and returns its final X velocity.
///
/// # Arguments
/// - `ball_friction`: The ball friction coefficient.
///
/// # Returns
/// Returns the absolute X velocity after stepping.
fn simulate_slide_velocity_x(ball_friction: f32) -> f32 {
  let mut world = PhysicsWorld2DBuilder::new().build().unwrap();
  build_ground(&mut world, 1.0, 0.0);

  let ball = RigidBody2DBuilder::new(RigidBodyType::Dynamic)
    .with_position(0.0, 2.0)
    .with_velocity(6.0, 0.0)
    .build(&mut world)
    .unwrap();

  Collider2DBuilder::circle(BALL_RADIUS)
    .with_friction(ball_friction)
    .with_restitution(0.0)
    .build(&mut world, ball)
    .unwrap();

  step_world(&mut world, DEFAULT_STEP_COUNT);

  let velocity = ball.velocity(&world).unwrap();
  return velocity[0].abs();
}

/// Ensures density affects dynamic body mass when mass is not explicitly set.
#[test]
fn physics_2d_density_affects_impulse_velocity_delta() {
  let low_density_velocity_x = simulate_impulse_velocity_x(None, 1.0);
  let high_density_velocity_x = simulate_impulse_velocity_x(None, 4.0);

  assert!(
    low_density_velocity_x > high_density_velocity_x * 2.5,
    "density did not change Δv enough: low={low_density_velocity_x}, \
high={high_density_velocity_x}",
  );

  let explicit_mass_velocity_x = simulate_impulse_velocity_x(Some(2.0), 50.0);
  assert!(
    (explicit_mass_velocity_x - 0.5).abs() < 0.05,
    "explicit mass did not override density: velocity_x={explicit_mass_velocity_x}",
  );

  return;
}

/// Applies an impulse to a ball and returns its X velocity.
///
/// # Arguments
/// - `explicit_mass_kg`: When `Some`, configures the body mass explicitly.
/// - `density`: The collider density in kg/m².
///
/// # Returns
/// Returns the X velocity after applying a unit impulse.
fn simulate_impulse_velocity_x(
  explicit_mass_kg: Option<f32>,
  density: f32,
) -> f32 {
  let mut world = PhysicsWorld2DBuilder::new()
    .with_gravity(0.0, 0.0)
    .build()
    .unwrap();

  let mut builder =
    RigidBody2DBuilder::new(RigidBodyType::Dynamic).with_position(0.0, 0.0);
  if let Some(mass_kg) = explicit_mass_kg {
    builder = builder.with_dynamic_mass_kg(mass_kg);
  }

  let body = builder.build(&mut world).unwrap();

  Collider2DBuilder::circle(BALL_RADIUS)
    .with_density(density)
    .with_friction(0.0)
    .with_restitution(0.0)
    .build(&mut world, body)
    .unwrap();

  body.apply_impulse(&mut world, 1.0, 0.0).unwrap();

  let velocity = body.velocity(&world).unwrap();
  return velocity[0];
}
