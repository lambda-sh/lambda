pub trait Runnable {
  fn setup(&mut self);
  fn run(self);
}

/// Builds & executes a Runnable all in one good. This is useful for when you
/// don't need to execute any code in between the building & execution stage of
/// the runnable
pub fn build_and_start_runnable<T: Default + Runnable>() {
  let app = T::default();
  start_runnable(app);
}

/// Simple function for starting any prebuilt Runnable.
pub fn start_runnable<T: Runnable>(mut app: T) {
  app.setup();
  app.run();
}
