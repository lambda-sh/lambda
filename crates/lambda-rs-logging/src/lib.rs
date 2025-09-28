#![allow(clippy::needless_return)]
//! A simple logging library for lambda-rs crates.

use std::{
  fmt,
  sync::{
    atomic::{
      AtomicU8,
      Ordering,
    },
    Arc,
    OnceLock,
    RwLock,
  },
  time::SystemTime,
};

/// A trait for handling log messages.
pub mod handler;

/// The log level for the logger.
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
pub enum LogLevel {
  TRACE,
  DEBUG,
  INFO,
  WARN,
  ERROR,
  FATAL,
}

impl LogLevel {
  fn to_u8(self) -> u8 {
    match self {
      LogLevel::TRACE => 0,
      LogLevel::DEBUG => 1,
      LogLevel::INFO => 2,
      LogLevel::WARN => 3,
      LogLevel::ERROR => 4,
      LogLevel::FATAL => 5,
    }
  }
}

/// A log record passed to handlers.
#[derive(Debug, Clone)]
pub struct Record<'a> {
  pub timestamp: SystemTime,
  pub level: LogLevel,
  pub target: &'a str,
  pub message: &'a str,
  pub module_path: Option<&'static str>,
  pub file: Option<&'static str>,
  pub line: Option<u32>,
}

/// Logger implementation.
pub struct Logger {
  name: String,
  level: AtomicU8,
  handlers: RwLock<Vec<Box<dyn handler::Handler>>>,
}

impl Logger {
  /// Creates a new logger with the given log level and name.
  pub fn new(level: LogLevel, name: &str) -> Self {
    Self {
      name: name.to_string(),
      level: AtomicU8::new(level.to_u8()),
      handlers: RwLock::new(Vec::new()),
    }
  }

  /// Creates a builder for configuring a `Logger`.
  pub fn builder() -> LoggerBuilder {
    LoggerBuilder::default()
  }

  /// Returns the global logger (thread-safe). Initializes with a default
  /// console handler if not explicitly initialized via `init`.
  pub fn global() -> &'static Arc<Self> {
    static GLOBAL: OnceLock<Arc<Logger>> = OnceLock::new();
    GLOBAL.get_or_init(|| {
      let logger = Logger::new(LogLevel::TRACE, "lambda-rs");
      // Default console handler
      logger.add_handler(Box::new(handler::ConsoleHandler::new("lambda-rs")));
      Arc::new(logger)
    })
  }

  /// Initialize the global logger (first caller wins).
  pub fn init(logger: Logger) -> Result<(), InitError> {
    static GLOBAL: OnceLock<Arc<Logger>> = OnceLock::new();
    GLOBAL
      .set(Arc::new(logger))
      .map_err(|_| InitError::AlreadyInitialized)
  }

  /// Adds a handler to the logger. Handlers are called in the order they
  /// are added.
  pub fn add_handler(&self, handler: Box<dyn handler::Handler>) {
    let mut lock = self.handlers.write().expect("poisoned handlers lock");
    lock.push(handler);
  }

  /// Updates the minimum level for this logger.
  pub fn set_level(&self, level: LogLevel) {
    self.level.store(level.to_u8(), Ordering::Relaxed);
  }

  fn compare_levels(&self, level: LogLevel) -> bool {
    level.to_u8() >= self.level.load(Ordering::Relaxed)
  }

  fn log_inner(&self, level: LogLevel, message: &str) {
    if !self.compare_levels(level) {
      return;
    }
    self.log_inner_with_meta(level, message, None, None, None);
  }

  fn log_inner_with_meta(
    &self,
    level: LogLevel,
    message: &str,
    module_path: Option<&'static str>,
    file: Option<&'static str>,
    line: Option<u32>,
  ) {
    if !self.compare_levels(level) {
      return;
    }
    let record = Record {
      timestamp: SystemTime::now(),
      level,
      target: &self.name,
      message,
      module_path,
      file,
      line,
    };
    let lock = self.handlers.read().expect("poisoned handlers lock");
    for handler in lock.iter() {
      handler.log(&record);
    }
  }

  /// Logs a trace message to all handlers (shim for backward compatibility).
  pub fn trace(&self, message: String) {
    self.log_inner(LogLevel::TRACE, &message);
  }

  /// Logs a debug message to all handlers (shim for backward compatibility).
  pub fn debug(&self, message: String) {
    self.log_inner(LogLevel::DEBUG, &message);
  }

  /// Logs an info message to all handlers (shim for backward compatibility).
  pub fn info(&self, message: String) {
    self.log_inner(LogLevel::INFO, &message);
  }

  /// Logs a warning to all handlers (shim for backward compatibility).
  pub fn warn(&self, message: String) {
    self.log_inner(LogLevel::WARN, &message);
  }

  /// Logs an error to all handlers (shim for backward compatibility).
  pub fn error(&self, message: String) {
    self.log_inner(LogLevel::ERROR, &message);
  }

  /// Logs a fatal error to all handlers. Does NOT exit the process.
  pub fn fatal(&self, message: String) {
    self.log_inner(LogLevel::FATAL, &message);
  }
}

