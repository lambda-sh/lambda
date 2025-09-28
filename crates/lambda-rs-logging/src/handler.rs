//! Log handling implementations for the logger.

use std::{
  fs::OpenOptions,
  io::{
    self,
    IsTerminal,
    Write,
  },
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

    // Select output stream based on level.
    let warn_or_higher = matches!(
      record.level,
      LogLevel::WARN | LogLevel::ERROR | LogLevel::FATAL
    );
    if warn_or_higher {
      let mut e = io::stderr().lock();
      let use_color = io::stderr().is_terminal();
      if use_color {
        let colored = match record.level {
          LogLevel::TRACE => format!("\x1B[37m{}\x1B[0m", log_message),
          LogLevel::DEBUG => format!("\x1B[35m{}\x1B[0m", log_message),
          LogLevel::INFO => format!("\x1B[32m{}\x1B[0m", log_message),
          LogLevel::WARN => format!("\x1B[33m{}\x1B[0m", log_message),
          LogLevel::ERROR | LogLevel::FATAL => {
            format!("\x1B[31;1m{}\x1B[0m", log_message)
          }
        };
        let _ = writeln!(e, "{}", colored);
      } else {
        let _ = writeln!(e, "{}", log_message);
      }
    } else {
      let mut o = io::stdout().lock();
      let use_color = io::stdout().is_terminal();
      if use_color {
        let colored = match record.level {
          LogLevel::TRACE => format!("\x1B[37m{}\x1B[0m", log_message),
          LogLevel::DEBUG => format!("\x1B[35m{}\x1B[0m", log_message),
          LogLevel::INFO => format!("\x1B[32m{}\x1B[0m", log_message),
          LogLevel::WARN => format!("\x1B[33m{}\x1B[0m", log_message),
          LogLevel::ERROR | LogLevel::FATAL => {
            format!("\x1B[31;1m{}\x1B[0m", log_message)
          }
        };
        let _ = writeln!(o, "{}", colored);
      } else {
        let _ = writeln!(o, "{}", log_message);
      }
    }
  }
}

/// A handler that writes newline-delimited JSON log records.
/// Uses minimal manual escaping to avoid external dependencies.
pub struct JsonHandler {
  inner: Mutex<io::BufWriter<std::fs::File>>,
}

impl JsonHandler {
  pub fn new(path: String) -> Self {
    let file = OpenOptions::new()
      .create(true)
      .append(true)
      .open(&path)
      .expect("open json log file");
    Self {
      inner: Mutex::new(io::BufWriter::new(file)),
    }
  }

  fn escape_json(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 8);
    for ch in s.chars() {
      match ch {
        '"' => out.push_str("\\\""),
        '\\' => out.push_str("\\\\"),
        '\n' => out.push_str("\\n"),
        '\r' => out.push_str("\\r"),
        '\t' => out.push_str("\\t"),
        c if c.is_control() => {
          use std::fmt::Write as _;
          let _ = write!(out, "\\u{:04x}", c as u32);
        }
        c => out.push(c),
      }
    }
    out
  }
}

impl Handler for JsonHandler {
  fn log(&self, record: &Record) {
    let ts = record
      .timestamp
      .duration_since(SystemTime::UNIX_EPOCH)
      .unwrap()
      .as_millis();
    let msg = Self::escape_json(record.message);
    let target = Self::escape_json(record.target);
    let module = record.module_path.unwrap_or("");
    let file = record.file.unwrap_or("");
    let line = record.line.unwrap_or(0);
    let level = match record.level {
      LogLevel::TRACE => "TRACE",
      LogLevel::DEBUG => "DEBUG",
      LogLevel::INFO => "INFO",
      LogLevel::WARN => "WARN",
      LogLevel::ERROR => "ERROR",
      LogLevel::FATAL => "FATAL",
    };
    let json = format!(
      "{{\"ts\":{},\"level\":\"{}\",\"target\":\"{}\",\"message\":\"{}\",\"module\":\"{}\",\"file\":\"{}\",\"line\":{}}}\n",
      ts, level, target, msg, module, file, line
    );
    let mut w = self.inner.lock().unwrap();
    let _ = w.write_all(json.as_bytes());
    let _ = w.flush();
  }
}

/// A handler that writes to a file and rotates when size exceeds `max_bytes`.
pub struct RotatingFileHandler {
  path: String,
  max_bytes: u64,
  backups: usize,
  lock: Mutex<()>,
}

impl RotatingFileHandler {
  pub fn new(path: String, max_bytes: u64, backups: usize) -> Self {
    Self {
      path,
      max_bytes,
      backups,
      lock: Mutex::new(()),
    }
  }

  fn rotate(&self) {
    // Rotate: file.(n-1) -> file.n, ..., file -> file.1, delete file.n if exists
    for i in (1..=self.backups).rev() {
      let from = if i == 1 {
        std::path::PathBuf::from(&self.path)
      } else {
        std::path::PathBuf::from(format!("{}.{}", &self.path, i - 1))
      };
      let to = std::path::PathBuf::from(format!("{}.{}", &self.path, i));
      if from.exists() {
        let _ = std::fs::rename(&from, &to);
      }
    }
  }
}

impl Handler for RotatingFileHandler {
  fn log(&self, record: &Record) {
    let _guard = self.lock.lock().unwrap();

    // Check file size and rotate if needed
    if let Ok(meta) = std::fs::metadata(&self.path) {
      if meta.len() >= self.max_bytes {
        self.rotate();
      }
    }

    let timestamp = record
      .timestamp
      .duration_since(SystemTime::UNIX_EPOCH)
      .unwrap()
      .as_secs();
    let line = format!(
      "[{}]-[{:?}]-[{}]: {}\n",
      timestamp, record.level, record.target, record.message
    );

    let mut f = OpenOptions::new()
      .create(true)
      .append(true)
      .open(&self.path)
      .expect("open rotating file");
    let _ = f.write_all(line.as_bytes());
  }
}
