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

/// The interface for a Component that can be rendered.
pub trait RenderableComponent<E>: Component<E> {
  fn on_attach(&mut self, render_context: &mut super::render::RenderContext);
  fn on_render(
    &mut self,
    render_context: &mut super::render::RenderContext,
    last_render: &Duration,
  ) -> Vec<super::render::command::RenderCommand>;
  fn on_detach(&mut self, render_context: &mut super::render::RenderContext);
}
