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
    let mut result = 0.0;
    for (a, b) in self.as_ref().iter().zip(other.as_ref().iter()) {
      result += a * b;
    }
    return result;
  }

  fn cross(&self, other: &Self) -> Self {
    let mut result = Self::default();
    self
      .as_ref()
      .iter()
      .zip(other.as_ref().iter())
      .enumerate()
      .for_each(|(i, (a, b))| {
        result.as_mut()[i] = a * b;
      });
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
    let mut result = Self::default();
    let length = self.length();
    self.as_ref().iter().enumerate().for_each(|(i, a)| {
      result.as_mut()[i] = a / length;
    });

    return result;
  }

  fn scale(&self, scalar: Self::Scalar) -> Self {
    let mut result = Self::default();
    let length = self.length();
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
}
