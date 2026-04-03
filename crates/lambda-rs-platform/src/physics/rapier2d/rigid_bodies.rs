use super::{
  helpers::{
    build_rapier_rigid_body,
    resolve_additional_mass_kg,
    resolve_explicit_dynamic_mass_kg,
    validate_force,
    validate_impulse,
    validate_position,
    validate_rotation,
    validate_velocity,
  },
  *,
};

impl PhysicsBackend2D {
  /// Creates a new empty 2D physics backend.
  ///
  /// # Arguments
  /// - `gravity`: The gravity vector in meters per second squared.
  /// - `timestep_seconds`: The fixed integration timestep in seconds.
  ///
  /// # Returns
  /// Returns a new `PhysicsBackend2D` with no bodies, colliders, or joints.
  pub fn new(gravity: [f32; 2], timestep_seconds: f32) -> Self {
    let gravity_vector = Vector::new(gravity[0], gravity[1]);

    // `lambda-rs` exposes substep control at the `PhysicsWorld2D` layer.
    // Rapier's default `num_solver_iterations = 4` also subdivides the step
    // for integration, which changes free-body motion even when the public
    // world is configured with one substep. Keep this at `1` so only
    // `PhysicsWorld2D::substeps` controls outer-step subdivision.
    let integration_parameters = IntegrationParameters {
      dt: timestep_seconds,
      num_solver_iterations: 1,
      ..Default::default()
    };

    return Self {
      gravity: gravity_vector,
      integration_parameters,
      islands: IslandManager::new(),
      broad_phase: BroadPhaseBvh::new(),
      narrow_phase: NarrowPhase::new(),
      bodies: RigidBodySet::new(),
      colliders: ColliderSet::new(),
      impulse_joints: ImpulseJointSet::new(),
      multibody_joints: MultibodyJointSet::new(),
      ccd_solver: CCDSolver::new(),
      pipeline: PhysicsPipeline::new(),
      rigid_body_slots_2d: Vec::new(),
      collider_slots_2d: Vec::new(),
      collider_parent_slots_2d: HashMap::new(),
      active_body_pairs_2d: HashSet::new(),
      active_body_pair_order_2d: Vec::new(),
      queued_collision_events_2d: Vec::new(),
    };
  }

  /// Creates and stores a new 2D rigid body without colliders.
  ///
  /// # Arguments
  /// - `body_type`: The integration mode for the rigid body.
  /// - `position`: The initial position in meters.
  /// - `rotation`: The initial rotation in radians.
  /// - `velocity`: The initial linear velocity in meters per second.
  /// - `dynamic_mass_kg`: The mass in kilograms for dynamic bodies.
  ///
  /// # Returns
  /// Returns a `(slot_index, slot_generation)` pair for the created body.
  ///
  /// # Errors
  /// Returns `RigidBody2DBackendError` if any input is invalid or unsupported.
  pub fn create_rigid_body_2d(
    &mut self,
    body_type: RigidBodyType2D,
    position: [f32; 2],
    rotation: f32,
    velocity: [f32; 2],
    dynamic_mass_kg: Option<f32>,
  ) -> Result<(u32, u32), RigidBody2DBackendError> {
    validate_position(position[0], position[1])?;
    validate_rotation(rotation)?;
    validate_velocity(velocity[0], velocity[1])?;

    let explicit_dynamic_mass_kg =
      resolve_explicit_dynamic_mass_kg(body_type, dynamic_mass_kg)?;
    let additional_mass_kg =
      resolve_additional_mass_kg(body_type, explicit_dynamic_mass_kg)?;

    let slot_index = self.rigid_body_slots_2d.len() as u32;
    let slot_generation = 1;

    let rapier_body = build_rapier_rigid_body(
      body_type,
      position,
      rotation,
      velocity,
      additional_mass_kg,
    );
    let rapier_handle = self.bodies.insert(rapier_body);

    if body_type == RigidBodyType2D::Dynamic {
      let Some(rapier_body) = self.bodies.get_mut(rapier_handle) else {
        return Err(RigidBody2DBackendError::BodyNotFound);
      };
      rapier_body.recompute_mass_properties_from_colliders(&self.colliders);
    }

    self.rigid_body_slots_2d.push(RigidBodySlot2D {
      body_type,
      rapier_handle,
      force_accumulator: [0.0, 0.0],
      explicit_dynamic_mass_kg,
      has_positive_density_colliders: false,
      generation: slot_generation,
    });

    return Ok((slot_index, slot_generation));
  }

