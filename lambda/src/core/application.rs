
// Data only struct for configuring the application.
pub struct AppConfig {
    name: String,
    start_on_create: bool,
    window_size: [u32; 2],
}

impl AppConfig {
    // Import
    pub fn from_default() -> AppConfig {
        return AppConfig{
            name: String::from("lambda"),
            start_on_create: false,
            window_size: [512, 512]
        }
    }

    pub fn with_name(mut self, app_name: String) -> AppConfig {
        self.name = app_name;
        return self;
    }

    pub fn start_on_create(mut self, should_start: bool) -> AppConfig {
        self.start_on_create = should_start;
        return self;
    }

    pub fn with_window_size(mut self, window_size: [u32; 2]) -> AppConfig {
        self.window_size = window_size;
        return self;
    }
}

// Trait for Runnable applications that can be plugged into lambda.
pub trait Runnable {
    fn run(&self);
    fn on_update(&self);
    fn on_render(&self);
    fn on_event(&self);
}

// Application is the runnable
pub struct Application {
    name: String,
    running: bool
}


impl Runnable for Application {
    fn run(&self) {}
    fn on_update(&self) {}
    fn on_render(&self) {}
    fn on_event(&self) {}
}

impl Application {
    pub fn get_name(&self) -> String {
        return self.name.clone();
    }

    pub fn is_running(&self) -> bool {
        return self.running;
    }
}


// Create's an Application instance that is ready to be executed.
pub fn create_application(
    configuration: AppConfig) -> Application {

    return Application{
        name: configuration.name,
        running: configuration.start_on_create,
    };
}
