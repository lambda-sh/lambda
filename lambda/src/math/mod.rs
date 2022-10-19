//! Very simple math types built strictly from primitive types.

pub type Vec2 = (f32, f32);

pub type Vec3 = (f32, f32, f32);

pub fn dot(a: Vec2, b: Vec2) -> f32 {
  a.0 * b.0 + a.1 * b.1
}

pub fn cross(a: Vec3, b: Vec3) -> Vec3 {
  (
    a.1 * b.2 - a.2 * b.1,
    a.2 * b.0 - a.0 * b.2,
    a.0 * b.1 - a.1 * b.0,
  )
}

fn length(a: Vec2) -> f32 {
  (a.0 * a.0 + a.1 * a.1).sqrt()
}

fn normalize(a: Vec2) -> Vec2 {
  let l = length(a);
  (a.0 / l, a.1 / l)
}

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