  /// Returns the rigid body type for the referenced body.
  ///
  /// # Arguments
  /// - `slot_index`: The body slot index.
  /// - `slot_generation`: The slot generation counter.
  ///
  /// # Returns
  /// Returns the rigid body type.
  ///
  /// # Errors
  /// Returns `RigidBody2DBackendError` if the referenced body does not exist.
  pub fn rigid_body_type_2d(
    &self,
    slot_index: u32,
    slot_generation: u32,
  ) -> Result<RigidBodyType2D, RigidBody2DBackendError> {
    let body_slot = self.rigid_body_slot_2d(slot_index, slot_generation)?;
    return Ok(body_slot.body_type);
  }

  /// Returns the current position for the referenced body.
  ///
  /// # Arguments
  /// - `slot_index`: The body slot index.
  /// - `slot_generation`: The slot generation counter.
  ///
  /// # Returns
  /// Returns the position in meters.
  ///
  /// # Errors
  /// Returns `RigidBody2DBackendError` if the referenced body does not exist.
  pub fn rigid_body_position_2d(
    &self,
    slot_index: u32,
    slot_generation: u32,
  ) -> Result<[f32; 2], RigidBody2DBackendError> {
    let rapier_body = self.rapier_rigid_body_2d(slot_index, slot_generation)?;
    let translation = rapier_body.translation();
    return Ok([translation.x, translation.y]);
  }

  /// Returns the current rotation for the referenced body.
  ///
  /// # Arguments
  /// - `slot_index`: The body slot index.
  /// - `slot_generation`: The slot generation counter.
  ///
  /// # Returns
  /// Returns the rotation in radians.
  ///
  /// # Errors
  /// Returns `RigidBody2DBackendError` if the referenced body does not exist.
  pub fn rigid_body_rotation_2d(
    &self,
    slot_index: u32,
    slot_generation: u32,
  ) -> Result<f32, RigidBody2DBackendError> {
    let rapier_body = self.rapier_rigid_body_2d(slot_index, slot_generation)?;
    return Ok(rapier_body.rotation().angle());
  }

  /// Returns the current linear velocity for the referenced body.
  ///
  /// # Arguments
  /// - `slot_index`: The body slot index.
  /// - `slot_generation`: The slot generation counter.
  ///
  /// # Returns
  /// Returns the linear velocity in meters per second.
  ///
  /// # Errors
  /// Returns `RigidBody2DBackendError` if the referenced body does not exist.
  pub fn rigid_body_velocity_2d(
    &self,
    slot_index: u32,
    slot_generation: u32,
  ) -> Result<[f32; 2], RigidBody2DBackendError> {
    let rapier_body = self.rapier_rigid_body_2d(slot_index, slot_generation)?;
    let velocity = rapier_body.linvel();
    return Ok([velocity.x, velocity.y]);
  }

  /// Returns whether the referenced body slot resolves to a live rigid body.
  ///
  /// # Arguments
  /// - `slot_index`: The body slot index.
  /// - `slot_generation`: The slot generation counter.
  ///
  /// # Returns
  /// Returns `true` when the slot is valid and the Rapier body still exists.
  pub fn rigid_body_exists_2d(
    &self,
    slot_index: u32,
    slot_generation: u32,
  ) -> bool {
    return self
      .rapier_rigid_body_2d(slot_index, slot_generation)
      .is_ok();
  }