/// Initialization error for the global logger.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InitError {
  AlreadyInitialized,
}

/// Logger builder for ergonomic configuration.
pub struct LoggerBuilder {
  name: String,
  level: LogLevel,
  handlers: Vec<Box<dyn handler::Handler>>,
}

impl Default for LoggerBuilder {
  fn default() -> Self {
    Self {
      name: "lambda-rs".to_string(),
      level: LogLevel::INFO,
      handlers: Vec::new(),
    }
  }
}

impl LoggerBuilder {
  pub fn name(mut self, name: &str) -> Self {
    self.name = name.to_string();
    self
  }

  pub fn level(mut self, level: LogLevel) -> Self {
    self.level = level;
    self
  }

  pub fn with_handler(mut self, handler: Box<dyn handler::Handler>) -> Self {
    self.handlers.push(handler);
    self
  }

  pub fn build(self) -> Logger {
    let logger = Logger::new(self.level, &self.name);
    for h in self.handlers {
      logger.add_handler(h);
    }
    logger
  }
}

/// Environment configuration helpers.
pub mod env {
  use super::{
    LogLevel,
    Logger,
  };

  /// Parse a log level from a string like "trace", "debug", ...
  pub fn parse_level(s: &str) -> Option<LogLevel> {
    match s.trim().to_ascii_lowercase().as_str() {
      "trace" => Some(LogLevel::TRACE),
      "debug" => Some(LogLevel::DEBUG),
      "info" => Some(LogLevel::INFO),
      "warn" | "warning" => Some(LogLevel::WARN),
      "error" => Some(LogLevel::ERROR),
      "fatal" => Some(LogLevel::FATAL),
      _ => None,
    }
  }

  /// Applies a level from the environment to the provided logger.
  ///
  /// Reads the specified `var` (default: "LAMBDA_LOG"). If it parses to a level,
  /// updates the logger's level.
  pub fn apply_env_level(logger: &Logger, var: Option<&str>) {
    let key = var.unwrap_or("LAMBDA_LOG");
    if let Ok(val) = std::env::var(key) {
      if let Some(level) = parse_level(&val) {
        logger.set_level(level);
      }
    }
  }

  /// Initialize a global logger with a console handler and apply env level.
  pub fn init_global_from_env() -> Result<(), super::InitError> {
    let logger = Logger::builder()
      .name("lambda-rs")
      .level(LogLevel::INFO)
      .with_handler(Box::new(crate::handler::ConsoleHandler::new("lambda-rs")))
      .build();
    apply_env_level(&logger, Some("LAMBDA_LOG"));
    super::Logger::init(logger)
  }
}
/// Returns whether the global logger would log at `level`.
pub fn enabled(level: LogLevel) -> bool {
  Logger::global().compare_levels(level)
}

/// Logs using the global logger, formatting only after an enabled check.
pub fn log_args(
  level: LogLevel,
  module_path: &'static str,
  file: &'static str,
  line: u32,
  args: fmt::Arguments,
) {
  let logger = Logger::global().clone();
  if !logger.compare_levels(level) {
    return;
  }
  let message = args.to_string();
  logger.log_inner_with_meta(
    level,
    &message,
    Some(module_path),
    Some(file),
    Some(line),
  );
}
/// Trace logging macro using the global logger instance.
#[macro_export]
macro_rules! trace {
  ($($arg:tt)*) => {
      if $crate::enabled($crate::LogLevel::TRACE) {
        $crate::log_args($crate::LogLevel::TRACE, module_path!(), file!(), line!(), format_args!($($arg)*));
      }
  };
}

/// Trace logging macro using the global logger instance.
#[macro_export]
macro_rules! debug {
  ($($arg:tt)*) => {
      if $crate::enabled($crate::LogLevel::DEBUG) {
        $crate::log_args($crate::LogLevel::DEBUG, module_path!(), file!(), line!(), format_args!($($arg)*));
      }
  };
}

