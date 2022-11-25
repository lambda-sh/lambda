//! Vector math types and functions.

/// Generalized Vector operations that can be implemented by any vector like
/// type.
pub trait Vector<T> {
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