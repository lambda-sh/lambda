use std::time::Instant;

use lambda_platform::winit::{
  create_event_loop,
  winit_exports::{
    ControlFlow,
    Event as WinitEvent,
    WindowEvent,
  },
  Loop,
};

use crate::core::{
  component::RenderableComponent,
  events::Event,
  kernel::Kernel,
  render::{
    window::{
      Window,
      WindowBuilder,
    },
    RenderContext,
    RenderContextBuilder,
  },
};

pub struct LambdaKernelBuilder {
  name: String,
  render_api: RenderContextBuilder,
}

impl LambdaKernelBuilder {
  pub fn new(name: &str) -> Self {
    return Self {
      name: name.to_string(),
      render_api: RenderContextBuilder::new(name),
    };
  }

  /// Update the name of the LambdaKernel.
  pub fn with_name(mut self, name: &str) -> Self {
    self.name = name.to_string();
    return self;
  }

  /// Configures the RenderAPIBuilder before the RenderingAPI is built using a
  /// callback provided by the user.
  pub fn configure_renderer(
    mut self,
    configure: impl FnOnce(RenderContextBuilder) -> RenderContextBuilder,
  ) -> Self {
    self.render_api = configure(self.render_api);
    return self;
  }

  /// Builds a LambdaKernel equipped with Windowing, an event loop, and a
  /// component stack that allows components to be dynamically pushed into the
  /// Kernel to receive events & render access.
  pub fn build(self) -> LambdaKernel {
    let name = self.name;
    let mut event_loop = create_event_loop::<Event>();
    let window = WindowBuilder::new()
      .with_name(name.as_str())
      .build(&mut event_loop);
    let component_stack = Vec::new();
    let render_api = self.render_api.build(&window);

    return LambdaKernel {
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
pub struct LambdaKernel {
  name: String,
  event_loop: Loop<Event>,
  window: Window,
  component_stack: Vec<Box<dyn RenderableComponent<Event>>>,
  render_api: RenderContext,
}

impl LambdaKernel {
  /// Attach a component to the current runnable.
  pub fn with_component<T: Default + RenderableComponent<Event> + 'static>(
    self,
    configure_component: impl FnOnce(Self, T) -> (Self, T),
  ) -> Self {
    let (mut kernel, component) = configure_component(self, T::default());
    kernel.component_stack.push(Box::new(component));
    return kernel;
  }
}

impl Kernel for LambdaKernel {
  /// Initiates an event loop that captures the context of the LambdaKernel
  /// and generates events from the windows event loop until the end of the event loops
  /// lifetime (Whether that be initiated intentionally or via error).
  fn run(self) {
    // Decompose Kernel components for transferring ownership to the
    // closure.
    let LambdaKernel {
      mut window,
      mut event_loop,
      mut component_stack,
      name,
      render_api,
    } = self;

    let mut active_render_api = Some(render_api);

    let publisher = event_loop.create_publisher();
    publisher.send_event(Event::Initialized);

    let mut last_frame = Instant::now();
    let mut current_frame = Instant::now();

    event_loop.run_forever(move |event, _, control_flow| {
      match event {
        WinitEvent::WindowEvent { event, .. } => match event {
          WindowEvent::CloseRequested => {
            // Issue a Shutdown event to deallocate resources and clean up.
            publisher.send_event(Event::Shutdown);
          }
          WindowEvent::Resized(dims) => publisher.send_event(Event::Resized {
            new_width: dims.width,
            new_height: dims.height,
          }),
          WindowEvent::ScaleFactorChanged { new_inner_size, .. } => publisher
            .send_event(Event::Resized {
              new_width: new_inner_size.width,
              new_height: new_inner_size.height,
            }),
          WindowEvent::Moved(_) => {}
          WindowEvent::Destroyed => {}
          WindowEvent::DroppedFile(_) => {}
          WindowEvent::HoveredFile(_) => {}
          WindowEvent::HoveredFileCancelled => {}
          WindowEvent::ReceivedCharacter(_) => {}
          WindowEvent::Focused(_) => {}
          WindowEvent::KeyboardInput {
            device_id,
            input,
            is_synthetic,
          } => {}
          WindowEvent::ModifiersChanged(_) => {}
          WindowEvent::CursorMoved {
            device_id,
            position,
            modifiers,
          } => {}
          WindowEvent::CursorEntered { device_id } => {}
          WindowEvent::CursorLeft { device_id } => {}
          WindowEvent::MouseWheel {
            device_id,
            delta,
            phase,
            modifiers,
          } => {}
          WindowEvent::MouseInput {
            device_id,
            state,
            button,
            modifiers,
          } => {}
          WindowEvent::TouchpadPressure {
            device_id,
            pressure,
            stage,
          } => {}
          WindowEvent::AxisMotion {
            device_id,
            axis,
            value,
          } => {}
          WindowEvent::Touch(_) => {}
          WindowEvent::ThemeChanged(_) => {}
        },
        WinitEvent::MainEventsCleared => {
          last_frame = current_frame.clone();
          current_frame = Instant::now();
          let duration = &current_frame.duration_since(last_frame);

          // Update and render commands.
          for component in &mut component_stack {
            component.on_update(duration);
            let commands = component
              .on_render(active_render_api.as_mut().unwrap(), duration);
            active_render_api.as_mut().unwrap().render(commands);
          }

          window.redraw();
        }
        WinitEvent::RedrawRequested(_) => {}
        WinitEvent::NewEvents(_) => {}
        WinitEvent::DeviceEvent { device_id, event } => {}
        WinitEvent::UserEvent(lambda_event) => {
          match lambda_event {
            Event::Initialized => {}
            Event::Shutdown => {
              // Once this has been set, the ControlFlow can no longer be
              // modified.

              println!("Detaching all components.");
              *control_flow = ControlFlow::Exit;
            }
            _ => {
              for component in &mut component_stack {
                component.on_event(&lambda_event);
              }
            }
          }
        }
        WinitEvent::Suspended => {}
        WinitEvent::Resumed => {}
        WinitEvent::RedrawEventsCleared => {}
        WinitEvent::LoopDestroyed => {
          for component in &mut component_stack {
            component.on_detach();
            component.on_renderer_detached(active_render_api.as_mut().unwrap());
          }
          active_render_api.take().unwrap().destroy();

          println!("All resources were successfully deleted.");
        }
      }
    });
  }

  fn on_start(&mut self) {
    println!("Starting {}", self.name);
    for component in &mut self.component_stack {
      component.on_attach();
      component.on_renderer_attached(&mut self.render_api);
    }
  }

  fn on_stop(&mut self) {
    println!("Stopping {}", self.name)
  }
}