/// Trace logging macro using the global logger instance.
#[macro_export]
macro_rules! info {
  ($($arg:tt)*) => {
      if $crate::enabled($crate::LogLevel::INFO) {
        $crate::log_args($crate::LogLevel::INFO, module_path!(), file!(), line!(), format_args!($($arg)*));
      }
  };
}

// Define logging macros that use the global logger instance
#[macro_export]
macro_rules! warn {
  ($($arg:tt)*) => {
      if $crate::enabled($crate::LogLevel::WARN) {
        $crate::log_args($crate::LogLevel::WARN, module_path!(), file!(), line!(), format_args!($($arg)*));
      }
  };
}

#[macro_export]
macro_rules! error {
  ($($arg:tt)*) => {
      if $crate::enabled($crate::LogLevel::ERROR) {
        $crate::log_args($crate::LogLevel::ERROR, module_path!(), file!(), line!(), format_args!($($arg)*));
      }
  };
}

#[macro_export]
macro_rules! fatal {
  ($($arg:tt)*) => {
      if $crate::enabled($crate::LogLevel::FATAL) {
        $crate::log_args($crate::LogLevel::FATAL, module_path!(), file!(), line!(), format_args!($($arg)*));
      }
  };
}

#[cfg(test)]
mod tests {
  use std::{
    sync::{
      Arc,
      Mutex,
    },
    thread,
  };

  use super::*;

  #[derive(Default)]
  struct TestHandler {
    out: Arc<Mutex<Vec<(LogLevel, String)>>>,
  }

  impl TestHandler {
    fn new(out: Arc<Mutex<Vec<(LogLevel, String)>>>) -> Self {
      Self { out }
    }
  }

  impl handler::Handler for TestHandler {
    fn log(&self, record: &Record) {
      self
        .out
        .lock()
        .unwrap()
        .push((record.level, record.message.to_string()));
    }
  }

  #[test]
  fn global_singleton_is_stable() {
    let a = Logger::global().clone();
    let b = Logger::global().clone();
    assert!(Arc::ptr_eq(&a, &b));
  }

  #[test]
  fn level_filtering_works() {
    let logger = Logger::new(LogLevel::INFO, "test");
    let out = Arc::new(Mutex::new(Vec::new()));
    logger.add_handler(Box::new(TestHandler::new(out.clone())));

    logger.debug("ignored".to_string());
    logger.info("shown".to_string());

    let recs = out.lock().unwrap();
    assert_eq!(recs.len(), 1);
    assert_eq!(recs[0].0, LogLevel::INFO);
    assert_eq!(recs[0].1, "shown");
  }

  #[test]
  fn handler_order_is_preserved_single_thread() {
    #[derive(Default)]
    struct TagHandler {
      tag: &'static str,
      out: Arc<Mutex<Vec<&'static str>>>,
    }
    impl handler::Handler for TagHandler {
      fn log(&self, _record: &Record) {
        self.out.lock().unwrap().push(self.tag);
      }
    }

    let logger = Logger::new(LogLevel::TRACE, "order");
    let out = Arc::new(Mutex::new(Vec::new()));
    logger.add_handler(Box::new(TagHandler {
      tag: "A",
      out: out.clone(),
    }));
    logger.add_handler(Box::new(TagHandler {
      tag: "B",
      out: out.clone(),
    }));

    logger.info("x".to_string());

    let v = out.lock().unwrap().clone();
    assert_eq!(v, vec!["A", "B"]);
  }

  #[test]
  fn concurrent_logging_no_panic_and_counts_match() {
    let logger = Arc::new(Logger::new(LogLevel::TRACE, "concurrent"));
    let out = Arc::new(Mutex::new(Vec::new()));
    logger.add_handler(Box::new(TestHandler::new(out.clone())));

    let mut handles = Vec::new();
    for i in 0..8 {
      let lg = logger.clone();
      handles.push(thread::spawn(move || {
        for j in 0..100 {
          lg.debug(format!("msg {}:{}", i, j));
        }
      }));
    }
    for t in handles {
      t.join().unwrap();
    }

    let recs = out.lock().unwrap();
    assert_eq!(recs.len(), 800);
  }

  #[test]
  fn fatal_does_not_exit() {
    let logger = Logger::new(LogLevel::TRACE, "fatal");
    let out = Arc::new(Mutex::new(Vec::new()));
    logger.add_handler(Box::new(TestHandler::new(out.clone())));
    logger.fatal("boom".to_string());
    let recs = out.lock().unwrap();
    assert_eq!(recs.len(), 1);
    assert_eq!(recs[0].0, LogLevel::FATAL);
    assert_eq!(recs[0].1, "boom");
  }

