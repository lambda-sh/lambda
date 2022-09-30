use winit::{
  dpi::{
    LogicalSize,
    PhysicalSize,
  },
  event::Event,
  event_loop::{
    ControlFlow,
    EventLoop,
    EventLoopProxy,
    EventLoopWindowTarget,
  },
  monitor::MonitorHandle,
  window::{
    Window,
    WindowBuilder,
  },
};

/// Embedded module for exporting data/types from winit as minimally/controlled
/// as possible. The exports from this module are not guaranteed to be stable.
pub mod winit_exports {
  pub use winit::{
    event::{
      Event,
      WindowEvent,
    },
    event_loop::{
      ControlFlow,
      EventLoop,
      EventLoopProxy,
      EventLoopWindowTarget,
    },
  };
}

/// Loop wrapping for the winit event loop.
pub struct Loop<E: 'static + std::fmt::Debug> {
  event_loop: EventLoop<E>,
}

pub fn create_event_loop<Events: 'static + std::fmt::Debug>() -> Loop<Events> {
  let event_loop = EventLoop::<Events>::with_user_event();
  return Loop { event_loop };
}

/// Structure that contains properties needed for building a window.
pub struct WindowProperties {
  pub name: String,
  pub dimensions: (u32, u32),
  pub monitor_handle: MonitorHandle,
}

/// Metadata for Lambda window sizing that supports Copy and move operations.
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

/// Construct WindowSize metdata from the window dimensions and scale factor of
/// the monitor being rendered to.
#[inline]
fn construct_window_size(
  window_size: (u32, u32),
  scale_factor: f64,
) -> WindowSize {
  let logical: LogicalSize<u32> = window_size.into();
  let physical: PhysicalSize<u32> = logical.to_physical(scale_factor);

  let (width, height) = window_size;
  return WindowSize {
    width,
    height,
    logical,
    physical,
  };
}

#[derive(Clone, Debug)]
pub struct EventLoopPublisher<E: 'static + std::fmt::Debug> {
  winit_proxy: EventLoopProxy<E>,
}

impl<E: 'static + std::fmt::Debug> EventLoopPublisher<E> {
  /// Instantiate a new EventLoopPublisher from an event loop proxy.
  #[inline]
  pub fn new(winit_proxy: EventLoopProxy<E>) -> Self {
    return EventLoopPublisher { winit_proxy };
  }

  /// Send an event
  #[inline]
  pub fn send_event(&self, event: E) {
    self
      .winit_proxy
      .send_event(event)
      .expect("Failed to send event");
  }
}

impl<E: 'static + std::fmt::Debug> Loop<E> {
  pub fn create_publisher(&mut self) -> EventLoopPublisher<E> {
    let proxy = self.event_loop.create_proxy();
    return EventLoopPublisher::new(proxy);
  }

  /// Returns the primary monitor for the current OS if detectable.
  pub fn get_primary_monitor(&self) -> Option<MonitorHandle> {
    return self.event_loop.primary_monitor();
  }

  /// Get all monitors available on the system.
  pub fn get_all_monitors(&self) -> impl Iterator<Item = MonitorHandle> {
    return self.event_loop.available_monitors();
  }

  pub fn get_any_available_monitors(&self) -> MonitorHandle {
    match self.event_loop.available_monitors().next() {
      Some(monitor) => monitor,
      None => panic!("No available monitors found."),
    }
  }

  /// Uses the winit event loop to run forever.
  pub fn run_forever<Callback>(self, callback: Callback)
  where
    Callback: 'static
      + FnMut(Event<E>, &EventLoopWindowTarget<E>, &mut ControlFlow) -> (),
  {
    self.event_loop.run(callback);
  }

  pub fn create_window_handle(
    &mut self,
    window_properties: WindowProperties,
  ) -> WindowHandle {
    let name = window_properties.name;
    let dimensions = window_properties.dimensions;
    let monitor_handle = window_properties.monitor_handle;

    let size = construct_window_size(dimensions, monitor_handle.scale_factor());

    let window_handle = WindowBuilder::new()
      .with_title(name)
      .with_inner_size(size.logical)
      .build(&self.event_loop)
      .expect("Failed to create a winit handle.");

    return WindowHandle {
      window_handle,
      size,
      monitor_handle,
    };
  }
}
