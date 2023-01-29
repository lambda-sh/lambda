//! Minimal application which configures a window & render context before
//! starting the runtime. You can use this as a starting point for your own
//! applications or to verify that your system is configured to run lambda
//! applications correctly.

#[macro_use]
use lambda::{
  core::runtime::start_runtime,
  runtimes::ApplicationRuntimeBuilder,
};

fn main() {
  let runtime = ApplicationRuntimeBuilder::new("Minimal Demo application")
    .with_renderer_configured_as(move |render_context_builder| {
      return render_context_builder.with_render_timeout(1_000_000_000);
    })
    .with_window_configured_as(move |window_builder| {
      return window_builder
        .with_dimensions(800, 600)
        .with_name("Minimal window");
    })
    .build();

  start_runtime(runtime);
}