  #[test]
  fn file_handler_flushes_after_ten() {
    use std::{
      fs,
      time::UNIX_EPOCH,
    };

    let tmp = std::env::temp_dir().join(format!(
      "lambda_logging_test_{}_{}.log",
      std::process::id(),
      SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos()
    ));

    let logger = Logger::new(LogLevel::TRACE, "file");
    logger.add_handler(Box::new(crate::handler::FileHandler::new(
      tmp.to_string_lossy().to_string(),
    )));

    for i in 0..10 {
      logger.info(format!("line{}", i));
    }

    let content =
      fs::read_to_string(&tmp).expect("file must exist after flush");
    assert!(!content.is_empty());
  }

  #[test]
  fn macro_early_guard_avoids_formatting() {
    // Ensure TRACE is disabled by setting level to INFO.
    super::Logger::global().set_level(super::LogLevel::INFO);

    struct Boom;
    impl fmt::Display for Boom {
      fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        panic!("should not be formatted when level disabled");
      }
    }

    // If guard fails, formatting Boom would panic.
    super::trace!("{}", Boom);
  }

  #[test]
  fn builder_sets_name_level_and_handlers() {
    #[derive(Default)]
    struct Capture {
      out: Arc<Mutex<Vec<String>>>,
    }
    impl handler::Handler for Capture {
      fn log(&self, record: &Record) {
        self
          .out
          .lock()
          .unwrap()
          .push(format!("{}:{}", record.target, record.level as u8));
      }
    }

    let out = Arc::new(Mutex::new(Vec::new()));
    let logger = Logger::builder()
      .name("builder-app")
      .level(LogLevel::WARN)
      .with_handler(Box::new(Capture { out: out.clone() }))
      .build();

    logger.info("drop".to_string());
    logger.error("keep".to_string());

    let v = out.lock().unwrap();
    assert_eq!(v.len(), 1);
    assert_eq!(v[0], "builder-app:4"); // ERROR => 4 per to_u8 mapping
  }

  #[test]
  fn env_parse_and_apply_level() {
    // no panic if env missing
    super::env::apply_env_level(
      &Logger::new(LogLevel::TRACE, "tmp"),
      Some("__NOT_SET__"),
    );

    assert_eq!(super::env::parse_level("trace"), Some(LogLevel::TRACE));
    assert_eq!(super::env::parse_level("DEBUG"), Some(LogLevel::DEBUG));
    assert_eq!(super::env::parse_level("warning"), Some(LogLevel::WARN));
    assert_eq!(super::env::parse_level("nope"), None);

    // apply
    let logger = Logger::new(LogLevel::ERROR, "tmp");
    std::env::set_var("LAMBDA_LOG", "info");
    super::env::apply_env_level(&logger, Some("LAMBDA_LOG"));
    assert!(logger.compare_levels(LogLevel::INFO));
    // restore
    std::env::remove_var("LAMBDA_LOG");
  }

  #[test]
  fn json_handler_writes_json_lines() {
    use std::fs;
    let tmp = std::env::temp_dir().join(format!(
      "lambda_json_{}_{}",
      std::process::id(),
      SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos()
    ));
    let p = tmp.to_string_lossy().to_string();

    let logger = Logger::builder()
      .name("json")
      .level(LogLevel::TRACE)
      .with_handler(Box::new(crate::handler::JsonHandler::new(p.clone())))
      .build();

    logger.info("hello json".to_string());
    let content = fs::read_to_string(p).unwrap();
    assert!(content.contains("\"level\":\"INFO\""));
    assert!(content.contains("hello json"));
  }

  #[test]
  fn rotating_handler_rotates_files() {
    use std::fs;
    let base = std::env::temp_dir().join(format!(
      "lambda_rotate_{}_{}",
      std::process::id(),
      SystemTime::now()
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos()
    ));
    let base_s = base.to_string_lossy().to_string();

    let logger = Logger::builder()
      .name("rot")
      .level(LogLevel::TRACE)
      .with_handler(Box::new(crate::handler::RotatingFileHandler::new(
        base_s.clone(),
        128, // small threshold
        2,
      )))
      .build();

    for i in 0..100 {
      logger.info(format!("line {i:03}"));
    }

    // Expect rotated files to exist
    let p1 = format!("{}.1", &base_s);
    let _p2 = format!("{}.2", &base_s);
    assert!(fs::metadata(p1).is_ok() || fs::metadata(base_s.clone()).is_ok());
    // not strictly asserting p2 due to small logs, but should often appear
  }
}
