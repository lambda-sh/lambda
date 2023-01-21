//! Lambda Math Types and operations

pub mod matrix;
pub mod vector;

pub enum Angle {
  Radians(f32),
  Degrees(f32),
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
