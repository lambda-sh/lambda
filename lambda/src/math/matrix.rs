//! Matrix math types and functions.

use super::vector::{
  Vector,
  Vector4,
};

/// Common Matrix operations that can be implemented by any matrix like type.
pub trait Matrix<T, V: Vector<T>> {
  fn identity() -> Self;
  fn transform(&self, vector: &V) -> V;
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

  fn transform(&self, vector: &Vector4) -> Vector4 {
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
