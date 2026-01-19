//! Matrix math types and functions.

use super::{
  turns_to_radians,
  vector::Vector,
  MathError,
};

// -------------------------------- MATRIX -------------------------------------

/// Matrix trait which defines the basic operations that can be performed on a
/// matrix. Lambda currently implements this trait for f32 arrays of arrays
/// for any size.
pub trait Matrix<V: Vector> {
  fn add(&self, other: &Self) -> Self;
  fn subtract(&self, other: &Self) -> Self;
  fn multiply(&self, other: &Self) -> Self;
  fn transpose(&self) -> Self;
  fn inverse(&self) -> Self;
  fn transform(&self, other: &V) -> V;
  /// Compute the determinant of the matrix.
  ///
  /// Returns an error when the matrix is empty or not square.
  fn determinant(&self) -> Result<f32, MathError>;
  fn size(&self) -> (usize, usize);
  fn row(&self, row: usize) -> &V;
  fn at(&self, row: usize, column: usize) -> V::Scalar;
  fn update(&mut self, row: usize, column: usize, value: V::Scalar);
}

// -------------------------------- FUNCTIONS ----------------------------------

/// Obtain the submatrix of the input matrix starting from the given row &
/// column.
pub fn submatrix<V: Vector<Scalar = f32>, MatrixLike: Matrix<V>>(
  matrix: MatrixLike,
  row: usize,
  column: usize,
) -> Vec<Vec<V::Scalar>> {
  let mut submatrix = Vec::new();
  let (rows, columns) = matrix.size();

  for k in 0..rows {
    if k != row {
      let mut row = Vec::new();
      for l in 0..columns {
        if l != column {
          row.push(matrix.at(k, l));
        }
      }
      submatrix.push(row);
    }
  }
  return submatrix;
}

/// Creates a translation matrix with the given translation vector. The output vector
pub fn translation_matrix<
  InputVector: Vector<Scalar = f32>,
  ResultingVector: Vector<Scalar = f32>,
  OutputMatrix: Matrix<ResultingVector> + Default,
>(
  vector: InputVector,
) -> OutputMatrix {
  let mut result = OutputMatrix::default();
  let (rows, columns) = result.size();
  assert_eq!(
    rows - 1,
    vector.size(),
    "Vector must contain one less element than the vectors of the input matrix"
  );

  for i in 0..rows {
    for j in 0..columns {
      if i == j {
        result.update(i, j, 1.0);
      } else if j == columns - 1 {
        result.update(i, j, vector.at(i));
      } else {
        result.update(i, j, 0.0);
      }
    }
  }

  return result;
}

/// Rotates the input matrix by the given number of turns around the given axis.
/// The axis must be a unit vector and the turns must be in the range [0, 1).
/// The rotation is counter-clockwise when looking down the axis.
///
/// Returns an error when the matrix is not 4x4, or when `axis_to_rotate` is not
/// a unit axis vector (`[1,0,0]`, `[0,1,0]`, `[0,0,1]`). A zero axis (`[0,0,0]`)
/// is treated as "no rotation".
pub fn rotate_matrix<
  V: Vector<Scalar = f32>,
  MatrixLike: Matrix<V> + Default + Clone,
>(
  matrix_to_rotate: MatrixLike,
  axis_to_rotate: [f32; 3],
  angle_in_turns: f32,
) -> Result<MatrixLike, MathError> {
  let (rows, columns) = matrix_to_rotate.size();
  if rows != columns {
    return Err(MathError::NonSquareMatrix {
      rows,
      cols: columns,
    });
  }
  if rows != 4 {
    return Err(MathError::InvalidRotationMatrixSize {
      rows,
      cols: columns,
    });
  }

  let angle_in_radians = turns_to_radians(angle_in_turns);
  let cosine_of_angle = angle_in_radians.cos();
  let sin_of_angle = angle_in_radians.sin();

  let mut rotation_matrix = MatrixLike::default();
  let [x, y, z] = axis_to_rotate;

  let rotation = if axis_to_rotate == [0.0, 0.0, 0.0] {
    return Ok(matrix_to_rotate);
  } else if axis_to_rotate == [0.0, 0.0, 1.0] {
    // Rotate around z-axis
    [
      [cosine_of_angle, sin_of_angle, 0.0, 0.0],
      [-sin_of_angle, cosine_of_angle, 0.0, 0.0],
      [0.0, 0.0, 1.0, 0.0],
      [0.0, 0.0, 0.0, 1.0],
    ]
  } else if axis_to_rotate == [0.0, 1.0, 0.0] {
    // Rotate around y-axis
    [
      [cosine_of_angle, 0.0, -sin_of_angle, 0.0],
      [0.0, 1.0, 0.0, 0.0],
      [sin_of_angle, 0.0, cosine_of_angle, 0.0],
      [0.0, 0.0, 0.0, 1.0],
    ]
  } else if axis_to_rotate == [1.0, 0.0, 0.0] {
    // Rotate around x-axis
    [
      [1.0, 0.0, 0.0, 0.0],
      [0.0, cosine_of_angle, sin_of_angle, 0.0],
      [0.0, -sin_of_angle, cosine_of_angle, 0.0],
      [0.0, 0.0, 0.0, 1.0],
    ]
  } else {
    return Err(MathError::InvalidRotationAxis { axis: [x, y, z] });
  };

  for (i, row) in rotation.iter().enumerate().take(rows) {
    for (j, value) in row.iter().enumerate().take(columns) {
      rotation_matrix.update(i, j, *value);
    }
  }

  return Ok(matrix_to_rotate.multiply(&rotation_matrix));
}

