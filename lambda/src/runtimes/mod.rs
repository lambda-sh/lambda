use std::time::Instant;

use lambda_platform::winit::{
  winit_exports::{
    ControlFlow,
    ElementState,
    Event as WinitEvent,
    WindowEvent as WinitWindowEvent,
  },
  Loop,
  LoopBuilder,
};

use crate::core::{
  component::Component,
  events::{
    ComponentEvent,
    Events,
    KeyEvent,
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

pub struct GenericRuntimeBuilder {
  app_name: String,
  render_context_builder: RenderContextBuilder,
  window_builder: WindowBuilder,
  components: Vec<Box<dyn Component>>,
}

impl GenericRuntimeBuilder {
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
  pub fn with_component<T: Default + Component + 'static>(
    self,
    configure_component: impl FnOnce(Self, T) -> (Self, T),
  ) -> Self {
    let (mut kernel_builder, component) =
      configure_component(self, T::default());
    kernel_builder.components.push(Box::new(component));
    return kernel_builder;
  }

  /// Builds a GenericRuntime equipped with Windowing, an event loop, and a
  /// component stack that allows components to be dynamically pushed into the
  /// Kernel to receive events & render access.
  pub fn build(self) -> GenericRuntime {
    let name = self.app_name;
    let mut event_loop = LoopBuilder::new().build();
    let window = self.window_builder.build(&mut event_loop);

    let component_stack = self.components;
    let render_context = self.render_context_builder.build(&window);

    return GenericRuntime {
      name,
      event_loop,
      window,
      render_context,
      component_stack,
    };
  }
}

/// A windowed and event-driven kernel that can be used to render a
/// scene on the primary GPU across Windows, MacOS, and Linux at this point in
/// time.
pub struct GenericRuntime {
  name: String,
  event_loop: Loop<Events>,
  window: Window,
  component_stack: Vec<Box<dyn Component>>,
  render_context: RenderContext,
}

impl GenericRuntime {}

impl Runtime for GenericRuntime {
  /// Runs the event loop for the GenericRuntime which takes ownership of all
  /// components, the windowing the render context, and anything else relevant
  /// to the runtime.
  fn run(self) {
    // Decompose Runtime components to transfer ownership from the runtime to
    // the event loop closure which will run until the app is closed.
    let GenericRuntime {
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

    event_loop.run_forever(move |event, _, control_flow| {
      match event {
        WinitEvent::WindowEvent { event, .. } => match event {
          WinitWindowEvent::CloseRequested => {
            // Issue a Shutdown event to deallocate resources and clean up.
            publisher.publish_event(Events::Runtime {
              event: RuntimeEvent::Shutdown,
              issued_at: Instant::now(),
            });

            *control_flow = ControlFlow::Exit;
          }
          WinitWindowEvent::Resized(dims) => {
            active_render_context
              .as_mut()
              .unwrap()
              .resize(dims.width, dims.height);

            publisher.publish_event(Events::Window {
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

            publisher.publish_event(Events::Window {
              event: WindowEvent::Resize {
                width: new_inner_size.width,
                height: new_inner_size.height,
              },
              issued_at: Instant::now(),
            })
          }
          WinitWindowEvent::Moved(_) => {}
          WinitWindowEvent::Destroyed => {}
          WinitWindowEvent::DroppedFile(_) => {}
          WinitWindowEvent::HoveredFile(_) => {}
          WinitWindowEvent::HoveredFileCancelled => {}
          WinitWindowEvent::ReceivedCharacter(_) => {}
          WinitWindowEvent::Focused(_) => {}
          WinitWindowEvent::KeyboardInput {
            device_id: _,
            input,
            is_synthetic,
          } => match (input.state, is_synthetic) {
            (ElementState::Pressed, false) => {
              publisher.publish_event(Events::Keyboard {
                event: KeyEvent::KeyPressed {
                  scan_code: input.scancode,
                  virtual_key: input.virtual_keycode,
                },
                issued_at: Instant::now(),
              })
            }
            (ElementState::Released, false) => {
              publisher.publish_event(Events::Keyboard {
                event: KeyEvent::KeyReleased {
                  scan_code: input.scancode,
                  virtual_key: input.virtual_keycode,
                },
                issued_at: Instant::now(),
              })
            }
            _ => {
              println!(
                "[WARN] Unhandled synthetic keyboard event: {:?}",
                input
              );
            }
          },
          WinitWindowEvent::ModifiersChanged(_) => {}
          WinitWindowEvent::CursorMoved {
            device_id,
            position,
            modifiers,
          } => {}
          WinitWindowEvent::CursorEntered { device_id } => {}
          WinitWindowEvent::CursorLeft { device_id } => {}
          WinitWindowEvent::MouseWheel {
            device_id,
            delta,
            phase,
            modifiers,
          } => {}
          WinitWindowEvent::MouseInput {
            device_id,
            state,
            button,
            modifiers,
          } => {}
          WinitWindowEvent::TouchpadPressure {
            device_id,
            pressure,
            stage,
          } => {}
          WinitWindowEvent::AxisMotion {
            device_id,
            axis,
            value,
          } => {}
          WinitWindowEvent::Touch(_) => {}
          WinitWindowEvent::ThemeChanged(_) => {}
          _ => {}
        },
        WinitEvent::MainEventsCleared => {
          let last_frame = current_frame.clone();
          current_frame = Instant::now();
          let duration = &current_frame.duration_since(last_frame);

          // Update and render commands.
          for component in &mut component_stack {
            component.on_update(duration);
          }

          window.redraw();
        }
        WinitEvent::RedrawRequested(_) => {
          for component in &mut component_stack {
            let commands = component.on_render(active_render_context.as_mut().unwrap());
            active_render_context.as_mut().unwrap().render(commands);
          }
        }
        WinitEvent::NewEvents(_) => {}
        WinitEvent::DeviceEvent { device_id, event } => {}
        WinitEvent::UserEvent(lambda_event) => match lambda_event {
          Events::Runtime { event, issued_at } => match event {
            RuntimeEvent::Initialized => {
              println!("[INFO] Initializing all of the components for the runtime: {}", name);
              for component in &mut component_stack {
                component.on_attach(active_render_context.as_mut().unwrap());
              }
            }
            RuntimeEvent::Shutdown => {
              for component in &mut component_stack {
                component.on_detach(active_render_context.as_mut().unwrap());
              }
            }
          },
          _ => {
            for component in &mut component_stack {
              component.on_event(lambda_event.clone());
            }
          }
        },
        WinitEvent::Suspended => {}
        WinitEvent::Resumed => {}
        WinitEvent::RedrawEventsCleared => {}
        WinitEvent::LoopDestroyed => {
          active_render_context
            .take()
            .expect("[ERROR] The render API has been already taken.")
            .destroy();

          println!("[INFO] All resources were successfully deleted.");
        }
      }
    });
  }

  /// When the generic runtime starts, it will attach all of the components that
  /// have been added during the construction phase in the users code.
  fn on_start(&mut self) {
    println!("[INFO] Starting the runtime {}", self.name);
  }

  fn on_stop(&mut self) {
    println!("[INFO] Stopping {}", self.name)
  }
}
