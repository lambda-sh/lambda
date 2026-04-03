//! Rapier-backed 2D physics backend.
//!
//! This module provides a minimal wrapper around `rapier2d` to support the
//! higher-level `lambda-rs` physics APIs without exposing vendor types outside
//! of the platform layer.

use std::{
  collections::{
    HashMap,
    HashSet,
  },
  error::Error,
  fmt,
};

use rapier2d::prelude::*;

mod colliders;
mod helpers;
mod queries;
mod rigid_bodies;
mod simulation;

#[cfg(test)]
mod tests;

/// The rigid body integration mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RigidBodyType2D {
  /// A body that does not move under simulation.
  Static,
  /// A body affected by gravity and forces.
  Dynamic,
  /// A body integrated only by user-provided motion.
  Kinematic,
}

/// Backend errors for 2D rigid body operations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RigidBody2DBackendError {
  /// The referenced rigid body was not found.
  BodyNotFound,
  /// The provided position is invalid.
  InvalidPosition { x: f32, y: f32 },
  /// The provided rotation is invalid.
  InvalidRotation { radians: f32 },
  /// The provided linear velocity is invalid.
  InvalidVelocity { x: f32, y: f32 },
  /// The provided force is invalid.
  InvalidForce { x: f32, y: f32 },
  /// The provided impulse is invalid.
  InvalidImpulse { x: f32, y: f32 },
  /// The provided dynamic mass is invalid.
  InvalidMassKg { mass_kg: f32 },
  /// The requested operation is unsupported for the body type.
  UnsupportedOperation { body_type: RigidBodyType2D },
}

impl fmt::Display for RigidBody2DBackendError {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::BodyNotFound => {
        return write!(formatter, "rigid body not found");
      }
      Self::InvalidPosition { x, y } => {
        return write!(formatter, "invalid position: ({x}, {y})");
      }
      Self::InvalidRotation { radians } => {
        return write!(formatter, "invalid rotation: {radians}");
      }
      Self::InvalidVelocity { x, y } => {
        return write!(formatter, "invalid velocity: ({x}, {y})");
      }
      Self::InvalidForce { x, y } => {
        return write!(formatter, "invalid force: ({x}, {y})");
      }
      Self::InvalidImpulse { x, y } => {
        return write!(formatter, "invalid impulse: ({x}, {y})");
      }
      Self::InvalidMassKg { mass_kg } => {
        return write!(formatter, "invalid mass_kg: {mass_kg}");
      }
      Self::UnsupportedOperation { body_type } => {
        return write!(
          formatter,
          "unsupported operation for body_type: {body_type:?}"
        );
      }
    }
  }
}

impl Error for RigidBody2DBackendError {}

/// Backend errors for 2D collider operations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Collider2DBackendError {
  /// The referenced rigid body was not found.
  BodyNotFound,
  /// The provided polygon could not be represented as a convex hull.
  InvalidPolygonDegenerate,
}

impl fmt::Display for Collider2DBackendError {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::BodyNotFound => {
        return write!(formatter, "rigid body not found");
      }
      Self::InvalidPolygonDegenerate => {
        return write!(formatter, "invalid polygon: degenerate");
      }
    }
  }
}

impl Error for Collider2DBackendError {}

/// Backend-agnostic data describing the nearest 2D raycast hit.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RaycastHit2DBackend {
  /// The hit rigid body's slot index.
  pub body_slot_index: u32,
  /// The hit rigid body's slot generation.
  pub body_slot_generation: u32,
  /// The world-space hit point.
  pub point: [f32; 2],
  /// The world-space unit hit normal.
  pub normal: [f32; 2],
  /// The non-negative hit distance in meters.
  pub distance: f32,
}

/// Indicates whether a backend collision pair started or ended contact.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollisionEventKind2DBackend {
  /// The body pair started touching during the current backend step.
  Started,
  /// The body pair stopped touching during the current backend step.
  Ended,
}

/// Backend-agnostic data describing one 2D collision event.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CollisionEvent2DBackend {
  /// The transition kind for the body pair.
  pub kind: CollisionEventKind2DBackend,
  /// The first rigid body's slot index.
  pub body_a_slot_index: u32,
  /// The first rigid body's slot generation.
  pub body_a_slot_generation: u32,
  /// The second rigid body's slot index.
  pub body_b_slot_index: u32,
  /// The second rigid body's slot generation.
  pub body_b_slot_generation: u32,
  /// The representative world-space contact point, when available.
  pub contact_point: Option<[f32; 2]>,
  /// The representative world-space contact normal, when available.
  pub normal: Option<[f32; 2]>,
  /// The representative penetration depth, when available.
  pub penetration: Option<f32>,
}

/// The fallback mass applied to dynamic bodies before density colliders exist.
const DYNAMIC_BODY_FALLBACK_MASS_KG: f32 = 1.0;

