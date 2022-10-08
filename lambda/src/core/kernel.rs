pub trait Runtime {
  fn on_start(&mut self);
  fn on_stop(&mut self);
  fn run(self);
}

/// Builds & executes a Runnable all in one good. This is useful for when you
/// don't need to execute any code in between the building & execution stage of
/// the runnable
pub fn build_and_start_kernel<T: Default + Runtime>() {
  let app = T::default();
  start_runtime(app);
}

/// Simple function for starting any prebuilt Runnable.
pub fn start_runtime<T: Runtime>(mut kernel: T) {
  kernel.on_start();
  kernel.run();
}
