use crate::platform::winit::create_event_loop;
use std::time::Instant;

use winit::{
  event::{Event as WinitEvent, WindowEvent},
  event_loop::ControlFlow,
};

use crate::platform::winit::Loop;

use super::core::{
  component::Component, event_loop::Event, runnable::Runnable,
};

use super::components::{ComponentStack, Renderer, Window};

///
/// LambdaRunnable is a pre configured composition of a generic set of
/// components from the lambda-rs codebase

pub struct LambdaRunnable {
  name: String,
  event_loop: Loop<Event>,
  window: Window,
  component_stack: ComponentStack,
  renderer: Renderer,
}

impl LambdaRunnable {
  /// Set the name for the current runnable
  pub fn with_name(mut self, name: &str) -> Self {
    self.name = String::from(name);
    return self;
  }

  /// Attach a component to the current runnable.
  pub fn with_component<T: Default + Component + 'static>(
    self,
    configure_component: impl FnOnce(Self, T) -> (Self, T),
  ) -> Self {
    let (mut runnable, component) = configure_component(self, T::default());
    runnable.component_stack.push_component(component);
    return runnable;
  }

  /// Attaches an active renderer to the runnable.
  pub fn with_renderable_component<T: Default + Component + 'static>(
    mut self,
    configure_component: impl FnOnce(&Renderer, T) -> T,
  ) -> Self {
    let component = T::default();
    let component = configure_component(&self.renderer, component);
    self.component_stack.push_component(component);
    return self;
  }
}

impl Default for LambdaRunnable {
  /// Constructs a LambdaRunanble with an event loop for publishing events to
  /// the application, a window with a renderable surface, a layer stack for
  /// storing layers into the engine.
  fn default() -> Self {
    let name = String::from("LambdaRunnable");
    let mut event_loop = create_event_loop::<Event>();
    let window = Window::new(name.as_str(), [480, 360], &mut event_loop);
    let component_stack = ComponentStack::new();
    let renderer = Renderer::new(name.as_str(), &window);

    return LambdaRunnable {
      name,
      event_loop,
      window,
      component_stack,
      renderer,
    };
  }
}

impl Runnable for LambdaRunnable {
  /// One setup to initialize the
  fn setup(&mut self) {}

  /// Initiates an event loop that captures the context of the LambdaRunnable
  /// and generates events from the windows event loop until the end of an
  /// applications lifetime.
  fn run(self) {
    // Decompose Runnable components for transferring ownership to the
    // closure.
    let app = self;
    let mut window = app.window;
    let mut event_loop = app.event_loop;

    // TODO(vmarcella): The renderer should most likely just act as
    let mut component_stack = app.component_stack;
    let mut renderer = app.renderer;

    let publisher = event_loop.create_publisher();

    let mut last_frame = Instant::now();
    let mut current_frame = Instant::now();

    event_loop.run_forever(move |event, _, control_flow| match event {
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
        component_stack.on_update(duration);
        renderer.on_update(duration);
      }
      WinitEvent::RedrawRequested(_) => {
        window.redraw();
      }
      WinitEvent::NewEvents(_) => {}
      WinitEvent::DeviceEvent { device_id, event } => {}
      WinitEvent::UserEvent(lambda_event) => {
        match lambda_event {
          Event::Initialized => {
            component_stack.on_attach();
            renderer.on_attach();
          }
          Event::Shutdown => {
            // Once this has been set, the ControlFlow can no longer be
            // modified.
            *control_flow = ControlFlow::Exit;
          }
          _ => {
            component_stack.on_event(&lambda_event);
            renderer.on_event(&lambda_event);
          }
        }
      }
      WinitEvent::Suspended => {}
      WinitEvent::Resumed => {}
      WinitEvent::RedrawEventsCleared => {}
      WinitEvent::LoopDestroyed => {
        component_stack.on_detach();
        renderer.on_detach();
      }
    });
  }
}

/// Create a generic lambda runnable. This provides you a Runnable
/// Application Instance that can be hooked into through attaching
/// a Layer.
pub fn create_lambda_runnable() -> LambdaRunnable {
  return LambdaRunnable::default();
}
