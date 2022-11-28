//! Matrix math types and functions.

use lambda_platform::rand::get_uniformly_random_floats_between;

// ------------------------------ MATRIX ---------------------------------------

pub struct Matrix<const COLUMNS: usize, const ROWS: usize, ValueType> {
  rows: usize,
  columns: usize,
  data: [[ValueType; COLUMNS]; ROWS],
}

impl<const COLUMNS: usize, const ROWS: usize, ValueType>
  Matrix<COLUMNS, ROWS, ValueType>
{
  pub fn new(data: [[ValueType; COLUMNS]; ROWS]) -> Self {
    Self {
      columns: COLUMNS,
      rows: ROWS,
      data,
    }
  }
}

pub type Matrix2x2f = Matrix<2, 2, f32>;
pub type Matrix3x3f = Matrix<3, 3, f32>;
pub type Matrix4x4f = Matrix<4, 4, f32>;

/// Common Matrix operations that can be implemented by any matrix like type.
pub trait MatrixProperties {
  fn is_square(&self) -> bool;
  fn rows(&self) -> usize;
  fn columns(&self) -> usize;
}

/// Common Initializers for Matrix
pub trait MatrixInitializers<ValueType> {
  fn identity() -> Self;
  fn zeroed() -> Self;
  fn random(start: ValueType, stop: ValueType) -> Self;
}

/// Common Matrix operations that can be implemented by any matrix like type so
/// long as it implements the `MatrixProperties` trait.
pub trait MatrixOperations<
  OtherMatrix: MatrixProperties,
  ResultingMatrix: MatrixProperties,
>
{
  fn multiply(&self, other: &OtherMatrix) -> ResultingMatrix;
}

impl<const COLUMNS: usize, const ROWS: usize, ValueType> MatrixProperties
  for Matrix<COLUMNS, ROWS, ValueType>
{
  fn is_square(&self) -> bool {
    return self.rows == self.columns;
  }

  fn rows(&self) -> usize {
    return self.rows;
  }

  fn columns(&self) -> usize {
    return self.columns;
  }
}

impl MatrixInitializers<f32> for Matrix4x4f {
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

  fn random(start: f32, stop: f32) -> Self {
    let random_floats = get_uniformly_random_floats_between(start, stop, 16);

    // TODO(vmarcella): Use an iterator over the returned vector to build the
    // matrix as opposed to these accesses. This will currently check every
    // array index for safety which incurs a performance penalty.
    return Matrix4x4f::new([
      [
        random_floats[0],
        random_floats[1],
        random_floats[2],
        random_floats[3],
      ],
      [
        random_floats[4],
        random_floats[5],
        random_floats[6],
        random_floats[7],
      ],
      [
        random_floats[8],
        random_floats[9],
        random_floats[10],
        random_floats[11],
      ],
      [
        random_floats[12],
        random_floats[13],
        random_floats[14],
        random_floats[15],
      ],
    ]);
  }
}

impl MatrixOperations<Matrix4x4f, Matrix4x4f> for Matrix4x4f {
  fn multiply(&self, other: &Matrix4x4f) -> Matrix4x4f {
    let mut result = Matrix4x4f::zeroed();
    return result;
  }
}

#[cfg(test)]
mod tests {
  use super::{
    Matrix4x4f,
    MatrixInitializers,
  };

  #[test]
  fn test_matrix4x4f_identity() {
    let identity = Matrix4x4f::identity();
    assert_eq!(identity.data[0][0], 1.0);
    assert_eq!(identity.data[0][1], 0.0);
    assert_eq!(identity.data[0][2], 0.0);
    assert_eq!(identity.data[0][3], 0.0);
    assert_eq!(identity.data[1][0], 0.0);
    assert_eq!(identity.data[1][1], 1.0);
    assert_eq!(identity.data[1][2], 0.0);
    assert_eq!(identity.data[1][3], 0.0);
    assert_eq!(identity.data[2][0], 0.0);
    assert_eq!(identity.data[2][1], 0.0);
    assert_eq!(identity.data[2][2], 1.0);
    assert_eq!(identity.data[2][3], 0.0);
    assert_eq!(identity.data[3][0], 0.0);
    assert_eq!(identity.data[3][1], 0.0);
    assert_eq!(identity.data[3][2], 0.0);
    assert_eq!(identity.data[3][3], 1.0);
  }

  #[test]
  fn test_matrix4x4f_zeroed() {
    let zeroed = Matrix4x4f::zeroed();

    for row in 0..4 {
      for column in 0..4 {
        assert_eq!(zeroed.data[row][column], 0.0);
      }
    }
  }

  #[test]
  fn test_matrix4x4f_random() {
    let random = Matrix4x4f::random(0.0, 1.0);

    for row in 0..4 {
      for column in 0..4 {
        assert!(random.data[row][column] >= 0.0);
        assert!(random.data[row][column] <= 1.0);
      }
    }
  }
}
