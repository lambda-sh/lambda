use std::time::Instant;

use winit::{
  event::{
    Event,
    WindowEvent,
  },
  event_loop::ControlFlow,
};

use super::{
  event_loop::{
    LambdaEvent,
    LambdaEventLoop,
  },
  layer::{
    Component,
    ComponentStack,
  },
  render::RenderAPI,
  window::{
    LambdaWindow,
    Window,
  },
};

pub trait Runnable {
  fn setup(&mut self);
  fn run(self);
}

/// LambdaRunnable is a pre configured composition of a generic set of
/// components from the lambda-rs codebase
pub struct LambdaRunnable {
  name: String,
  window: LambdaWindow,
  event_loop: LambdaEventLoop,
  component_stack: ComponentStack,
  renderer: RenderAPI,
}

impl LambdaRunnable {
  pub fn with_component_attached<T: Default + Component + 'static>(
    mut self,
  ) -> Self {
    self.component_stack.push_component::<T>();
    return self;
  }
}

impl Default for LambdaRunnable {
  /// Constructs a LambdaRunanble with an event loop for publishing events to
  /// the application, a window with a renderable surface, a layer stack for
  /// storing layers into the engine.
  fn default() -> Self {
    let name = String::from("LambdaRunnable");
    let event_loop = LambdaEventLoop::new();
    let window = LambdaWindow::new().with_event_loop(&event_loop);
    let component_stack = ComponentStack::new();
    let renderer = RenderAPI::new(&name, Some(&window));

    return LambdaRunnable {
      name,
      window,
      event_loop,
      component_stack,
      renderer,
    };
  }
}

impl Runnable for LambdaRunnable {
  /// One setup to initialize the
  fn setup(&mut self) {
    let publisher = self.event_loop.create_publisher();
    publisher.send_event(LambdaEvent::Initialized);
  }

  /// Initiates an event loop that captures the context of the LambdaRunnable
  /// and generates events from the windows event loop until the end of an
  /// applications lifetime.
  fn run(self) {
    // Decompose Runnable components for transferring ownership to the
    // closure.
    let app = self;
    let publisher = app.event_loop.create_publisher();
    let event_loop = app.event_loop;
    let window = app.window;
    let mut component_stack = app.component_stack;
    let mut renderer = app.renderer;

    let mut last_frame = Instant::now();
    let mut current_frame = Instant::now();

    event_loop.run_forever(move |event, _, control_flow| match event {
      Event::WindowEvent { event, .. } => match event {
        WindowEvent::CloseRequested => {
          // Issue a Shutdown event to deallocate resources and clean up.
          publisher.send_event(LambdaEvent::Shutdown)
        }
        WindowEvent::Resized(dims) => {
          publisher.send_event(LambdaEvent::Resized {
            new_width: dims.width,
            new_height: dims.height,
          })
        }
        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => publisher
          .send_event(LambdaEvent::Resized {
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
      Event::MainEventsCleared => {
        last_frame = current_frame.clone();
        current_frame = Instant::now();
        let duration = &current_frame.duration_since(last_frame);
        renderer.on_update(duration);

        component_stack.on_update(duration);
      }
      Event::RedrawRequested(_) => {
        window.redraw();
      }
      Event::NewEvents(_) => {}
      Event::DeviceEvent { device_id, event } => {}
      Event::UserEvent(lambda_event) => {
        match lambda_event {
          LambdaEvent::Initialized => {
            component_stack.attach();
            renderer.attach();
          }
          LambdaEvent::Shutdown => {
            // Once this has been set, the ControlFlow can no longer be
            // modified.
            *control_flow = ControlFlow::Exit;
          }
          _ => {
            renderer.on_event(&lambda_event);
            component_stack.on_event(&lambda_event);
          }
        }
      }
      Event::Suspended => {}
      Event::Resumed => {}
      Event::RedrawEventsCleared => {}
      Event::LoopDestroyed => {
        component_stack.detach();
        renderer.detach();
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

/// Builds & executes a Runnable all in one good. This is useful for when you
/// don't need to execute any code in between the building & execution stage of
/// the runnable
pub fn build_and_start_runnable<T: Default + Runnable>() {
  let app = T::default();

  start_runnable(app);
}

/// Simple function for starting any prebuilt Runnable.
pub fn start_runnable<T: Runnable>(mut app: T) {
  app.setup();
  app.run();
}
