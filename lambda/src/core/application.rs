use std::time::{Duration, Instant};

use winit::{
    event::Event,
    event::WindowEvent,
    event_loop::ControlFlow
};

use crate::{
    core::event_loop::LambdaEvent,
    core::event_loop::LambdaEventLoop,
    core::window::Window,
    core::window::LambdaWindow
};


pub trait Layer {
    fn attach(&self);
    fn detach(&self);
    fn on_event(&self, event: &LambdaEvent);
    fn on_update(&self, last_frame: &Duration);
}

pub struct DefaultLayer {}

impl Layer for DefaultLayer {
    fn attach(&self) {
        println!("Attached to the lambda application");
    }

    fn detach(&self) {
        todo!()
    }

    fn on_event(&self, event: &LambdaEvent) {
    }

    fn on_update(&self, last_frame: &Duration) {
        println!("{} since last tick", last_frame.as_millis());
    }
}

impl Default for DefaultLayer {
    fn default() -> Self {
        return DefaultLayer{};
    }
}

pub struct LayerStack {
    layers: Vec<Box<dyn Layer + 'static>>
}

impl LayerStack {
    pub fn new() -> Self {
        return LayerStack{
            layers: Vec::new()
        };
    }

    pub fn push_layer<T>(&mut self) 
        where T: Default + Layer + 'static {
            let layer = Box::new(T::default());
            self.layers.push(layer);
    }

    pub fn pop_layer(&mut self) -> Option<Box<dyn Layer + 'static>> {
        let layer = self.layers.pop();
        return layer; 
    }

    pub fn on_event(&self, event: &LambdaEvent) {
        for layer in &self.layers {
            layer.on_event(&event);
        }
    }

    pub fn on_update(&self, last_frame: &Duration) {
        for layer in &self.layers {
            layer.on_update(last_frame);
        }
    }
}

pub trait Runnable {
    fn setup(&self);
    fn run(self);
}

pub struct LambdaRunnable {
    name: String,
    window: LambdaWindow,
    event_loop: LambdaEventLoop,
    layer_stack: LayerStack,
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

        return LambdaRunnable{
            name,
            window,
            event_loop,
            layer_stack
        }
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

        let mut last_frame= Instant::now();
        let mut current_frame = Instant::now();

        event_loop.run_forever(
                move |
                        event, 
                        _, 
                        control_flow| {
                match event {
                    Event::WindowEvent { event, .. } => {
                        match event {
                            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                            WindowEvent::Resized(dims) => {},
                            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => { },
                            WindowEvent::Moved(_) => {},
                            WindowEvent::Destroyed => {},
                            WindowEvent::DroppedFile(_) => {},
                            WindowEvent::HoveredFile(_) => {},
                            WindowEvent::HoveredFileCancelled => {},
                            WindowEvent::ReceivedCharacter(_) => {},
                            WindowEvent::Focused(_) => {},
                            WindowEvent::KeyboardInput { device_id, input, is_synthetic } => {},
                            WindowEvent::ModifiersChanged(_) => {},
                            WindowEvent::CursorMoved { device_id, position, modifiers } => {},
                            WindowEvent::CursorEntered { device_id } => {},
                            WindowEvent::CursorLeft { device_id } => {},
                            WindowEvent::MouseWheel { device_id, delta, phase, modifiers } => {},
                            WindowEvent::MouseInput { device_id, state, button, modifiers } => {},
                            WindowEvent::TouchpadPressure { device_id, pressure, stage } => {},
                            WindowEvent::AxisMotion { device_id, axis, value } => {},
                            WindowEvent::Touch(_) => {},
                            WindowEvent::ThemeChanged(_) => {},
                        }
                    },
                    Event::MainEventsCleared => { 
                        last_frame = current_frame;
                        current_frame = Instant::now();
                        layer_stack.on_update(&current_frame.duration_since(last_frame));
                    },
                    Event::RedrawRequested(_) => {
                        window.redraw();
                    }
                    Event::NewEvents(_) => {

                    },
                    Event::DeviceEvent { device_id, event } => {},
                    Event::UserEvent(lambda_event) => {
                        match lambda_event {
                            LambdaEvent::Initialized => {
                                println!("Initialized Lambda");
                            }
                            LambdaEvent::Shutdown => todo!(),
                        }
                    },
                    Event::Suspended => {},
                    Event::Resumed => {},
                    Event::RedrawEventsCleared => {},
                    Event::LoopDestroyed => {},
                }

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