//! Log handling implementations for the logger.

use std::{
  fs::OpenOptions,
  io::Write,
  sync::Mutex,
  time::SystemTime,
};

use crate::{
  LogLevel,
  Record,
};

/// Pluggable sink for log records emitted by the `Logger`.
/// Implementors decide how to format and where to deliver messages.
pub trait Handler: Send + Sync {
  fn log(&self, record: &Record);
}

/// A handler that logs to a file.
#[derive(Debug)]
pub struct FileHandler {
  file: String,
  log_buffer: Mutex<Vec<String>>,
}

impl FileHandler {
  pub fn new(file: String) -> Self {
    Self {
      file,
      log_buffer: Mutex::new(Vec::new()),
    }
  }
}

impl Handler for FileHandler {
  fn log(&self, record: &Record) {
    let timestamp = record
      .timestamp
      .duration_since(SystemTime::UNIX_EPOCH)
      .unwrap()
      .as_secs();

    let log_message =
      format!("[{}]-[{:?}]: {}", timestamp, record.level, record.message);

    // Preserve existing behavior: color codes even in file output.
    let colored_message = match record.level {
      LogLevel::TRACE => format!("\x1B[37m{}\x1B[0m", log_message),
      LogLevel::DEBUG => format!("\x1B[35m{}\x1B[0m", log_message),
      LogLevel::INFO => format!("\x1B[32m{}\x1B[0m", log_message),
      LogLevel::WARN => format!("\x1B[33m{}\x1B[0m", log_message),
      LogLevel::ERROR => format!("\x1B[31;1m{}\x1B[0m", log_message),
      LogLevel::FATAL => format!("\x1B[31;1m{}\x1B[0m", log_message),
    };

    let mut buf = self.log_buffer.lock().unwrap();
    buf.push(colored_message);

    // Flush buffer every ten messages.
    if buf.len() < 10 {
      return;
    }

    let log_message = buf.join("\n");

    let mut file = OpenOptions::new()
      .append(true)
      .create(true)
      .open(self.file.clone())
      .unwrap();

    file
      .write_all(log_message.as_bytes())
      .expect("Unable to write data");

    buf.clear();
  }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct ConsoleHandler {
  name: String,
}

impl ConsoleHandler {
  pub fn new(name: &str) -> Self {
    Self {
      name: name.to_string(),
    }
  }
}

impl Handler for ConsoleHandler {
  fn log(&self, record: &Record) {
    let timestamp = record
      .timestamp
      .duration_since(SystemTime::UNIX_EPOCH)
      .unwrap()
      .as_secs();

    let log_message = format!(
      "[{}]-[{:?}]-[{}]: {}",
      timestamp, record.level, self.name, record.message
    );

    let colored_message = match record.level {
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
