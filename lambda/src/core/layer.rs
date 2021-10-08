use std::time::Duration;

use super::{event_loop::LambdaEvent, render::RenderAPI};

pub trait Layer {
  fn attach(&self);
  fn detach(&self);
  fn on_event(&self, event: &LambdaEvent);
  fn on_update(&self, last_frame: &Duration, renderer: &mut RenderAPI);
}

/// A stack based Vec that can Push & Pop layers.
pub struct LayerStack {
  layers: Vec<Box<dyn Layer + 'static>>,
}

impl LayerStack {
  pub fn new() -> Self {
    return LayerStack { layers: Vec::new() };
  }

  pub fn push_layer<T>(&mut self)
  where
    T: Default + Layer + 'static,
  {
    let layer = Box::new(T::default());
    self.layers.push(layer);
  }

  pub fn pop_layer(&mut self) -> Option<Box<dyn Layer + 'static>> {
    let layer = self.layers.pop();
    return layer;
  }

  pub fn on_event(&self, event: &LambdaEvent) {
    for layer in &self.layers {
      layer.on_event(&event);
    }
  }

	pub fn get_layers(&mut self) -> &Vec<Box<dyn Layer + 'static>> {
		return &self.layers;
	}
}