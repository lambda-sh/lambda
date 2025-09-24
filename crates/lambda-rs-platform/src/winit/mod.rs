//! Winit wrapper to easily construct cross platform windows

use winit::{
  dpi::{LogicalSize, PhysicalSize},
  event::Event,
  event_loop::{
    ControlFlow, EventLoop, EventLoopBuilder, EventLoopProxy,
    EventLoopWindowTarget,
  },
  monitor::MonitorHandle,
  window::{Window, WindowBuilder},
};

/// Embedded module for exporting data/types from winit as minimally/controlled
/// as possible. The exports from this module are not guaranteed to be stable.
pub mod winit_exports {
  pub use winit::{
    event::{ElementState, Event, KeyEvent, MouseButton, WindowEvent},
    event_loop::{
      ControlFlow, EventLoop, EventLoopProxy, EventLoopWindowTarget,
    },
    keyboard::{KeyCode, PhysicalKey},
  };
}

/// LoopBuilder - Putting this here for consistency.
pub struct LoopBuilder;

impl LoopBuilder {
  pub fn new() -> Self {
    return Self;
  }

  pub fn build<Events: 'static + std::fmt::Debug>(self) -> Loop<Events> {
    let event_loop = EventLoopBuilder::<Events>::with_user_event()
      .build()
      .expect("Failed to build event loop");
    return Loop { event_loop };
  }
}

/// Loop wrapping for the winit event loop.
pub struct Loop<E: 'static + std::fmt::Debug> {
  event_loop: EventLoop<E>,
}

/// Structure that contains properties needed for building a window.
pub struct WindowProperties {
  pub name: String,
  pub dimensions: (u32, u32),
  pub monitor_handle: MonitorHandle,
}

/// Metadata for Lambda window sizing that supports Copy and Move operations.
#[derive(Clone, Copy)]
pub struct WindowSize {
  pub width: u32,
  pub height: u32,
  pub logical: LogicalSize<u32>,
  pub physical: PhysicalSize<u32>,
}

pub struct WindowHandle {
  pub window_handle: Window,
  pub size: WindowSize,
  pub monitor_handle: MonitorHandle,
}

// Should we take the loop as a field right here? Probably a ref or something? IDK
pub struct WindowHandleBuilder {
  window_handle: Option<Window>,
  size: WindowSize,
  monitor_handle: Option<MonitorHandle>,
}

impl WindowHandleBuilder {
  /// Instantiate an empty builder
  pub fn new() -> Self {
    // Initialize the window size with some default values.
    let logical: LogicalSize<u32> = [0, 0].into();
    let physical = logical.to_physical(1.0);
    let size = WindowSize {
      width: 0,
      height: 0,
      logical,
      physical,
    };

    return Self {
      window_handle: None,
      size,
      monitor_handle: None,
    };
  }

  /// Set the window size for the WindowHandle
  fn with_window_size(
    mut self,
    window_size: (u32, u32),
    scale_factor: f64,
  ) -> Self {
    let logical: LogicalSize<u32> = window_size.into();
    let physical: PhysicalSize<u32> = logical.to_physical(scale_factor);
    let (width, height) = window_size;

    let window_size = WindowSize {
      width,
      height,
      logical,
      physical,
    };

    self.size = window_size;
    return self;
  }

  /// Probably the function that'll be used the most
  pub fn with_window_properties<E: 'static + std::fmt::Debug>(
    mut self,
    window_properties: WindowProperties,
    lambda_loop: &Loop<E>,
  ) -> Self {
    let WindowProperties {
      name,
      dimensions,
      monitor_handle,
    } = window_properties;

    // TODO(ahlawat) = Find out if there's a better way to do this. Looks kinda ugly.
    self = self.with_window_size(dimensions, monitor_handle.scale_factor());

    let window_handle = WindowBuilder::new()
      .with_title(name)
      .with_inner_size(self.size.logical)
      .build(&lambda_loop.event_loop)
      .expect("Failed creation of window handle");

    self.monitor_handle = Some(monitor_handle);
    self.window_handle = Some(window_handle);
    return self;
  }

  /// Build the WindowHandle
  pub fn build(self) -> WindowHandle {
    return WindowHandle {
      monitor_handle: self
        .monitor_handle
        .expect("Unable to find a MonitorHandle."),
      size: self.size,
      window_handle: self.window_handle.expect("Unable to find WindowHandle."),
    };
  }
}

/// Event loop publisher wrapper for pushing events into a winit event loop.
pub struct LoopPublisher<E: 'static> {
  winit_proxy: EventLoopProxy<E>,
}

impl<E: 'static + std::fmt::Debug> LoopPublisher<E> {
  /// New LoopPublishers are created from a lambda_loop directly and don't need
  #[inline]
  pub fn new(lambda_loop: &Loop<E>) -> Self {
    let winit_proxy = lambda_loop.event_loop.create_proxy();
    return LoopPublisher { winit_proxy };
  }

  /// Publishes an event into the event loop that created this publisher.
  #[inline]
  pub fn publish_event(&self, event: E) {
    self
      .winit_proxy
      .send_event(event)
      .expect("Failed to send event");
  }
}

impl<E: 'static + std::fmt::Debug> Loop<E> {
  /// Create an event publisher for this Loop.
  pub fn create_event_publisher(&mut self) -> LoopPublisher<E> {
    return LoopPublisher::new(&self);
  }

  /// Returns the primary monitor for the current OS if detectable.
  pub fn get_primary_monitor(&self) -> Option<MonitorHandle> {
    return self.event_loop.primary_monitor();
  }

  /// Get all monitors available on the system.
  pub fn get_all_monitors(&self) -> impl Iterator<Item = MonitorHandle> {
    return self.event_loop.available_monitors();
  }

  /// Gets the first available monitor or panics.
  pub fn get_any_available_monitors(&self) -> Option<MonitorHandle> {
    return self.event_loop.available_monitors().next();
  }

  /// Uses the winit event loop to run forever
  pub fn run_forever<Callback>(self, mut callback: Callback)
  where
    Callback: 'static + FnMut(Event<E>, &EventLoopWindowTarget<E>),
  {
    self
      .event_loop
      .run(move |event, target| {
        target.set_control_flow(ControlFlow::Poll);
        callback(event, target);
      })
      .expect("Event loop terminated unexpectedly");
  }
}
