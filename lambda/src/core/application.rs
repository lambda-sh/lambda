
use crate::core::{
    LambdaEventLoop,
};

use crate::core::{
    Window,
    LambdaWindow,
};

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
    running: bool
}

impl LambdaRunnable {
    pub fn new() -> Self {
        let event_loop = LambdaEventLoop::new();
        return LambdaRunnable{
            name: String::from("f"),
            window: LambdaWindow::new().with_event_loop(&event_loop),
            event_loop,
            running: false,
        }
    }

    pub fn get_name(&self) -> String {
        return self.name.clone();
    }

    // Get a cloned copy of the window
    pub fn get_window_data(&self) -> &LambdaWindow {
        return &self.window;
    }

    pub fn get_running(&self) -> bool {
        return self.running;
    }
}

impl Runnable for LambdaRunnable {
    /// One setup to initialize the
    fn setup(&self) {
        println!("Just hit lambda application runner setup!")
    }

    fn run(self) {
        self.event_loop.run_in_main_thread();
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
