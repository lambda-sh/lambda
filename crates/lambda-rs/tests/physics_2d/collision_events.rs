//! 2D collision event integration tests.
//!
//! These tests validate post-step collision event collection through the
//! public `lambda-rs` 2D physics API.

use lambda::physics::{
  Collider2DBuilder,
  CollisionEvent,
  CollisionEventKind,
  PhysicsWorld2D,
  PhysicsWorld2DBuilder,
  RigidBody2D,
  RigidBody2DBuilder,
  RigidBodyType,
};

const MAX_STEP_COUNT_UNTIL_CONTACT: u32 = 240;
const STEADY_CONTACT_STEP_COUNT: u32 = 30;

/// Creates a static ground body.
///
/// # Arguments
/// - `world`: The world that will own the ground body.
///
/// # Returns
/// Returns the created ground rigid body handle.
fn build_ground(world: &mut PhysicsWorld2D) -> RigidBody2D {
  let ground = RigidBody2DBuilder::new(RigidBodyType::Static)
    .with_position(0.0, -1.0)
    .build(world)
    .unwrap();

  Collider2DBuilder::rectangle(10.0, 0.5)
    .build(world, ground)
    .unwrap();

  return ground;
}

/// Creates a falling dynamic ball body.
///
/// # Arguments
/// - `world`: The world that will own the ball body.
///
/// # Returns
/// Returns the created ball rigid body handle.
fn build_ball(world: &mut PhysicsWorld2D) -> RigidBody2D {
  let ball = RigidBody2DBuilder::new(RigidBodyType::Dynamic)
    .with_position(0.0, 3.0)
    .build(world)
    .unwrap();

  Collider2DBuilder::circle(0.5).build(world, ball).unwrap();

  return ball;
}

/// Creates a static body with two overlapping circle colliders.
///
/// # Arguments
/// - `world`: The world that will own the compound body.
///
/// # Returns
/// Returns the created compound rigid body handle.
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

/// Creates a dynamic ball already positioned in overlap.
///
/// # Arguments
/// - `world`: The world that will own the body.
/// - `position`: The initial body position in meters.
///
/// # Returns
/// Returns the created rigid body handle.
fn build_overlapping_ball(
  world: &mut PhysicsWorld2D,
  position: [f32; 2],
) -> RigidBody2D {
  let ball = RigidBody2DBuilder::new(RigidBodyType::Dynamic)
    .with_position(position[0], position[1])
    .build(world)
    .unwrap();

  Collider2DBuilder::circle(0.5).build(world, ball).unwrap();

  return ball;
}

/// Steps until at least one collision event is produced.
///
/// # Arguments
/// - `world`: The world to step.
/// - `max_steps`: The maximum number of steps to attempt.
///
/// # Returns
/// Returns the drained events from the first step that produced any events.
fn step_until_collision_events(
  world: &mut PhysicsWorld2D,
  max_steps: u32,
) -> Vec<CollisionEvent> {
  for _ in 0..max_steps {
    world.step();

    let events: Vec<CollisionEvent> = world.collision_events().collect();
    if !events.is_empty() {
      return events;
    }
  }

  panic!("expected collision events within {max_steps} steps");
}

/// Steps until a collision event of the requested kind is produced.
///
/// # Arguments
/// - `world`: The world to step.
/// - `kind`: The event kind to wait for.
/// - `max_steps`: The maximum number of steps to attempt.
///
/// # Returns
/// Returns all drained events from the first step that produced the requested
/// event kind.
fn step_until_collision_event_kind(
  world: &mut PhysicsWorld2D,
  kind: CollisionEventKind,
  max_steps: u32,
) -> Vec<CollisionEvent> {
  for _ in 0..max_steps {
    world.step();

    let events: Vec<CollisionEvent> = world.collision_events().collect();
    if events.iter().any(|event| event.kind == kind) {
      return events;
    }
  }

  panic!("expected {kind:?} event within {max_steps} steps");
}

/// Ensures first contact emits a single `Started` event.
#[test]
fn physics_2d_collision_events_first_contact_emits_started() {
  let mut world = PhysicsWorld2DBuilder::new().build().unwrap();

  let ground = build_ground(&mut world);
  let ball = build_ball(&mut world);
  let events =
    step_until_collision_events(&mut world, MAX_STEP_COUNT_UNTIL_CONTACT);

  assert_eq!(events.len(), 1);
  assert_eq!(events[0].kind, CollisionEventKind::Started);
  assert_eq!(events[0].body_a, ground);
  assert_eq!(events[0].body_b, ball);

  return;
}

/// Ensures steady-state contact does not re-emit `Started` every step.
#[test]
fn physics_2d_collision_events_steady_contact_emits_no_extra_started() {
  let mut world = PhysicsWorld2DBuilder::new().build().unwrap();

  build_ground(&mut world);
  build_ball(&mut world);
  step_until_collision_events(&mut world, MAX_STEP_COUNT_UNTIL_CONTACT);

  for _ in 0..STEADY_CONTACT_STEP_COUNT {
    world.step();
    assert!(
      world
        .collision_events()
        .all(|event| event.kind != CollisionEventKind::Started),
      "steady-state contact emitted an unexpected Started event",
    );
  }

  return;
}

/// Ensures draining the queue leaves the next read empty.
#[test]
fn physics_2d_collision_events_queue_drains_after_read() {
  let mut world = PhysicsWorld2DBuilder::new().build().unwrap();

  build_ground(&mut world);
  build_ball(&mut world);

  let events =
    step_until_collision_events(&mut world, MAX_STEP_COUNT_UNTIL_CONTACT);

  assert_eq!(events.len(), 1);
  assert_eq!(world.collision_events().count(), 0);

  return;
}

