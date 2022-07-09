use lambda_platform::winit::{
  Loop,
  WindowHandle,
  WindowProperties,
};

use super::internal::RenderBackend;
use crate::core::events::Event;

pub struct WindowBuilder {
  name: String,
  dimensions: [u32; 2],
}

impl WindowBuilder {
  /// Construct a new window window builder.
  pub fn new() -> Self {
    return Self {
      name: "Window".to_string(),
      dimensions: [480, 360],
    };
  }

  /// The name of the window (Will also appear as the title of the window/application)
  pub fn with_name(mut self, name: &str) -> Self {
    self.name = name.to_string();
    return self;
  }

  /// Specify the dimensions for the window (Defaults to 480 x 360)
  pub fn with_dimensions(mut self, width: u32, height: u32) -> Self {
    self.dimensions = [width, height];
    return self;
  }

  // TODO(vmarcella): Remove new call for window and construct the window directly.
  pub fn build(self, event_loop: &mut Loop<Event>) -> Window {
    return Window::new(self.name.as_str(), self.dimensions, event_loop);
  }
}

pub struct Window {
  window_handle: WindowHandle,
}

impl Window {
  /// Create a new
  fn new(
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

  /// Redraws the window.
  pub fn redraw(&mut self) {
    self.window_handle.window_handle.request_redraw();
  }

  /// Returns the window handle.
  pub fn window_handle(&self) -> &WindowHandle {
    return &self.window_handle;
  }

  /// Returns the dimensions of the current window.
  pub fn dimensions(&self) -> [u32; 2] {
    return [
      self.window_handle.size.width,
      self.window_handle.size.height,
    ];
  }
}