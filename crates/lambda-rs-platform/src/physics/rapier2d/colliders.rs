use super::{
  helpers::{
    build_collision_groups_2d,
    encode_rapier_friction_coefficient,
    resolve_collider_attachment_mass_plan_2d,
  },
  *,
};

impl PhysicsBackend2D {
  /// Creates and attaches a circle collider to a rigid body.
  ///
  /// The caller MUST validate all collider inputs before reaching this backend.
  /// `lambda-rs` performs that validation in `Collider2DBuilder::build()`.
  ///
  /// # Arguments
  /// - `parent_slot_index`: The rigid body slot index.
  /// - `parent_slot_generation`: The rigid body slot generation counter.
  /// - `radius`: The circle radius in meters.
  /// - `local_offset`: The collider local translation in meters.
  /// - `local_rotation`: The collider local rotation in radians.
  /// - `density`: The density in kg/m².
  /// - `friction`: The friction coefficient (unitless).
  /// - `restitution`: The restitution coefficient in `[0.0, 1.0]`.
  /// - `collision_group`: The collider membership bitfield.
  /// - `collision_mask`: The collider interaction mask bitfield.
  ///
  /// # Returns
  /// Returns a `(slot_index, slot_generation)` pair for the created collider.
  ///
  /// # Errors
  /// Returns `Collider2DBackendError::BodyNotFound` if the parent body does
  /// not exist.
  #[allow(clippy::too_many_arguments)]
  pub fn create_circle_collider_2d(
    &mut self,
    parent_slot_index: u32,
    parent_slot_generation: u32,
    radius: f32,
    local_offset: [f32; 2],
    local_rotation: f32,
    density: f32,
    friction: f32,
    restitution: f32,
    collision_group: u32,
    collision_mask: u32,
  ) -> Result<(u32, u32), Collider2DBackendError> {
    return self.attach_collider_2d(
      parent_slot_index,
      parent_slot_generation,
      ColliderBuilder::ball(radius),
      local_offset,
      local_rotation,
      density,
      friction,
      restitution,
      collision_group,
      collision_mask,
    );
  }

  /// Creates and attaches a rectangle collider to a rigid body.
  ///
  /// The caller MUST validate all collider inputs before reaching this backend.
  /// `lambda-rs` performs that validation in `Collider2DBuilder::build()`.
  ///
  /// # Arguments
  /// - `parent_slot_index`: The rigid body slot index.
  /// - `parent_slot_generation`: The rigid body slot generation counter.
  /// - `half_width`: The rectangle half-width in meters.
  /// - `half_height`: The rectangle half-height in meters.
  /// - `local_offset`: The collider local translation in meters.
  /// - `local_rotation`: The collider local rotation in radians.
  /// - `density`: The density in kg/m².
  /// - `friction`: The friction coefficient (unitless).
  /// - `restitution`: The restitution coefficient in `[0.0, 1.0]`.
  /// - `collision_group`: The collider membership bitfield.
  /// - `collision_mask`: The collider interaction mask bitfield.
  ///
  /// # Returns
  /// Returns a `(slot_index, slot_generation)` pair for the created collider.
  ///
  /// # Errors
  /// Returns `Collider2DBackendError::BodyNotFound` if the parent body does
  /// not exist.
  #[allow(clippy::too_many_arguments)]
  pub fn create_rectangle_collider_2d(
    &mut self,
    parent_slot_index: u32,
    parent_slot_generation: u32,
    half_width: f32,
    half_height: f32,
    local_offset: [f32; 2],
    local_rotation: f32,
    density: f32,
    friction: f32,
    restitution: f32,
    collision_group: u32,
    collision_mask: u32,
  ) -> Result<(u32, u32), Collider2DBackendError> {
    return self.attach_collider_2d(
      parent_slot_index,
      parent_slot_generation,
      ColliderBuilder::cuboid(half_width, half_height),
      local_offset,
      local_rotation,
      density,
      friction,
      restitution,
      collision_group,
      collision_mask,
    );
  }

