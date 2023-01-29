pub mod application;
pub use application::{
  ApplicationRuntime,
  ApplicationRuntimeBuilder,
};

pub trait Runtime {
  fn on_start(&mut self);
  fn on_stop(&mut self);
  fn run(self);
}

/// Builds & executes a runtime all in one good. It's a good idea to use this if you
/// don't need to execute any code in between the building & execution stage of
/// the runnable, but will not impact or modify the runtime in any way.
pub fn build_and_start_kernel<T: Default + Runtime>() {
  let runtime = T::default();
  start_runtime(runtime);
}

/// Simple function for starting any prebuilt Runnable.
pub fn start_runtime<T: Runtime>(mut runtime: T) {
  runtime.on_start();
  runtime.run();
}
