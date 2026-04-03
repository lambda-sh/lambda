use super::{
  helpers::{
    cast_live_collider_raycast_hit_2d,
    normalize_query_vector_2d,
    validate_position,
    validate_velocity,
  },
  *,
};

impl PhysicsBackend2D {
  /// Returns all rigid bodies whose colliders contain the provided point.
  ///
  /// This walks the live collider set instead of Rapier's query pipeline
  /// because gameplay queries are expected to work immediately after collider
  /// creation. Before the world steps, broad-phase acceleration structures are
  /// not guaranteed to be synchronized with newly attached colliders.
  ///
  /// # Arguments
  /// - `point`: The world-space point to test.
  ///
  /// # Returns
  /// Returns backend rigid body slot pairs for each matching collider.
  pub fn query_point_2d(&self, point: [f32; 2]) -> Vec<(u32, u32)> {
    if validate_position(point[0], point[1]).is_err() {
      return Vec::new();
    }

    let point = Vector::new(point[0], point[1]);
    let mut body_slots = Vec::new();

    for (collider_handle, collider) in self.colliders.iter() {
      if !collider.shape().contains_point(collider.position(), point) {
        continue;
      }

      let Some(body_slot) =
        self.query_hit_to_parent_body_slot_2d(collider_handle)
      else {
        continue;
      };

      body_slots.push(body_slot);
    }

    return body_slots;
  }

  /// Returns all rigid bodies whose colliders overlap the provided AABB.
  ///
  /// This performs exact shape-vs-shape tests over the live collider set for
  /// the same reason as `query_point_2d()`: overlap queries need to be correct
  /// before the first simulation step, when broad-phase data may still be
  /// stale. Using exact tests here also avoids broad-phase false positives in
  /// the backend result.
  ///
  /// # Arguments
  /// - `min`: The minimum world-space corner of the query box.
  /// - `max`: The maximum world-space corner of the query box.
  ///
  /// # Returns
  /// Returns backend rigid body slot pairs for each matching collider.
  pub fn query_aabb_2d(&self, min: [f32; 2], max: [f32; 2]) -> Vec<(u32, u32)> {
    if validate_position(min[0], min[1]).is_err()
      || validate_position(max[0], max[1]).is_err()
    {
      return Vec::new();
    }

    let half_extents =
      Vector::new((max[0] - min[0]) * 0.5, (max[1] - min[1]) * 0.5);
    let center = Vector::new((min[0] + max[0]) * 0.5, (min[1] + max[1]) * 0.5);
    let query_shape = Cuboid::new(half_extents);
    let query_pose = Pose::from_translation(center);
    let query_dispatcher = self.narrow_phase.query_dispatcher();
    let mut body_slots = Vec::new();

    for (collider_handle, collider) in self.colliders.iter() {
      // Express the query box in the collider's local frame because Parry's
      // exact intersection test compares one shape pose relative to the other.
      let shape_to_collider = query_pose.inv_mul(collider.position());
      let intersects = query_dispatcher.intersection_test(
        &shape_to_collider,
        &query_shape,
        collider.shape(),
      );

      if intersects != Ok(true) {
        continue;
      }

      let Some(body_slot) =
        self.query_hit_to_parent_body_slot_2d(collider_handle)
      else {
        continue;
      };

      body_slots.push(body_slot);
    }

    return body_slots;
  }

  /// Returns the nearest rigid body hit by the provided finite ray segment.
  ///
  /// This iterates the live collider set directly instead of using Rapier's
  /// broad-phase query pipeline because raycasts are expected to see colliders
  /// that were just created or attached earlier in the frame. Keeping queries
  /// on the live collider set makes the result match gameplay expectations even
  /// before the world has advanced.
  ///
  /// # Arguments
  /// - `origin`: The world-space ray origin.
  /// - `dir`: The world-space ray direction.
  /// - `max_dist`: The maximum query distance in meters.
  ///
  /// # Returns
  /// Returns the nearest hit data when any live collider intersects the ray.
  pub fn raycast_2d(
    &self,
    origin: [f32; 2],
    dir: [f32; 2],
    max_dist: f32,
  ) -> Option<RaycastHit2DBackend> {
    if validate_position(origin[0], origin[1]).is_err()
      || validate_velocity(dir[0], dir[1]).is_err()
      || !max_dist.is_finite()
      || max_dist <= 0.0
    {
      return None;
    }

    let normalized_dir = normalize_query_vector_2d(dir)?;
    let ray = Ray::new(
      Vector::new(origin[0], origin[1]),
      Vector::new(normalized_dir[0], normalized_dir[1]),
    );
    let mut nearest_hit = None;

    for (collider_handle, collider) in self.colliders.iter() {
      // Resolve the public body handle data up front so the final hit payload
      // stays backend-agnostic and does not expose Rapier collider handles.
      let Some(body_slot) =
        self.query_hit_to_parent_body_slot_2d(collider_handle)
      else {
        continue;
      };

      let Some(hit) =
        cast_live_collider_raycast_hit_2d(collider, &ray, max_dist)
      else {
        continue;
      };
      let hit_point = ray.point_at(hit.time_of_impact);
      let candidate = RaycastHit2DBackend {
        body_slot_index: body_slot.0,
        body_slot_generation: body_slot.1,
        point: [hit_point.x, hit_point.y],
        normal: [hit.normal.x, hit.normal.y],
        distance: hit.time_of_impact,
      };

      // The public API only returns the nearest hit, so keep the first minimum
      // distance we observe while scanning the live collider set.
      if nearest_hit
        .as_ref()
        .is_some_and(|nearest: &RaycastHit2DBackend| {
          candidate.distance >= nearest.distance
        })
      {
        continue;
      }

      nearest_hit = Some(candidate);
    }

    return nearest_hit;
  }
}
