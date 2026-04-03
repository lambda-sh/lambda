use super::{
  helpers::{
    encode_rapier_friction_coefficient,
    resolve_collider_attachment_mass_plan_2d,
  },
  *,
};

/// Verifies that the backend integrates gravity using symplectic Euler.
#[test]
fn dynamic_body_integrates_with_symplectic_euler() {
  let mut backend = PhysicsBackend2D::new([0.0, -10.0], 1.0);

  let (slot_index, slot_generation) = backend
    .create_rigid_body_2d(
      RigidBodyType2D::Dynamic,
      [0.0, 0.0],
      0.0,
      [0.0, 0.0],
      Some(1.0),
    )
    .unwrap();

  let rapier_handle =
    backend.rigid_body_slots_2d[slot_index as usize].rapier_handle;
  let rapier_body = backend.bodies.get(rapier_handle).unwrap();
  assert_eq!(rapier_body.linear_damping(), 0.0);
  assert_eq!(rapier_body.gravity_scale(), 1.0);
  assert_eq!(backend.integration_parameters.dt, 1.0);

  backend.step_with_timestep_seconds(1.0);

  assert_eq!(
    backend
      .rigid_body_velocity_2d(slot_index, slot_generation)
      .unwrap(),
    [0.0, -10.0]
  );
  assert_eq!(
    backend
      .rigid_body_position_2d(slot_index, slot_generation)
      .unwrap(),
    [0.0, -10.0]
  );

  backend.step_with_timestep_seconds(1.0);

  assert_eq!(
    backend
      .rigid_body_velocity_2d(slot_index, slot_generation)
      .unwrap(),
    [0.0, -20.0]
  );
  assert_eq!(
    backend
      .rigid_body_position_2d(slot_index, slot_generation)
      .unwrap(),
    [0.0, -30.0]
  );

  return;
}

/// Verifies that kinematic bodies advance via their linear velocity.
#[test]
fn kinematic_body_integrates_using_velocity() {
  let mut backend = PhysicsBackend2D::new([0.0, -10.0], 1.0);

  let (slot_index, slot_generation) = backend
    .create_rigid_body_2d(
      RigidBodyType2D::Kinematic,
      [0.0, 0.0],
      0.0,
      [2.0, 0.0],
      None,
    )
    .unwrap();

  backend.step_with_timestep_seconds(1.0);

  assert_eq!(
    backend
      .rigid_body_position_2d(slot_index, slot_generation)
      .unwrap(),
    [2.0, 0.0]
  );

  return;
}

/// Verifies that static bodies remain fixed during stepping.
#[test]
fn static_body_does_not_move_during_step() {
  let mut backend = PhysicsBackend2D::new([0.0, -10.0], 1.0);

  let (slot_index, slot_generation) = backend
    .create_rigid_body_2d(
      RigidBodyType2D::Static,
      [1.0, 2.0],
      0.0,
      [3.0, 4.0],
      None,
    )
    .unwrap();

  backend.step_with_timestep_seconds(1.0);

  assert_eq!(
    backend
      .rigid_body_position_2d(slot_index, slot_generation)
      .unwrap(),
    [1.0, 2.0]
  );

  return;
}

/// Verifies force accumulation persists until explicitly cleared.
#[test]
fn force_accumulates_until_cleared() {
  let mut backend = PhysicsBackend2D::new([0.0, 0.0], 1.0);

  let (slot_index, slot_generation) = backend
    .create_rigid_body_2d(
      RigidBodyType2D::Dynamic,
      [0.0, 0.0],
      0.0,
      [0.0, 0.0],
      Some(2.0),
    )
    .unwrap();

  backend
    .rigid_body_apply_force_2d(slot_index, slot_generation, [10.0, 0.0])
    .unwrap();

  backend.step_with_timestep_seconds(1.0);
  assert_eq!(
    backend
      .rigid_body_velocity_2d(slot_index, slot_generation)
      .unwrap(),
    [5.0, 0.0]
  );
  assert_eq!(
    backend
      .rigid_body_position_2d(slot_index, slot_generation)
      .unwrap(),
    [5.0, 0.0]
  );

  backend.clear_rigid_body_forces_2d();
  backend.step_with_timestep_seconds(1.0);

  assert_eq!(
    backend
      .rigid_body_velocity_2d(slot_index, slot_generation)
      .unwrap(),
    [5.0, 0.0]
  );
  assert_eq!(
    backend
      .rigid_body_position_2d(slot_index, slot_generation)
      .unwrap(),
    [10.0, 0.0]
  );

  return;
}

