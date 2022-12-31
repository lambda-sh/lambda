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
      for (j, b) in a.as_ref().iter().enumerate() {
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
}

#[cfg(test)]
mod tests {

  use super::Matrix;

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
}
