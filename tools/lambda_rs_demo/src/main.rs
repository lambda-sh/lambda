use lambda::core::application as app;

fn main() {

    let configuration = app::ApplicationConfiguration{
        name: String::from("Demo application"),
        start_on_create: false,
    };

    let demo = app::create_application(configuration);
    println!("{} is currently running.", demo.get_name());
}
