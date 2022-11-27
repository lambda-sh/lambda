//! Matrix math types and functions.

use rand::thread_rng;

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
pub trait MatrixProperties<ValueType> {
  fn identity() -> Self;
  fn is_square(&self) -> bool;
  fn rows(&self) -> usize;
  fn columns(&self) -> usize;
}

/// Common Initializers for Matrix
pub trait CommonMatrixInitializers {
  fn identity() -> Self;
  fn zeroed() -> Self;
  fn random() -> Self;
}

impl CommonMatrixInitializers for Matrix4x4f {
  fn identity() -> Self {
    return Matrix4x4f::new([
      [1.0, 0.0, 0.0, 0.0],
      [0.0, 1.0, 0.0, 0.0],
      [0.0, 0.0, 1.0, 0.0],
      [0.0, 0.0, 0.0, 1.0],
    ]);
  }

  fn zeroed() -> Self {
    return Matrix4x4f::new([[0.0; 4]; 4]);
  }

  fn random() -> Self {
    todo!();
  }
}

pub trait Operations<OtherMatrix, ResultingMatrix> {
  fn multiply(&self, other: &OtherMatrix) -> ResultingMatrix;
}

impl Operations<Matrix4x4f, Matrix4x4f> for Matrix4x4f {
  fn multiply(&self, other: &Matrix4x4f) -> Matrix4x4f {
    let mut result = Matrix4x4f::zeroed();
    return result;
  }
}

pub struct Matrix<const columns: usize, const rows: usize, ValueType> {
  rows: usize,
  columns: usize,
  data: [[ValueType; columns]; rows],
}

impl<const columns: usize, const rows: usize, ValueType>
  Matrix<columns, rows, ValueType>
{
  pub fn new(data: [[ValueType; columns]; rows]) -> Self {
    Self {
      rows,
      columns,
      data,
    }
  }
}

pub type Matrix2x2f = Matrix<2, 2, f32>;
pub type Matrix3x3f = Matrix<3, 3, f32>;
pub type Matrix4x4f = Matrix<4, 4, f32>;
