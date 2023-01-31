//! Log handling implementations for the logger.

use std::{
  fmt::Debug,
  fs::OpenOptions,
  io::Write,
  time::SystemTime,
};

use crate::LogLevel;

pub trait Handler {
  fn trace(&mut self, message: String);
  fn debug(&mut self, message: String);
  fn info(&mut self, message: String);
  fn warn(&mut self, message: String);
  fn error(&mut self, message: String);
  fn fatal(&mut self, message: String);
}

/// A handler that logs to a file.

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct FileHandler {
  file: String,
  log_buffer: Vec<String>,
}

impl FileHandler {
  pub fn new(file: String) -> Self {
    Self {
      file,
      log_buffer: Vec::new(),
    }
  }

  /// Logs a message to the file.
  fn log(&mut self, log_level: LogLevel, message: String) {
    let timestamp = SystemTime::now()
      .duration_since(SystemTime::UNIX_EPOCH)
      .unwrap()
      .as_secs();

    let log_message = format!("[{}]-[{:?}]: {}", timestamp, log_level, message);

    let colored_message = match log_level {
      LogLevel::TRACE => format!("\x1B[37m{}\x1B[0m", log_message),
      LogLevel::DEBUG => format!("\x1B[35m{}\x1B[0m", log_message),
      LogLevel::INFO => format!("\x1B[32m{}\x1B[0m", log_message),
      LogLevel::WARN => format!("\x1B[33m{}\x1B[0m", log_message),
      LogLevel::ERROR => format!("\x1B[31;1m{}\x1B[0m", log_message),
      LogLevel::FATAL => format!("\x1B[31;1m{}\x1B[0m", log_message),
    };

    self.log_buffer.push(colored_message);

    // Flush buffer every ten messages.
    if self.log_buffer.len() < 10 {
      return;
    }

    let log_message = self.log_buffer.join("\n");

    let mut file = OpenOptions::new()
      .append(true)
      .create(true)
      .open(self.file.clone())
      .unwrap();

    file
      .write_all(log_message.as_bytes())
      .expect("Unable to write data");

    self.log_buffer.clear();
  }
}

impl Handler for FileHandler {
  fn trace(&mut self, message: String) {
    self.log(LogLevel::TRACE, message)
  }

  fn debug(&mut self, message: String) {
    self.log(LogLevel::DEBUG, message)
  }

  fn info(&mut self, message: String) {
    self.log(LogLevel::INFO, message)
  }

  fn warn(&mut self, message: String) {
    self.log(LogLevel::WARN, message)
  }

  fn error(&mut self, message: String) {
    self.log(LogLevel::ERROR, message)
  }

  fn fatal(&mut self, message: String) {
    self.log(LogLevel::FATAL, message)
  }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct ConsoleHandler {
  name: String,
}

impl ConsoleHandler {
  pub fn new(name: &str) -> Self {
    return Self {
      name: name.to_string(),
    };
  }

  fn log(&mut self, log_level: LogLevel, message: String) {
    let timestamp = SystemTime::now()
      .duration_since(SystemTime::UNIX_EPOCH)
      .unwrap()
      .as_secs();

    let log_message = format!(
      "[{}]-[{:?}]-[{}]: {}",
      timestamp, log_level, self.name, message
    );

    let colored_message = match log_level {
      LogLevel::TRACE => format!("\x1B[37m{}\x1B[0m", log_message),
      LogLevel::DEBUG => format!("\x1B[35m{}\x1B[0m", log_message),
      LogLevel::INFO => format!("\x1B[32m{}\x1B[0m", log_message),
      LogLevel::WARN => format!("\x1B[33m{}\x1B[0m", log_message),
      LogLevel::ERROR => format!("\x1B[31;1m{}\x1B[0m", log_message),
      LogLevel::FATAL => format!("\x1B[31;1m{}\x1B[0m", log_message),
    };

    println!("{}", colored_message);
  }
}

impl Handler for ConsoleHandler {
  fn trace(&mut self, message: String) {
    self.log(LogLevel::TRACE, message);
  }

  fn debug(&mut self, message: String) {
    self.log(LogLevel::DEBUG, message);
  }

  fn info(&mut self, message: String) {
    self.log(LogLevel::INFO, message);
  }

  fn warn(&mut self, message: String) {
    self.log(LogLevel::WARN, message);
  }

  fn error(&mut self, message: String) {
    self.log(LogLevel::ERROR, message);
  }

  fn fatal(&mut self, message: String) {
    self.log(LogLevel::FATAL, message);
  }
}
