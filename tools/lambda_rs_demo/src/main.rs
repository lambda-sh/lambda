use lambda::core::ApplicationConfiguration;
use lambda::core::create_application;

fn main() {

    let configuration = ApplicationConfiguration{
        name: String::from("Demo application"),
        start_on_create: false,
    };

    let app = create_application(configuration);
    println!("{} is currently running.", app.get_name());
}
