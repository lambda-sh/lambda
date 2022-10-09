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
  component::RenderableComponent,
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
  render_api: RenderContextBuilder,
  window_size: (u32, u32),
  components: Vec<Box<dyn RenderableComponent<Events>>>,
}

impl GenericRuntimeBuilder {
  pub fn new(app_name: &str) -> Self {
    return Self {
      app_name: app_name.to_string(),
      render_api: RenderContextBuilder::new(app_name),
      window_size: (800, 600),
      components: Vec::new(),
    };
  }

  /// Update the name of the LambdaKernel.
  pub fn with_app_name(mut self, name: &str) -> Self {
    self.app_name = name.to_string();
    return self;
  }

  pub fn with_window_size(mut self, width: u32, height: u32) -> Self {
    self.window_size = (width, height);
    return self;
  }

  /// Configures the RenderAPIBuilder before the RenderingAPI is built using a
  /// callback provided by the user.
  pub fn with_renderer(
    mut self,
    configure: impl FnOnce(RenderContextBuilder) -> RenderContextBuilder,
  ) -> Self {
    self.render_api = configure(self.render_api);
    return self;
  }
  /// Attach a component to the current runnable.
  pub fn with_component<T: Default + RenderableComponent<Events> + 'static>(
    self,
    configure_component: impl FnOnce(Self, T) -> (Self, T),
  ) -> Self {
    let (mut kernel_builder, component) =
      configure_component(self, T::default());
    kernel_builder.components.push(Box::new(component));
    return kernel_builder;
  }

  /// Builds a LambdaKernel equipped with Windowing, an event loop, and a
  /// component stack that allows components to be dynamically pushed into the
  /// Kernel to receive events & render access.
  pub fn build(self) -> GenericRuntime {
    let name = self.app_name;
    let mut event_loop = LoopBuilder::new().build();
    let (width, height) = self.window_size;

    let window = WindowBuilder::new()
      .with_name(name.as_str())
      .with_dimensions(width, height)
      .build(&mut event_loop);
    let component_stack = self.components;
    let render_api = self.render_api.build(&window);

    return GenericRuntime {
      name,
      event_loop,
      window,
      render_api,
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
  component_stack: Vec<Box<dyn RenderableComponent<Events>>>,
  render_api: RenderContext,
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
      render_api,
    } = self;

    let mut active_render_api = Some(render_api);

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
          }
          WinitWindowEvent::Resized(dims) => {
            publisher.publish_event(Events::Window {
              event: WindowEvent::Resize {
                width: dims.width,
                height: dims.height,
              },
              issued_at: Instant::now(),
            })
          }
          WinitWindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
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
              println!("Unhandled synthetic keyboard event: {:?}", input);
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
        },
        WinitEvent::MainEventsCleared => {
          let last_frame = current_frame.clone();
          current_frame = Instant::now();
          let duration = &current_frame.duration_since(last_frame);

          let render_api = active_render_api.as_mut().unwrap();
          // Update and render commands.
          for component in &mut component_stack {
            component.on_update(duration);
            let commands = component.on_render(render_api, duration);
            render_api.render(commands);
          }

          window.redraw();
        }
        WinitEvent::RedrawRequested(_) => {}
        WinitEvent::NewEvents(_) => {}
        WinitEvent::DeviceEvent { device_id, event } => {}
        WinitEvent::UserEvent(lambda_event) => match lambda_event {
          Events::Runtime { event, issued_at } => match event {
            RuntimeEvent::Initialized => {
              println!("Starting the kernel {}", name);
              for component in &mut component_stack {
                component.on_attach();
                component
                  .on_renderer_attached(active_render_api.as_mut().unwrap());
              }
            }
            RuntimeEvent::Shutdown => {
              for component in &mut component_stack {
                component.on_detach();
                component
                  .on_renderer_detached(active_render_api.as_mut().unwrap());
              }
              *control_flow = ControlFlow::Exit;
            }
          },
          _ => {
            for component in &mut component_stack {
              component.on_event(&lambda_event);
            }
          }
        },
        WinitEvent::Suspended => {}
        WinitEvent::Resumed => {}
        WinitEvent::RedrawEventsCleared => {}
        WinitEvent::LoopDestroyed => {
          active_render_api.take().unwrap().destroy();

          println!("All resources were successfully deleted.");
        }
      }
    });
  }

  /// When the generic runtime starts, it will attach all of the components that
  /// have been added during the construction phase in the users code.
  fn on_start(&mut self) {
    println!("Starting the runtime {}", self.name);
  }

  fn on_stop(&mut self) {
    println!("Stopping {}", self.name)
  }
}
