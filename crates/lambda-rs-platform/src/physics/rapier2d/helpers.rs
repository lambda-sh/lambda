use super::*;

/// Builds a Rapier rigid body builder with `lambda-rs` invariants applied.
///
/// Bodies created by this backend lock 2D rotation so `RigidBody2D` rotation
/// changes only through explicit `set_rotation()` calls. This preserves the
/// current public 2D rigid-body contract, which excludes angular dynamics from
/// simulation stepping.
///
/// # Arguments
/// - `body_type`: The integration mode for the rigid body.
/// - `position`: The initial position in meters.
/// - `rotation`: The initial rotation in radians.
/// - `velocity`: The initial linear velocity in meters per second.
/// - `additional_mass_kg`: The additional mass in kilograms for dynamic
///   bodies.
///
/// # Returns
/// Returns a configured Rapier `RigidBodyBuilder`.
pub(super) fn build_rapier_rigid_body(
  body_type: RigidBodyType2D,
  position: [f32; 2],
  rotation: f32,
  velocity: [f32; 2],
  additional_mass_kg: f32,
) -> RigidBodyBuilder {
  let translation = Vector::new(position[0], position[1]);
  let linear_velocity = Vector::new(velocity[0], velocity[1]);

  match body_type {
    RigidBodyType2D::Static => {
      return RigidBodyBuilder::fixed()
        .translation(translation)
        .rotation(rotation)
        .angvel(0.0)
        .lock_rotations()
        .linvel(linear_velocity);
    }
    RigidBodyType2D::Kinematic => {
      return RigidBodyBuilder::kinematic_velocity_based()
        .translation(translation)
        .rotation(rotation)
        .angvel(0.0)
        .lock_rotations()
        .linvel(linear_velocity);
    }
    RigidBodyType2D::Dynamic => {
      return RigidBodyBuilder::dynamic()
        .translation(translation)
        .rotation(rotation)
        .angvel(0.0)
        .lock_rotations()
        .linvel(linear_velocity)
        .additional_mass(additional_mass_kg);
    }
  }
}

/// Converts public collision filter bitfields into Rapier interaction groups.
///
/// # Arguments
/// - `collision_group`: The collider membership bitfield.
/// - `collision_mask`: The collider interaction mask bitfield.
///
/// # Returns
/// Returns Rapier interaction groups using AND-based matching.
pub(super) fn build_collision_groups_2d(
  collision_group: u32,
  collision_mask: u32,
) -> InteractionGroups {
  return InteractionGroups::new(
    Group::from_bits_retain(collision_group),
    Group::from_bits_retain(collision_mask),
    InteractionTestMode::And,
  );
}

/// Normalizes a body-pair key into a stable `body_a`/`body_b` ordering.
///
/// Stable ordering lets the backend deduplicate collider contacts that belong
/// to the same bodies and keeps the public event stream from oscillating based
/// on Rapier's internal pair ordering. The returned boolean tells callers
/// whether normals reported from collider/body 1 toward collider/body 2 must
/// be flipped to match the normalized body ordering.
///
/// # Arguments
/// - `body_a`: The first raw backend body slot pair.
/// - `body_b`: The second raw backend body slot pair.
///
/// # Returns
/// Returns the normalized body-pair key and whether contact normals should be
/// flipped to point from normalized body A toward normalized body B.
pub(super) fn normalize_body_pair_key_2d(
  body_a: (u32, u32),
  body_b: (u32, u32),
) -> (BodyPairKey2D, bool) {
  if body_a <= body_b {
    return (
      BodyPairKey2D {
        body_a_slot_index: body_a.0,
        body_a_slot_generation: body_a.1,
        body_b_slot_index: body_b.0,
        body_b_slot_generation: body_b.1,
      },
      false,
    );
  }

  return (
    BodyPairKey2D {
      body_a_slot_index: body_b.0,
      body_a_slot_generation: body_b.1,
      body_b_slot_index: body_a.0,
      body_b_slot_generation: body_a.1,
    },
    true,
  );
}

