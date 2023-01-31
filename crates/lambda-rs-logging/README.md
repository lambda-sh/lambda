# lambda-rs-logging
![lambda-rs](https://img.shields.io/crates/d/lambda-rs-logging)
![lambda-rs](https://img.shields.io/crates/v/lambda-rs-logging)

A simple logger implementation for lamba-rs crates. Inspired by
python's [logging](https://docs.python.org/3/library/logging.html) module.


# Installation
First, add the following to your `Cargo.toml`:
```toml
[dependencies]
lambda-rs-logging = "2023.1.30"
```

or run this command from your project directory:
```bash
cargo add lambda-rs-logging
```

# Getting started
## Using the global logger
```rust
use logging;

fn main() {
  logging::trace!("Hello world");
  logging::debug!("Hello world");
  logging::info!("Hello world");
  logging::warn!("Hello world");
  logging::error!("Hello world");
  logging::fatal!("Hello world");
}
```

## Using an instance of the logger
```rust
use logging::Logger;

fn main() {
  let logger = Logger::new("my-logger");
  logger.trace("Hello world");
  logger.debug("Hello world");
  logger.info("Hello world");
  logger.warn("Hello world");
  logger.error("Hello world");
  logger.fatal("Hello world");
}
```
