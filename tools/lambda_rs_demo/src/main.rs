use lambda::core::render::shader::FragmentShaders;
use lambda::core::render::shader::PrepackagedShaders;
use lambda::core::render::shader::VertexShaders;
use lambda::core::{
  component::Component, render::shader::Shader, runnable::start_runnable,
};

use lambda::runnables::create_lambda_runnable;

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
  let app = create_lambda_runnable().with_renderable_component(
    move |renderer, demo: DemoComponent| {
      let vertex_shader = Shader::from_lambda(PrepackagedShaders::Vertex(
        VertexShaders::Triangle,
      ));

      let fragment_shader = Shader::from_lambda(PrepackagedShaders::Fragment(
        FragmentShaders::Triangle,
      ));
      return demo;
    },
  );

  start_runnable(app);
}

// These 40 lines of code create what you saw before
