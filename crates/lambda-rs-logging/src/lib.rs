//! A simple logging library for lambda-rs crates.

use std::fmt::Debug;

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

/// Logger implementation.
pub struct Logger {
  name: String,
  level: LogLevel,
  handlers: Vec<Box<dyn handler::Handler>>,
}

impl Logger {
  /// Creates a new logger with the given log level and name.
  pub fn new(level: LogLevel, name: &str) -> Self {
    Self {
      name: name.to_string(),
      level,
      handlers: Vec::new(),
    }
  }

  /// Returns the global logger.
  pub fn global() -> &'static mut Self {
    // TODO(vmarcella): Fix the instantiation for the global logger.
    unsafe {
      if LOGGER.is_none() {
        LOGGER = Some(Logger {
          level: LogLevel::TRACE,
          name: "lambda-rs".to_string(),
          handlers: vec![Box::new(handler::ConsoleHandler::new("lambda-rs"))],
        });
      }
    };
    return unsafe { &mut LOGGER }
      .as_mut()
      .expect("Logger not initialized");
  }

  /// Adds a handler to the logger. Handlers are called in the order they
  /// are added.
  pub fn add_handler(&mut self, handler: Box<dyn handler::Handler>) {
    self.handlers.push(handler);
  }

  fn compare_levels(&self, level: LogLevel) -> bool {
    level as u8 >= self.level as u8
  }

  /// Logs a trace message to all handlers.
  pub fn trace(&mut self, message: String) {
    if !self.compare_levels(LogLevel::TRACE) {
      return;
    }

    for handler in self.handlers.iter_mut() {
      handler.trace(message.clone());
    }
  }

  /// Logs a debug message to all handlers.
  pub fn debug(&mut self, message: String) {
    if !self.compare_levels(LogLevel::DEBUG) {
      return;
    }
    for handler in self.handlers.iter_mut() {
      handler.debug(message.clone());
    }
  }

  /// Logs an info message to all handlers.
  pub fn info(&mut self, message: String) {
    if !self.compare_levels(LogLevel::INFO) {
      return;
    }

    for handler in self.handlers.iter_mut() {
      handler.info(message.clone());
    }
  }

  /// Logs a warning to all handlers.
  pub fn warn(&mut self, message: String) {
    if !self.compare_levels(LogLevel::WARN) {
      return;
    }
    for handler in self.handlers.iter_mut() {
      handler.warn(message.clone());
    }
  }

  /// Logs an error to all handlers.
  pub fn error(&mut self, message: String) {
    if !self.compare_levels(LogLevel::ERROR) {
      return;
    }

    for handler in self.handlers.iter_mut() {
      handler.error(message.clone());
    }
  }

  ///  Logs a fatal error to all handlers and exits the program.
  pub fn fatal(&mut self, message: String) {
    if !self.compare_levels(LogLevel::FATAL) {
      return;
    }

    for handler in self.handlers.iter_mut() {
      handler.fatal(message.clone());
    }
    std::process::exit(1);
  }
}

pub(crate) static mut LOGGER: Option<Logger> = None;

/// Trace logging macro using the global logger instance.
#[macro_export]
macro_rules! trace {
  ($($arg:tt)*) => {
      logging::Logger::global().trace(format!("{}", format_args!($($arg)*)));
  };
}

/// Trace logging macro using the global logger instance.
#[macro_export]
macro_rules! debug {
  ($($arg:tt)*) => {
      logging::Logger::global().debug(format!("{}", format_args!($($arg)*)));
  };
}

/// Trace logging macro using the global logger instance.
#[macro_export]
macro_rules! info {
  ($($arg:tt)*) => {
      logging::Logger::global().info(format!("{}", format_args!($($arg)*)));
  };
}

// Define logging macros that use the global logger instance
#[macro_export]
macro_rules! warn {
  ($($arg:tt)*) => {
      logging::Logger::global().warn(format!("{}", format_args!($($arg)*)));
  };
}

#[macro_export]
macro_rules! error {
  ($($arg:tt)*) => {
      logging::Logger::global().error(format!("{}", format_args!($($arg)*)));
  };
}

#[macro_export]
macro_rules! fatal {
  ($($arg:tt)*) => {
      logging::Logger::global().fatal(format!("{}", format_args!($($arg)*)));
  };
}
