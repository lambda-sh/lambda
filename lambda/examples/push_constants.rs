use lambda::{
  core::{
    component::Component,
    render::shader,
    runtime::start_runtime,
  },
  math,
  runtimes::GenericRuntimeBuilder,
};

const VERTEX_SHADER_SOURCE: &str = r#"
#version 450

layout (location = 0) in vec2 vertex_position;
layout (location = 1) in vec3 vertex_normal;
layout (location = 2) in vec3 vertex_color;

layout (location = 0) out vec3 frag_color;

layout ( push_constant ) uniform PushConstant {
  vec4 data;
  mat4 render_matrix;
} push_constants;

void main() {
  gl_Position = push_constants.render_matrix * vec4(vertex_position, 0.0, 1.0);
  frag_color = vertex_color;
}

"#;

struct PushConstantsExample {}

impl Component for PushConstantsExample {
  fn on_attach(
    &mut self,
    render_context: &mut lambda::core::render::RenderContext,
  ) {
    todo!()
  }

  fn on_detach(
    &mut self,
    render_context: &mut lambda::core::render::RenderContext,
  ) {
    todo!()
  }

  fn on_event(&mut self, event: lambda::core::events::Events) {
    todo!()
  }

  fn on_update(&mut self, last_frame: &std::time::Duration) {
    todo!()
  }

  fn on_render(
    &mut self,
    render_context: &mut lambda::core::render::RenderContext,
  ) -> Vec<lambda::core::render::command::RenderCommand> {
    use math::vector::Vector;
    let mut camera = [0.0, 0.0, -2.0];
    let view: [[f32; 4]; 4] = math::matrix::translation_matrix(camera);
    let mut projection: [[f32; 4]; 4] =
      math::matrix::perspective_matrix(1.0 / 2.0, 1700.0 / 900.0, 0.1, 200.0);
    projection.as_mut()[1].as_mut()[1] *= -1.0;

    todo!()
  }
}

impl Default for PushConstantsExample {
  fn default() -> Self {
    return Self {};
  }
}

fn main() {
  let runtime = GenericRuntimeBuilder::new("Multiple Triangles Demo")
    .with_renderer_configured_as(move |render_context_builder| {
      return render_context_builder.with_render_timeout(1_000_000_000);
    })
    .with_window_configured_as(move |window_builder| {
      return window_builder
        .with_dimensions(800, 600)
        .with_name("Triangles");
    })
    .with_component(move |runtime, triangles: PushConstantsExample| {
      return (runtime, triangles);
    })
    .build();

  start_runtime(runtime);
}
