//! Very simple math types built strictly off of Rust primitive types.

/// Generalized Vector operations that can be implemented by any vector like
/// type.
trait Vector<T> {
  fn add(&self, other: &Self) -> Self;
  fn subtract(&self, other: &Self) -> Self;
  fn multiply(&self, other: &Self) -> Self;
  fn dot(&self, other: &Self) -> T;
  fn cross(&self, other: &Self) -> Self;
  fn length(&self) -> T;
  fn normalize(&self) -> Self;
}

//  ------------------------------- VECTOR2 -----------------------------------

pub type Vector2 = (f32, f32);

impl Vector<f32> for Vector2 {
  fn add(&self, other: &Self) -> Self {
    return (self.0 + other.0, self.1 + other.1);
  }

  fn subtract(&self, other: &Self) -> Self {
    return (self.0 - other.0, self.1 - other.1);
  }

  fn multiply(&self, other: &Self) -> Self {
    return (self.0 * other.0, self.1 * other.1);
  }

  fn dot(&self, other: &Self) -> f32 {
    return self.0 * other.0 + self.1 * other.1;
  }

  fn cross(&self, other: &Self) -> Self {
    return (self.0 * other.1, self.1 * other.0);
  }

  fn length(&self) -> f32 {
    return (self.0 * self.0 + self.1 * self.1).sqrt();
  }

  fn normalize(&self) -> Self {
    let length = self.length();

    if length == 0.0 {
      return (0.0, 0.0);
    }

    return (self.0 / length, self.1 / length);
  }
}

impl Vector<f32> for Vector3 {
  fn add(&self, other: &Self) -> Self {
    todo!()
  }

  fn subtract(&self, other: &Self) -> Self {
    todo!()
  }

  fn multiply(&self, other: &Self) -> Self {
    todo!()
  }

  fn dot(&self, other: &Self) -> f64 {
    todo!()
  }

  fn cross(&self, other: &Self) -> Self {
    todo!()
  }

  fn length(&self) -> f64 {
    todo!()
  }

  fn normalize(&self) -> Self {
    todo!()
  }
}

/// Compute the dot product of two `Vec2`.
pub fn dot(a: Vector2, b: Vector2) -> f32 {
  a.0 * b.0 + a.1 * b.1
}

/// Compute the length of a vector.
pub fn length(a: Vector2) -> f32 {
  return (a.0 * a.0 + a.1 * a.1).sqrt();
}

/// Normalized vector.
pub fn normalize(a: Vector2) -> Vector2 {
  let length = length(a);

  if length == 0.0 {
    return a;
  }

  return (a.0 / length, a.1 / length);
}

//  ------------------------------- VECTOR3 -----------------------------------

pub type Vector3 = (f32, f32, f32);

/// Compute the cross product of two `Vec3`.
pub fn cross(a: Vector3, b: Vector3) -> Vector3 {
  return (
    a.1 * b.2 - a.2 * b.1,
    a.2 * b.0 - a.0 * b.2,
    a.0 * b.1 - a.1 * b.0,
  );
}

//  ------------------------------- VECTOR4 -----------------------------------

pub type Vector4 = (f32, f32, f32, f32);

//  ------------------------------- MATRIX4 -----------------------------------

pub type Matrix4<Vector> = (Vector, Vector, Vector, Vector);

pub fn translate(
  matrix: Matrix4<Vector3>,
  vector: Vector3,
) -> Matrix4<Vector3> {
  (
    (
      matrix.0 .0 + vector.0,
      matrix.0 .1 + vector.1,
      matrix.0 .2 + vector.2,
    ),
    (
      matrix.1 .0 + vector.0,
      matrix.1 .1 + vector.1,
      matrix.1 .2 + vector.2,
    ),
    (
      matrix.2 .0 + vector.0,
      matrix.2 .1 + vector.1,
      matrix.2 .2 + vector.2,
    ),
    (
      matrix.3 .0 + vector.0,
      matrix.3 .1 + vector.1,
      matrix.3 .2 + vector.2,
    ),
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