  /// Creates and attaches a capsule collider to a rigid body.
  ///
  /// The capsule is aligned with the collider local Y axis.
  /// The caller MUST validate all collider inputs before reaching this backend.
  /// `lambda-rs` performs that validation in `Collider2DBuilder::build()`.
  ///
  /// # Arguments
  /// - `parent_slot_index`: The rigid body slot index.
  /// - `parent_slot_generation`: The rigid body slot generation counter.
  /// - `half_height`: The half-height of the capsule segment in meters.
  /// - `radius`: The capsule radius in meters.
  /// - `local_offset`: The collider local translation in meters.
  /// - `local_rotation`: The collider local rotation in radians.
  /// - `density`: The density in kg/m².
  /// - `friction`: The friction coefficient (unitless).
  /// - `restitution`: The restitution coefficient in `[0.0, 1.0]`.
  /// - `collision_group`: The collider membership bitfield.
  /// - `collision_mask`: The collider interaction mask bitfield.
  ///
  /// # Returns
  /// Returns a `(slot_index, slot_generation)` pair for the created collider.
  ///
  /// # Errors
  /// Returns `Collider2DBackendError::BodyNotFound` if the parent body does
  /// not exist.
  #[allow(clippy::too_many_arguments)]
  pub fn create_capsule_collider_2d(
    &mut self,
    parent_slot_index: u32,
    parent_slot_generation: u32,
    half_height: f32,
    radius: f32,
    local_offset: [f32; 2],
    local_rotation: f32,
    density: f32,
    friction: f32,
    restitution: f32,
    collision_group: u32,
    collision_mask: u32,
  ) -> Result<(u32, u32), Collider2DBackendError> {
    let rapier_builder = if half_height == 0.0 {
      ColliderBuilder::ball(radius)
    } else {
      ColliderBuilder::capsule_y(half_height, radius)
    };

    return self.attach_collider_2d(
      parent_slot_index,
      parent_slot_generation,
      rapier_builder,
      local_offset,
      local_rotation,
      density,
      friction,
      restitution,
      collision_group,
      collision_mask,
    );
  }

  /// Creates and attaches a convex polygon collider to a rigid body.
  ///
  /// The polygon vertices are interpreted as points in collider local space.
  /// The caller MUST validate and normalize polygon inputs before reaching this
  /// backend. `lambda-rs` performs that validation in
  /// `Collider2DBuilder::build()`.
  ///
  /// # Arguments
  /// - `parent_slot_index`: The rigid body slot index.
  /// - `parent_slot_generation`: The rigid body slot generation counter.
  /// - `vertices`: The polygon vertices in meters.
  /// - `local_offset`: The collider local translation in meters.
  /// - `local_rotation`: The collider local rotation in radians.
  /// - `density`: The density in kg/m².
  /// - `friction`: The friction coefficient (unitless).
  /// - `restitution`: The restitution coefficient in `[0.0, 1.0]`.
  /// - `collision_group`: The collider membership bitfield.
  /// - `collision_mask`: The collider interaction mask bitfield.
  ///
  /// # Returns
  /// Returns a `(slot_index, slot_generation)` pair for the created collider.
  ///
  /// # Errors
  /// Returns `Collider2DBackendError::BodyNotFound` if the parent body does
  /// not exist. Returns `Collider2DBackendError::InvalidPolygonDegenerate` if
  /// the validated polygon still cannot be represented as a Rapier convex
  /// hull.
  #[allow(clippy::too_many_arguments)]
  pub fn create_convex_polygon_collider_2d(
    &mut self,
    parent_slot_index: u32,
    parent_slot_generation: u32,
    vertices: Vec<[f32; 2]>,
    local_offset: [f32; 2],
    local_rotation: f32,
    density: f32,
    friction: f32,
    restitution: f32,
    collision_group: u32,
    collision_mask: u32,
  ) -> Result<(u32, u32), Collider2DBackendError> {
    let rapier_vertices: Vec<Vector> = vertices
      .iter()
      .map(|vertex| Vector::new(vertex[0], vertex[1]))
      .collect();

    let Some(rapier_builder) =
      ColliderBuilder::convex_hull(rapier_vertices.as_slice())
    else {
      return Err(Collider2DBackendError::InvalidPolygonDegenerate);
    };

    return self.attach_collider_2d(
      parent_slot_index,
      parent_slot_generation,
      rapier_builder,
      local_offset,
      local_rotation,
      density,
      friction,
      restitution,
      collision_group,
      collision_mask,
    );
  }

