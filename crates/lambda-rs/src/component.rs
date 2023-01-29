use std::{
  fmt::Debug,
  time::Duration,
};

use crate::{
  events::Events,
  render::{
    command::RenderCommand,
    RenderContext,
  },
};

/// The Component Interface for allowing Component based data structures
/// like the ComponentStack to store components with various purposes
/// and implementations to work together.
pub trait Component<R, E>
where
  R: Sized + Debug,
  E: Sized + Debug,
{
  /// The attach function is called when the component is added to the
  /// component data storage a runtime is using.
  fn on_attach(&mut self, render_context: &mut RenderContext) -> Result<R, E>;

  /// The detach function is called when the component is removed from the
  /// component data storage a runtime is using.
  fn on_detach(&mut self, render_context: &mut RenderContext) -> Result<R, E>;

  /// The event function is called every time an event is received from
  /// the windowing system/event loop.
  fn on_event(&mut self, event: Events) -> Result<R, E>;

  /// The update function is called every frame and is used to update
  /// the state of the component.
  fn on_update(&mut self, last_frame: &Duration) -> Result<R, E>;

  /// Render commands returned from this function will be executed
  /// by the renderer immediately.
  fn on_render(
    &mut self,
    render_context: &mut RenderContext,
  ) -> Vec<RenderCommand>;
}
