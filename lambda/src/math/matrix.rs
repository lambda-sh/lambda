//! Matrix math types and functions.

use lambda_platform::rand::get_uniformly_random_floats_between;

use super::vector::Vector;
pub trait Matrix<V: Vector> {
  fn add(&self, other: &Self) -> Self;
  fn subtract(&self, other: &Self) -> Self;
  fn multiply(&self, other: &Self) -> Self;
  fn transpose(&self) -> Self;
  fn inverse(&self) -> Self;
  fn transform(&self, other: &V) -> V;
  fn determinant(&self) -> f32;
  fn size(&self) -> (usize, usize);
  fn row(&self, row: usize) -> &V;
  fn at(&self, row: usize, column: usize) -> V::Scalar;
}

/// Obtain the submatrix of the input matrix where the submatrix
pub fn submatrix<V: Vector<Scalar = f32>, M: Matrix<V>>(
  matrix: M,
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

/// Matrix implementations for arrays backed by vectors.
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
        result.as_mut()[i].as_mut()[j] += a.dot(&b);
      }
    }
    return result;
  }

  fn transpose(&self) -> Self {
    let mut result = Self::default();
    for (i, a) in self.as_ref().iter().enumerate() {
      for j in 0..a.as_ref().len() {
        result.as_mut()[i].as_mut()[j] = self.as_ref()[j].as_ref()[i];
      }
    }
    return result;
  }

  fn inverse(&self) -> Self {
    todo!()
  }

  fn transform(&self, other: &V) -> V {
    todo!()
  }

  /// Computes the determinant of any square matrix using the Laplace expansion.
  fn determinant(&self) -> f32 {
    let (width, height) =
      (self.as_ref()[0].as_ref().len(), self.as_ref().len());

    if width != height {
      panic!("Cannot compute determinant of non-square matrix");
    }

    return match height {
      1 => self.as_ref()[0].as_ref()[0],
      2 => {
        let a = self.as_ref()[0].as_ref()[0];
        let b = self.as_ref()[0].as_ref()[1];
        let c = self.as_ref()[1].as_ref()[0];
        let d = self.as_ref()[1].as_ref()[1];
        a * d - b * c
      }
      _ => {
        let mut result = 0.0;
        for i in 0..height {
          let mut submatrix: Vec<Vec<f32>> = Vec::with_capacity(height - 1);
          for j in 1..height {
            let mut row = Vec::new();
            for k in 0..height {
              if k != i {
                row.push(self.as_ref()[j].as_ref()[k]);
              }
            }
            submatrix.push(row);
          }
          result += self.as_ref()[0].as_ref()[i]
            * submatrix.determinant()
            * (-1.0 as f32).powi(i as i32);
        }
        result
      }
    };
  }

  /// Return the size as a (rows, columns).
  fn size(&self) -> (usize, usize) {
    return (self.as_ref().len(), self.as_ref()[0].as_ref().len());
  }

  /// Return a reference to the row.
  fn row(&self, row: usize) -> &V {
    return &self.as_ref()[row];
  }

  ///
  fn at(&self, row: usize, column: usize) -> <V as Vector>::Scalar {
    return self.as_ref()[row].as_ref()[column];
  }
}

#[cfg(test)]
mod tests {

  use super::{
    submatrix,
    Matrix,
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
    assert_eq!(m.determinant(), -14.0);

    let m2 = [[6.0, 1.0, 1.0], [4.0, -2.0, 5.0], [2.0, 8.0, 7.0]];
    assert_eq!(m2.determinant(), -306.0);
  }

  #[test]
  fn non_square_matrix_determinant() {
    let m = [[3.0, 8.0], [4.0, 6.0], [0.0, 1.0]];
    let result = std::panic::catch_unwind(|| m.determinant());
    assert_eq!(false, result.is_ok());
  }

  #[test]
  fn submatrix_on_matrix_array() {
    let matrix = [[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]];

    let expected_submatrix = vec![vec![2.0, 3.0], vec![8.0, 9.0]];
    let actual_submatrix = submatrix(matrix, 1, 0);

    assert_eq!(expected_submatrix, actual_submatrix);
  }
}
