use crate::core::window::{
    Window,
    LambdaWindow,
};

pub trait Runnable {
    fn setup(&self);
    fn run(&self);
    fn on_update(&self);
    fn on_render(&self);
    fn on_event(&self);
}

pub struct LambdaRunnable {
    name: String,
    window: LambdaWindow,
    running: bool
}

impl LambdaRunnable {
    pub fn new() -> Self {
        return LambdaRunnable{
            name: String::from("f"),
            window: LambdaWindow::new(),
            running: false
        }
    }

    pub fn get_name(&self) -> String {
        return self.name.clone();
    }

    // Get a cloned copy of the window
    pub fn get_window_data(&self) -> LambdaWindow {
        return self.window;
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

    fn run(&self) {
        while self.running {
            self.on_update();
        }
        println!("Just hit lambda application runner loop!")
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
