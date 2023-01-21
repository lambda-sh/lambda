use std::time::Duration;

use super::{
  events::Events,
  render::{
    command::RenderCommand,
    RenderContext,
  },
};

/// The Component Interface for allowing Component based data structures
/// like the ComponentStack to store components with various purposes
/// and implementations to work together.
pub trait Component {
  fn on_attach(&mut self, render_context: &mut RenderContext);
  fn on_detach(&mut self, render_context: &mut RenderContext);
  fn on_event(&mut self, event: Events);

  /// The update function is called every frame and is used to update
  /// the state of the component.
  fn on_update(&mut self, last_frame: &Duration);

  fn on_render(
    &mut self,
    render_context: &mut RenderContext,
  ) -> Vec<RenderCommand>;
}