  /// Sets the current position for the referenced body.
  ///
  /// # Arguments
  /// - `slot_index`: The body slot index.
  /// - `slot_generation`: The slot generation counter.
  /// - `position`: The new position in meters.
  ///
  /// # Returns
  /// Returns `()` after applying the mutation.
  ///
  /// # Errors
  /// Returns `RigidBody2DBackendError` if the input is invalid or the
  /// referenced body does not exist.
  pub fn rigid_body_set_position_2d(
    &mut self,
    slot_index: u32,
    slot_generation: u32,
    position: [f32; 2],
  ) -> Result<(), RigidBody2DBackendError> {
    validate_position(position[0], position[1])?;
    let rapier_body =
      self.rapier_rigid_body_2d_mut(slot_index, slot_generation)?;
    rapier_body.set_translation(Vector::new(position[0], position[1]), true);
    return Ok(());
  }

  /// Sets the current rotation for the referenced body.
  ///
  /// # Arguments
  /// - `slot_index`: The body slot index.
  /// - `slot_generation`: The slot generation counter.
  /// - `rotation`: The new rotation in radians.
  ///
  /// # Returns
  /// Returns `()` after applying the mutation.
  ///
  /// # Errors
  /// Returns `RigidBody2DBackendError` if the input is invalid or the
  /// referenced body does not exist.
  pub fn rigid_body_set_rotation_2d(
    &mut self,
    slot_index: u32,
    slot_generation: u32,
    rotation: f32,
  ) -> Result<(), RigidBody2DBackendError> {
    validate_rotation(rotation)?;
    let rapier_body =
      self.rapier_rigid_body_2d_mut(slot_index, slot_generation)?;
    rapier_body.set_rotation(Rotation::new(rotation), true);
    return Ok(());
  }

  /// Sets the current linear velocity for the referenced body.
  ///
  /// # Arguments
  /// - `slot_index`: The body slot index.
  /// - `slot_generation`: The slot generation counter.
  /// - `velocity`: The new linear velocity in meters per second.
  ///
  /// # Returns
  /// Returns `()` after applying the mutation.
  ///
  /// # Errors
  /// Returns `RigidBody2DBackendError` if the input is invalid, if the
  /// operation is unsupported for the body type, or if the referenced body
  /// does not exist.
  pub fn rigid_body_set_velocity_2d(
    &mut self,
    slot_index: u32,
    slot_generation: u32,
    velocity: [f32; 2],
  ) -> Result<(), RigidBody2DBackendError> {
    validate_velocity(velocity[0], velocity[1])?;
    let (body_type, rapier_handle) = {
      let body_slot = self.rigid_body_slot_2d(slot_index, slot_generation)?;
      (body_slot.body_type, body_slot.rapier_handle)
    };

    if body_type == RigidBodyType2D::Static {
      return Err(RigidBody2DBackendError::UnsupportedOperation { body_type });
    }

    let Some(rapier_body) = self.bodies.get_mut(rapier_handle) else {
      return Err(RigidBody2DBackendError::BodyNotFound);
    };
    rapier_body.set_linvel(Vector::new(velocity[0], velocity[1]), true);
    return Ok(());
  }

  /// Applies a force, in Newtons, at the center of mass.
  ///
  /// # Arguments
  /// - `slot_index`: The body slot index.
  /// - `slot_generation`: The slot generation counter.
  /// - `force`: The force in Newtons.
  ///
  /// # Returns
  /// Returns `()` after accumulating the force.
  ///
  /// # Errors
  /// Returns `RigidBody2DBackendError` if the input is invalid, if the
  /// operation is unsupported for the body type, or if the referenced body
  /// does not exist.
  pub fn rigid_body_apply_force_2d(
    &mut self,
    slot_index: u32,
    slot_generation: u32,
    force: [f32; 2],
  ) -> Result<(), RigidBody2DBackendError> {
    validate_force(force[0], force[1])?;
    let body = self.rigid_body_slot_2d_mut(slot_index, slot_generation)?;

    if body.body_type != RigidBodyType2D::Dynamic {
      return Err(RigidBody2DBackendError::UnsupportedOperation {
        body_type: body.body_type,
      });
    }

    body.force_accumulator[0] += force[0];
    body.force_accumulator[1] += force[1];

    return Ok(());
  }

