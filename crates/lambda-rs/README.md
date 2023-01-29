# lambda-rs
![lambda-rs](https://img.shields.io/crates/d/lambda-rs)
![lambda-rs](https://img.shields.io/crates/v/lambda-rs)

The lambda-rs crate provides a safe, cross-platform API for building applications on the Lambda platform.

## Getting started
### First window
Getting started with lambda is easy. The following example will create a window with the title "Hello lambda!" and a size of 800x600.
```rust
#[macro_use]
use lambda::{
  core::runtime::start_runtime,
  runtimes::ApplicationRuntimeBuilder,
};

fn main() {
  let runtime = ApplicationRuntimeBuilder::new("Hello lambda!")
    .with_window_configured_as(move |window_builder| {
      return window_builder
        .with_dimensions(800, 600)
        .with_name("Hello lambda!");
    })
    .build();

  start_runtime(runtime);
}
```
