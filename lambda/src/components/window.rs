use std::time::Duration;

use crate::{
  core::{
    component::Component,
    events::Event,
  },
  platform::winit::{
    Loop,
    WindowHandle,
    WindowProperties,
  },
};

pub struct Window {
  window_handle: WindowHandle,
}

impl Window {
  /// Create a new
  pub fn new(
    name: &str,
    dimensions: [u32; 2],
    event_loop: &mut Loop<Event>,
  ) -> Self {
    let monitor_handle = event_loop
      .get_primary_monitor()
      .unwrap_or(event_loop.get_any_available_monitors());

    let window_properties = WindowProperties {
      name: name.to_string(),
      dimensions,
      monitor_handle,
    };

    let window_handle = event_loop.create_window_handle(window_properties);

    return Self { window_handle };
  }

  pub fn redraw(&mut self) {
    self.window_handle.window_handle.request_redraw();
  }

  pub fn window_handle(&self) -> &WindowHandle {
    return &self.window_handle;
  }

  pub fn dimensions(&self) -> [u32; 2] {
    return [
      self.window_handle.size.width,
      self.window_handle.size.height,
    ];
  }
}

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
