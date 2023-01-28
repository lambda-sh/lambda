use rand::{
  distributions::Uniform,
  Rng,
};

/// Generate a random float within any given range.
#[inline(always)]
pub fn get_random_float_between(min: f32, max: f32) -> f32 {
  let mut rng = rand::thread_rng();
  return rng.gen_range(min..max);
}

/// Generate a vector of uniformally distributed random floats within any given
/// range.
pub fn get_uniformly_random_floats_between(
  min: f32,
  max: f32,
  count: usize,
) -> Vec<f32> {
  let distribution = rand::distributions::Uniform::new(min, max);
  let mut rng = rand::thread_rng();
  let mut result = Vec::with_capacity(count);
  for _ in 0..count {
    result.push(rng.sample(distribution));
  }
  return result;
}
