use std::borrow::Borrow;

use winit::event::{Event, WindowEvent};
use winit::event_loop::ControlFlow;

use crate::core::{
    LambdaEventLoop,
};

use crate::core::{
    Window,
    LambdaWindow,
};

use crate::core::event_loop::{EventLoopPublisher, LambdaEvent};

pub trait Runnable {
    fn setup(&self);
    fn run(self);
    fn on_update(&self);
    fn on_render(&self);
    fn on_event(&self);
}

pub struct LambdaRunnable {
    name: String,
    window: LambdaWindow,
    event_loop: LambdaEventLoop,
    running: bool,
}

impl LambdaRunnable {
    pub fn new() -> Self {
        let name = String::from("LambdaRunnable");
        let event_loop = LambdaEventLoop::new();
        let window = LambdaWindow::new().with_event_loop(&event_loop);

        return LambdaRunnable{
            name,
            window,
            event_loop,
            running: false,
        }
    }

    pub fn get_name(&self) -> String {
        return self.name.clone();
    }

    pub fn is_running(&self) -> bool {
        return self.running;
    }
}

impl Runnable for LambdaRunnable {
    /// One setup to initialize the
    fn setup(&self) {
        println!("Just hit lambda application runner setup!")
    }

    fn run(self) {
        let publisher = self.event_loop.create_publisher();
        publisher.send_event(LambdaEvent::Initialized);

        // Decompose Runnable components for transferring ownership to the 
        // closure.
        let app = self;
        let event_loop = app.event_loop;
        let window = app.window;

        event_loop.run_forever(
                move |event, windows, control_flow| {
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
                        window.winit_window().request_redraw();
                    },
                    Event::RedrawRequested(_) => {
                    }
                    Event::NewEvents(_) => {},
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
    fn on_update(&self) {

    }
    fn on_render(&self) {}
    fn on_event(&self) {}
}


pub fn create_lambda_application() -> LambdaRunnable {
    return LambdaRunnable::new();
}

pub fn start_application<T: Runnable>(app: T) {
    app.setup();
    app.run();
}
