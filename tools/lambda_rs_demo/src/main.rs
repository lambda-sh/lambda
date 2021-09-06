use lambda::core::{
  application::{
    create_lambda_runnable,
    start_runnable,
  },
  layer::Layer,
};

pub struct FirstLayer {
  name: String,
}

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
    println!("[layer 1] {} ns since last update", last_frame.as_nanos());
  }
}

impl Default for FirstLayer {
  fn default() -> Self {
    return FirstLayer {
      name: String::from("Hello, lambda!"),
    };
  }
}

fn main() {
  let app = create_lambda_runnable().with_layer_attached::<FirstLayer>();

  start_runnable(app);
}