  /// Attaches a prepared Rapier collider builder to a parent rigid body.
  ///
  /// This helper applies the shared local transform and material properties,
  /// inserts the built collider into Rapier, recomputes parent mass
  /// properties, and allocates the public collider slot.
  ///
  /// Lambda material semantics are encoded using Rapier's built-in combine
  /// rules instead of a custom contact hook:
  /// - friction stores `sqrt(requested_friction)` and uses `Multiply`
  /// - restitution stores the requested value and uses `Max`
  ///
  /// # Arguments
  /// - `parent_slot_index`: The parent rigid body slot index.
  /// - `parent_slot_generation`: The parent slot generation counter.
  /// - `rapier_builder`: The shape-specific Rapier collider builder.
  /// - `local_offset`: The collider local translation in meters.
  /// - `local_rotation`: The collider local rotation in radians.
  /// - `density`: The requested density in kg/m².
  /// - `friction`: The friction coefficient (unitless).
  /// - `restitution`: The restitution coefficient in `[0.0, 1.0]`.
  /// - `collision_group`: The collider membership bitfield.
  /// - `collision_mask`: The collider interaction mask bitfield.
  ///
  /// # Returns
  /// Returns a `(slot_index, slot_generation)` pair for the created collider.
  ///
  /// # Errors
  /// Returns `Collider2DBackendError` if the parent body does not exist.
  #[allow(clippy::too_many_arguments)]
  fn attach_collider_2d(
    &mut self,
    parent_slot_index: u32,
    parent_slot_generation: u32,
    rapier_builder: ColliderBuilder,
    local_offset: [f32; 2],
    local_rotation: f32,
    density: f32,
    friction: f32,
    restitution: f32,
    collision_group: u32,
    collision_mask: u32,
  ) -> Result<(u32, u32), Collider2DBackendError> {
    let (rapier_parent_handle, rapier_density) = self
      .prepare_parent_body_for_collider_attachment_2d(
        parent_slot_index,
        parent_slot_generation,
        density,
      )?;
    let interaction_groups =
      build_collision_groups_2d(collision_group, collision_mask);

    let rapier_collider = rapier_builder
      .translation(Vector::new(local_offset[0], local_offset[1]))
      .rotation(local_rotation)
      .density(rapier_density)
      .friction(encode_rapier_friction_coefficient(friction))
      .friction_combine_rule(CoefficientCombineRule::Multiply)
      .restitution(restitution)
      .restitution_combine_rule(CoefficientCombineRule::Max)
      .collision_groups(interaction_groups)
      .solver_groups(interaction_groups)
      .build();

    let rapier_handle = self.colliders.insert_with_parent(
      rapier_collider,
      rapier_parent_handle,
      &mut self.bodies,
    );

    self.recompute_parent_mass_after_collider_attachment_2d(
      parent_slot_index,
      parent_slot_generation,
      rapier_parent_handle,
    )?;

    let slot_index = self.collider_slots_2d.len() as u32;
    let slot_generation = 1;
    self.collider_slots_2d.push(ColliderSlot2D {
      rapier_handle,
      parent_slot_index,
      parent_slot_generation,
      generation: slot_generation,
    });
    self
      .collider_parent_slots_2d
      .insert(rapier_handle, (parent_slot_index, parent_slot_generation));

    return Ok((slot_index, slot_generation));
  }

