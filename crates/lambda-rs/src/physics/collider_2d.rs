//! 2D collider support.
//!
//! This module defines backend-agnostic collider shapes and builders for
//! attaching collision geometry to a `RigidBody2D`.
//!
//! Collision detection and response is implemented by the platform backend.

use std::{
  error::Error,
  fmt,
};

use lambda_platform::physics::Collider2DBackendError;

use super::{
  PhysicsWorld2D,
  RigidBody2D,
  RigidBody2DError,
};

/// Maximum supported vertices for `ColliderShape2D::ConvexPolygon`.
pub const MAX_CONVEX_POLYGON_VERTICES: usize = 64;

const DEFAULT_LOCAL_OFFSET_X: f32 = 0.0;
const DEFAULT_LOCAL_OFFSET_Y: f32 = 0.0;
const DEFAULT_LOCAL_ROTATION_RADIANS: f32 = 0.0;

const DEFAULT_DENSITY: f32 = 1.0;
const DEFAULT_FRICTION: f32 = 0.5;
const DEFAULT_RESTITUTION: f32 = 0.0;

/// An opaque handle to a collider stored in a `PhysicsWorld2D`.
///
/// This handle is world-scoped. All collider operations MUST validate that the
/// handle belongs to the provided world and that it references a live collider.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Collider2D {
  world_id: u64,
  slot_index: u32,
  slot_generation: u32,
}

/// Supported 2D collider shapes.
#[derive(Debug, Clone, PartialEq)]
pub enum ColliderShape2D {
  /// A circle centered at the collider origin.
  Circle { radius: f32 },
  /// An axis-aligned rectangle in collider local space.
  Rectangle { half_width: f32, half_height: f32 },
  /// A capsule aligned with the collider local Y axis.
  Capsule { half_height: f32, radius: f32 },
  /// An arbitrary convex polygon in collider local space.
  ConvexPolygon { vertices: Vec<[f32; 2]> },
}

/// Material parameters for a `Collider2D`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColliderMaterial2D {
  density: f32,
  friction: f32,
  restitution: f32,
}

impl ColliderMaterial2D {
  /// Creates a new collider material configuration.
  ///
  /// The returned value is validated when `Collider2DBuilder::build()` is
  /// called.
  ///
  /// # Arguments
  /// - `density`: The density in kg/m².
  /// - `friction`: The friction coefficient (unitless).
  /// - `restitution`: The restitution coefficient in `[0.0, 1.0]`.
  ///
  /// # Returns
  /// Returns a new `ColliderMaterial2D` value.
  pub fn new(density: f32, friction: f32, restitution: f32) -> Self {
    return Self {
      density,
      friction,
      restitution,
    };
  }

  /// Returns the density in kg/m².
  ///
  /// # Returns
  /// Returns the configured density.
  pub fn density(self) -> f32 {
    return self.density;
  }

  /// Returns the friction coefficient (unitless).
  ///
  /// # Returns
  /// Returns the configured friction coefficient.
  pub fn friction(self) -> f32 {
    return self.friction;
  }

  /// Returns the restitution coefficient in `[0.0, 1.0]`.
  ///
  /// # Returns
  /// Returns the configured restitution coefficient.
  pub fn restitution(self) -> f32 {
    return self.restitution;
  }
}

impl Default for ColliderMaterial2D {
  fn default() -> Self {
    return Self {
      density: DEFAULT_DENSITY,
      friction: DEFAULT_FRICTION,
      restitution: DEFAULT_RESTITUTION,
    };
  }
}

/// Builder for `Collider2D`.
#[derive(Debug, Clone)]
pub struct Collider2DBuilder {
  shape: ColliderShape2D,
  local_offset: [f32; 2],
  local_rotation: f32,
  material: ColliderMaterial2D,
}

impl Collider2DBuilder {
  /// Creates a circle collider builder with stable defaults.
  ///
  /// Defaults
  /// - Local offset: `(0.0, 0.0)`
  /// - Local rotation: `0.0`
  /// - Density: `1.0`
  /// - Friction: `0.5`
  /// - Restitution: `0.0`
  ///
  /// # Arguments
  /// - `radius`: The circle radius in meters.
  ///
  /// # Returns
  /// Returns a new builder instance.
  pub fn circle(radius: f32) -> Self {
    return Self::new(ColliderShape2D::Circle { radius });
  }

