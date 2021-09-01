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

pub struct LambdaApplication {
    name: String,
    window: LambdaWindow,
    running: bool
}

impl LambdaApplication {
    pub fn new() -> Self {
        return LambdaApplication{
            name: String::from("f"),
            window: LambdaWindow::new(),
            running: false
        }
    }

    pub fn get_name(&self) -> String {
        return self.name.clone();
    }

    pub fn get_window(&self) -> LambdaWindow {
        return self.window.clone();
    }

    pub fn get_running(&self) -> bool {
        return self.running;
    }
}

impl Runnable for LambdaApplication {
    fn setup(&self) {
        println!("Just hit lambda application runner setup!")
    }
    fn run(&self) {
        println!("Just hit lambda application runner loop!")
    }
    fn on_update(&self) {}
    fn on_render(&self) {}
    fn on_event(&self) {}
}


pub fn create_lambda_application() -> LambdaApplication {
    return LambdaApplication::new();
}

pub fn start_application<T: Runnable>(app: T) {
    app.setup();
    app.run();
}
