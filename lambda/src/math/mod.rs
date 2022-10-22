//! Very simple math types built strictly off of Rust primitive types.

//  ------------------------------- VECTOR2 -----------------------------------
pub type Vector2 = (f32, f32);

/// Compute the dot product of two `Vec2`.
pub fn dot(a: Vector2, b: Vector2) -> f32 {
  a.0 * b.0 + a.1 * b.1
}

/// Compute the length of a vector.
fn length(a: Vector2) -> f32 {
  (a.0 * a.0 + a.1 * a.1).sqrt()
}

/// Normalized vector.
fn normalize(a: Vector2) -> Vector2 {
  let l = length(a);
  (a.0 / l, a.1 / l)
}

//  ------------------------------- VECTOR3 -----------------------------------

pub type Vector3 = (f32, f32, f32);

/// Compute the cross product of two `Vec3`.
pub fn cross(a: Vector3, b: Vector3) -> Vector3 {
  (
    a.1 * b.2 - a.2 * b.1,
    a.2 * b.0 - a.0 * b.2,
    a.0 * b.1 - a.1 * b.0,
  )
}

// ---------------------------------- TESTS -----------------------------------

#[cfg(test)]
mod tests {

  #[test]
  fn test_dot() {
    assert_eq!(super::dot((1.0, 2.0), (3.0, 4.0)), 11.0);
  }

  #[test]
  fn test_cross() {
    assert_eq!(
      super::cross((1.0, 2.0, 3.0), (4.0, 5.0, 6.0)),
      (-3.0, 6.0, -3.0)
    );
  }

  #[test]
  fn test_length() {
    assert_eq!(super::length((3.0, 4.0)), 5.0);
  }

  #[test]
  fn test_normalize() {
    assert_eq!(super::normalize((3.0, 4.0)), (0.6, 0.8));
  }
}
