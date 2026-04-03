use super::{
  helpers::{
    normalize_body_pair_key_2d,
    representative_contact_from_pair_2d,
  },
  *,
};

impl PhysicsBackend2D {
  /// Advances the simulation by one fixed timestep.
  ///
  /// # Returns
  /// Returns `()` after applying integration and constraint solving for the
  /// configured timestep.
  pub fn step(&mut self) {
    return self.step_with_timestep_seconds(self.integration_parameters.dt);
  }

  /// Advances the simulation by the given timestep.
  ///
  /// # Arguments
  /// - `timestep_seconds`: The timestep used for this step.
  ///
  /// # Returns
  /// Returns `()` after applying integration and constraint solving.
  pub fn step_with_timestep_seconds(&mut self, timestep_seconds: f32) {
    self.integration_parameters.dt = timestep_seconds;

    if cfg!(debug_assertions) {
      self.debug_validate_collider_slots_2d();
    }

    // Rapier consumes user forces during each integration step, so
    // accumulated public forces must be re-synchronized before every substep.
    self.sync_force_accumulators_2d();

    self.pipeline.step(
      self.gravity,
      &self.integration_parameters,
      &mut self.islands,
      &mut self.broad_phase,
      &mut self.narrow_phase,
      &mut self.bodies,
      &mut self.colliders,
      &mut self.impulse_joints,
      &mut self.multibody_joints,
      &mut self.ccd_solver,
      &(),
      &(),
    );

    self.collect_collision_events_2d();

    return;
  }

  /// Drains backend collision events queued by prior step calls.
  ///
  /// The backend accumulates transition events across substeps so
  /// `PhysicsWorld2D` can expose one post-step drain point without exposing
  /// Rapier's event machinery or borrowing backend internals through the
  /// public iterator.
  ///
  /// # Returns
  /// Returns all queued backend collision events in insertion order.
  pub fn drain_collision_events_2d(&mut self) -> Vec<CollisionEvent2DBackend> {
    return std::mem::take(&mut self.queued_collision_events_2d);
  }

  /// Collects body-pair collision transitions from the current narrow phase.
  ///
  /// The public API reports one event per body pair, not one event per
  /// collider pair. This pass aggregates Rapier collider contacts by owning
  /// bodies, keeps the deepest active contact seen for each body pair, and
  /// compares the resulting active set against the previous step to detect
  /// both newly started and newly ended contacts without emitting
  /// collider-pair duplicates for compound bodies.
  ///
  /// # Returns
  /// Returns `()` after appending any newly-started or newly-ended events to
  /// the backend queue.
  fn collect_collision_events_2d(&mut self) {
    let mut current_body_pair_contacts: HashMap<
      BodyPairKey2D,
      BodyPairContact2D,
    > = HashMap::new();
    let mut current_body_pair_order = Vec::new();

    for contact_pair in self.narrow_phase.contact_pairs() {
      if !contact_pair.has_any_active_contact() {
        continue;
      }

      let Some((body_pair_key, body_pair_contact)) =
        self.body_pair_contact_from_contact_pair_2d(contact_pair)
      else {
        continue;
      };

      if let Some(existing_contact) =
        current_body_pair_contacts.get_mut(&body_pair_key)
      {
        if body_pair_contact.penetration > existing_contact.penetration {
          *existing_contact = body_pair_contact;
        }

        continue;
      }

      current_body_pair_order.push(body_pair_key);
      current_body_pair_contacts.insert(body_pair_key, body_pair_contact);
    }

    for body_pair_key in current_body_pair_order.iter().copied() {
      if self.active_body_pairs_2d.contains(&body_pair_key) {
        continue;
      }

      let Some(contact) = current_body_pair_contacts.get(&body_pair_key) else {
        continue;
      };

      self
        .queued_collision_events_2d
        .push(CollisionEvent2DBackend {
          kind: CollisionEventKind2DBackend::Started,
          body_a_slot_index: body_pair_key.body_a_slot_index,
          body_a_slot_generation: body_pair_key.body_a_slot_generation,
          body_b_slot_index: body_pair_key.body_b_slot_index,
          body_b_slot_generation: body_pair_key.body_b_slot_generation,
          contact_point: Some(contact.point),
          normal: Some(contact.normal),
          penetration: Some(contact.penetration),
        });
    }

    // Check for ended contacts by looking for body pairs that were active in
    // the previous step but are missing from the current step.
    for body_pair_key in self.active_body_pair_order_2d.iter().copied() {
      if current_body_pair_contacts.contains_key(&body_pair_key) {
        continue;
      }

      self
        .queued_collision_events_2d
        .push(CollisionEvent2DBackend {
          kind: CollisionEventKind2DBackend::Ended,
          body_a_slot_index: body_pair_key.body_a_slot_index,
          body_a_slot_generation: body_pair_key.body_a_slot_generation,
          body_b_slot_index: body_pair_key.body_b_slot_index,
          body_b_slot_generation: body_pair_key.body_b_slot_generation,
          contact_point: None,
          normal: None,
          penetration: None,
        });
    }

    self.active_body_pairs_2d =
      current_body_pair_contacts.keys().copied().collect();
    self.active_body_pair_order_2d = current_body_pair_order;

    return;
  }

  /// Resolves one Rapier contact pair into a normalized body-pair contact.
  ///
  /// Rapier stores contacts per collider pair. The public API is body-oriented,
  /// so this helper maps each collider pair back to its owning bodies,
  /// discards self-contacts, normalizes body ordering for stable
  /// deduplication, and returns the deepest active solver contact for that
  /// body pair.
  ///
  /// # Arguments
  /// - `contact_pair`: The Rapier contact pair to inspect.
  ///
  /// # Returns
  /// Returns the normalized body-pair key and representative contact when the
  /// pair belongs to two distinct tracked bodies and has at least one active
  /// solver contact.
  fn body_pair_contact_from_contact_pair_2d(
    &self,
    contact_pair: &ContactPair,
  ) -> Option<(BodyPairKey2D, BodyPairContact2D)> {
    let body_a =
      self.query_hit_to_parent_body_slot_2d(contact_pair.collider1)?;
    let body_b =
      self.query_hit_to_parent_body_slot_2d(contact_pair.collider2)?;

    if body_a == body_b {
      return None;
    }

    let (body_pair_key, should_flip_normal) =
      normalize_body_pair_key_2d(body_a, body_b);
    let representative_contact =
      representative_contact_from_pair_2d(contact_pair, should_flip_normal)?;

    return Some((body_pair_key, representative_contact));
  }
}
