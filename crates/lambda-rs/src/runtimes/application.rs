//! The application runtime is the default runtime for Lambda applications. It
//! provides a window and a render context which can be used to render
//! both 2D and 3D graphics to the screen.

use std::time::Instant;

use lambda_platform::winit::{
  winit_exports::{
    ElementState,
    Event as WinitEvent,
    MouseButton,
    WindowEvent as WinitWindowEvent,
  },
  Loop,
  LoopBuilder,
};
use logging;

use crate::{
  component::Component,
  events::{
    Button,
    ComponentEvent,
    Events,
    Key,
    Mouse,
    RuntimeEvent,
    WindowEvent,
  },
  render::{
    window::{
      Window,
      WindowBuilder,
    },
    RenderContext,
    RenderContextBuilder,
  },
  runtime::Runtime,
};

#[derive(Clone, Debug)]
pub enum ComponentResult {
  Success,
  Failure,
}

pub struct ApplicationRuntimeBuilder {
  app_name: String,
  render_context_builder: RenderContextBuilder,
  window_builder: WindowBuilder,
  components: Vec<Box<dyn Component<ComponentResult, String>>>,
}

impl ApplicationRuntimeBuilder {
  pub fn new(app_name: &str) -> Self {
    return Self {
      app_name: app_name.to_string(),
      render_context_builder: RenderContextBuilder::new(app_name),
      window_builder: WindowBuilder::new(),
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
    let name = self.app_name;
    let mut event_loop = LoopBuilder::new().build();
    let window = self.window_builder.build(&mut event_loop);

    let component_stack = self.components;
    let render_context = self.render_context_builder.build(&window);

    return ApplicationRuntime {
      name,
      event_loop,
      window,
      render_context,
      component_stack,
    };
  }
}

/// A windowed and event-driven runtime that can be used to render a
/// scene on the primary GPU across Windows, MacOS, and Linux.
pub struct ApplicationRuntime {
  name: String,
  event_loop: Loop<Events>,
  window: Window,
  component_stack: Vec<Box<dyn Component<ComponentResult, String>>>,
  render_context: RenderContext,
}

impl ApplicationRuntime {}

impl Runtime<(), String> for ApplicationRuntime {
  type Component = Box<dyn Component<ComponentResult, String>>;
  /// Runs the event loop for the Application Runtime which takes ownership
  /// of all components, the windowing the render context, and anything
  /// else relevant to the runtime.
  fn run(self) -> Result<(), String> {
    // Decompose Runtime components to transfer ownership from the runtime to
    // the event loop closure which will run until the app is closed.
    let ApplicationRuntime {
      window,
      mut event_loop,
      mut component_stack,
      name,
      render_context,
    } = self;

    let mut active_render_context = Some(render_context);

    let publisher = event_loop.create_event_publisher();
    publisher.publish_event(Events::Runtime {
      event: RuntimeEvent::Initialized,
      issued_at: Instant::now(),
    });

    let mut current_frame = Instant::now();
    let mut runtime_result: Box<Result<(), String>> = Box::new(Ok(()));

    event_loop.run_forever(move |event, _, control_flow| {
      let mapped_event: Option<Events> = match event {
        WinitEvent::WindowEvent { event, .. } => match event {
          WinitWindowEvent::CloseRequested => {
            // Issue a Shutdown event to deallocate resources and clean up.
            control_flow.set_exit();
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
          WinitWindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
            active_render_context
              .as_mut()
              .unwrap()
              .resize(new_inner_size.width, new_inner_size.height);

            Some(Events::Window {
              event: WindowEvent::Resize {
                width: new_inner_size.width,
                height: new_inner_size.height,
              },
              issued_at: Instant::now(),
            })
          }
          WinitWindowEvent::Moved(_) => None,
          WinitWindowEvent::Destroyed => None,
          WinitWindowEvent::DroppedFile(_) => None,
          WinitWindowEvent::HoveredFile(_) => None,
          WinitWindowEvent::HoveredFileCancelled => None,
          WinitWindowEvent::ReceivedCharacter(_) => None,
          WinitWindowEvent::Focused(_) => None,
          WinitWindowEvent::KeyboardInput {
            device_id: _,
            input,
            is_synthetic,
          } => match (input.state, is_synthetic) {
            (ElementState::Pressed, false) => Some(Events::Keyboard {
              event: Key::Pressed {
                scan_code: input.scancode,
                virtual_key: input.virtual_keycode,
              },
              issued_at: Instant::now(),
            }),
            (ElementState::Released, false) => Some(Events::Keyboard {
              event: Key::Released {
                scan_code: input.scancode,
                virtual_key: input.virtual_keycode,
              },
              issued_at: Instant::now(),
            }),
            _ => {
              logging::warn!("Unhandled synthetic keyboard event: {:?}", input);
              None
            }
          },
          WinitWindowEvent::ModifiersChanged(_) => None,
          WinitWindowEvent::CursorMoved {
            device_id,
            position,
            modifiers,
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
          WinitWindowEvent::CursorEntered { device_id } => {
            Some(Events::Mouse {
              event: Mouse::EnteredWindow { device_id: 0 },
              issued_at: Instant::now(),
            })
          }
          WinitWindowEvent::CursorLeft { device_id } => Some(Events::Mouse {
            event: Mouse::LeftWindow { device_id: 0 },
            issued_at: Instant::now(),
          }),
          WinitWindowEvent::MouseWheel {
            device_id,
            delta,
            phase,
            modifiers,
          } => Some(Events::Mouse {
            event: Mouse::Scrolled { device_id: 0 },
            issued_at: Instant::now(),
          }),
          WinitWindowEvent::MouseInput {
            device_id,
            state,
            button,
            modifiers,
          } => {
            // Map winit button to our button type
            let button = match button {
              MouseButton::Left => Button::Left,
              MouseButton::Right => Button::Right,
              MouseButton::Middle => Button::Middle,
              MouseButton::Other(other) => Button::Other(other),
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
          WinitWindowEvent::TouchpadPressure {
            device_id,
            pressure,
            stage,
          } => None,
          WinitWindowEvent::AxisMotion {
            device_id,
            axis,
            value,
          } => None,
          WinitWindowEvent::Touch(_) => None,
          WinitWindowEvent::ThemeChanged(_) => None,
          _ => None,
        },
        WinitEvent::MainEventsCleared => {
          let last_frame = current_frame.clone();
          current_frame = Instant::now();
          let duration = &current_frame.duration_since(last_frame);

          let active_render_context = active_render_context
            .as_mut()
            .expect("Couldn't get the active render context. ");
          for component in &mut component_stack {
            component.on_update(duration);
            let commands = component.on_render(active_render_context);
            active_render_context.render(commands);
          }

          // Warn if frames dropped below 32 ms (30 fps).
          match duration.as_millis() > 32 {
            true => {
              logging::warn!(
                "Frame took too long to render: {:?} ms",
                duration.as_millis()
              );
            }
            false => {
              // Disable until frametimes can be determined via monitor
              // std::thread::sleep(std::time::Duration::from_millis(16 - duration.as_millis() as u64));
            }
          }

          None
        }
        WinitEvent::RedrawRequested(_) => None,
        WinitEvent::NewEvents(_) => None,
        WinitEvent::DeviceEvent { device_id, event } => None,
        WinitEvent::UserEvent(lambda_event) => match lambda_event {
          Events::Runtime { event, issued_at } => match event {
            RuntimeEvent::Initialized => {
              logging::debug!(
                "Initializing all of the components for the runtime: {}",
                name
              );
              for component in &mut component_stack {
                component.on_attach(active_render_context.as_mut().unwrap());
              }
              None
            }
            RuntimeEvent::Shutdown => {
              for component in &mut component_stack {
                component.on_detach(active_render_context.as_mut().unwrap());
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
        WinitEvent::RedrawEventsCleared => None,
        WinitEvent::LoopDestroyed => {
          active_render_context
            .take()
            .expect("[ERROR] The render API has been already taken.")
            .destroy();

          logging::info!("All resources were successfully deleted.");
          None
        }
      };

      match mapped_event {
        Some(event) => {
          logging::trace!("Sending event: {:?} to all components", event);

          for component in &mut component_stack {
            let event_result = component.on_event(event.clone());
            match event_result {
              Ok(_) => {}
              Err(e) => {
                let error = format!(
                  "A component has panicked while handling an event. {:?}",
                  e
                );
                logging::error!(
                  "A component has panicked while handling an event. {:?}",
                  e
                );
                publisher.publish_event(Events::Runtime {
                  event: RuntimeEvent::ComponentPanic { message: error },
                  issued_at: Instant::now(),
                });
              }
            }
          }
        }
        None => {}
      }
    });
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
