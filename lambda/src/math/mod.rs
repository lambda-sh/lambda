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

//  ------------------------------- VECTOR3 -----------------------------------

pub type Vector3 = (f32, f32, f32);

impl Vector<f32> for Vector3 {
  fn add(&self, other: &Self) -> Self {
    return (self.0 + other.0, self.1 + other.1, self.2 + other.2);
  }

  fn subtract(&self, other: &Self) -> Self {
    return (self.0 - other.0, self.1 - other.1, self.2 - other.2);
  }

  fn multiply(&self, other: &Self) -> Self {
    return (self.0 * other.0, self.1 * other.1, self.2 * other.2);
  }

  fn dot(&self, other: &Self) -> f32 {
    return self.0 * other.0 + self.1 * other.1 + self.2 * other.2;
  }

  fn cross(&self, other: &Self) -> Self {
    return (
      self.1 * other.2 - self.2 * other.1,
      self.2 * other.0 - self.0 * other.2,
      self.0 * other.1 - self.1 * other.0,
    );
  }

  fn length(&self) -> f32 {
    return (self.0 * self.0 + self.1 * self.1 + self.2 * self.2).sqrt();
  }

  fn normalize(&self) -> Self {
    let length = self.length();

    if length == 0.0 {
      return (0.0, 0.0, 0.0);
    }

    return (self.0 / length, self.1 / length, self.2 / length);
  }
}

//  ------------------------------- VECTOR4 -----------------------------------

pub type Vector4 = (f32, f32, f32, f32);

impl Vector<f32> for Vector4 {
  fn add(&self, other: &Self) -> Self {
    return (
      self.0 + other.0,
      self.1 + other.1,
      self.2 + other.2,
      self.3 + other.3,
    );
  }

  fn subtract(&self, other: &Self) -> Self {
    return (
      self.0 - other.0,
      self.1 - other.1,
      self.2 - other.2,
      self.3 - other.3,
    );
  }

  fn multiply(&self, other: &Self) -> Self {
    return (
      self.0 * other.0,
      self.1 * other.1,
      self.2 * other.2,
      self.3 * other.3,
    );
  }

  fn dot(&self, other: &Self) -> f32 {
    return self.0 * other.0
      + self.1 * other.1
      + self.2 * other.2
      + self.3 * other.3;
  }

  fn cross(&self, other: &Self) -> Self {
    return (
      self.1 * other.2 - self.2 * other.1,
      self.2 * other.0 - self.0 * other.2,
      self.0 * other.1 - self.1 * other.0,
      0.0,
    );
  }

  fn length(&self) -> f32 {
    return (self.0 * self.0
      + self.1 * self.1
      + self.2 * self.2
      + self.3 * self.3)
      .sqrt();
  }

  fn normalize(&self) -> Self {
    let length = self.length();

    if length == 0.0 {
      return (0.0, 0.0, 0.0, 0.0);
    }

    return (
      self.0 / length,
      self.1 / length,
      self.2 / length,
      self.3 / length,
    );
  }
}

/// Common Matrix operations that can be implemented by any matrix like type.
trait Matrix<T, V: Vector<T>> {
  fn identity() -> Self;
  fn translate(&self, vector: &V) -> V;
  fn rotate(&self, vector: &V) -> V;
  fn scale(&self, scale: &V) -> Self;
  fn multiply(&self, other: &Self) -> Self;
}

//  ------------------------------- MATRIX4 -----------------------------------

pub type Matrix4<Vector> = (Vector, Vector, Vector, Vector);

impl Matrix<f32, Vector4> for Matrix4<Vector4> {
  fn identity() -> Self {
    return (
      (1.0, 0.0, 0.0, 0.0),
      (0.0, 1.0, 0.0, 0.0),
      (0.0, 0.0, 1.0, 0.0),
      (0.0, 0.0, 0.0, 1.0),
    );
  }

  fn translate(&self, vector: &Vector4) -> Vector4 {
    return (
      vector.0 * self.0 .0
        + vector.1 * self.1 .0
        + vector.2 * self.2 .0
        + vector.3 * self.3 .0,
      vector.0 * self.0 .1
        + vector.1 * self.1 .1
        + vector.2 * self.2 .1
        + vector.3 * self.3 .1,
      vector.0 * self.0 .2
        + vector.1 * self.1 .2
        + vector.2 * self.2 .2
        + vector.3 * self.3 .2,
      vector.0 * self.0 .3
        + vector.1 * self.1 .3
        + vector.2 * self.2 .3
        + vector.3 * self.3 .3,
    );
  }

