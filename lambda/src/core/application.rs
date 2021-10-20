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
    Layer,
    LayerStack,
  },
  render::{
    LambdaRenderer,
    RenderAPI,
  },
  window::{
    LambdaWindow,
    Window,
  },
};

pub trait Runnable {
  fn setup(&self);
  fn run(self);
}

pub struct LambdaRunnable {
  name: String,
  window: LambdaWindow,
  event_loop: LambdaEventLoop,
  layer_stack: LayerStack,
  renderer: RenderAPI,
}

impl LambdaRunnable {
  pub fn with_layer_attached<T: Default + Layer + 'static>(mut self) -> Self {
    self.layer_stack.push_layer::<T>();
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
    let layer_stack = LayerStack::new();
    let renderer = RenderAPI::new(&name, Some(&window));

    return LambdaRunnable {
      name,
      window,
      event_loop,
      layer_stack,
      renderer,
    };
  }
}

impl Runnable for LambdaRunnable {
  /// One setup to initialize the
  fn setup(&self) {
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
    let event_loop = app.event_loop;
    let window = app.window;
    let mut layer_stack = app.layer_stack;
    let mut renderer = app.renderer;

    let mut last_frame = Instant::now();
    let mut current_frame = Instant::now();

    event_loop.run_forever(move |event, _, control_flow| match event {
      Event::WindowEvent { event, .. } => match event {
        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
        WindowEvent::Resized(dims) => {}
        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {}
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

        // Update all layers with the duration since the last frame &
        for layer in layer_stack.get_layers() {
          layer.on_update(duration, &mut renderer);
        }
      }
      Event::RedrawRequested(_) => {
        window.redraw();
      }
      Event::NewEvents(_) => {}
      Event::DeviceEvent { device_id, event } => {}
      Event::UserEvent(lambda_event) => match lambda_event {
        LambdaEvent::Initialized => {
          println!("Initialized Lambda");
          renderer.init();
        }
        LambdaEvent::Shutdown => {
          // TODO(vmarcella): Clean up resources during shutdown. All owned
          // resources will call into here to gracefully shutdown.
          renderer.shutdown();
        }
      },
      Event::Suspended => {}
      Event::Resumed => {}
      Event::RedrawEventsCleared => {}
      Event::LoopDestroyed => {}
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
pub fn start_runnable<T: Runnable>(app: T) {
  app.setup();
  app.run();
}
