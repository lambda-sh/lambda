use std::{
  fmt::Debug,
  time::Duration,
};

use crate::{
  events::{
    ComponentEvent,
    EventMask,
    Key,
    Mouse,
    RuntimeEvent,
    WindowEvent,
  },
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

  /// Declares which event categories this component wants to receive.
  ///
  /// Runtimes MAY use this to skip dispatch for components that are not
  /// interested in a given event category.
  fn event_mask(&self) -> EventMask {
    return EventMask::NONE;
  }

  /// Called when a window event is received.
  fn on_window_event(&mut self, _event: &WindowEvent) -> Result<(), E> {
    return Ok(());
  }

  /// Called when a keyboard event is received.
  fn on_keyboard_event(&mut self, _event: &Key) -> Result<(), E> {
    return Ok(());
  }

  /// Called when a mouse event is received.
  fn on_mouse_event(&mut self, _event: &Mouse) -> Result<(), E> {
    return Ok(());
  }

  /// Called when a runtime event is received.
  fn on_runtime_event(&mut self, _event: &RuntimeEvent) -> Result<(), E> {
    return Ok(());
  }

  /// Called when a component event is received.
  fn on_component_event(&mut self, _event: &ComponentEvent) -> Result<(), E> {
    return Ok(());
  }

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
