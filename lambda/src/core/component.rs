use std::time::Duration;

use super::event_loop::Event;

/// The Component Interface for allowing Component based data structures
/// like the ComponentStack to store components with various purposes
/// and implementations to work together.
pub trait Component {
  fn on_attach(&mut self);
  fn on_detach(&mut self);
  fn on_event(&mut self, event: &Event);
  fn on_update(&mut self, last_frame: &Duration);
}
