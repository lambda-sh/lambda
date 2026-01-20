//! Lambda Math Types and operations

pub mod error;
pub mod matrix;
pub mod vector;

pub use error::MathError;

/// Angle units used by conversion helpers and matrix transforms.
///
/// Prefer `Angle::Turns` for ergonomic quarter/half rotations when building
/// camera and model transforms. One full turn equals `2π` radians.
pub enum Angle {
  /// Angle expressed in radians.
  Radians(f32),
  /// Angle expressed in degrees.
  Degrees(f32),
  /// Angle expressed in turns, where `1.0` is a full revolution.
  Turns(f32),
}

/// Convert a turn into radians.
fn turns_to_radians(turns: f32) -> f32 {
  return turns * std::f32::consts::PI * 2.0;
}

#[macro_export]
/// Assert that two values are equal, with a given tolerance.
macro_rules! assert_approximately_equal {
  ($a:expr, $b:expr, $eps:expr) => {{
    let (a, b, eps) = ($a, $b, $eps);
    assert!(
      (a - b).abs() < eps,
      "{} is not approximately equal to {} with an epsilon of {}",
      a,
      b,
      eps
    );
  }};
}
