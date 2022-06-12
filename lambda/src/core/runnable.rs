use super::component::Component;

pub trait Kernel {
  fn on_start(&mut self);
  fn on_stop(&mut self);
  fn run(self);
}

/// Builds & executes a Runnable all in one good. This is useful for when you
/// don't need to execute any code in between the building & execution stage of
/// the runnable
pub fn build_and_start_kernel<T: Default + Kernel>() {
  let app = T::default();
  start_kernel(app);
}

/// Simple function for starting any prebuilt Runnable.
pub fn start_kernel<T: Kernel>(mut kernel: T) {
  kernel.on_start();
  kernel.run();
}