  /// Creates a rectangle collider builder with stable defaults.
  ///
  /// # Arguments
  /// - `half_width`: The half-width in meters.
  /// - `half_height`: The half-height in meters.
  ///
  /// # Returns
  /// Returns a new builder instance.
  pub fn rectangle(half_width: f32, half_height: f32) -> Self {
    return Self::new(ColliderShape2D::Rectangle {
      half_width,
      half_height,
    });
  }

  /// Creates a capsule collider builder with stable defaults.
  ///
  /// # Arguments
  /// - `half_height`: The half-height of the capsule segment in meters.
  /// - `radius`: The capsule radius in meters.
  ///
  /// # Returns
  /// Returns a new builder instance.
  pub fn capsule(half_height: f32, radius: f32) -> Self {
    return Self::new(ColliderShape2D::Capsule {
      half_height,
      radius,
    });
  }

  /// Creates a convex polygon collider builder with stable defaults.
  ///
  /// # Arguments
  /// - `vertices`: The polygon vertices in meters, in collider local space.
  ///
  /// # Returns
  /// Returns a new builder instance.
  pub fn polygon(vertices: Vec<[f32; 2]>) -> Self {
    return Self::new(ColliderShape2D::ConvexPolygon { vertices });
  }

  /// Creates a convex polygon collider builder by copying from a slice.
  ///
  /// # Arguments
  /// - `vertices`: The polygon vertices in meters, in collider local space.
  ///
  /// # Returns
  /// Returns a new builder instance.
  pub fn polygon_from_slice(vertices: &[[f32; 2]]) -> Self {
    return Self::polygon(vertices.to_vec());
  }

  /// Sets the collider local offset relative to the owning body origin.
  ///
  /// # Arguments
  /// - `x`: The local translation X component in meters.
  /// - `y`: The local translation Y component in meters.
  ///
  /// # Returns
  /// Returns the updated builder.
  pub fn with_offset(mut self, x: f32, y: f32) -> Self {
    self.local_offset = [x, y];
    return self;
  }

  /// Sets the collider local rotation relative to the owning body.
  ///
  /// # Arguments
  /// - `radians`: The local rotation in radians.
  ///
  /// # Returns
  /// Returns the updated builder.
  pub fn with_local_rotation(mut self, radians: f32) -> Self {
    self.local_rotation = radians;
    return self;
  }

  /// Sets the collider material parameters.
  ///
  /// # Arguments
  /// - `material`: The collider material configuration.
  ///
  /// # Returns
  /// Returns the updated builder.
  pub fn with_material(mut self, material: ColliderMaterial2D) -> Self {
    self.material = material;
    return self;
  }

  /// Sets the collider density, in kg/m².
  ///
  /// When attaching a collider with `density > 0.0` to a dynamic body that did
  /// not explicitly configure mass via `RigidBody2DBuilder::with_dynamic_mass_kg`,
  /// the body's mass MAY change as mass properties are recomputed from attached
  /// colliders.
  ///
  /// When the owning body explicitly configures mass, collider density MUST
  /// NOT affect body mass properties.
  ///
  /// # Arguments
  /// - `density`: The density in kg/m².
  ///
  /// # Returns
  /// Returns the updated builder.
  pub fn with_density(mut self, density: f32) -> Self {
    self.material = ColliderMaterial2D::new(
      density,
      self.material.friction,
      self.material.restitution,
    );
    return self;
  }

  /// Sets the friction coefficient (unitless, `>= 0.0`).
  ///
  /// # Arguments
  /// - `friction`: The friction coefficient (unitless).
  ///
  /// # Returns
  /// Returns the updated builder.
  pub fn with_friction(mut self, friction: f32) -> Self {
    self.material = ColliderMaterial2D::new(
      self.material.density,
      friction,
      self.material.restitution,
    );
    return self;
  }