/// Stores per-body state that `lambda-rs` tracks alongside Rapier.
///
/// This slot exists because `lambda-rs` defines integration semantics that are
/// stricter than the vendor backend:
/// - Forces are accumulated and cleared explicitly by the public API.
/// - Impulses update velocity immediately.
///
/// # Invariants
/// - `rapier_handle` MUST reference a body in `PhysicsBackend2D::bodies`.
/// - `explicit_dynamic_mass_kg` MUST be `Some` only for dynamic bodies.
/// - `generation` MUST be non-zero and is used to validate handles.
#[derive(Debug, Clone, Copy)]
struct RigidBodySlot2D {
  /// The rigid body's integration mode.
  body_type: RigidBodyType2D,
  /// The handle to the Rapier rigid body stored in the `RigidBodySet`.
  rapier_handle: RigidBodyHandle,
  /// Accumulated forces applied by the public API, in Newtons.
  force_accumulator: [f32; 2],
  /// The explicitly configured body mass in kilograms, if set.
  ///
  /// When this value is `Some`, collider density MUST NOT affect body mass
  /// properties. The backend enforces this by creating attached colliders with
  /// zero density and using the configured value as the body's additional mass.
  explicit_dynamic_mass_kg: Option<f32>,
  /// Tracks whether the body has at least one positive-density collider.
  ///
  /// This flag supports the spec requirement that bodies with no positive
  /// density colliders default to `1.0` kg, while bodies with at least one
  /// positive-density collider compute mass from collider density alone.
  has_positive_density_colliders: bool,
  /// A monotonically increasing counter used to validate stale handles.
  generation: u32,
}

/// Stores per-collider state that `lambda-rs` tracks alongside Rapier.
///
/// # Invariants
/// - `rapier_handle` MUST reference a collider in `PhysicsBackend2D::colliders`.
/// - `generation` MUST be non-zero and is used to validate stale handles.
#[derive(Debug, Clone, Copy)]
struct ColliderSlot2D {
  /// The handle to the Rapier collider stored in the `ColliderSet`.
  rapier_handle: ColliderHandle,
  /// The parent rigid body slot index that owns this collider.
  parent_slot_index: u32,
  /// The parent rigid body slot generation that owns this collider.
  parent_slot_generation: u32,
  /// A monotonically increasing counter used to validate stale handles.
  generation: u32,
}

/// Describes how collider attachment should affect dynamic-body mass semantics.
///
/// This helper isolates `lambda-rs` mass rules from the Rapier attachment flow
/// so body creation and collider attachment share one backend policy source.
#[derive(Debug, Clone, Copy, PartialEq)]
struct ColliderAttachmentMassPlan2D {
  /// The density value that MUST be passed to the Rapier collider builder.
  rapier_density: f32,
  /// Whether attaching this collider transitions the body to density-driven
  /// mass computation.
  should_mark_has_positive_density_colliders: bool,
  /// Whether the initial fallback mass MUST be removed before insertion.
  should_remove_fallback_mass: bool,
}

/// A normalized body-pair key used for backend collision tracking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct BodyPairKey2D {
  /// The first body slot index.
  body_a_slot_index: u32,
  /// The first body slot generation.
  body_a_slot_generation: u32,
  /// The second body slot index.
  body_b_slot_index: u32,
  /// The second body slot generation.
  body_b_slot_generation: u32,
}

/// The representative contact selected for a body pair during one step.
#[derive(Debug, Clone, Copy, PartialEq)]
struct BodyPairContact2D {
  /// The representative world-space contact point.
  point: [f32; 2],
  /// The representative world-space normal from body A toward body B.
  normal: [f32; 2],
  /// The non-negative penetration depth.
  penetration: f32,
}

/// A 2D physics backend powered by `rapier2d`.
///
/// This type is an internal implementation detail used by `lambda-rs`.
pub struct PhysicsBackend2D {
  gravity: Vector,
  integration_parameters: IntegrationParameters,
  islands: IslandManager,
  broad_phase: BroadPhaseBvh,
  narrow_phase: NarrowPhase,
  bodies: RigidBodySet,
  colliders: ColliderSet,
  impulse_joints: ImpulseJointSet,
  multibody_joints: MultibodyJointSet,
  ccd_solver: CCDSolver,
  pipeline: PhysicsPipeline,
  rigid_body_slots_2d: Vec<RigidBodySlot2D>,
  collider_slots_2d: Vec<ColliderSlot2D>,
  collider_parent_slots_2d: HashMap<ColliderHandle, (u32, u32)>,
  active_body_pairs_2d: HashSet<BodyPairKey2D>,
  active_body_pair_order_2d: Vec<BodyPairKey2D>,
  queued_collision_events_2d: Vec<CollisionEvent2DBackend>,
}
