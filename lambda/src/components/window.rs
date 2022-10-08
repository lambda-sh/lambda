use std::time::Duration;

use lambda_platform::winit::{
  Loop,
  WindowHandle,
  WindowProperties,
};

use crate::core::{
  component::Component,
  events::Event,
  render::Window,
};

impl Component for Window {
  fn on_attach(&mut self) {
    todo!()
  }

  fn on_detach(&mut self) {
    todo!()
  }

  fn on_event(&mut self, event: &Event) {
    todo!()
  }

  fn on_update(&mut self, _: &Duration) {
    todo!()
  }
}