  /// Applies an impulse, in Newton-seconds, at the center of mass.
  ///
  /// # Arguments
  /// - `slot_index`: The body slot index.
  /// - `slot_generation`: The slot generation counter.
  /// - `impulse`: The impulse in Newton-seconds.
  ///
  /// # Returns
  /// Returns `()` after applying the impulse.
  ///
  /// # Errors
  /// Returns `RigidBody2DBackendError` if the input is invalid, if the
  /// operation is unsupported for the body type, or if the referenced body
  /// does not exist.
  pub fn rigid_body_apply_impulse_2d(
    &mut self,
    slot_index: u32,
    slot_generation: u32,
    impulse: [f32; 2],
  ) -> Result<(), RigidBody2DBackendError> {
    validate_impulse(impulse[0], impulse[1])?;
    let (body_type, rapier_handle) = {
      let body_slot = self.rigid_body_slot_2d(slot_index, slot_generation)?;
      (body_slot.body_type, body_slot.rapier_handle)
    };

    if body_type != RigidBodyType2D::Dynamic {
      return Err(RigidBody2DBackendError::UnsupportedOperation { body_type });
    }

    let Some(rapier_body) = self.bodies.get_mut(rapier_handle) else {
      return Err(RigidBody2DBackendError::BodyNotFound);
    };
    rapier_body.apply_impulse(Vector::new(impulse[0], impulse[1]), true);

    return Ok(());
  }

  /// Clears accumulated forces for all stored bodies.
  ///
  /// # Returns
  /// Returns `()` after clearing force accumulators.
  pub fn clear_rigid_body_forces_2d(&mut self) {
    for index in 0..self.rigid_body_slots_2d.len() {
      let rapier_handle = {
        let body_slot = &mut self.rigid_body_slots_2d[index];
        body_slot.force_accumulator = [0.0, 0.0];
        body_slot.rapier_handle
      };

      let Some(rapier_body) = self.bodies.get_mut(rapier_handle) else {
        continue;
      };
      rapier_body.reset_forces(true);
    }

    return;
  }

  /// Returns the gravity vector used by this backend.
  ///
  /// # Returns
  /// Returns the gravity vector in meters per second squared.
  pub fn gravity(&self) -> [f32; 2] {
    return [self.gravity.x, self.gravity.y];
  }

  /// Returns the fixed integration timestep in seconds.
  ///
  /// # Returns
  /// Returns the timestep used for each simulation step.
  pub fn timestep_seconds(&self) -> f32 {
    return self.integration_parameters.dt;
  }

  /// Returns an immutable reference to a rigid body slot after validation.
  ///
  /// # Arguments
  /// - `slot_index`: The body slot index.
  /// - `slot_generation`: The slot generation counter.
  ///
  /// # Returns
  /// Returns an immutable reference to the validated `RigidBodySlot2D`.
  ///
  /// # Errors
  /// Returns `RigidBody2DBackendError::BodyNotFound` when the slot is missing
  /// or the generation does not match.
  pub(super) fn rigid_body_slot_2d(
    &self,
    slot_index: u32,
    slot_generation: u32,
  ) -> Result<&RigidBodySlot2D, RigidBody2DBackendError> {
    let Some(body) = self.rigid_body_slots_2d.get(slot_index as usize) else {
      return Err(RigidBody2DBackendError::BodyNotFound);
    };

    if body.generation != slot_generation {
      return Err(RigidBody2DBackendError::BodyNotFound);
    }

    return Ok(body);
  }

