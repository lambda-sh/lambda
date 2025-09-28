fn main() {
  let base = std::env::temp_dir().join("lambda_rotate_example.log");
  let base_s = base.to_string_lossy().to_string();

  let logger = logging::Logger::builder()
    .name("rotate-example")
    .level(logging::LogLevel::TRACE)
    .with_handler(Box::new(logging::handler::RotatingFileHandler::new(
      base_s.clone(),
      256, // bytes
      3,   // keep 3 backups
    )))
    .build();

  for i in 0..200 {
    logger.info(format!("log line {:03}", i));
  }

  println!("rotation base: {} (check .1, .2, .3)", base_s);
}
