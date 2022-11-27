//! Matrix math types and functions.

use lambda_platform::rand::get_uniformly_random_floats_between;

// ------------------------------ MATRIX ---------------------------------------

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

/// Common Matrix operations that can be implemented by any matrix like type.
pub trait MatrixProperties {
  fn is_square(&self) -> bool;
  fn rows(&self) -> usize;
  fn columns(&self) -> usize;
}

/// Common Initializers for Matrix
pub trait MatrixInitializers {
  fn identity() -> Self;
  fn zeroed() -> Self;
  fn random() -> Self;
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

impl<const columns: usize, const rows: usize, ValueType> MatrixProperties
  for Matrix<columns, rows, ValueType>
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

impl MatrixInitializers for Matrix4x4f {
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
    let random_floats = get_uniformly_random_floats_between(0.0, 1.0, 16);

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
