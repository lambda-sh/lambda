use lambda::core as core;

fn main() {
    let configuration = core::ApplicationConfiguration{
        name: String::from("Demo application"),
        start_on_create: false,
    };

    let demo = core::create_application(configuration);
    println!("{} is currently running.", demo.get_name());
}