  /// Sets the restitution coefficient in `[0.0, 1.0]`.
  ///
  /// # Arguments
  /// - `restitution`: The restitution coefficient in `[0.0, 1.0]`.
  ///
  /// # Returns
  /// Returns the updated builder.
  pub fn with_restitution(mut self, restitution: f32) -> Self {
    self.material = ColliderMaterial2D::new(
      self.material.density,
      self.material.friction,
      restitution,
    );
    return self;
  }

  /// Attaches the collider to a body and returns a world-scoped handle.
  ///
  /// Attaching a collider to a dynamic body that does not explicitly configure
  /// mass MAY recompute the body's mass properties based on collider density.
  ///
  /// # Arguments
  /// - `world`: The physics world that owns the body.
  /// - `body`: The rigid body handle to attach the collider to.
  ///
  /// # Returns
  /// Returns a `Collider2D` handle on success.
  ///
  /// # Errors
  /// Returns `Collider2DError` if any configuration value is invalid or the
  /// `body` handle is invalid for the provided world.
  pub fn build(
    mut self,
    world: &mut PhysicsWorld2D,
    body: RigidBody2D,
  ) -> Result<Collider2D, Collider2DError> {
    validate_local_offset(self.local_offset[0], self.local_offset[1])?;
    validate_local_rotation(self.local_rotation)?;
    validate_material(self.material)?;
    validate_and_normalize_shape(&mut self.shape)?;

    validate_body_handle(world, body)?;

    let (body_slot_index, body_slot_generation) = body.backend_slot();

    let (slot_index, slot_generation) = match self.shape {
      ColliderShape2D::Circle { radius } => world
        .backend
        .create_circle_collider_2d(
          body_slot_index,
          body_slot_generation,
          radius,
          self.local_offset,
          self.local_rotation,
          self.material.density(),
          self.material.friction(),
          self.material.restitution(),
        )
        .map_err(map_backend_error)?,
      ColliderShape2D::Rectangle {
        half_width,
        half_height,
      } => world
        .backend
        .create_rectangle_collider_2d(
          body_slot_index,
          body_slot_generation,
          half_width,
          half_height,
          self.local_offset,
          self.local_rotation,
          self.material.density(),
          self.material.friction(),
          self.material.restitution(),
        )
        .map_err(map_backend_error)?,
      ColliderShape2D::Capsule {
        half_height,
        radius,
      } => world
        .backend
        .create_capsule_collider_2d(
          body_slot_index,
          body_slot_generation,
          half_height,
          radius,
          self.local_offset,
          self.local_rotation,
          self.material.density(),
          self.material.friction(),
          self.material.restitution(),
        )
        .map_err(map_backend_error)?,
      ColliderShape2D::ConvexPolygon { vertices } => world
        .backend
        .create_convex_polygon_collider_2d(
          body_slot_index,
          body_slot_generation,
          vertices,
          self.local_offset,
          self.local_rotation,
          self.material.density(),
          self.material.friction(),
          self.material.restitution(),
        )
        .map_err(map_backend_error)?,
    };

    return Ok(Collider2D {
      world_id: world.world_id,
      slot_index,
      slot_generation,
    });
  }

  /// Creates a builder for the given shape with stable defaults.
  fn new(shape: ColliderShape2D) -> Self {
    return Self {
      shape,
      local_offset: [DEFAULT_LOCAL_OFFSET_X, DEFAULT_LOCAL_OFFSET_Y],
      local_rotation: DEFAULT_LOCAL_ROTATION_RADIANS,
      material: ColliderMaterial2D::default(),
    };
  }
}

