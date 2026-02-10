//! Winit wrapper to easily construct cross platform windows

use std::time::{
  Duration,
  Instant,
};

use winit::{
  dpi::{
    LogicalSize,
    PhysicalSize,
  },
  event::Event,
  event_loop::{
    ControlFlow,
    EventLoop,
    EventLoopBuilder,
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
      ElementState,
      Event,
      KeyEvent,
      MouseButton,
      WindowEvent,
    },
    event_loop::{
      ControlFlow,
      EventLoop,
      EventLoopProxy,
      EventLoopWindowTarget,
    },
    keyboard::{
      KeyCode,
      PhysicalKey,
    },
  };
}

/// Control flow policy for the winit event loop.
///
/// Lambda defaults to [`EventLoopPolicy::Poll`] for backwards compatibility.
/// Applications that don't require continuous updates (e.g., editors/tools)
/// should prefer [`EventLoopPolicy::Wait`] to reduce CPU usage when idle.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EventLoopPolicy {
  /// Continuous polling for games and real-time applications.
  Poll,
  /// Sleep until events arrive; ideal for tools and editors.
  Wait,
  /// Sleep until the next frame deadline to target a fixed update rate.
  ///
  /// Note: this is not a frame-pacing / vsync guarantee; it only controls how
  /// long the event loop waits between wakeups.
  WaitUntil { target_fps: u32 },
}

const MAX_TARGET_FPS: u32 = 1000;

fn div_ceil_u64(numerator: u64, denominator: u64) -> u64 {
  let div = numerator / denominator;
  let rem = numerator % denominator;
  if rem == 0 {
    return div;
  }
  return div + 1;
}

fn frame_interval_for_target_fps(target_fps: u32) -> Option<Duration> {
  if target_fps == 0 {
    return None;
  }

  // Clamp to a sane max to avoid impractically small intervals (which can
  // busy-loop or require large catch-up work after sleeps).
  let clamped_fps = target_fps.min(MAX_TARGET_FPS) as u64;

  // Compute a non-zero interval in integer nanoseconds (ceil to ensure at
  // least 1ns).
  let nanos_per_frame = div_ceil_u64(1_000_000_000, clamped_fps);
  return Some(Duration::from_nanos(nanos_per_frame));
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

impl Default for LoopBuilder {
  fn default() -> Self {
    return Self::new();
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
}

/// Metadata for Lambda window sizing that supports Copy and Move operations.
#[derive(Clone, Copy)]
pub struct WindowSize {
  pub width: u32,
  pub height: u32,
  pub logical: LogicalSize<u32>,
  pub physical: PhysicalSize<u32>,
}

/// Aggregated window handle with cached sizing and monitor metadata.
pub struct WindowHandle {
  pub window_handle: Window,
  pub size: WindowSize,
  pub monitor_handle: Option<MonitorHandle>,
}

// Should we take the loop as a field right here? Probably a ref or something? IDK
/// Builder for constructing a `WindowHandle` from window properties.
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
    let WindowProperties { name, dimensions } = window_properties;

    // Initialize using a neutral scale factor; recompute after creating the window.
    self = self.with_window_size(dimensions, 1.0);

    let window_handle: Window = WindowBuilder::new()
      .with_title(name)
      .with_inner_size(self.size.logical)
      .build(&lambda_loop.event_loop)
      .expect("Failed creation of window handle");

    // Recompute size using the actual window scale factor and cache current monitor if available.
    let scale_factor = window_handle.scale_factor();
    self = self.with_window_size(dimensions, scale_factor);
    self.monitor_handle = window_handle.current_monitor();
    self.window_handle = Some(window_handle);
    return self;
  }

  /// Build the WindowHandle
  pub fn build(self) -> WindowHandle {
    return WindowHandle {
      monitor_handle: self.monitor_handle,
      size: self.size,
      window_handle: self.window_handle.expect("Unable to find WindowHandle."),
    };
  }
}

impl Default for WindowHandleBuilder {
  fn default() -> Self {
    return Self::new();
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
    return LoopPublisher::new(self);
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
  pub fn run_forever<Callback>(self, callback: Callback)
  where
    Callback: 'static + FnMut(Event<E>, &EventLoopWindowTarget<E>),
  {
    self.run_forever_with_policy(EventLoopPolicy::Poll, callback);
  }

  /// Uses the winit event loop to run forever with the provided control-flow
  /// policy.
  pub fn run_forever_with_policy<Callback>(
    self,
    policy: EventLoopPolicy,
    mut callback: Callback,
  ) where
    Callback: 'static + FnMut(Event<E>, &EventLoopWindowTarget<E>),
  {
    let frame_interval = match policy {
      EventLoopPolicy::WaitUntil { target_fps } => {
        frame_interval_for_target_fps(target_fps)
      }
      _ => None,
    };
    let mut next_frame_deadline: Option<Instant> = None;

    self
      .event_loop
      .run(move |event, target| {
        match policy {
          EventLoopPolicy::Poll => {
            target.set_control_flow(ControlFlow::Poll);
          }
          EventLoopPolicy::Wait => {
            target.set_control_flow(ControlFlow::Wait);
          }
          EventLoopPolicy::WaitUntil { target_fps: 0 } => {
            target.set_control_flow(ControlFlow::Wait);
          }
          EventLoopPolicy::WaitUntil { .. } => {
            let now = Instant::now();
            let interval = frame_interval.unwrap_or(Duration::from_secs(1));

            // Guarantee the deadline always advances and stays in the future.
            let deadline = match next_frame_deadline {
              Some(deadline) if deadline > now => deadline,
              _ => now + interval,
            };

            next_frame_deadline = Some(deadline);
            target.set_control_flow(ControlFlow::WaitUntil(deadline));
          }
        }

        callback(event, target);
      })
      .expect("Event loop terminated unexpectedly");
  }
}
