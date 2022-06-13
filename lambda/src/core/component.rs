use std::time::Duration;

/// The Component Interface for allowing Component based data structures
/// like the ComponentStack to store components with various purposes
/// and implementations to work together.
pub trait Component<E> {
  fn on_attach(&mut self);
  fn on_detach(&mut self);
  fn on_event(&mut self, event: &E);
  fn on_update(&mut self, last_frame: &Duration);
}
