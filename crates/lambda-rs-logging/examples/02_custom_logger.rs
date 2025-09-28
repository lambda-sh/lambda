fn main() {
  let logger = logging::Logger::new(logging::LogLevel::DEBUG, "custom");
  logger.add_handler(Box::new(logging::handler::ConsoleHandler::new("custom")));

  logger.trace("this will be filtered unless level <= TRACE".to_string());
  logger.debug("debug from custom logger".to_string());
  logger.info("info from custom logger".to_string());
}
