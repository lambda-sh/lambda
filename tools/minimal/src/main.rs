use lambda::{
  core::runtime::start_runtime,
  runtimes::GenericRuntimeBuilder,
};

fn main() {
  let runtime = GenericRuntimeBuilder::new("Minimal Demo application")
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