/// Validates a 2D position.
///
/// # Arguments
/// - `x`: The X position in meters.
/// - `y`: The Y position in meters.
///
/// # Returns
/// Returns `()` when the input is finite.
///
/// # Errors
/// Returns `RigidBody2DBackendError::InvalidPosition` when any component is
/// non-finite.
pub(super) fn validate_position(
  x: f32,
  y: f32,
) -> Result<(), RigidBody2DBackendError> {
  if !x.is_finite() || !y.is_finite() {
    return Err(RigidBody2DBackendError::InvalidPosition { x, y });
  }

  return Ok(());
}

/// Validates a rotation angle.
///
/// # Arguments
/// - `radians`: The rotation in radians.
///
/// # Returns
/// Returns `()` when the input is finite.
///
/// # Errors
/// Returns `RigidBody2DBackendError::InvalidRotation` when the angle is
/// non-finite.
pub(super) fn validate_rotation(
  radians: f32,
) -> Result<(), RigidBody2DBackendError> {
  if !radians.is_finite() {
    return Err(RigidBody2DBackendError::InvalidRotation { radians });
  }

  return Ok(());
}

/// Validates a 2D linear velocity.
///
/// # Arguments
/// - `x`: The X velocity in meters per second.
/// - `y`: The Y velocity in meters per second.
///
/// # Returns
/// Returns `()` when the input is finite.
///
/// # Errors
/// Returns `RigidBody2DBackendError::InvalidVelocity` when any component is
/// non-finite.
pub(super) fn validate_velocity(
  x: f32,
  y: f32,
) -> Result<(), RigidBody2DBackendError> {
  if !x.is_finite() || !y.is_finite() {
    return Err(RigidBody2DBackendError::InvalidVelocity { x, y });
  }

  return Ok(());
}

/// Normalizes a finite 2D query vector.
///
/// Query directions are normalized inside the backend so geometric helpers can
/// treat Rapier's `time_of_impact` as world-space distance. Keeping that
/// normalization in one helper avoids subtle drift between different query
/// paths and keeps zero-length rejection consistent.
///
/// # Arguments
/// - `vector`: The vector to normalize.
///
/// # Returns
/// Returns the normalized vector when the input has non-zero finite length.
pub(super) fn normalize_query_vector_2d(vector: [f32; 2]) -> Option<[f32; 2]> {
  let length = vector[0].hypot(vector[1]);

  if !length.is_finite() || length <= 0.0 {
    return None;
  }

  return Some([vector[0] / length, vector[1] / length]);
}

/// Selects the deepest active solver contact from one Rapier contact pair.
///
/// Rapier's collider pair may expose several manifolds and several active
/// solver contacts per manifold. The public start event only needs one
/// representative contact, so this helper keeps the active solver contact with
/// the greatest penetration depth. Solver contacts are used instead of raw
/// tracked contacts because they already provide world-space points and
/// reflect the contacts that actually participated in collision resolution
/// this step.
///
/// # Arguments
/// - `contact_pair`: The Rapier contact pair to inspect.
/// - `should_flip_normal`: Whether the selected normal should be inverted to
///   point from normalized body A toward normalized body B.
///
/// # Returns
/// Returns the deepest active solver contact for the pair, if one exists.
pub(super) fn representative_contact_from_pair_2d(
  contact_pair: &ContactPair,
  should_flip_normal: bool,
) -> Option<BodyPairContact2D> {
  let mut representative_contact = None;

  for manifold in &contact_pair.manifolds {
    let mut normal = [manifold.data.normal.x, manifold.data.normal.y];

    if should_flip_normal {
      normal = [-normal[0], -normal[1]];
    }

    for solver_contact in &manifold.data.solver_contacts {
      let penetration = (-solver_contact.dist).max(0.0);
      let candidate_contact = BodyPairContact2D {
        point: [solver_contact.point.x, solver_contact.point.y],
        normal,
        penetration,
      };

      if representative_contact.as_ref().is_some_and(
        |existing: &BodyPairContact2D| {
          candidate_contact.penetration <= existing.penetration
        },
      ) {
        continue;
      }

      representative_contact = Some(candidate_contact);
    }
  }

  return representative_contact;
}