/// Errors for collider construction and operations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Collider2DError {
  /// The rigid body handle encoding is invalid.
  InvalidBodyHandle,
  /// The rigid body handle does not belong to the provided world.
  WorldMismatch,
  /// The referenced rigid body was not found.
  BodyNotFound,
  /// The collider backend is not available for this build.
  BackendUnsupported,
  /// The provided collider local offset is invalid.
  InvalidLocalOffset { x: f32, y: f32 },
  /// The provided collider local rotation is invalid.
  InvalidLocalRotation { radians: f32 },
  /// The provided circle radius is invalid.
  InvalidCircleRadius { radius: f32 },
  /// The provided rectangle half-extents are invalid.
  InvalidRectangleHalfExtents { half_width: f32, half_height: f32 },
  /// The provided capsule half-height is invalid.
  InvalidCapsuleHalfHeight { half_height: f32 },
  /// The provided capsule radius is invalid.
  InvalidCapsuleRadius { radius: f32 },
  /// The provided polygon has too few vertices.
  InvalidPolygonTooFewVertices { vertex_count: usize },
  /// The provided polygon exceeds the maximum supported vertex count.
  InvalidPolygonTooManyVertices {
    vertex_count: usize,
    max_vertices: usize,
  },
  /// The provided polygon contains a non-finite vertex.
  InvalidPolygonVertex { index: usize, x: f32, y: f32 },
  /// The provided polygon has zero signed area.
  InvalidPolygonZeroArea,
  /// The provided polygon is not strictly convex.
  InvalidPolygonNonConvex,
  /// The provided density is invalid.
  InvalidDensity { density: f32 },
  /// The provided friction coefficient is invalid.
  InvalidFriction { friction: f32 },
  /// The provided restitution coefficient is invalid.
  InvalidRestitution { restitution: f32 },
}

impl fmt::Display for Collider2DError {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::InvalidBodyHandle => {
        return write!(formatter, "invalid rigid body handle");
      }
      Self::WorldMismatch => {
        return write!(formatter, "rigid body handle does not match the world");
      }
      Self::BodyNotFound => {
        return write!(formatter, "rigid body not found");
      }
      Self::BackendUnsupported => {
        return write!(formatter, "colliders are not supported by this build");
      }
      Self::InvalidLocalOffset { x, y } => {
        return write!(formatter, "invalid local_offset: ({x}, {y})");
      }
      Self::InvalidLocalRotation { radians } => {
        return write!(formatter, "invalid local_rotation: {radians}");
      }
      Self::InvalidCircleRadius { radius } => {
        return write!(formatter, "invalid circle radius: {radius}");
      }
      Self::InvalidRectangleHalfExtents {
        half_width,
        half_height,
      } => {
        return write!(
          formatter,
          "invalid rectangle half extents: ({half_width}, {half_height})"
        );
      }
      Self::InvalidCapsuleHalfHeight { half_height } => {
        return write!(formatter, "invalid capsule half_height: {half_height}");
      }
      Self::InvalidCapsuleRadius { radius } => {
        return write!(formatter, "invalid capsule radius: {radius}");
      }
      Self::InvalidPolygonTooFewVertices { vertex_count } => {
        return write!(
          formatter,
          "invalid polygon vertex_count (too few): {vertex_count}"
        );
      }
      Self::InvalidPolygonTooManyVertices {
        vertex_count,
        max_vertices,
      } => {
        return write!(
          formatter,
          "invalid polygon vertex_count (too many): {vertex_count} (max: \
{max_vertices})"
        );
      }
      Self::InvalidPolygonVertex { index, x, y } => {
        return write!(
          formatter,
          "invalid polygon vertex at index {index}: ({x}, {y})"
        );
      }
      Self::InvalidPolygonZeroArea => {
        return write!(formatter, "invalid polygon: zero area");
      }
      Self::InvalidPolygonNonConvex => {
        return write!(formatter, "invalid polygon: not strictly convex");
      }
      Self::InvalidDensity { density } => {
        return write!(formatter, "invalid density: {density}");
      }
      Self::InvalidFriction { friction } => {
        return write!(formatter, "invalid friction: {friction}");
      }
      Self::InvalidRestitution { restitution } => {
        return write!(formatter, "invalid restitution: {restitution}");
      }
    }
  }
}

impl Error for Collider2DError {}

/// Validates that the provided rigid body handle is usable in the world.
fn validate_body_handle(
  world: &PhysicsWorld2D,
  body: RigidBody2D,
) -> Result<(), Collider2DError> {
  let result = body.position(world).map(|_| ());
  return result.map_err(map_body_error);
}

