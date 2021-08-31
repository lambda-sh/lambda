use lambda::{
    core::create_default_application,
    core::Application,
    core::Runnable,
    core::RunnableApplication,
    core::start_application,
};

pub struct DemoApp;

impl Application for DemoApp {
    fn get_name(&self) -> std::string::String { return String::from("Demo app") }
    fn get_window(&self) -> &lambda::core::LambdaWindow { todo!() }
    fn is_running(&self) -> bool { todo!() }
}

impl Runnable for DemoApp {
    fn setup(&self){
        println!("Demo application runner setup!")
    }
    fn run(&self){
        println!("Demo applicaiton runner loop!")
    }
    fn on_update(&self){}
    fn on_render(&self){}
    fn on_event(&self){}
}

impl DemoApp {
    pub fn new() -> Self {
        return DemoApp{};
    }
}

fn main() {
    let lambda = create_default_application();
    println!("{} was just created", lambda.get_name());
    start_application(lambda);

    let demo = DemoApp::new();
    println!("{} was just created.", demo.get_name());
    start_application(demo);
}
