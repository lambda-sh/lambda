use lambda::{
  core::{
    application::{
      create_lambda_runnable,
      start_runnable,
    },
    layer::Layer,
    render::RenderAPI,
  },
  platform::gfx::create_default_gfx_instance,
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

  fn on_update(
    &self,
    last_frame: &std::time::Duration,
    renderer: &mut RenderAPI,
  ) {
    println!(
      "This layer was last updated: {} nanoseconds ago",
      last_frame.as_nanos()
    );

    renderer.init();
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

struct AILayer {}

impl Default for AILayer {
  fn default() -> Self {
    return Self {};
  }
}
impl Layer for AILayer {
  fn attach(&self) {
    println!("Attached AI Layer")
  }

  fn detach(&self) {
    todo!()
  }

  fn on_event(&self, event: &lambda::core::event_loop::LambdaEvent) {
    todo!()
  }

  fn on_update(
    &self,
    last_frame: &std::time::Duration,
    renderer: &mut RenderAPI,
  ) {
    println!("Updating AI");
  }
}

fn main() {
  let app = create_lambda_runnable()
    .with_layer_attached::<FirstLayer>()
    .with_layer_attached::<AILayer>();

  start_runnable(app);
}

// These 40 lines of code create what you saw before