/// Maps a `RigidBody2DError` into the collider error surface.
fn map_body_error(error: RigidBody2DError) -> Collider2DError {
  match error {
    RigidBody2DError::InvalidHandle => {
      return Collider2DError::InvalidBodyHandle;
    }
    RigidBody2DError::WorldMismatch => {
      return Collider2DError::WorldMismatch;
    }
    RigidBody2DError::BodyNotFound => {
      return Collider2DError::BodyNotFound;
    }
    other => {
      debug_assert!(
        matches!(
          other,
          RigidBody2DError::InvalidPosition { .. }
            | RigidBody2DError::InvalidRotation { .. }
            | RigidBody2DError::InvalidVelocity { .. }
            | RigidBody2DError::InvalidForce { .. }
            | RigidBody2DError::InvalidImpulse { .. }
            | RigidBody2DError::InvalidMassKg { .. }
            | RigidBody2DError::UnsupportedOperation { .. }
        ),
        "unexpected RigidBody2DError mapping: {other:?}"
      );
      return Collider2DError::BodyNotFound;
    }
  }
}

/// Maps a backend collider error into the public collider error surface.
fn map_backend_error(error: Collider2DBackendError) -> Collider2DError {
  match error {
    Collider2DBackendError::BodyNotFound => {
      return Collider2DError::BodyNotFound;
    }
    Collider2DBackendError::InvalidPolygonDegenerate => {
      return Collider2DError::InvalidPolygonZeroArea;
    }
  }
}

/// Validates that the provided local offset is finite.
fn validate_local_offset(x: f32, y: f32) -> Result<(), Collider2DError> {
  if !x.is_finite() || !y.is_finite() {
    return Err(Collider2DError::InvalidLocalOffset { x, y });
  }

  return Ok(());
}

/// Validates that the provided local rotation is finite.
fn validate_local_rotation(radians: f32) -> Result<(), Collider2DError> {
  if !radians.is_finite() {
    return Err(Collider2DError::InvalidLocalRotation { radians });
  }

  return Ok(());
}

/// Validates material parameters.
fn validate_material(
  material: ColliderMaterial2D,
) -> Result<(), Collider2DError> {
  if !material.density.is_finite() || material.density < 0.0 {
    return Err(Collider2DError::InvalidDensity {
      density: material.density,
    });
  }

  if !material.friction.is_finite() || material.friction < 0.0 {
    return Err(Collider2DError::InvalidFriction {
      friction: material.friction,
    });
  }

  if !material.restitution.is_finite()
    || material.restitution < 0.0
    || material.restitution > 1.0
  {
    return Err(Collider2DError::InvalidRestitution {
      restitution: material.restitution,
    });
  }

  return Ok(());
}

/// Validates a shape and normalizes polygon winding to counterclockwise.
fn validate_and_normalize_shape(
  shape: &mut ColliderShape2D,
) -> Result<(), Collider2DError> {
  match shape {
    ColliderShape2D::Circle { radius } => {
      if !radius.is_finite() || *radius <= 0.0 {
        return Err(Collider2DError::InvalidCircleRadius { radius: *radius });
      }

      return Ok(());
    }
    ColliderShape2D::Rectangle {
      half_width,
      half_height,
    } => {
      if !half_width.is_finite()
        || !half_height.is_finite()
        || *half_width <= 0.0
        || *half_height <= 0.0
      {
        return Err(Collider2DError::InvalidRectangleHalfExtents {
          half_width: *half_width,
          half_height: *half_height,
        });
      }

      return Ok(());
    }
    ColliderShape2D::Capsule {
      half_height,
      radius,
    } => {
      if !half_height.is_finite() || *half_height < 0.0 {
        return Err(Collider2DError::InvalidCapsuleHalfHeight {
          half_height: *half_height,
        });
      }

      if !radius.is_finite() || *radius <= 0.0 {
        return Err(Collider2DError::InvalidCapsuleRadius { radius: *radius });
      }

      return Ok(());
    }
    ColliderShape2D::ConvexPolygon { vertices } => {
      return validate_and_normalize_convex_polygon(vertices.as_mut_slice());
    }
  }
}