/// Ensures `Started` includes representative contact data.
#[test]
fn physics_2d_collision_events_started_includes_contact_data() {
  let mut world = PhysicsWorld2DBuilder::new().build().unwrap();

  build_ground(&mut world);
  build_ball(&mut world);

  let events =
    step_until_collision_events(&mut world, MAX_STEP_COUNT_UNTIL_CONTACT);
  let started_event = events
    .into_iter()
    .find(|event| event.kind == CollisionEventKind::Started)
    .unwrap();

  let contact_point = started_event.contact_point.unwrap();
  let normal = started_event.normal.unwrap();
  let penetration = started_event.penetration.unwrap();

  assert!(contact_point[0].is_finite());
  assert!(contact_point[1].is_finite());
  assert!(normal[0].is_finite());
  assert!(normal[1].is_finite());
  assert!(penetration.is_finite());
  assert!(penetration >= 0.0);
  assert!(
    (normal[0] * normal[0] + normal[1] * normal[1] - 1.0).abs() <= 1.0e-4,
    "expected a unit normal, got {:?}",
    normal,
  );

  return;
}

/// Ensures separation emits one `Ended` event.
#[test]
fn physics_2d_collision_events_separation_emits_ended() {
  let mut world = PhysicsWorld2DBuilder::new()
    .with_gravity(0.0, 0.0)
    .build()
    .unwrap();

  let ground = build_ground(&mut world);
  let ball = build_overlapping_ball(&mut world, [0.0, 0.0]);

  let started_events =
    step_until_collision_event_kind(&mut world, CollisionEventKind::Started, 1);
  assert_eq!(
    started_events
      .iter()
      .filter(|event| event.kind == CollisionEventKind::Started)
      .count(),
    1,
  );

  ball.set_position(&mut world, 0.0, 4.0).unwrap();
  let ended_events =
    step_until_collision_event_kind(&mut world, CollisionEventKind::Ended, 1);
  let ended_event = ended_events
    .into_iter()
    .find(|event| event.kind == CollisionEventKind::Ended)
    .unwrap();

  assert_eq!(ended_event.body_a, ground);
  assert_eq!(ended_event.body_b, ball);

  return;
}

/// Ensures `Ended` omits contact payload fields.
#[test]
fn physics_2d_collision_events_ended_has_no_contact_payload() {
  let mut world = PhysicsWorld2DBuilder::new()
    .with_gravity(0.0, 0.0)
    .build()
    .unwrap();

  build_ground(&mut world);
  let ball = build_overlapping_ball(&mut world, [0.0, 0.0]);

  step_until_collision_event_kind(&mut world, CollisionEventKind::Started, 1);
  ball.set_position(&mut world, 0.0, 4.0).unwrap();

  let ended_event =
    step_until_collision_event_kind(&mut world, CollisionEventKind::Ended, 1)
      .into_iter()
      .find(|event| event.kind == CollisionEventKind::Ended)
      .unwrap();

  assert_eq!(ended_event.contact_point, None);
  assert_eq!(ended_event.normal, None);
  assert_eq!(ended_event.penetration, None);

  return;
}

/// Ensures compound colliders still emit one event per body pair.
#[test]
fn physics_2d_collision_events_compound_colliders_emit_one_body_pair_event() {
  let mut world = PhysicsWorld2DBuilder::new()
    .with_gravity(0.0, 0.0)
    .build()
    .unwrap();

  let compound_body = build_compound_circle_body(&mut world);
  let ball = build_overlapping_ball(&mut world, [0.0, 0.0]);

  let started_events =
    step_until_collision_event_kind(&mut world, CollisionEventKind::Started, 1);

  assert_eq!(
    started_events
      .iter()
      .filter(|event| event.kind == CollisionEventKind::Started)
      .count(),
    1,
  );
  assert_eq!(
    started_events
      .iter()
      .find(|event| event.kind == CollisionEventKind::Started)
      .unwrap()
      .body_a,
    compound_body,
  );
  assert_eq!(
    started_events
      .iter()
      .find(|event| event.kind == CollisionEventKind::Started)
      .unwrap()
      .body_b,
    ball,
  );

  ball.set_position(&mut world, 3.0, 0.0).unwrap();
  let ended_events =
    step_until_collision_event_kind(&mut world, CollisionEventKind::Ended, 1);

  assert_eq!(
    ended_events
      .iter()
      .filter(|event| event.kind == CollisionEventKind::Ended)
      .count(),
    1,
  );

  return;
}

/// Ensures queued events survive multiple steps until drained.
#[test]
fn physics_2d_collision_events_preserve_queue_across_multiple_steps() {
  let mut world = PhysicsWorld2DBuilder::new()
    .with_gravity(0.0, 0.0)
    .build()
    .unwrap();

  build_ground(&mut world);
  let ball = build_overlapping_ball(&mut world, [0.0, 0.0]);

  world.step();
  ball.set_position(&mut world, 0.0, 4.0).unwrap();
  world.step();

  let events: Vec<CollisionEvent> = world.collision_events().collect();

  assert_eq!(events.len(), 2);
  assert_eq!(events[0].kind, CollisionEventKind::Started);
  assert_eq!(events[1].kind, CollisionEventKind::Ended);

  return;
}
