# lambda-rs-logging
![lambda-rs](https://img.shields.io/crates/d/lambda-rs-logging)
![lambda-rs](https://img.shields.io/crates/v/lambda-rs-logging)

Simple, lightweight logging for lambda-rs crates. Inspired by Python’s
[logging](https://docs.python.org/3/library/logging.html) module.


## Installation
Add to your `Cargo.toml`:
```toml
[dependencies]
# Option A: use the crate name in code as `lambda_rs_logging`
lambda-rs-logging = "2023.1.30"

# Option B: rename dependency so you can write `use logging;`
# logging = { package = "lambda-rs-logging", version = "2023.1.30" }
```

Or from your project directory:
```bash
cargo add lambda-rs-logging
```

Then in code, either import with the default name:
```rust
use lambda_rs_logging as logging;
```
or, if you used the rename in Cargo.toml (Option B), simply:
```rust
use logging; // renamed in Cargo.toml
```

## Getting Started
### Global logger via macros
```rust
use lambda_rs_logging as logging;

fn main() {
  logging::trace!("trace {}", 1);
  logging::debug!("debug {}", 2);
  logging::info!("info {}", 3);
  logging::warn!("warn {}", 4);
  logging::error!("error {}", 5);
  logging::fatal!("fatal {}", 6); // note: does not exit
}
```

### Custom logger instance
```rust
use lambda_rs_logging as logging;

fn main() {
  let logger = logging::Logger::new(logging::LogLevel::INFO, "my-app");
  logger.add_handler(Box::new(logging::handler::ConsoleHandler::new("my-app")));

  logger.info("Hello world".to_string());
  logger.warn("Be careful".to_string());
}
```

### Initialize a custom global
```rust
use lambda_rs_logging as logging;

fn main() {
  let logger = logging::Logger::new(logging::LogLevel::DEBUG, "app");
  logger.add_handler(Box::new(logging::handler::ConsoleHandler::new("app")));

  // Set the global logger before any macros are used
  logging::Logger::init(logger).expect("global logger can only be initialized once");

  logging::debug!("from global");
}
```

## Notes
- Thread-safe global with `OnceLock<Arc<Logger>>`.
- Handlers are `Send + Sync` and receive a `Record` internally (phase 1 refactor).
- `fatal!` logs at FATAL level but does not exit the process. Prefer explicit exits in your app logic.

## Examples
This crate ships with examples. From the repository root:
```bash
cargo run -p lambda-rs-logging --example 01_global_macros
cargo run -p lambda-rs-logging --example 02_custom_logger
cargo run -p lambda-rs-logging --example 03_global_init
```