/// Creates a 4x4 perspective matrix given the fov in turns (unit between
/// 0..2pi radians), aspect ratio, near clipping plane (also known as z_near),
/// and far clipping plane (also known as z_far). Enforces that the matrix being
/// created is square in both debug and release builds, but only enforces that
/// the output matrix is 4x4 in debug builds.
pub fn perspective_matrix<
  V: Vector<Scalar = f32>,
  MatrixLike: Matrix<V> + Default,
>(
  fov: V::Scalar,
  aspect_ratio: V::Scalar,
  near_clipping_plane: V::Scalar,
  far_clipping_plane: V::Scalar,
) -> MatrixLike {
  let mut result = MatrixLike::default();
  let (rows, columns) = result.size();
  assert_eq!(
    rows, columns,
    "Matrix must be square to be a perspective matrix"
  );
  debug_assert_eq!(rows, 4, "Matrix must be 4x4 to be a perspective matrix");
  let fov_in_radians = turns_to_radians(fov);
  let f = 1.0 / (fov_in_radians / 2.0).tan();
  let range = near_clipping_plane - far_clipping_plane;

  result.update(0, 0, f / aspect_ratio);
  result.update(1, 1, f);
  result.update(2, 2, (near_clipping_plane + far_clipping_plane) / range);
  result.update(2, 3, -1.0);
  result.update(
    3,
    2,
    (2.0 * near_clipping_plane * far_clipping_plane) / range,
  );

  return result;
}

/// Create a matrix of any size that is filled with zeros.
pub fn zeroed_matrix<
  V: Vector<Scalar = f32>,
  MatrixLike: Matrix<V> + Default,
>(
  rows: usize,
  columns: usize,
) -> MatrixLike {
  let mut result = MatrixLike::default();
  for i in 0..rows {
    for j in 0..columns {
      result.update(i, j, 0.0);
    }
  }
  return result;
}

/// Creates a new matrix with the given number of rows and columns, and fills it
/// with the given value.
pub fn filled_matrix<
  V: Vector<Scalar = f32>,
  MatrixLike: Matrix<V> + Default,
>(
  rows: usize,
  columns: usize,
  value: V::Scalar,
) -> MatrixLike {
  let mut result = MatrixLike::default();
  for i in 0..rows {
    for j in 0..columns {
      result.update(i, j, value);
    }
  }
  return result;
}

/// Creates an identity matrix of the given size.
pub fn identity_matrix<
  V: Vector<Scalar = f32>,
  MatrixLike: Matrix<V> + Default,
>(
  rows: usize,
  columns: usize,
) -> MatrixLike {
  assert_eq!(
    rows, columns,
    "Matrix must be square to be an identity matrix"
  );
  let mut result = MatrixLike::default();
  for i in 0..rows {
    for j in 0..columns {
      if i == j {
        result.update(i, j, 1.0);
      } else {
        result.update(i, j, 0.0);
      }
    }
  }
  return result;
}

// -------------------------- ARRAY IMPLEMENTATION -----------------------------