/// Reports rigid-body slot liveness without reading body state.
#[test]
fn rigid_body_exists_2d_reports_live_slots() {
  let mut backend = PhysicsBackend2D::new([0.0, 0.0], 1.0);

  let (slot_index, slot_generation) = backend
    .create_rigid_body_2d(
      RigidBodyType2D::Dynamic,
      [0.0, 0.0],
      0.0,
      [0.0, 0.0],
      Some(1.0),
    )
    .unwrap();

  assert!(backend.rigid_body_exists_2d(slot_index, slot_generation));
  assert!(!backend.rigid_body_exists_2d(slot_index, slot_generation + 1));
  assert!(!backend.rigid_body_exists_2d(slot_index + 1, 1));

  return;
}

/// Removes fallback mass only for the first positive-density collider.
#[test]
fn collider_attachment_mass_plan_marks_first_positive_density_collider() {
  let plan = resolve_collider_attachment_mass_plan_2d(
    RigidBodyType2D::Dynamic,
    None,
    false,
    1.0,
  );

  assert_eq!(
    plan,
    ColliderAttachmentMassPlan2D {
      rapier_density: 1.0,
      should_mark_has_positive_density_colliders: true,
      should_remove_fallback_mass: true,
    }
  );

  return;
}

/// Preserves density-driven mass state after the first positive collider.
#[test]
fn collider_attachment_mass_plan_does_not_remove_fallback_mass_twice() {
  let plan = resolve_collider_attachment_mass_plan_2d(
    RigidBodyType2D::Dynamic,
    None,
    true,
    1.0,
  );

  assert_eq!(
    plan,
    ColliderAttachmentMassPlan2D {
      rapier_density: 1.0,
      should_mark_has_positive_density_colliders: false,
      should_remove_fallback_mass: false,
    }
  );

  return;
}

/// Keeps explicit dynamic mass authoritative over collider density.
#[test]
fn collider_attachment_mass_plan_ignores_density_for_explicit_mass() {
  let plan = resolve_collider_attachment_mass_plan_2d(
    RigidBodyType2D::Dynamic,
    Some(2.0),
    false,
    5.0,
  );

  assert_eq!(
    plan,
    ColliderAttachmentMassPlan2D {
      rapier_density: 0.0,
      should_mark_has_positive_density_colliders: false,
      should_remove_fallback_mass: false,
    }
  );

  return;
}

/// Encodes friction so Rapier `Multiply` matches the public rule.
#[test]
fn rapier_friction_encoding_preserves_public_combination_semantics() {
  let encoded_friction_1 = encode_rapier_friction_coefficient(4.0);
  let encoded_friction_2 = encode_rapier_friction_coefficient(9.0);

  assert_eq!(encoded_friction_1, 2.0);
  assert_eq!(encoded_friction_2, 3.0);
  assert_eq!(encoded_friction_1 * encoded_friction_2, 6.0);

  return;
}

/// Verifies that applying an impulse updates velocity immediately.
#[test]
fn impulse_updates_velocity_immediately() {
  let mut backend = PhysicsBackend2D::new([0.0, 0.0], 1.0);

  let (slot_index, slot_generation) = backend
    .create_rigid_body_2d(
      RigidBodyType2D::Dynamic,
      [0.0, 0.0],
      0.0,
      [0.0, 0.0],
      Some(2.0),
    )
    .unwrap();

  backend
    .rigid_body_apply_impulse_2d(slot_index, slot_generation, [2.0, 0.0])
    .unwrap();

  assert_eq!(
    backend
      .rigid_body_velocity_2d(slot_index, slot_generation)
      .unwrap(),
    [1.0, 0.0]
  );

  return;
}
