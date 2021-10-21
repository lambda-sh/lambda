use std::time::Duration;

use super::{
  event_loop::LambdaEvent,
  render::RenderAPI,
};

/// The Component Interface for allowing Component based data structures
/// like the ComponentStack to store components with various purposes
/// and implementations to work together.
pub trait Component {
  fn attach(&mut self);
  fn detach(&mut self);
  fn on_event(&mut self, event: &LambdaEvent);
  fn on_update(&mut self, last_frame: &Duration, renderer: &mut RenderAPI);
}

/// A stack based Vec that can Push & Pop layers.
pub struct ComponentStack {
  components: Vec<Box<dyn Component + 'static>>,
}

impl Component for ComponentStack {
  /// Attaches all the components that are currently on the component graph
  fn attach(&mut self) {
    for component in &mut self.components {
      component.attach();
    }
  }

  /// Detaches all components currently on the component stack.
  fn detach(&mut self) {
    for component in &mut self.components {
      component.detach();
    }
  }

  /// Pass events to all components in the component stack.
  fn on_event(&mut self, event: &LambdaEvent) {
    for component in &mut self.components {
      component.on_event(&event);
    }
  }

  /// Update all components currently in the component stack.
  fn on_update(&mut self, last_frame: &Duration, renderer: &mut RenderAPI) {
    for component in &mut self.components {
      component.on_update(last_frame, renderer);
    }
  }
}

impl ComponentStack {
  /// Return a new component stack with an empty array of components.
  pub fn new() -> Self {
    return ComponentStack {
      components: Vec::new(),
    };
  }

  /// Push a component on to the component stack.
  pub fn push_component<T>(&mut self)
  where
    T: Default + Component + 'static,
  {
    let layer = Box::new(T::default());
    self.components.push(layer);
  }

  /// Pop a component from the component stack.
  pub fn pop_component(&mut self) -> Option<Box<dyn Component + 'static>> {
    let layer = self.components.pop();
    return layer;
  }

  pub fn get_layers(&mut self) -> &Vec<Box<dyn Component + 'static>> {
    return &self.components;
  }
}
