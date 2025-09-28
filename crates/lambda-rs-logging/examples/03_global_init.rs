fn main() {
  let logger = logging::Logger::new(logging::LogLevel::INFO, "app");
  logger.add_handler(Box::new(logging::handler::ConsoleHandler::new("app")));

  logging::Logger::init(logger)
    .expect("global logger can only be initialized once");

  logging::info!("hello from initialized global");
}
