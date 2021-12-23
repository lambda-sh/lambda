use std::time::Duration;

use crate::{core::component::Component, core::events::Event};

/// A stack based Vec that can Push & Pop layers.
pub struct ComponentStack {
  components: Vec<Box<dyn Component>>,
}

impl Component for ComponentStack {
  /// Attaches all the components that are currently on the component graph
  fn on_attach(&mut self) {
    for component in &mut self.components {
      component.on_attach();
    }
  }

  /// Detaches all components currently on the component stack.
  fn on_detach(&mut self) {
    for component in &mut self.components {
      component.on_detach();
    }
  }

  /// Pass events to all components in the component stack.
  fn on_event(&mut self, event: &Event) {
    for component in &mut self.components {
      component.on_event(&event);
    }
  }

  /// Update all components currently in the component stack.
  fn on_update(&mut self, last_frame: &Duration) {
    for component in &mut self.components {
      component.on_update(last_frame);
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
  pub fn push_component<T>(&mut self, component: T)
  where
    T: Component + 'static,
  {
    let component = Box::new(component);
    self.components.push(component);
  }

  /// Pop a component from the component stack. Doesn't delete or detach
  /// the component.
  pub fn pop_component(&mut self) -> Option<Box<dyn Component + 'static>> {
    let component = self.components.pop();
    return component;
  }
}
