//! Matrix math types and functions.

use super::vector::{
  Vector,
  Vector3,
  Vector4,
};

pub enum Axes {
  X,
  Y,
  Z,
}

/// Common Matrix operations that can be implemented by any matrix like type.
pub trait MatrixOperations<T, V: Vector<T>> {
  fn identity() -> Self;
  fn transform(&self, vector: &V) -> V;
  fn rotate(&self, vector: &V, axis: Axes) -> V;
  fn scale(&self, scale: &V) -> Self;
  fn multiply(&self, other: &Self) -> Self;
}

pub type Matrix1x3f = Vector3;
pub type Matrix2x3f = (Vector3, Vector3);
pub type Matrix3x3f = (Vector3, Vector3, Vector3);
pub type Matrix4x3f = (Vector3, Vector3, Vector3, Vector3);

impl MatrixOperations<f32, Vector3> for Matrix4x3f {
  fn identity() -> Self {
    todo!()
  }

  fn transform(&self, vector: &Vector3) -> Vector3 {
    todo!();
  }

  fn rotate(&self, vector: &Vector3, axis: Axes) -> Vector3 {
    todo!()
  }

  fn scale(&self, scale: &Vector3) -> Self {
    todo!()
  }

  fn multiply(&self, other: &Self) -> Self {
    todo!()
  }
}

pub struct Matrix<const columns: usize, const rows: usize> {
  rows: usize,
  columns: usize,
  data: [[f32; columns]; rows],
}

impl<const columns: usize, const rows: usize> Matrix<columns, rows> {
  pub fn new(data: [[f32; columns]; rows]) -> Self {
    Self {
      rows,
      columns,
      data,
    }
  }
}

pub type Matrix1x4f = Vector4;
pub type Matrix2x4f = (Vector4, Vector4);
pub type Matrix3x4f = (Vector4, Vector4, Vector4);
pub type Matrix4x4f = (Vector4, Vector4, Vector4, Vector4);

impl MatrixOperations<f32, Vector4> for Matrix3x4f {
  fn identity() -> Self {
    todo!()
  }

  fn transform(&self, vector: &Vector4) -> Vector4 {
    todo!()
  }

  fn rotate(&self, vector: &Vector4, axis: Axes) -> Vector4 {
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
          + self.0 .2 * other.2 .0,
        self.0 .0 * other.0 .1
          + self.0 .1 * other.1 .1
          + self.0 .2 * other.2 .1,
        self.0 .0 * other.0 .2
          + self.0 .1 * other.1 .2
          + self.0 .2 * other.2 .2,
        self.0 .0 * other.0 .3
          + self.0 .1 * other.1 .3
          + self.0 .2 * other.2 .3,
      ),
      (
        self.1 .0 * other.0 .0
          + self.1 .1 * other.1 .0
          + self.1 .2 * other.2 .0,
        self.1 .0 * other.0 .1
          + self.1 .1 * other.1 .1
          + self.1 .2 * other.2 .1,
        self.1 .0 * other.0 .2
          + self.1 .1 * other.1 .2
          + self.1 .2 * other.2 .2,
        self.1 .0 * other.0 .3
          + self.1 .1 * other.1 .3
          + self.1 .2 * other.2 .3,
      ),
      (
        self.2 .0 * other.0 .0
          + self.2 .1 * other.1 .0
          + self.2 .2 * other.2 .0,
        self.2 .0 * other.0 .1
          + self.2 .1 * other.1 .1
          + self.2 .2 * other.2 .1,
        self.2 .0 * other.0 .2
          + self.2 .1 * other.1 .2
          + self.2 .2 * other.2 .2,
        self.2 .0 * other.0 .3
          + self.2 .1 * other.1 .3
          + self.2 .2 * other.2 .3,
      ),
    );
  }
}

impl MatrixOperations<f32, Vector4> for Matrix4x4f {
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

  fn rotate(&self, rotation: &Vector4, axis: Axes) -> Vector4 {
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