/// Casts a ray against one live collider and normalizes the reported normal.
///
/// When the origin lies inside a collider, Parry may report a zero normal for
/// the solid hit at distance `0.0`. In that case, this helper performs one
/// non-solid cast along the same ray to recover a stable outward exit normal
/// while preserving the `0.0` contact distance expected by the public API.
///
/// # Arguments
/// - `collider`: The live Rapier collider to test.
/// - `ray`: The normalized query ray.
/// - `max_dist`: The maximum query distance in meters.
///
/// # Returns
/// Returns normalized hit data when the collider intersects the finite ray.
pub(super) fn cast_live_collider_raycast_hit_2d(
  collider: &Collider,
  ray: &Ray,
  max_dist: f32,
) -> Option<RayIntersection> {
  // Use a solid cast so callers starting inside geometry receive an immediate
  // hit at distance `0.0` instead of only the exit point.
  let mut hit = collider.shape().cast_ray_and_get_normal(
    collider.position(),
    ray,
    max_dist,
    true,
  )?;

  let normalized_normal =
    normalize_query_vector_2d([hit.normal.x, hit.normal.y]).or_else(|| {
      // Some inside hits report a zero normal. A follow-up non-solid cast
      // recovers the exit-face normal without changing the public `0.0`
      // distance contract for origin-inside queries.
      let exit_hit = collider.shape().cast_ray_and_get_normal(
        collider.position(),
        ray,
        max_dist,
        false,
      )?;

      return normalize_query_vector_2d([exit_hit.normal.x, exit_hit.normal.y]);
    })?;

  hit.normal = Vector::new(normalized_normal[0], normalized_normal[1]);
  return Some(hit);
}

/// Validates a 2D force vector.
///
/// # Arguments
/// - `x`: The X force component in Newtons.
/// - `y`: The Y force component in Newtons.
///
/// # Returns
/// Returns `()` when the input is finite.
///
/// # Errors
/// Returns `RigidBody2DBackendError::InvalidForce` when any component is
/// non-finite.
pub(super) fn validate_force(
  x: f32,
  y: f32,
) -> Result<(), RigidBody2DBackendError> {
  if !x.is_finite() || !y.is_finite() {
    return Err(RigidBody2DBackendError::InvalidForce { x, y });
  }

  return Ok(());
}

/// Validates a 2D impulse vector.
///
/// # Arguments
/// - `x`: The X impulse component in Newton-seconds.
/// - `y`: The Y impulse component in Newton-seconds.
///
/// # Returns
/// Returns `()` when the input is finite.
///
/// # Errors
/// Returns `RigidBody2DBackendError::InvalidImpulse` when any component is
/// non-finite.
pub(super) fn validate_impulse(
  x: f32,
  y: f32,
) -> Result<(), RigidBody2DBackendError> {
  if !x.is_finite() || !y.is_finite() {
    return Err(RigidBody2DBackendError::InvalidImpulse { x, y });
  }

  return Ok(());
}

/// Resolves the explicitly configured mass for a rigid body.
///
/// # Arguments
/// - `body_type`: The integration mode for the rigid body.
/// - `dynamic_mass_kg`: The requested mass in kilograms for dynamic bodies.
///
/// # Returns
/// Returns the explicitly configured dynamic mass.
///
/// # Errors
/// Returns `RigidBody2DBackendError` if a mass is provided for a non-dynamic
/// body, or if the mass is non-finite or non-positive.
pub(super) fn resolve_explicit_dynamic_mass_kg(
  body_type: RigidBodyType2D,
  dynamic_mass_kg: Option<f32>,
) -> Result<Option<f32>, RigidBody2DBackendError> {
  let Some(mass_kg) = dynamic_mass_kg else {
    return Ok(None);
  };

  if body_type != RigidBodyType2D::Dynamic {
    return Err(RigidBody2DBackendError::UnsupportedOperation { body_type });
  }

  if !mass_kg.is_finite() || mass_kg <= 0.0 {
    return Err(RigidBody2DBackendError::InvalidMassKg { mass_kg });
  }

  return Ok(Some(mass_kg));
}

