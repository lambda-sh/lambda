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
// TODO(ahlawat) = Remove all these except WindowEvent since we're abstracting them already? Double check
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

/// LoopBuilder - Putting this here for consistency.
pub struct LoopBuilder;

impl LoopBuilder {
  pub fn build<Events: 'static>() -> Loop<Events> {
    let event_loop = EventLoop::<Events>::with_user_event();
    return Loop { event_loop };
  }
}

/// Loop wrapping for the winit event loop.
pub struct Loop<E: 'static> {
  event_loop: EventLoop<E>,
}

/// TODO(ahlawat) = Remove this and refactor the code depending directly on it.
pub fn create_event_loop<Events: 'static>() -> Loop<Events> {
  let event_loop = EventLoop::<Events>::with_user_event();
  return Loop { event_loop };
}

/// Structure that contains properties needed for building a window.
pub struct WindowProperties {
  pub name: String,
  pub dimensions: [u32; 2],
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
  size: Option<WindowSize>,
  monitor_handle: Option<MonitorHandle>,
}

impl WindowHandleBuilder {
  /// Instantiate an empty builder
  pub fn new() -> Self {
    return Self;
  }

  /// Set the window size for the WindowHandle
  fn with_window_size(
    &mut self,
    window_size: [u32; 2],
    scale_factor: f64,
  ) -> self {
    let logical: LogicalSize<u32> = window_size.into();
    let physical: PhysicalSize<u32> = logical_size.to_physical(scale_factor);

    let window_size = WindowSize {
      width: window_size[0],
      height: window_size[1],
      logical,
      physical,
    };

    self.size = Some(window_size);
    return self;
  }

  /// Probably the function that'll be used the most
  pub fn with_window_properties<E: 'static>(
    &mut self,
    window_properties: WindowProperties,
    lambda_loop: &Loop<E>,
  ) -> self {
    let WindowProperties {
      name,
      dimensions,
      monitor_handle,
    } = window_properties;

    self.with_window_size(dimensions, monitor_handle.scale_factor());

    let window_handle = WindowBuilder::new()
      .with_title(name)
      .with_inner_size(self.size.expect("No window size found.").logical)
      .build(&lambda_loop.event_loop)
      .expect("Failed creation of window handle");

    self.monitor_handle = Some(monitor_handle);
    self.window_handle = Some(window_handle);
    return self;
  }

  /// Build the WindowHandle
  pub fn build(&self) -> WindowHandle {
    return WindowHandle {
      monitor_handle: self
        .monitor_handle
        .expect("Unable to find a MonitorHandle."),
      size: self.size.expect("Unable to find WindowSize."),
      window_handle: self.window_handle.expect("Unable to find WindowHandle."),
    };
  }
}

// TODO(ahlawat) = Remove this as well?
/// Construct WindowSize metdata from the window dimensions and scale factor of
/// the monitor being rendered to.
#[inline]
fn construct_window_size(
  window_size: [u32; 2],
  scale_factor: f64,
) -> WindowSize {
  let logical: LogicalSize<u32> = window_size.into();
  let physical: PhysicalSize<u32> = logical.to_physical(scale_factor);

  return WindowSize {
    width: window_size[0],
    height: window_size[1],
    logical,
    physical,
  };
}

pub struct LoopPublisher<E: 'static> {
  winit_proxy: EventLoopProxy<E>,
}

impl<E: 'static> LoopPublisher<E> {
  /// Instantiate a new LoopPublisher from an event loop proxy.
  #[inline]
  pub fn new(winit_proxy: EventLoopProxy<E>) -> Self {
    return LoopPublisher { winit_proxy };
  }

  /// Instantiate a new LoopPublisher from a loop
  pub fn from<E: 'static>(lambda_loop: &Loop<E>) -> Self {
    let winit_proxy = lambda_loop.event_loop.create_proxy();
    return LoopPublisher { winit_proxy };
  }

  /// Send an event
  #[inline]
  pub fn send_event(&self, event: E) {
    self.winit_proxy.send_event(event);
  }
}

impl<E: 'static> Loop<E> {
  // TODO(ahlawat) = Possibly remove this?
  pub fn create_publisher(&mut self) -> LoopPublisher<E> {
    let proxy = self.event_loop.create_proxy();
    return LoopPublisher::new(proxy);
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

  // TODO(ahlawat) = Should this be here?
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
