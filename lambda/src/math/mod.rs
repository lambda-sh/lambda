//! Lambda Math Types and operations

pub mod matrix;
pub mod vector;

/// Convert a turn into radians.
fn turns_to_radians(turns: f32) -> f32 {
  return turns * std::f32::consts::PI * 2.0;
}

#[macro_export]
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