  /// Returns a mutable reference to a rigid body slot after validation.
  ///
  /// # Arguments
  /// - `slot_index`: The body slot index.
  /// - `slot_generation`: The slot generation counter.
  ///
  /// # Returns
  /// Returns a mutable reference to the validated `RigidBodySlot2D`.
  ///
  /// # Errors
  /// Returns `RigidBody2DBackendError::BodyNotFound` when the slot is missing
  /// or the generation does not match.
  pub(super) fn rigid_body_slot_2d_mut(
    &mut self,
    slot_index: u32,
    slot_generation: u32,
  ) -> Result<&mut RigidBodySlot2D, RigidBody2DBackendError> {
    let Some(body) = self.rigid_body_slots_2d.get_mut(slot_index as usize)
    else {
      return Err(RigidBody2DBackendError::BodyNotFound);
    };

    if body.generation != slot_generation {
      return Err(RigidBody2DBackendError::BodyNotFound);
    }

    return Ok(body);
  }

  /// Returns an immutable reference to the Rapier rigid body for a slot.
  ///
  /// # Arguments
  /// - `slot_index`: The body slot index.
  /// - `slot_generation`: The slot generation counter.
  ///
  /// # Returns
  /// Returns an immutable reference to the underlying Rapier `RigidBody`.
  ///
  /// # Errors
  /// Returns `RigidBody2DBackendError::BodyNotFound` when the slot is invalid
  /// or the Rapier body has been removed.
  fn rapier_rigid_body_2d(
    &self,
    slot_index: u32,
    slot_generation: u32,
  ) -> Result<&RigidBody, RigidBody2DBackendError> {
    let body_slot = self.rigid_body_slot_2d(slot_index, slot_generation)?;
    let Some(rapier_body) = self.bodies.get(body_slot.rapier_handle) else {
      return Err(RigidBody2DBackendError::BodyNotFound);
    };

    return Ok(rapier_body);
  }

  /// Returns a mutable reference to the Rapier rigid body for a slot.
  ///
  /// # Arguments
  /// - `slot_index`: The body slot index.
  /// - `slot_generation`: The slot generation counter.
  ///
  /// # Returns
  /// Returns a mutable reference to the underlying Rapier `RigidBody`.
  ///
  /// # Errors
  /// Returns `RigidBody2DBackendError::BodyNotFound` when the slot is invalid
  /// or the Rapier body has been removed.
  fn rapier_rigid_body_2d_mut(
    &mut self,
    slot_index: u32,
    slot_generation: u32,
  ) -> Result<&mut RigidBody, RigidBody2DBackendError> {
    let rapier_handle = {
      let body_slot = self.rigid_body_slot_2d(slot_index, slot_generation)?;
      body_slot.rapier_handle
    };

    let Some(rapier_body) = self.bodies.get_mut(rapier_handle) else {
      return Err(RigidBody2DBackendError::BodyNotFound);
    };

    return Ok(rapier_body);
  }

  /// Syncs accumulated forces into Rapier prior to stepping.
  ///
  /// `lambda-rs` exposes a force accumulation API that persists forces until
  /// explicitly cleared. Rapier stores forces on each rigid body. This
  /// function overwrites Rapier's stored force with the value tracked by
  /// `lambda-rs` so Rapier can integrate forces and gravity consistently
  /// during stepping. Bodies with zero accumulated force are skipped because
  /// `clear_*` methods and Rapier step completion already leave them with no
  /// user force to reapply.
  ///
  /// # Returns
  /// Returns `()` after updating Rapier force state for all dynamic bodies.
  pub(super) fn sync_force_accumulators_2d(&mut self) {
    for index in 0..self.rigid_body_slots_2d.len() {
      let (body_type, rapier_handle, force_accumulator) = {
        let body_slot = &self.rigid_body_slots_2d[index];
        (
          body_slot.body_type,
          body_slot.rapier_handle,
          body_slot.force_accumulator,
        )
      };

      if body_type != RigidBodyType2D::Dynamic {
        continue;
      }

      if force_accumulator[0] == 0.0 && force_accumulator[1] == 0.0 {
        continue;
      }

      let Some(rapier_body) = self.bodies.get_mut(rapier_handle) else {
        continue;
      };

      let should_wake =
        force_accumulator[0] != 0.0 || force_accumulator[1] != 0.0;
      rapier_body.reset_forces(false);
      rapier_body.add_force(
        Vector::new(force_accumulator[0], force_accumulator[1]),
        should_wake,
      );
    }

    return;
  }
}
