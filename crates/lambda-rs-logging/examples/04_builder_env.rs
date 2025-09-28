// When building examples inside the crate, refer to the library as `logging` directly.

fn main() {
  // Build a custom logger and apply env level
  let logger = logging::Logger::builder()
    .name("builder-env")
    .level(logging::LogLevel::INFO)
    .with_handler(Box::new(logging::handler::ConsoleHandler::new(
      "builder-env",
    )))
    .build();

  logging::env::apply_env_level(&logger, Some("LAMBDA_LOG"));

  logger.debug("filtered unless LAMBDA_LOG=debug".to_string());
  logger.info("visible at info".to_string());
}