/// Resolves the additional mass in kilograms applied when creating a body.
///
/// # Arguments
/// - `body_type`: The rigid body integration mode.
/// - `explicit_dynamic_mass_kg`: The explicitly configured mass for dynamic
///   bodies.
///
/// # Returns
/// Returns the additional mass value in kilograms.
///
/// # Errors
/// Returns `RigidBody2DBackendError::InvalidMassKg` if the fallback mass
/// cannot be represented as a positive finite value.
pub(super) fn resolve_additional_mass_kg(
  body_type: RigidBodyType2D,
  explicit_dynamic_mass_kg: Option<f32>,
) -> Result<f32, RigidBody2DBackendError> {
  if body_type != RigidBodyType2D::Dynamic {
    return Ok(0.0);
  }

  if let Some(explicit_mass_kg) = explicit_dynamic_mass_kg {
    return Ok(explicit_mass_kg);
  }

  let fallback_mass_kg = DYNAMIC_BODY_FALLBACK_MASS_KG;
  if !fallback_mass_kg.is_finite() || fallback_mass_kg <= 0.0 {
    return Err(RigidBody2DBackendError::InvalidMassKg {
      mass_kg: fallback_mass_kg,
    });
  }

  return Ok(fallback_mass_kg);
}

/// Encodes a public friction coefficient for Rapier's `Multiply` rule.
///
/// Lambda specifies `sqrt(friction_a * friction_b)` as the effective contact
/// friction. Rapier cannot express that rule directly, so the backend stores
/// `sqrt(requested_friction)` on each collider and relies on
/// `CoefficientCombineRule::Multiply` to recover the public result.
///
/// # Arguments
/// - `requested_friction`: The public friction coefficient.
///
/// # Returns
/// Returns the Rapier friction coefficient to store on the collider.
pub(super) fn encode_rapier_friction_coefficient(
  requested_friction: f32,
) -> f32 {
  return requested_friction.sqrt();
}

/// Resolves how attaching a collider affects a body's backend mass state.
///
/// This helper encodes the public density semantics without directly mutating
/// Rapier state or backend slots.
///
/// # Arguments
/// - `body_type`: The parent rigid body integration mode.
/// - `explicit_dynamic_mass_kg`: The explicitly configured dynamic mass, if
///   any.
/// - `has_positive_density_colliders`: Whether the body already has at least
///   one collider with `density > 0.0`.
/// - `requested_density`: The density requested for the new collider.
///
/// # Returns
/// Returns a plan describing the Rapier density and any required backend state
/// transitions.
pub(super) fn resolve_collider_attachment_mass_plan_2d(
  body_type: RigidBodyType2D,
  explicit_dynamic_mass_kg: Option<f32>,
  has_positive_density_colliders: bool,
  requested_density: f32,
) -> ColliderAttachmentMassPlan2D {
  if body_type != RigidBodyType2D::Dynamic {
    return ColliderAttachmentMassPlan2D {
      rapier_density: requested_density,
      should_mark_has_positive_density_colliders: false,
      should_remove_fallback_mass: false,
    };
  }

  if explicit_dynamic_mass_kg.is_some() {
    return ColliderAttachmentMassPlan2D {
      rapier_density: 0.0,
      should_mark_has_positive_density_colliders: false,
      should_remove_fallback_mass: false,
    };
  }

  let is_first_positive_density_collider =
    requested_density > 0.0 && !has_positive_density_colliders;

  return ColliderAttachmentMassPlan2D {
    rapier_density: requested_density,
    should_mark_has_positive_density_colliders:
      is_first_positive_density_collider,
    should_remove_fallback_mass: is_first_positive_density_collider,
  };
}
