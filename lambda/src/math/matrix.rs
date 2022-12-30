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

/// Matrix implementations for arrays
impl<T, V> Matrix<V> for T
where
  T: AsMut<[V]> + AsRef<[V]> + Default,
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
    for (i, a) in self.as_ref().iter().enumerate() {
      for (j, b) in other.as_ref().iter().enumerate() {
        todo!("Matrix multiplication");
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
