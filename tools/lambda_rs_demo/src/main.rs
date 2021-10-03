use lambda::core::{
  application::{
    create_lambda_runnable,
    start_runnable,
  },
  layer::Layer,
};

pub struct FirstLayer {}

impl Layer for FirstLayer {
  fn attach(&self) {
    println!("Attached the first layer to lambda");
  }

  fn detach(&self) {
    todo!()
  }

  fn on_event(&self, event: &lambda::core::event_loop::LambdaEvent) {
    todo!()
  }

  fn on_update(&self, last_frame: &std::time::Duration) {
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

impl Default for FirstLayer {
  fn default() -> Self {
    return FirstLayer {};
  }
}

fn main() {
  let app = create_lambda_runnable().with_layer_attached::<FirstLayer>();

  start_runnable(app);
}

// These 40 lines of code create what you saw before