/// Validates that the polygon is strictly convex, non-degenerate, and finite.
fn validate_and_normalize_convex_polygon(
  vertices: &mut [[f32; 2]],
) -> Result<(), Collider2DError> {
  let vertex_count = vertices.len();

  if vertex_count < 3 {
    return Err(Collider2DError::InvalidPolygonTooFewVertices { vertex_count });
  }

  if vertex_count > MAX_CONVEX_POLYGON_VERTICES {
    return Err(Collider2DError::InvalidPolygonTooManyVertices {
      vertex_count,
      max_vertices: MAX_CONVEX_POLYGON_VERTICES,
    });
  }

  for (index, vertex) in vertices.iter().enumerate() {
    let x = vertex[0];
    let y = vertex[1];
    if !x.is_finite() || !y.is_finite() {
      return Err(Collider2DError::InvalidPolygonVertex { index, x, y });
    }
  }

  let area2 = polygon_signed_area_times_two(vertices);
  if area2 == 0.0 {
    return Err(Collider2DError::InvalidPolygonZeroArea);
  }

  if area2 < 0.0 {
    vertices.reverse();
  }

  validate_polygon_is_strictly_convex(vertices)?;

  return Ok(());
}

/// Computes twice the signed area of the polygon using the shoelace formula.
fn polygon_signed_area_times_two(vertices: &[[f32; 2]]) -> f32 {
  let mut area2 = 0.0;
  let vertex_count = vertices.len();

  for index in 0..vertex_count {
    let next_index = (index + 1) % vertex_count;
    let a = vertices[index];
    let b = vertices[next_index];
    area2 += (a[0] * b[1]) - (b[0] * a[1]);
  }

  return area2;
}

