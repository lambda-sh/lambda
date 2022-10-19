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

  /// When the application state should perform logic updates.
  fn on_update(&mut self, last_frame: &Duration);

  /// When the application state should perform rendering.
  fn on_render(
    &mut self,
    render_context: &mut RenderContext,
  ) -> Vec<RenderCommand>;
}
