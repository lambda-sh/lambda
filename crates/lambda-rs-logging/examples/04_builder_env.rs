// When building examples inside the crate, the library is available as `logging`.
// If you copy this example to another crate, import with `use lambda_rs_logging as logging;`
use logging;

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
