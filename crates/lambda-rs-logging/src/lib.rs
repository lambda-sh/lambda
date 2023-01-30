use std::{
  fs::OpenOptions,
  io::Write,
  time::{
    SystemTime,
    UNIX_EPOCH,
  },
};

/// A trait for handling log messages.
pub mod handler;

/// The log level for the logger.
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy)]
pub enum LogLevel {
  Trace,
  Debug,
  Info,
  Warn,
  Error,
  Fatal,
}

/// Logger implementation.
pub struct Logger {
  name: String,
  level: LogLevel,
  handlers: Vec<Box<dyn handler::Handler>>,
}

impl Logger {
  pub fn new(level: LogLevel, name: String) -> Self {
    Self {
      name,
      level,
      handlers: Vec::new(),
    }
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
    if !self.compare_levels(LogLevel::Trace) {
      return;
    }

    for handler in self.handlers.iter_mut() {
      handler.trace(message.clone());
    }
  }

  /// Logs a debug message to all handlers.
  pub fn debug(&mut self, message: String) {
    if !self.compare_levels(LogLevel::Debug) {
      return;
    }
    for handler in self.handlers.iter_mut() {
      handler.debug(message.clone());
    }
  }

  /// Logs an info message to all handlers.
  pub fn info(&mut self, message: String) {
    if !self.compare_levels(LogLevel::Info) {
      return;
    }

    for handler in self.handlers.iter_mut() {
      handler.info(message.clone());
    }
  }

  /// Logs a warning to all handlers.
  pub fn warn(&mut self, message: String) {
    if !self.compare_levels(LogLevel::Warn) {
      return;
    }
    for handler in self.handlers.iter_mut() {
      handler.warn(message.clone());
    }
  }

  /// Logs an error to all handlers.
  pub fn error(&mut self, message: String) {
    if !self.compare_levels(LogLevel::Error) {
      return;
    }

    for handler in self.handlers.iter_mut() {
      handler.error(message.clone());
    }
  }

  ///  Logs a fatal error to all handlers and exits the program.
  pub fn fatal(&mut self, message: String) {
    if !self.compare_levels(LogLevel::Fatal) {
      return;
    }

    for handler in self.handlers.iter_mut() {
      handler.fatal(message.clone());
    }
    std::process::exit(1);
  }
}
