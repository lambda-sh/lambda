
pub struct Application {
    name: String,
    running: bool
}

impl Application {
    pub fn get_name(&self) -> String {
        return self.name.clone();
    }

    pub fn is_running(&self) -> bool {
        return self.running;
    }
}

// Data only struct for configuring the application.
pub struct ApplicationConfiguration {
    pub name: String,
    pub start_on_create: bool,
}

pub fn create_application(
    configuration: ApplicationConfiguration) -> Application {
    return Application{
        name: configuration.name,
        running: configuration.start_on_create,
    };
}
