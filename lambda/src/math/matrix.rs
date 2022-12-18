//! Matrix math types and functions.

use lambda_platform::rand::get_uniformly_random_floats_between;

use super::vector::Vector;
pub trait Matrix {
  type Scalar: Copy;
  type Vector: Vector<Scalar = Self::Scalar>;

  fn transform(&self, vector: &Self::Vector) -> Self::Vector;
  fn determinant(&self) -> Self::Scalar;
}

impl Matrix for [[f32; 4]; 4] {
  type Scalar = f32;
  type Vector = [f32; 4];

  fn transform(&self, vector: &Self::Vector) -> Self::Vector {
    let mut result = [0.0; 4];
    for (i, row) in self.iter().enumerate() {
      for (j, value) in row.iter().enumerate() {
        result[i] += value * vector[j];
      }
    }
    return result;
  }

  fn determinant(&self) -> Self::Scalar {
    let mut result = 0.0;
    for (i, value) in self[0].iter().enumerate() {}
    return result;
  }
}
