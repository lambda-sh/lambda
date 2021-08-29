fn main() {
    use lambda::core::AppConfig;
    use lambda::core::create_application;

    let configuration = AppConfig::from_default()
        .with_name(String::from("Demo app"))
        .with_window_size([720, 480])
        .start_on_create(false);

    let demo = create_application(configuration);
    println!("{} is currently running.", demo.get_name());
}