  fn rotate(&self, rotation: &Vector4) -> Vector4 {
    todo!()
  }

  fn scale(&self, scale: &Vector4) -> Self {
    todo!()
  }

  fn multiply(&self, other: &Self) -> Self {
    return (
      (
        self.0 .0 * other.0 .0
          + self.0 .1 * other.1 .0
          + self.0 .2 * other.2 .0
          + self.0 .3 * other.3 .0,
        self.0 .0 * other.0 .1
          + self.0 .1 * other.1 .1
          + self.0 .2 * other.2 .1
          + self.0 .3 * other.3 .1,
        self.0 .0 * other.0 .2
          + self.0 .1 * other.1 .2
          + self.0 .2 * other.2 .2
          + self.0 .3 * other.3 .2,
        self.0 .0 * other.0 .3
          + self.0 .1 * other.1 .3
          + self.0 .2 * other.2 .3
          + self.0 .3 * other.3 .3,
      ),
      (
        self.1 .0 * other.0 .0
          + self.1 .1 * other.1 .0
          + self.1 .2 * other.2 .0
          + self.1 .3 * other.3 .0,
        self.1 .0 * other.0 .1
          + self.1 .1 * other.1 .1
          + self.1 .2 * other.2 .1
          + self.1 .3 * other.3 .1,
        self.1 .0 * other.0 .2
          + self.1 .1 * other.1 .2
          + self.1 .2 * other.2 .2
          + self.1 .3 * other.3 .2,
        self.1 .0 * other.0 .3
          + self.1 .1 * other.1 .3
          + self.1 .2 * other.2 .3
          + self.1 .3 * other.3 .3,
      ),
      (
        self.2 .0 * other.0 .0
          + self.2 .1 * other.1 .0
          + self.2 .2 * other.2 .0
          + self.2 .3 * other.3 .0,
        self.2 .0 * other.0 .1
          + self.2 .1 * other.1 .1
          + self.2 .2 * other.2 .1
          + self.2 .3 * other.3 .1,
        self.2 .0 * other.0 .2
          + self.2 .1 * other.1 .2
          + self.2 .2 * other.2 .2
          + self.2 .3 * other.3 .2,
        self.2 .0 * other.0 .3
          + self.2 .1 * other.1 .3
          + self.2 .2 * other.2 .3
          + self.2 .3 * other.3 .3,
      ),
      (
        self.3 .0 * other.0 .0
          + self.3 .1 * other.1 .0
          + self.3 .2 * other.2 .0
          + self.3 .3 * other.3 .0,
        self.3 .0 * other.0 .1
          + self.3 .1 * other.1 .1
          + self.3 .2 * other.2 .1
          + self.3 .3 * other.3 .1,
        self.3 .0 * other.0 .2
          + self.3 .1 * other.1 .2
          + self.3 .2 * other.2 .2
          + self.3 .3 * other.3 .2,
        self.3 .0 * other.0 .3
          + self.3 .1 * other.1 .3
          + self.3 .2 * other.2 .3
          + self.3 .3 * other.3 .3,
      ),
    );
  }
}

// ---------------------------------- TESTS -----------------------------------

#[cfg(test)]
mod vector2_tests {
  use super::Vector;

  #[test]
  fn test_dot() {
    assert_eq!((1.0, 2.0).dot(&(3.0, 4.0)), 11.0);
  }

  #[test]
  fn test_cross() {
    assert_eq!((1.0, 2.0, 3.0).cross(&(4.0, 5.0, 6.0)), (-3.0, 6.0, -3.0));
  }

  #[test]
  fn test_length() {
    assert_eq!((3.0, 4.0).length(), 5.0);
  }

  #[test]
  fn test_normalize() {
    assert_eq!((3.0, 4.0).normalize(), (0.6, 0.8));
  }
}

#[cfg(test)]
mod vector3_tests {
  use super::Vector;

  #[test]
  fn test_dot() {
    assert_eq!((1.0, 2.0, 3.0).dot(&(4.0, 5.0, 6.0)), 32.0);
  }

  #[test]
  fn test_cross() {
    assert_eq!((1.0, 2.0, 3.0).cross(&(4.0, 5.0, 6.0)), (-3.0, 6.0, -3.0));
  }

  #[test]
  fn test_length() {
    assert_eq!((1.0, 2.0, 2.0).length(), 3.0);
  }

  #[test]
  fn test_normalize() {
    assert_eq!(
      (1.0, 2.0, 2.0).normalize(),
      (0.33333334, 0.6666667, 0.6666667)
    );
  }
}
