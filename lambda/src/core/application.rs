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
  render::LambdaRenderer,
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
  renderer: LambdaRenderer<backend::Backend>,
}

impl LambdaRunnable {
  pub fn with_layer_attached<T: Default + Layer + 'static>(mut self) -> Self {
    self.layer_stack.push_layer::<T>();
    return self;
  }
}

impl Default for LambdaRunnable {
  fn default() -> Self {
    let name = String::from("LambdaRunnable");
    let event_loop = LambdaEventLoop::new();
    let window = LambdaWindow::new().with_event_loop(&event_loop);
    let layer_stack = LayerStack::new();
    let renderer = LambdaRenderer::new(&name, Some(&window));

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

  fn run(self) {
    // Decompose Runnable components for transferring ownership to the
    // closure.
    let app = self;
    let event_loop = app.event_loop;
    let window = app.window;
    let layer_stack = app.layer_stack;

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
        layer_stack.on_update(&current_frame.duration_since(last_frame));
      }
      Event::RedrawRequested(_) => {
        window.redraw();
      }
      Event::NewEvents(_) => {}
      Event::DeviceEvent { device_id, event } => {}
      Event::UserEvent(lambda_event) => match lambda_event {
        LambdaEvent::Initialized => {
          println!("Initialized Lambda");
        }
        LambdaEvent::Shutdown => todo!(),
      },
      Event::Suspended => {}
      Event::Resumed => {}
      Event::RedrawEventsCleared => {}
      Event::LoopDestroyed => {}
    });
  }
}

pub fn create_lambda_runnable() -> LambdaRunnable {
  return LambdaRunnable::default();
}

pub fn build_and_start_runnable<T: Default + Runnable>() {
  let app = T::default();

  start_runnable(app);
}

pub fn start_runnable<T: Runnable>(app: T) {
  app.setup();
  app.run();
}
