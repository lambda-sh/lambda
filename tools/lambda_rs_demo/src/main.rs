use lambda::{
  core::{
    component::Component,
    events::Event,
    kernel::start_kernel,
    render::RenderContextBuilder,
  },
  kernels::LambdaKernelBuilder,
};

pub struct DemoComponent {}

impl Component<Event> for DemoComponent {
  fn on_attach(&mut self) {
    println!("Attached the first layer to lambda");
  }

  fn on_detach(self: &mut DemoComponent) {}

  fn on_event(self: &mut DemoComponent, event: &lambda::core::events::Event) {}

  fn on_update(self: &mut DemoComponent, last_frame: &std::time::Duration) {
    println!(
      "This layer was last updated: {} nanoseconds ago",
      last_frame.as_nanos()
    );

    println!(
      "This layer was last updated: {} milliseconds ago",
      last_frame.as_millis()
    );
  }
}

impl DemoComponent {}

impl Default for DemoComponent {
  fn default() -> Self {
    return DemoComponent {};
  }
}

/// This function demonstrates how to configure the renderer that comes with
/// the LambdaKernel. This is where you can upload shaders, configure render
/// passes, and generally allocate the resources you need from a completely safe
/// Rust API.
fn configure_renderer(builder: RenderContextBuilder) -> RenderContextBuilder {
  return builder;
}

fn main() {
  let kernel = LambdaKernelBuilder::new("Lambda 2D Demo")
    .configure_renderer(configure_renderer)
    .build()
    .with_component(move |kernel, demo: DemoComponent| {
      return (kernel, demo);
    });

  start_kernel(kernel);
}

// These 40 lines of code create what you saw before
