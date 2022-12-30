//! Vector math types and functions.

/// Generalized Vector operations that can be implemented by any vector like
/// type.
pub trait Vector {
  type Scalar: Copy;
  fn add(&self, other: &Self) -> Self;
  fn subtract(&self, other: &Self) -> Self;
  fn scale(&self, scalar: Self::Scalar) -> Self;
  fn dot(&self, other: &Self) -> Self::Scalar;
  fn cross(&self, other: &Self) -> Self;
  fn length(&self) -> Self::Scalar;
  fn normalize(&self) -> Self;
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

  /// Cross product of two 3D vectors. Panics if the vectors are not 3D.
  fn cross(&self, other: &Self) -> Self {
    assert_eq!(
      self.as_ref().len(),
      other.as_ref().len(),
      "Vectors must be the same length"
    );

    let mut result = Self::default();
    let a = self.as_ref();
    let b = other.as_ref();

    // TODO: This is only for 3D vectors
    match a.len() {
      3 => {
        result.as_mut()[0] = a[1] * b[2] - a[2] * b[1];
        result.as_mut()[1] = a[2] * b[0] - a[0] * b[2];
        result.as_mut()[2] = a[0] * b[1] - a[1] * b[0];
      }
      _ => {
        panic!("Cross product is only defined for 3 dimensional vectors.")
      }
    }
    return result;
  }

  fn length(&self) -> Self::Scalar {
    let mut result = 0.0;
    for a in self.as_ref().iter() {
      result += a * a;
    }
    result.sqrt()
  }

  fn normalize(&self) -> Self {
    assert_ne!(self.length(), 0.0, "Cannot normalize a zero length vector");
    let mut result = Self::default();
    let length = self.length();

    self.as_ref().iter().enumerate().for_each(|(i, a)| {
      result.as_mut()[i] = a / length;
    });

    return result;
  }

  fn scale(&self, scalar: Self::Scalar) -> Self {
    let mut result = Self::default();
    self.as_ref().iter().enumerate().for_each(|(i, a)| {
      result.as_mut()[i] = a * scalar;
    });

    return result;
  }
}

#[cfg(test)]
mod tests {
  use super::Vector;

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

    let result = a.cross(&b);
    assert_eq!(result, c);
  }

  #[test]
  fn cross_product_fails_for_non_3d_vectors() {
    let a = [1.0, 2.0];
    let b = [4.0, 5.0];

    let result = std::panic::catch_unwind(|| a.cross(&b));
    assert!(result.is_err());
  }

  #[test]
  fn length() {
    let a = [1.0, 2.0, 3.0];
    let b = 3.7416573867739413;

    let result = a.length();
    assert_eq!(result, b);

    let c = [1.0, 2.0, 3.0, 4.0];
    let d = 5.477225575051661;
    let result = c.length();
    assert_eq!(result, d);
  }

  #[test]
  fn normalize() {
    let a = [4.0, 3.0, 2.0];
    let b = [0.74278135, 0.55708605, 0.37139067];
    let result = a.normalize();
    assert_eq!(result, b);
  }

  #[test]
  fn normalize_fails_for_zero_length_vector() {
    let a = [0.0, 0.0, 0.0];

    let result = std::panic::catch_unwind(|| a.normalize());
    assert!(result.is_err());
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