/// Matrix implementations for arrays of f32 arrays. Including the trait Matrix into
/// your code will allow you to use these function implementation for any array
/// of f32 arrays.
impl<Array, V> Matrix<V> for Array
where
  Array: AsMut<[V]> + AsRef<[V]> + Default,
  V: AsMut<[f32]> + AsRef<[f32]> + Vector<Scalar = f32> + Sized,
{
  fn add(&self, other: &Self) -> Self {
    let mut result = Self::default();
    for (i, (a, b)) in
      self.as_ref().iter().zip(other.as_ref().iter()).enumerate()
    {
      result.as_mut()[i] = a.add(b);
    }
    return result;
  }

  fn subtract(&self, other: &Self) -> Self {
    let mut result = Self::default();

    for (i, (a, b)) in
      self.as_ref().iter().zip(other.as_ref().iter()).enumerate()
    {
      result.as_mut()[i] = a.subtract(b);
    }
    return result;
  }

  fn multiply(&self, other: &Self) -> Self {
    let mut result = Self::default();

    // We transpose the other matrix to convert the columns into rows, allowing
    // us to compute the new values of each index using the dot product
    // function.
    let transposed = other.transpose();

    for (i, a) in self.as_ref().iter().enumerate() {
      for (j, b) in transposed.as_ref().iter().enumerate() {
        result.update(i, j, a.dot(b));
      }
    }
    return result;
  }

  /// Transposes the matrix, swapping the rows and columns.
  fn transpose(&self) -> Self {
    let mut result = Self::default();
    for (i, a) in self.as_ref().iter().enumerate() {
      for j in 0..a.as_ref().len() {
        result.update(i, j, self.at(j, i));
      }
    }
    return result;
  }

  fn inverse(&self) -> Self {
    todo!()
  }

  fn transform(&self, _other: &V) -> V {
    todo!()
  }

  /// Computes the determinant of any square matrix using Laplace expansion.
  fn determinant(&self) -> Result<f32, MathError> {
    let rows = self.as_ref().len();
    if rows == 0 {
      return Err(MathError::EmptyMatrix);
    }

    let cols = self.as_ref()[0].as_ref().len();
    if cols == 0 {
      return Err(MathError::EmptyMatrix);
    }

    if cols != rows {
      return Err(MathError::NonSquareMatrix { rows, cols });
    }

    return match rows {
      1 => Ok(self.as_ref()[0].as_ref()[0]),
      2 => {
        let a = self.at(0, 0);
        let b = self.at(0, 1);
        let c = self.at(1, 0);
        let d = self.at(1, 1);
        return Ok(a * d - b * c);
      }
      _ => {
        let mut result = 0.0;
        for i in 0..rows {
          let mut submatrix: Vec<Vec<f32>> = Vec::with_capacity(rows - 1);
          for j in 1..rows {
            let mut row = Vec::new();
            for k in 0..rows {
              if k != i {
                row.push(self.at(j, k));
              }
            }
            submatrix.push(row);
          }
          let sub_determinant = submatrix.determinant()?;
          result += self.at(0, i) * sub_determinant * (-1.0_f32).powi(i as i32);
        }
        return Ok(result);
      }
    };
  }

  /// Return the size as a (rows, columns).
  fn size(&self) -> (usize, usize) {
    return (self.as_ref().len(), self.row(0).size());
  }

  /// Return a reference to the row.
  fn row(&self, row: usize) -> &V {
    return &self.as_ref()[row];
  }

  fn at(&self, row: usize, column: usize) -> <V as Vector>::Scalar {
    return self.as_ref()[row].as_ref()[column];
  }

  fn update(&mut self, row: usize, column: usize, new_value: V::Scalar) {
    self.as_mut()[row].as_mut()[column] = new_value;
  }
}

// ---------------------------------- TESTS ------------------------------------

#[cfg(test)]
mod tests {

  use super::{
    filled_matrix,
    perspective_matrix,
    rotate_matrix,
    submatrix,
    Matrix,
  };
  use crate::math::{
    matrix::translation_matrix,
    turns_to_radians,
    MathError,
  };

  #[test]
  fn square_matrix_add() {
    let a = [[1.0, 2.0], [3.0, 4.0]];
    let b = [[5.0, 6.0], [7.0, 8.0]];
    let c = a.add(&b);
    assert_eq!(c, [[6.0, 8.0], [10.0, 12.0]]);
  }

  #[test]
  fn square_matrix_subtract() {
    let a = [[1.0, 2.0], [3.0, 4.0]];
    let b = [[5.0, 6.0], [7.0, 8.0]];
    let c = a.subtract(&b);
    assert_eq!(c, [[-4.0, -4.0], [-4.0, -4.0]]);
  }