/// Validates that all internal angles have the same winding and are non-zero.
fn validate_polygon_is_strictly_convex(
  vertices: &[[f32; 2]],
) -> Result<(), Collider2DError> {
  let vertex_count = vertices.len();
  let mut cross_sign: f32 = 0.0;

  for index in 0..vertex_count {
    let a = vertices[index];
    let b = vertices[(index + 1) % vertex_count];
    let c = vertices[(index + 2) % vertex_count];

    let ab_x = b[0] - a[0];
    let ab_y = b[1] - a[1];
    let bc_x = c[0] - b[0];
    let bc_y = c[1] - b[1];

    let cross = (ab_x * bc_y) - (ab_y * bc_x);
    if cross == 0.0 {
      return Err(Collider2DError::InvalidPolygonNonConvex);
    }

    if cross_sign == 0.0 {
      cross_sign = cross.signum();
      continue;
    }

    if cross.signum() != cross_sign {
      return Err(Collider2DError::InvalidPolygonNonConvex);
    }
  }

  return Ok(());
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::physics::{
    PhysicsWorld2DBuilder,
    RigidBody2DBuilder,
    RigidBodyType,
  };

  /// Rejects non-positive circle radii during build validation.
  #[test]
  fn build_rejects_non_positive_circle_radius() {
    let mut world = PhysicsWorld2DBuilder::new().build().unwrap();
    let body = RigidBody2DBuilder::new(RigidBodyType::Static)
      .build(&mut world)
      .unwrap();

    let error = Collider2DBuilder::circle(0.0)
      .build(&mut world, body)
      .unwrap_err();

    assert_eq!(error, Collider2DError::InvalidCircleRadius { radius: 0.0 });

    return;
  }

  /// Rejects non-positive rectangle half-extents during build validation.
  #[test]
  fn build_rejects_non_positive_rectangle_half_extents() {
    let mut world = PhysicsWorld2DBuilder::new().build().unwrap();
    let body = RigidBody2DBuilder::new(RigidBodyType::Static)
      .build(&mut world)
      .unwrap();

    let error = Collider2DBuilder::rectangle(1.0, 0.0)
      .build(&mut world, body)
      .unwrap_err();

    assert_eq!(
      error,
      Collider2DError::InvalidRectangleHalfExtents {
        half_width: 1.0,
        half_height: 0.0,
      }
    );

    return;
  }

  /// Rejects invalid restitution values during build validation.
  #[test]
  fn build_rejects_invalid_restitution() {
    let mut world = PhysicsWorld2DBuilder::new().build().unwrap();
    let body = RigidBody2DBuilder::new(RigidBodyType::Static)
      .build(&mut world)
      .unwrap();

    let error = Collider2DBuilder::circle(1.0)
      .with_restitution(2.0)
      .build(&mut world, body)
      .unwrap_err();

    assert_eq!(
      error,
      Collider2DError::InvalidRestitution { restitution: 2.0 }
    );

    return;
  }

  /// Rejects non-finite local offsets during build validation.
  #[test]
  fn build_rejects_non_finite_local_offset() {
    let mut world = PhysicsWorld2DBuilder::new().build().unwrap();
    let body = RigidBody2DBuilder::new(RigidBodyType::Static)
      .build(&mut world)
      .unwrap();

    let error = Collider2DBuilder::circle(1.0)
      .with_offset(f32::NAN, 0.0)
      .build(&mut world, body)
      .unwrap_err();

    match error {
      Collider2DError::InvalidLocalOffset { x, y } => {
        assert!(x.is_nan());
        assert_eq!(y, 0.0);
      }
      other => panic!("unexpected error: {other:?}"),
    }

    return;
  }

  /// Rejects negative friction during build validation.
  #[test]
  fn build_rejects_negative_friction() {
    let mut world = PhysicsWorld2DBuilder::new().build().unwrap();
    let body = RigidBody2DBuilder::new(RigidBodyType::Static)
      .build(&mut world)
      .unwrap();

    let error = Collider2DBuilder::circle(1.0)
      .with_friction(-0.1)
      .build(&mut world, body)
      .unwrap_err();

    assert_eq!(error, Collider2DError::InvalidFriction { friction: -0.1 });

    return;
  }

  /// Accepts clockwise polygon winding by reversing the vertex order.
  #[test]
  fn build_accepts_clockwise_polygon_by_reversing_vertices() {
    let mut world = PhysicsWorld2DBuilder::new().build().unwrap();
    let body = RigidBody2DBuilder::new(RigidBodyType::Static)
      .build(&mut world)
      .unwrap();

    let vertices = vec![[0.0, 0.0], [0.0, 1.0], [1.0, 0.0]];
    assert!(Collider2DBuilder::polygon(vertices)
      .build(&mut world, body)
      .is_ok());

    return;
  }

  /// Rejects polygons that exceed the maximum supported vertex count.
  #[test]
  fn build_rejects_polygon_too_many_vertices() {
    let mut world = PhysicsWorld2DBuilder::new().build().unwrap();
    let body = RigidBody2DBuilder::new(RigidBodyType::Static)
      .build(&mut world)
      .unwrap();

    let vertices = vec![[0.0, 0.0]; MAX_CONVEX_POLYGON_VERTICES + 1];
    let error = Collider2DBuilder::polygon(vertices)
      .build(&mut world, body)
      .unwrap_err();

    assert_eq!(
      error,
      Collider2DError::InvalidPolygonTooManyVertices {
        vertex_count: MAX_CONVEX_POLYGON_VERTICES + 1,
        max_vertices: MAX_CONVEX_POLYGON_VERTICES,
      }
    );

    return;
  }

  /// Rejects non-convex polygons during build validation.
  #[test]
  fn build_rejects_non_convex_polygon() {
    let mut world = PhysicsWorld2DBuilder::new().build().unwrap();
    let body = RigidBody2DBuilder::new(RigidBodyType::Static)
      .build(&mut world)
      .unwrap();

    let vertices =
      vec![[0.0, 0.0], [2.0, 0.0], [1.0, 0.5], [2.0, 1.0], [0.0, 1.0]];

    let error = Collider2DBuilder::polygon(vertices)
      .build(&mut world, body)
      .unwrap_err();

    assert_eq!(error, Collider2DError::InvalidPolygonNonConvex);

    return;
  }

  /// Rejects using a rigid body from a different world.
  #[test]
  fn build_rejects_cross_world_body_handle() {
    let mut world_a = PhysicsWorld2DBuilder::new().build().unwrap();
    let mut world_b = PhysicsWorld2DBuilder::new().build().unwrap();

    let body = RigidBody2DBuilder::new(RigidBodyType::Static)
      .build(&mut world_a)
      .unwrap();

    let error = Collider2DBuilder::circle(1.0)
      .build(&mut world_b, body)
      .unwrap_err();

    assert_eq!(error, Collider2DError::WorldMismatch);

    return;
  }
}
