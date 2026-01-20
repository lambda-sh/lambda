//! Vector math types and functions.

use super::MathError;

/// Generalized Vector operations that can be implemented by any vector like
/// type.
pub trait Vector {
  type Scalar: Copy;
  fn add(&self, other: &Self) -> Self;
  fn subtract(&self, other: &Self) -> Self;
  fn scale(&self, scalar: Self::Scalar) -> Self;
  fn dot(&self, other: &Self) -> Self::Scalar;
  /// Cross product of two vectors.
  ///
  /// Returns an error when the vectors are not 3D, or when the vectors have
  /// mismatched dimensions.
  fn cross(&self, other: &Self) -> Result<Self, MathError>
  where
    Self: Sized;
  fn length(&self) -> Self::Scalar;
  /// Normalize the vector to unit length.
  ///
  /// Returns an error when the vector has zero length.
  fn normalize(&self) -> Result<Self, MathError>
  where
    Self: Sized;
  fn size(&self) -> usize;
  fn at(&self, index: usize) -> Self::Scalar;
  fn update(&mut self, index: usize, value: Self::Scalar);
}

impl<T> Vector for T
where
  T: AsMut<[f32]> + AsRef<[f32]> + Default,
{
  type Scalar = f32;

  /// Add two vectors of any size together.
  fn add(&self, other: &Self) -> Self {
    let mut result = Self::default();

    self
      .as_ref()
      .iter()
      .zip(other.as_ref().iter())
      .enumerate()
      .for_each(|(i, (a, b))| result.as_mut()[i] = a + b);

    return result;
  }

  /// Subtract two vectors of any size.
  fn subtract(&self, other: &Self) -> Self {
    let mut result = Self::default();

    self
      .as_ref()
      .iter()
      .zip(other.as_ref().iter())
      .enumerate()
      .for_each(|(i, (a, b))| result.as_mut()[i] = a - b);

    return result;
  }

  fn dot(&self, other: &Self) -> Self::Scalar {
    assert_eq!(
      self.as_ref().len(),
      other.as_ref().len(),
      "Vectors must be the same length"
    );

    let mut result = 0.0;
    for (a, b) in self.as_ref().iter().zip(other.as_ref().iter()) {
      result += a * b;
    }
    return result;
  }

  /// Cross product of two 3D vectors.
  ///
  /// Returns an error when either vector is not 3D, or when the vectors have
  /// mismatched dimensions.
  fn cross(&self, other: &Self) -> Result<Self, MathError> {
    let left_size = self.as_ref().len();
    let right_size = other.as_ref().len();
    if left_size != right_size {
      return Err(MathError::MismatchedVectorDimensions {
        left: left_size,
        right: right_size,
      });
    }

    let mut result = Self::default();
    let a = self.as_ref();
    let b = other.as_ref();

    if a.len() != 3 {
      return Err(MathError::CrossProductDimension { actual: a.len() });
    }

    result.as_mut()[0] = a[1] * b[2] - a[2] * b[1];
    result.as_mut()[1] = a[2] * b[0] - a[0] * b[2];
    result.as_mut()[2] = a[0] * b[1] - a[1] * b[0];
    return Ok(result);
  }

  fn length(&self) -> Self::Scalar {
    let mut result = 0.0;
    for a in self.as_ref().iter() {
      result += a * a;
    }
    result.sqrt()
  }

  fn normalize(&self) -> Result<Self, MathError> {
    let mut result = Self::default();
    let length = self.length();
    if length == 0.0 {
      return Err(MathError::ZeroLengthVector);
    }

    self.as_ref().iter().enumerate().for_each(|(i, a)| {
      result.as_mut()[i] = a / length;
    });

    return Ok(result);
  }

  fn scale(&self, scalar: Self::Scalar) -> Self {
    let mut result = Self::default();
    self.as_ref().iter().enumerate().for_each(|(i, a)| {
      result.as_mut()[i] = a * scalar;
    });

    return result;
  }

  fn size(&self) -> usize {
    return self.as_ref().len();
  }

  fn at(&self, index: usize) -> Self::Scalar {
    return self.as_ref()[index];
  }

  fn update(&mut self, index: usize, value: Self::Scalar) {
    self.as_mut()[index] = value;
  }
}

#[cfg(test)]
mod tests {
  use super::Vector;
  use crate::math::MathError;

  #[test]
  fn adding_vectors() {
    let a = [1.0, 2.0, 3.0];
    let b = [4.0, 5.0, 6.0];
    let c = [5.0, 7.0, 9.0];

    let result = a.add(&b);

    assert_eq!(result, c);
  }

  #[test]
  fn subtracting_vectors() {
    let a = [1.0, 2.0, 3.0];
    let b = [4.0, 5.0, 6.0];
    let c = [-3.0, -3.0, -3.0];

    let result = a.subtract(&b);

    assert_eq!(result, c);
  }

  #[test]
  fn scaling_vectors() {
    let a = [1.0, 2.0, 3.0];
    let b = [2.0, 4.0, 6.0];
    let scalar = 2.0;

    let result = a.scale(scalar);
    assert_eq!(result, b);
  }

  #[test]
  fn dot_product() {
    let a = [1.0, 2.0, 3.0];
    let b = [4.0, 5.0, 6.0];
    let c = 32.0;

    let result = a.dot(&b);
    assert_eq!(result, c);
  }

  #[test]
  fn cross_product() {
    let a = [1.0, 2.0, 3.0];
    let b = [4.0, 5.0, 6.0];
    let c = [-3.0, 6.0, -3.0];

    let result = a.cross(&b).expect("cross product inputs are 3D vectors");
    assert_eq!(result, c);
  }

  #[test]
  fn cross_product_fails_for_non_3d_vectors() {
    let a = [1.0, 2.0];
    let b = [4.0, 5.0];

    let result = a.cross(&b);
    assert_eq!(result, Err(MathError::CrossProductDimension { actual: 2 }));
  }

  /// Verify that `cross` returns `MismatchedVectorDimensions` when vectors of
  /// different dimensions are provided.
  #[test]
  fn cross_product_fails_for_mismatched_dimensions() {
    let a: Vec<f32> = vec![1.0, 2.0];
    let b: Vec<f32> = vec![4.0, 5.0, 6.0];

    let result = a.cross(&b);
    assert_eq!(
      result,
      Err(MathError::MismatchedVectorDimensions { left: 2, right: 3 })
    );
  }

  #[test]
  fn length() {
    let a = [1.0, 2.0, 3.0];
    let b = 3.741_657_5;

    let result = a.length();
    assert_eq!(result, b);

    let c = [1.0, 2.0, 3.0, 4.0];
    let d = 5.477_226;
    let result = c.length();
    assert_eq!(result, d);
  }

  #[test]
  fn normalize() {
    let a = [4.0, 3.0, 2.0];
    let b = [0.74278135, 0.55708605, 0.37139067];
    let result = a.normalize().expect("vector has non-zero length");
    assert_eq!(result, b);
  }

  #[test]
  fn normalize_fails_for_zero_length_vector() {
    let a = [0.0, 0.0, 0.0];

    let result = a.normalize();
    assert_eq!(result, Err(MathError::ZeroLengthVector));
  }

  #[test]
  fn scale() {
    let a = [1.0, 2.0, 3.0];
    let b = [2.0, 4.0, 6.0];
    let scalar = 2.0;

    let result = a.scale(scalar);
    assert_eq!(result, b);
  }
}