  /// Prepares a parent body for collider attachment and resolves the Rapier
  /// density value to apply.
  ///
  /// This function enforces spec mass semantics:
  /// - When a dynamic body's mass is explicitly configured, collider density
  ///   MUST NOT affect mass properties. The returned density is `0.0`.
  /// - When a dynamic body's mass is not explicitly configured, the backend
  ///   starts with a `1.0` kg fallback mass. When attaching the first
  ///   positive-density collider, the fallback mass is removed so the body's
  ///   mass becomes the sum of collider mass contributions.
  ///
  /// # Arguments
  /// - `parent_slot_index`: The parent rigid body slot index.
  /// - `parent_slot_generation`: The parent slot generation counter.
  /// - `requested_density`: The density requested by the public API.
  ///
  /// # Returns
  /// Returns the Rapier body handle and the density to apply to the Rapier
  /// collider.
  ///
  /// # Errors
  /// Returns `Collider2DBackendError::BodyNotFound` if the parent body is
  /// missing or the handle is stale.
  fn prepare_parent_body_for_collider_attachment_2d(
    &mut self,
    parent_slot_index: u32,
    parent_slot_generation: u32,
    requested_density: f32,
  ) -> Result<(RigidBodyHandle, f32), Collider2DBackendError> {
    let (
      body_type,
      rapier_handle,
      explicit_dynamic_mass_kg,
      has_positive_density_colliders,
    ) = {
      let body_slot = self
        .rigid_body_slot_2d(parent_slot_index, parent_slot_generation)
        .map_err(|_| Collider2DBackendError::BodyNotFound)?;
      (
        body_slot.body_type,
        body_slot.rapier_handle,
        body_slot.explicit_dynamic_mass_kg,
        body_slot.has_positive_density_colliders,
      )
    };

    if self.bodies.get(rapier_handle).is_none() {
      return Err(Collider2DBackendError::BodyNotFound);
    }

    let attachment_mass_plan = resolve_collider_attachment_mass_plan_2d(
      body_type,
      explicit_dynamic_mass_kg,
      has_positive_density_colliders,
      requested_density,
    );

    if attachment_mass_plan.should_mark_has_positive_density_colliders {
      let body_slot = self
        .rigid_body_slot_2d_mut(parent_slot_index, parent_slot_generation)
        .map_err(|_| Collider2DBackendError::BodyNotFound)?;
      body_slot.has_positive_density_colliders = true;
    }

    if attachment_mass_plan.should_remove_fallback_mass {
      let Some(rapier_body) = self.bodies.get_mut(rapier_handle) else {
        return Err(Collider2DBackendError::BodyNotFound);
      };
      rapier_body.set_additional_mass(0.0, true);
    }

    return Ok((rapier_handle, attachment_mass_plan.rapier_density));
  }

  /// Recomputes mass properties for a parent body after collider attachment.
  ///
  /// # Arguments
  /// - `parent_slot_index`: The parent rigid body slot index.
  /// - `parent_slot_generation`: The parent slot generation counter.
  /// - `rapier_parent_handle`: The Rapier body handle.
  ///
  /// # Returns
  /// Returns `()` after recomputing mass properties.
  ///
  /// # Errors
  /// Returns `Collider2DBackendError::BodyNotFound` if the parent body is
  /// missing or the handle is stale.
  fn recompute_parent_mass_after_collider_attachment_2d(
    &mut self,
    _parent_slot_index: u32,
    _parent_slot_generation: u32,
    rapier_parent_handle: RigidBodyHandle,
  ) -> Result<(), Collider2DBackendError> {
    let Some(rapier_body) = self.bodies.get_mut(rapier_parent_handle) else {
      return Err(Collider2DBackendError::BodyNotFound);
    };
    rapier_body.recompute_mass_properties_from_colliders(&self.colliders);

    return Ok(());
  }

  /// Validates that collider slots reference live Rapier colliders.
  ///
  /// This debug-only validation reads slot fields to prevent stale-handle
  /// regressions during backend refactors. The direct collider-handle lookup
  /// map is validated alongside the slot table because queries and contact
  /// collection rely on O(1) parent resolution in hot paths.
  ///
  /// # Returns
  /// Returns `()` after completing validation.
  pub(super) fn debug_validate_collider_slots_2d(&self) {
    debug_assert_eq!(
      self.collider_slots_2d.len(),
      self.collider_parent_slots_2d.len(),
      "collider parent lookup map diverged from collider slot table"
    );

    for slot in self.collider_slots_2d.iter() {
      debug_assert!(slot.generation > 0, "collider slot generation is zero");
      debug_assert!(
        self.colliders.get(slot.rapier_handle).is_some(),
        "collider slot references missing Rapier collider"
      );
      debug_assert!(
        self
          .rigid_body_slot_2d(
            slot.parent_slot_index,
            slot.parent_slot_generation
          )
          .is_ok(),
        "collider slot references missing parent rigid body slot"
      );
      debug_assert_eq!(
        self
          .collider_parent_slots_2d
          .get(&slot.rapier_handle)
          .copied(),
        Some((slot.parent_slot_index, slot.parent_slot_generation)),
        "collider parent lookup map references wrong parent rigid body slot"
      );
    }

    return;
  }

  /// Resolves a query hit collider back to its owning rigid body slot.
  ///
  /// # Arguments
  /// - `collider_handle`: The Rapier collider handle returned by a query.
  ///
  /// # Returns
  /// Returns the owning backend rigid body slot pair when the collider slot is
  /// still tracked by the backend.
  pub(super) fn query_hit_to_parent_body_slot_2d(
    &self,
    collider_handle: ColliderHandle,
  ) -> Option<(u32, u32)> {
    return self.collider_parent_slots_2d.get(&collider_handle).copied();
  }
}
