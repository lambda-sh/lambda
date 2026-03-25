//! Spatial query integration tests.
//!
//! These tests validate point and axis-aligned overlap queries through the
//! public `lambda-rs` 2D physics API.

use lambda::physics::{
  Collider2DBuilder,
  PhysicsWorld2D,
  PhysicsWorld2DBuilder,
  RigidBody2D,
  RigidBody2DBuilder,
  RigidBodyType,
};

/// Builds a static rectangle body with one rectangle collider.
///
/// # Arguments
/// - `world`: The physics world to mutate.
/// - `position`: The body position in meters.
/// - `half_width`: The rectangle half-width in meters.
/// - `half_height`: The rectangle half-height in meters.
///
/// # Returns
/// Returns the created rigid body handle.
fn build_static_rectangle(
  world: &mut PhysicsWorld2D,
  position: [f32; 2],
  half_width: f32,
  half_height: f32,
) -> RigidBody2D {
  let body = RigidBody2DBuilder::new(RigidBodyType::Static)
    .with_position(position[0], position[1])
    .build(world)
    .unwrap();

  Collider2DBuilder::rectangle(half_width, half_height)
    .build(world, body)
    .unwrap();

  return body;
}

/// Builds a static circle body with one circle collider.
///
/// # Arguments
/// - `world`: The physics world to mutate.
/// - `position`: The body position in meters.
/// - `radius`: The circle radius in meters.
///
/// # Returns
/// Returns the created rigid body handle.
fn build_static_circle(
  world: &mut PhysicsWorld2D,
  position: [f32; 2],
  radius: f32,
) -> RigidBody2D {
  let body = RigidBody2DBuilder::new(RigidBodyType::Static)
    .with_position(position[0], position[1])
    .build(world)
    .unwrap();

  Collider2DBuilder::circle(radius)
    .build(world, body)
    .unwrap();

  return body;
}

/// Builds a static body with two overlapping circle colliders.
///
/// # Arguments
/// - `world`: The physics world to mutate.
///
/// # Returns
/// Returns the created rigid body handle.
fn build_compound_circle_body(world: &mut PhysicsWorld2D) -> RigidBody2D {
  let body = RigidBody2DBuilder::new(RigidBodyType::Static)
    .build(world)
    .unwrap();

  Collider2DBuilder::circle(0.5)
    .with_offset(-0.25, 0.0)
    .build(world, body)
    .unwrap();
  Collider2DBuilder::circle(0.5)
    .with_offset(0.25, 0.0)
    .build(world, body)
    .unwrap();

  return body;
}

/// Ensures point queries include interior and boundary hits.
#[test]
fn physics_2d_queries_point_hits_interior_and_boundary() {
  let mut world = PhysicsWorld2DBuilder::new()
    .with_gravity(0.0, 0.0)
    .build()
    .unwrap();

  let rectangle = build_static_rectangle(&mut world, [0.0, 0.0], 1.0, 1.0);

  assert_eq!(world.query_point([0.0, 0.0]), vec![rectangle]);
  assert_eq!(world.query_point([1.0, 0.0]), vec![rectangle]);

  return;
}

/// Ensures point queries miss when the point lies outside all colliders.
#[test]
fn physics_2d_queries_point_misses_outside_geometry() {
  let mut world = PhysicsWorld2DBuilder::new()
    .with_gravity(0.0, 0.0)
    .build()
    .unwrap();

  build_static_rectangle(&mut world, [0.0, 0.0], 1.0, 1.0);

  assert!(world.query_point([1.1, 0.0]).is_empty());

  return;
}

/// Ensures AABB queries return all overlapping bodies.
#[test]
fn physics_2d_queries_aabb_hits_overlapping_bodies() {
  let mut world = PhysicsWorld2DBuilder::new()
    .with_gravity(0.0, 0.0)
    .build()
    .unwrap();

  let rectangle = build_static_rectangle(&mut world, [-2.0, 0.0], 0.75, 1.0);
  let circle = build_static_circle(&mut world, [2.0, 0.0], 0.75);

  let hits = world.query_aabb([-3.0, -1.0], [3.0, 1.0]);

  assert_eq!(hits.len(), 2);
  assert!(hits.contains(&rectangle));
  assert!(hits.contains(&circle));

  return;
}

/// Ensures AABB queries normalize inverted bounds before testing.
#[test]
fn physics_2d_queries_aabb_accepts_inverted_bounds() {
  let mut world = PhysicsWorld2DBuilder::new()
    .with_gravity(0.0, 0.0)
    .build()
    .unwrap();

  let circle = build_static_circle(&mut world, [1.0, 0.0], 0.5);
  let hits = world.query_aabb([2.0, 1.0], [0.0, -1.0]);

  assert_eq!(hits, vec![circle]);

  return;
}

/// Ensures compound collider point hits are deduplicated to one body handle.
#[test]
fn physics_2d_queries_compound_colliders_return_one_body_handle() {
  let mut world = PhysicsWorld2DBuilder::new()
    .with_gravity(0.0, 0.0)
    .build()
    .unwrap();

  let body = build_compound_circle_body(&mut world);
  let hits = world.query_point([0.0, 0.0]);

  assert_eq!(hits, vec![body]);

  return;
}