  #[test]
  // Test square matrix multiplication.
  fn square_matrix_multiply() {
    let m1 = [[1.0, 2.0], [3.0, 4.0]];
    let m2 = [[2.0, 0.0], [1.0, 2.0]];

    let mut result = m1.multiply(&m2);
    assert_eq!(result, [[4.0, 4.0], [10.0, 8.0]]);

    result = m2.multiply(&m1);
    assert_eq!(result, [[2.0, 4.0], [7.0, 10.0]])
  }

  #[test]
  fn transpose_square_matrix() {
    let m = [[1.0, 2.0], [5.0, 6.0]];
    let t = m.transpose();
    assert_eq!(t, [[1.0, 5.0], [2.0, 6.0]]);
  }

  #[test]
  fn square_matrix_determinant() {
    let m = [[3.0, 8.0], [4.0, 6.0]];
    assert_eq!(m.determinant(), Ok(-14.0));

    let m2 = [[6.0, 1.0, 1.0], [4.0, -2.0, 5.0], [2.0, 8.0, 7.0]];
    assert_eq!(m2.determinant(), Ok(-306.0));
  }

  #[test]
  fn non_square_matrix_determinant() {
    let m = [[3.0, 8.0], [4.0, 6.0], [0.0, 1.0]];
    let result = m.determinant();
    assert_eq!(result, Err(MathError::NonSquareMatrix { rows: 3, cols: 2 }));
  }

  #[test]
  fn submatrix_for_matrix_array() {
    let matrix = [[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]];

    let expected_submatrix = vec![vec![2.0, 3.0], vec![8.0, 9.0]];
    let actual_submatrix = submatrix(matrix, 1, 0);

    assert_eq!(expected_submatrix, actual_submatrix);
  }

  #[test]
  fn translate_matrix() {
    let translation: [[f32; 3]; 3] = translation_matrix([56.0, 5.0]);
    assert_eq!(
      translation,
      [[1.0, 0.0, 56.0], [0.0, 1.0, 5.0], [0.0, 0.0, 1.0]]
    );

    let translation: [[f32; 4]; 4] = translation_matrix([10.0, 2.0, 3.0]);
    let expected = [
      [1.0, 0.0, 0.0, 10.0],
      [0.0, 1.0, 0.0, 2.0],
      [0.0, 0.0, 1.0, 3.0],
      [0.0, 0.0, 0.0, 1.0],
    ];
    assert_eq!(translation, expected);
  }

  #[test]
  fn perspective_matrix_test() {
    let perspective: [[f32; 4]; 4] =
      perspective_matrix(1.0 / 4.0, 1.0, 1.0, 0.0);

    // Compute the field of view values used by the perspective matrix by hand.
    let fov_radians = turns_to_radians(1.0 / 4.0);
    let f = 1.0 / (fov_radians / 2.0).tan();

    let expected: [[f32; 4]; 4] = [
      [f, 0.0, 0.0, 0.0],
      [0.0, f, 0.0, 0.0],
      [0.0, 0.0, 1.0, -1.0],
      [0.0, 0.0, 0.0, 0.0],
    ];

    assert_eq!(perspective, expected);
  }

  /// Test the rotation matrix for a 3D rotation.
  #[test]
  fn rotate_matrices() {
    // Test a zero turn rotation.
    let matrix: [[f32; 4]; 4] = filled_matrix(4, 4, 1.0);
    let rotated_matrix =
      rotate_matrix(matrix, [0.0, 0.0, 1.0], 0.0).expect("valid axis");
    assert_eq!(rotated_matrix, matrix);

    // Test a 90 degree rotation.
    let matrix = [
      [1.0, 2.0, 3.0, 4.0],
      [5.0, 6.0, 7.0, 8.0],
      [9.0, 10.0, 11.0, 12.0],
      [13.0, 14.0, 15.0, 16.0],
    ];
    let rotated =
      rotate_matrix(matrix, [0.0, 1.0, 0.0], 0.25).expect("valid axis");
    let expected = [
      [3.0, 1.9999999, -1.0000001, 4.0],
      [7.0, 5.9999995, -5.0000005, 8.0],
      [11.0, 9.999999, -9.000001, 12.0],
      [14.999999, 13.999999, -13.000001, 16.0],
    ];

    for i in 0..4 {
      for j in 0..4 {
        crate::assert_approximately_equal!(
          rotated.at(i, j),
          expected.at(i, j),
          0.1
        );
      }
    }
  }
}
