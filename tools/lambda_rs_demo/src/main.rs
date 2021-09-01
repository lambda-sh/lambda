use lambda::{
    core::create_lambda_application,
    core::Runnable,
    core::start_application,
};

pub struct DemoApp;

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

    pub fn get_name(&self) -> String {
        return String::from("Demo app");
    }
}

fn main() {
    let lambda = create_lambda_application();
    println!("{} was just created", lambda.get_name());
    start_application(lambda);

    let demo = DemoApp::new();
    println!("{} was just created.", demo.get_name());
    start_application(demo);
}
