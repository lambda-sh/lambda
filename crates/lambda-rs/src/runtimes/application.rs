//! The application runtime is the default runtime for Lambda applications. It
//! provides a window and a render context which can be used to render
//! both 2D and 3D graphics to the screen.

use std::time::{
  Duration,
  Instant,
};

use lambda_platform::winit::{
  winit_exports::{
    ElementState,
    Event as WinitEvent,
    MouseButton,
    PhysicalKey as WinitPhysicalKey,
    WindowEvent as WinitWindowEvent,
  },
  EventLoopPolicy,
  LoopBuilder,
};
use logging;

use crate::{
  component::Component,
  events::{
    Button,
    EventMask,
    Events,
    Key,
    Mouse,
    RuntimeEvent,
    WindowEvent,
  },
  render::{
    window::WindowBuilder,
    RenderContextBuilder,
  },
  runtime::Runtime,
};

/// Result value used by component callbacks executed under
/// `ApplicationRuntime`.
#[derive(Clone, Debug)]
///
/// Components can return `Success` when work completed as expected or
/// `Failure` to signal a non‑fatal error to the runtime.
pub enum ComponentResult {
  Success,
  Failure,
}

/// Builder for constructing an `ApplicationRuntime` with a window, a
/// configured `RenderContext`, and a stack of components that receive events
/// and render access.
pub struct ApplicationRuntimeBuilder {
  app_name: String,
  render_context_builder: RenderContextBuilder,
  window_builder: WindowBuilder,
  event_loop_policy: EventLoopPolicy,
  components: Vec<Box<dyn Component<ComponentResult, String>>>,
}

impl ApplicationRuntimeBuilder {
  /// Create a new builder seeded with sensible defaults.
  pub fn new(app_name: &str) -> Self {
    return Self {
      app_name: app_name.to_string(),
      render_context_builder: RenderContextBuilder::new(app_name),
      window_builder: WindowBuilder::new(),
      event_loop_policy: EventLoopPolicy::Poll,
      components: Vec::new(),
    };
  }

  /// Update the name of the LambdaKernel.
  pub fn with_app_name(mut self, name: &str) -> Self {
    self.app_name = name.to_string();
    return self;
  }

  /// Configures the `RenderAPIBuilder` before the `RenderContext` is built
  /// using a callback provided by the user. The renderer in it's default
  /// state will be good enough for most applications, but if you need to
  /// customize the renderer you can do so here.
  pub fn with_renderer_configured_as(
    mut self,
    configuration: impl FnOnce(RenderContextBuilder) -> RenderContextBuilder,
  ) -> Self {
    self.render_context_builder = configuration(self.render_context_builder);
    return self;
  }

  /// Configures the WindowBuilder before the Window is built using a callback
  /// provided by the user. If you need to customize the window you can do so
  /// here.
  pub fn with_window_configured_as(
    mut self,
    configuration: impl FnOnce(WindowBuilder) -> WindowBuilder,
  ) -> Self {
    self.window_builder = configuration(self.window_builder);
    return self;
  }

  /// Set the winit event loop control-flow policy.
  ///
  /// - [`EventLoopPolicy::Poll`]: Continuous updates, highest CPU usage, lowest latency
  /// - [`EventLoopPolicy::Wait`]: Sleep until events arrive, minimal CPU usage when idle
  /// - [`EventLoopPolicy::WaitUntil`]: Wake at a fixed cadence (best effort)
  pub fn with_event_loop_policy(mut self, policy: EventLoopPolicy) -> Self {
    self.event_loop_policy = policy;
    return self;
  }

  /// Attach a component to the current runnable.
  pub fn with_component<
    T: Default + Component<ComponentResult, String> + 'static,
  >(
    self,
    configure_component: impl FnOnce(Self, T) -> (Self, T),
  ) -> Self {
    let (mut kernel_builder, component) =
      configure_component(self, T::default());
    kernel_builder.components.push(Box::new(component));
    return kernel_builder;
  }

  /// Builds an `ApplicationRuntime` equipped with windowing, an event loop, and a
  /// component stack that allows components to be dynamically pushed into the
  /// Kernel to receive events & render access.
  pub fn build(self) -> ApplicationRuntime {
    ApplicationRuntime {
      name: self.app_name,
      render_context_builder: self.render_context_builder,
      window_builder: self.window_builder,
      event_loop_policy: self.event_loop_policy,
      component_stack: self.components,
    }
  }
}

