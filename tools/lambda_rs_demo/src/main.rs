use lambda::{
  core::{
    component::Component,
    runnable::start_kernel,
  },
  runnables::LambdaKernelBuilder,
};

pub struct DemoComponent {}

impl Component for DemoComponent {
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

impl DemoComponent {
  fn new() -> Self {
    return DemoComponent {};
  }
}

impl Default for DemoComponent {
  fn default() -> Self {
    return DemoComponent {};
  }
}

fn main() {
  let kernel = LambdaKernelBuilder::new()
    .with_name("Lambda RS 2D Demo")
    .build()
    .with_component(move |runnable, demo: DemoComponent| {
      return (runnable, demo);
    });

  start_kernel(kernel);
}

// These 40 lines of code create what you saw before
