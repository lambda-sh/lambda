
// Data only struct for configuring the application.
pub struct ApplicationConfiguration {
    pub name: String,
    pub start_on_create: bool,
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



pub fn create_application(
    configuration: ApplicationConfiguration) -> Application {
    return Application{
        name: configuration.name,
        running: configuration.start_on_create,
    };
}