/// A windowed and event-driven runtime that can be used to render a
/// scene on the primary GPU across Windows, MacOS, and Linux.
pub struct ApplicationRuntime {
  name: String,
  render_context_builder: RenderContextBuilder,
  window_builder: WindowBuilder,
  event_loop_policy: EventLoopPolicy,
  component_stack: Vec<Box<dyn Component<ComponentResult, String>>>,
}

impl ApplicationRuntime {}

fn format_component_handler_failure(error: &String) -> String {
  return format!(
    "A component has panicked while handling an event. {:?}",
    error
  );
}

fn dispatch_event_to_component(
  event: &Events,
  event_mask: EventMask,
  component: &mut dyn Component<ComponentResult, String>,
) -> Result<(), String> {
  let component_mask = component.event_mask();

  if !component_mask.contains(event_mask) {
    return Ok(());
  }

  let result = match event {
    Events::Window { event, .. } => component.on_window_event(event),
    Events::Keyboard { event, .. } => component.on_keyboard_event(event),
    Events::Mouse { event, .. } => component.on_mouse_event(event),
    Events::Runtime { event, .. } => component.on_runtime_event(event),
    Events::Component { event, .. } => component.on_component_event(event),
  };

  match result {
    Ok(()) => {
      return Ok(());
    }
    Err(err) => {
      return Err(format_component_handler_failure(&err));
    }
  }
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

impl Runtime<(), String> for ApplicationRuntime {
  type Component = Box<dyn Component<ComponentResult, String>>;
  /// Runs the event loop for the Application Runtime which takes ownership
  /// of all components, the windowing the render context, and anything
  /// else relevant to the runtime.
  fn run(self) -> Result<(), String> {
    let name = self.name;
    let event_loop_policy = self.event_loop_policy;
    let frame_warn_threshold: Option<Duration> = match event_loop_policy {
      EventLoopPolicy::Poll => Some(Duration::from_millis(32)),
      EventLoopPolicy::Wait => None,
      EventLoopPolicy::WaitUntil { target_fps } if target_fps > 0 => {
        // Compute an expected frame interval (1 / FPS) and warn only if the
        // observed frame time exceeds it by a slack factor (25%) to avoid
        // spamming on small scheduling jitter.
        let clamped_fps = target_fps.min(MAX_TARGET_FPS) as u64;
        let nanos_per_frame = div_ceil_u64(1_000_000_000, clamped_fps);
        let expected_interval = Duration::from_nanos(nanos_per_frame);
        Some(expected_interval.mul_f64(1.25))
      }
      EventLoopPolicy::WaitUntil { .. } => None,
    };
    let mut event_loop = LoopBuilder::new().build();
    let window = self.window_builder.build(&mut event_loop);
    let mut component_stack = self.component_stack;
    let render_context = match self.render_context_builder.build(&window) {
      Ok(ctx) => ctx,
      Err(err) => {
        let msg = format!("Failed to initialize render context: {}", err);
        logging::error!("{}", msg);
        return Err(msg);
      }
    };
    let mut active_render_context = Some(render_context);

    let publisher = event_loop.create_event_publisher();
    publisher.publish_event(Events::Runtime {
      event: RuntimeEvent::Initialized,
      issued_at: Instant::now(),
    });

    let mut current_frame = Instant::now();
    let mut runtime_result: Box<Result<(), String>> = Box::new(Ok(()));

    event_loop.run_forever_with_policy(
      event_loop_policy,
      move |event, target| {
        let mapped_event: Option<Events> = match event {
          WinitEvent::WindowEvent { event, .. } => match event {
            WinitWindowEvent::CloseRequested => {
              // Issue a Shutdown event to deallocate resources and clean up.
              target.exit();
              Some(Events::Runtime {
                event: RuntimeEvent::Shutdown,
                issued_at: Instant::now(),
              })
            }
            WinitWindowEvent::Resized(dims) => {
              active_render_context
                .as_mut()
                .unwrap()
                .resize(dims.width, dims.height);

              Some(Events::Window {
                event: WindowEvent::Resize {
                  width: dims.width,
                  height: dims.height,
                },
                issued_at: Instant::now(),
              })
            }
            WinitWindowEvent::ScaleFactorChanged { .. } => None,
            WinitWindowEvent::Moved(_) => None,
            WinitWindowEvent::Destroyed => None,
            WinitWindowEvent::DroppedFile(_) => None,
            WinitWindowEvent::HoveredFile(_) => None,
            WinitWindowEvent::HoveredFileCancelled => None,
            // Character input is delivered via IME; ignore here for now
            WinitWindowEvent::Focused(_) => None,
            WinitWindowEvent::KeyboardInput {
              event: key_event,
              is_synthetic,
              ..
            } => match (key_event.state, is_synthetic) {
              (ElementState::Pressed, false) => {
                let (scan_code, virtual_key) = match key_event.physical_key {
                  WinitPhysicalKey::Code(code) => (0, Some(code)),
                  _ => (0, None),
                };
                Some(Events::Keyboard {
                  event: Key::Pressed {
                    scan_code,
                    virtual_key,
                  },
                  issued_at: Instant::now(),
                })
              }
              (ElementState::Released, false) => {
                let (scan_code, virtual_key) = match key_event.physical_key {
                  WinitPhysicalKey::Code(code) => (0, Some(code)),
                  _ => (0, None),
                };
                Some(Events::Keyboard {
                  event: Key::Released {
                    scan_code,
                    virtual_key,
                  },
                  issued_at: Instant::now(),
                })
              }
              _ => None,
            },
            WinitWindowEvent::ModifiersChanged(_) => None,
            WinitWindowEvent::CursorMoved {
              device_id: _,
              position,
            } => Some(Events::Mouse {
              event: Mouse::Moved {
                x: position.x,
                y: position.y,
                dx: 0.0,
                dy: 0.0,
                device_id: 0,
              },
              issued_at: Instant::now(),
            }),
            WinitWindowEvent::CursorEntered { device_id: _ } => {
              Some(Events::Mouse {
                event: Mouse::EnteredWindow { device_id: 0 },
                issued_at: Instant::now(),
              })
            }
            WinitWindowEvent::CursorLeft { device_id: _ } => {
              Some(Events::Mouse {
                event: Mouse::LeftWindow { device_id: 0 },
                issued_at: Instant::now(),
              })
            }
            WinitWindowEvent::MouseWheel {
              device_id: _,
              delta: _,
              phase: _,
            } => Some(Events::Mouse {
              event: Mouse::Scrolled { device_id: 0 },
              issued_at: Instant::now(),
            }),
            WinitWindowEvent::MouseInput {
              device_id: _,
              state,
              button,
            } => {
              // Map winit button to our button type
              let button = match button {
                MouseButton::Left => Button::Left,
                MouseButton::Right => Button::Right,
                MouseButton::Middle => Button::Middle,
                MouseButton::Other(other) => Button::Other(other),
                MouseButton::Back => Button::Other(8),
                MouseButton::Forward => Button::Other(9),
              };

              let event = match state {
                ElementState::Pressed => Mouse::Pressed {
                  button,
                  x: 0.0,
                  y: 0.0,
                  device_id: 0,
                },
                ElementState::Released => Mouse::Released {
                  button,
                  x: 0.0,
                  y: 0.0,
                  device_id: 0,
                },
              };

              Some(Events::Mouse {
                event,
                issued_at: Instant::now(),
              })
            }
            WinitWindowEvent::TouchpadPressure { .. } => None,
            WinitWindowEvent::AxisMotion { .. } => None,
            WinitWindowEvent::Touch(_) => None,
            WinitWindowEvent::ThemeChanged(_) => None,
            _ => None,
          },
          WinitEvent::AboutToWait => {
            let last_frame = current_frame;
            current_frame = Instant::now();
            let duration = &current_frame.duration_since(last_frame);

            let active_render_context = active_render_context
              .as_mut()
              .expect("Couldn't get the active render context. ");
            for component in &mut component_stack {
              let update_result = component.on_update(duration);
              if let Err(error) = update_result {
                logging::error!("{}", error);
                publisher.publish_event(Events::Runtime {
                  event: RuntimeEvent::ComponentPanic { message: error },
                  issued_at: Instant::now(),
                });
                continue;
              }
              let commands = component.on_render(active_render_context);
              active_render_context.render(commands);
            }

            // Warn if the time between frames significantly exceeds the expected
            // interval for the selected event loop policy.
            //
            // - Poll: uses a fixed 32 ms threshold (~30 fps).
            // - WaitUntil: uses a threshold derived from the target FPS.
            // - Wait: disabled (duration includes idle sleep time).
            if let Some(threshold) = frame_warn_threshold {
              if *duration > threshold {
                logging::warn!(
                  "Frame took too long to render: {:?} ms",
                  duration.as_millis()
                );
              }
            }

            None
          }
          // Redraw requests are handled implicitly when AboutToWait fires; ignore explicit requests
          WinitEvent::NewEvents(_) => None,
          WinitEvent::DeviceEvent {
            device_id: _,
            event: _,
          } => None,
          WinitEvent::UserEvent(lambda_event) => match lambda_event {
            Events::Runtime {
              event,
              issued_at: _,
            } => match event {
              RuntimeEvent::Initialized => {
                logging::debug!(
                  "Initializing all of the components for the runtime: {}",
                  name
                );
                for component in &mut component_stack {
                  let attach_result = component
                    .on_attach(active_render_context.as_mut().unwrap());
                  if let Err(error) = attach_result {
                    logging::error!("{}", error);
                    publisher.publish_event(Events::Runtime {
                      event: RuntimeEvent::ComponentPanic { message: error },
                      issued_at: Instant::now(),
                    });
                  }
                }
                None
              }
              RuntimeEvent::Shutdown => {
                for component in &mut component_stack {
                  let detach_result = component
                    .on_detach(active_render_context.as_mut().unwrap());
                  if let Err(error) = detach_result {
                    logging::error!("{}", error);
                    publisher.publish_event(Events::Runtime {
                      event: RuntimeEvent::ComponentPanic { message: error },
                      issued_at: Instant::now(),
                    });
                  }
                }
                *runtime_result = Ok(());
                None
              }
              RuntimeEvent::ComponentPanic { message } => {
                *runtime_result = Err(message);
                None
              }
            },
            _ => None,
          },
          WinitEvent::Suspended => None,
          WinitEvent::Resumed => None,
          WinitEvent::MemoryWarning => None,
          // No RedrawEventsCleared in winit 0.29
          WinitEvent::LoopExiting => {
            active_render_context
              .take()
              .expect("[ERROR] The render API has been already taken.")
              .destroy();

            logging::info!("All resources were successfully deleted.");
            None
          }
        };

        if let Some(event) = mapped_event {
          logging::trace!("Sending event: {:?} to all components", event);

          let event_mask = event.mask();
          for component in &mut component_stack {
            let event_result = dispatch_event_to_component(
              &event,
              event_mask,
              component.as_mut(),
            );

            if let Err(error) = event_result {
              logging::error!("{}", error);
              publisher.publish_event(Events::Runtime {
                event: RuntimeEvent::ComponentPanic { message: error },
                issued_at: Instant::now(),
              });
            }
          }
        }
      },
    );
    return Ok(());
  }

  /// When an application runtime starts, it will attach all of the components that
  /// have been added during the construction phase in the users code.
  fn on_start(&mut self) {
    logging::info!("Starting the runtime: {}", self.name);
  }

  fn on_stop(&mut self) {
    logging::info!("Stopping the runtime: {}", self.name);
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{
    events::{
      ComponentEvent,
      Key,
      RuntimeEvent,
      WindowEvent,
    },
    render::RenderContext,
  };

  #[derive(Default)]
  struct RecordingComponent {
    mask: EventMask,
    window_event_count: u32,
    keyboard_event_count: u32,
    mouse_event_count: u32,
    runtime_event_count: u32,
    component_event_count: u32,
    fail_window: bool,
  }

  impl Component<ComponentResult, String> for RecordingComponent {
    fn on_attach(
      &mut self,
      _render_context: &mut RenderContext,
    ) -> Result<ComponentResult, String> {
      return Ok(ComponentResult::Success);
    }

    fn on_detach(
      &mut self,
      _render_context: &mut RenderContext,
    ) -> Result<ComponentResult, String> {
      return Ok(ComponentResult::Success);
    }

    fn event_mask(&self) -> EventMask {
      return self.mask;
    }

    fn on_window_event(&mut self, _event: &WindowEvent) -> Result<(), String> {
      self.window_event_count += 1;
      if self.fail_window {
        return Err("window failure".to_string());
      }
      return Ok(());
    }

    fn on_keyboard_event(&mut self, _event: &Key) -> Result<(), String> {
      self.keyboard_event_count += 1;
      return Ok(());
    }

    fn on_mouse_event(
      &mut self,
      _event: &crate::events::Mouse,
    ) -> Result<(), String> {
      self.mouse_event_count += 1;
      return Ok(());
    }

    fn on_runtime_event(
      &mut self,
      _event: &RuntimeEvent,
    ) -> Result<(), String> {
      self.runtime_event_count += 1;
      return Ok(());
    }

    fn on_component_event(
      &mut self,
      _event: &ComponentEvent,
    ) -> Result<(), String> {
      self.component_event_count += 1;
      return Ok(());
    }

    fn on_update(
      &mut self,
      _last_frame: &std::time::Duration,
    ) -> Result<ComponentResult, String> {
      return Ok(ComponentResult::Success);
    }

    fn on_render(
      &mut self,
      _render_context: &mut RenderContext,
    ) -> Vec<crate::render::command::RenderCommand> {
      return Vec::new();
    }
  }

  #[test]
  fn dispatch_skips_component_when_mask_is_none() {
    let mut component = RecordingComponent {
      mask: EventMask::NONE,
      ..Default::default()
    };

    let event = Events::Window {
      event: WindowEvent::Close,
      issued_at: Instant::now(),
    };
    let event_mask = event.mask();

    dispatch_event_to_component(&event, event_mask, &mut component).unwrap();

    assert_eq!(component.window_event_count, 0);
  }

  #[test]
  fn dispatch_skips_component_when_mask_does_not_contain_event() {
    let mut component = RecordingComponent {
      mask: EventMask::KEYBOARD,
      ..Default::default()
    };

    let event = Events::Window {
      event: WindowEvent::Close,
      issued_at: Instant::now(),
    };
    let event_mask = event.mask();

    dispatch_event_to_component(&event, event_mask, &mut component).unwrap();

    assert_eq!(component.window_event_count, 0);
    assert_eq!(component.keyboard_event_count, 0);
  }

  #[test]
  fn dispatch_calls_exact_handler_when_mask_contains_event() {
    let mut component = RecordingComponent {
      mask: EventMask::WINDOW | EventMask::KEYBOARD,
      ..Default::default()
    };

    let window_event = Events::Window {
      event: WindowEvent::Resize {
        width: 1,
        height: 2,
      },
      issued_at: Instant::now(),
    };
    let window_event_mask = window_event.mask();

    dispatch_event_to_component(
      &window_event,
      window_event_mask,
      &mut component,
    )
    .unwrap();

    assert_eq!(component.window_event_count, 1);
    assert_eq!(component.keyboard_event_count, 0);

    let keyboard_event = Events::Keyboard {
      event: Key::Pressed {
        scan_code: 0,
        virtual_key: None,
      },
      issued_at: Instant::now(),
    };
    let keyboard_event_mask = keyboard_event.mask();

    dispatch_event_to_component(
      &keyboard_event,
      keyboard_event_mask,
      &mut component,
    )
    .unwrap();

    assert_eq!(component.window_event_count, 1);
    assert_eq!(component.keyboard_event_count, 1);
  }

  #[test]
  fn dispatch_returns_fatal_error_message_on_handler_failure() {
    let mut component = RecordingComponent {
      mask: EventMask::WINDOW,
      fail_window: true,
      ..Default::default()
    };

    let event = Events::Window {
      event: WindowEvent::Close,
      issued_at: Instant::now(),
    };
    let event_mask = event.mask();

    let error = dispatch_event_to_component(&event, event_mask, &mut component)
      .unwrap_err();

    assert!(error.contains("A component has panicked while handling an event."));
    assert!(error.contains("window failure"));
  }
}
