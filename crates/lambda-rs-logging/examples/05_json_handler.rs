// Inside this crate, refer to the lib as `logging` directly.

fn main() {
  let path = std::env::temp_dir().join("lambda_json_example.log");
  let path_s = path.to_string_lossy().to_string();

  let logger = logging::Logger::builder()
    .name("json-example")
    .level(logging::LogLevel::TRACE)
    .with_handler(Box::new(logging::handler::JsonHandler::new(path_s.clone())))
    .build();

  logger.info("json info".to_string());
  logger.error("json error".to_string());

  println!("wrote JSON to {}", path_s);
}
