use lambda::core::{
  application::{
    create_lambda_runnable,
    start_runnable,
  },
  component::Component,
};

pub struct DemoComponent {}

impl Component for DemoComponent {
  fn attach(&mut self) {
    println!("Attached the first layer to lambda");
  }

  fn detach(self: &mut DemoComponent) {}

  fn on_event(
    self: &mut DemoComponent,
    event: &lambda::core::event_loop::LambdaEvent,
  ) {
  }

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

impl Default for DemoComponent {
  fn default() -> Self {
    return DemoComponent {};
  }
}

fn main() {
  let app = create_lambda_runnable().with_component::<DemoComponent>();

  start_runnable(app);
}

// These 40 lines of code create what you saw before
