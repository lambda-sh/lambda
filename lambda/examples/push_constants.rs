use lambda::{
  core::{
    component::Component,
    render::{
      command::RenderCommand,
      pipeline::{
        RenderPipeline,
        RenderPipelineBuilder,
      },
      render_pass::{
        RenderPass,
        RenderPassBuilder,
      },
      shader::{
        Shader,
        ShaderBuilder,
      },
      viewport,
      ResourceId,
    },
    runtime::start_runtime,
  },
  math::{
    matrix::{
      self,
      Matrix,
    },
    vector::Vector,
  },
  runtimes::GenericRuntimeBuilder,
};
use lambda_platform::{
  gfx::pipeline::PipelineStage,
  shaderc::{
    ShaderKind,
    VirtualShader,
  },
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

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PushConstant {
  data: [f32; 4],
  render_matrix: [[f32; 4]; 4],
}

pub struct PushConstantsExample {
  frame_number: u64,
  shader: Shader,
  render_pipeline: Option<ResourceId>,
  render_pass: Option<ResourceId>,
}

pub fn push_constants_to_bytes(push_constants: &PushConstant) -> &[u32] {
  let bytes = unsafe {
    let size_in_bytes = std::mem::size_of::<PushConstant>();
    let size_in_u32 = size_in_bytes / std::mem::size_of::<u32>();
    let ptr = push_constants as *const PushConstant as *const u32;
    std::slice::from_raw_parts(ptr, size_in_u32)
  };

  return bytes;
}

impl Component for PushConstantsExample {
  fn on_attach(
    &mut self,
    render_context: &mut lambda::core::render::RenderContext,
  ) {
    let render_pass = RenderPassBuilder::new().build(&render_context);
    let push_constant_size = std::mem::size_of::<PushConstant>() as u32;
    let pipeline = RenderPipelineBuilder::new()
      .with_push_constant(PipelineStage::VERTEX, push_constant_size)
      .build(render_context, &render_pass, &self.shader, None);

    self.render_pass = Some(render_context.attach_render_pass(render_pass));
    self.render_pipeline = Some(render_context.attach_pipeline(pipeline));
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

  /// Update the frame number every frame.
  fn on_update(&mut self, last_frame: &std::time::Duration) {
    self.frame_number += 1;
  }

  fn on_render(
    &mut self,
    render_context: &mut lambda::core::render::RenderContext,
  ) -> Vec<lambda::core::render::command::RenderCommand> {
    let mut camera = [0.0, 0.0, -2.0];
    let view: [[f32; 4]; 4] = matrix::translation_matrix(camera);

    // Create a projection matrix.
    let mut projection: [[f32; 4]; 4] =
      matrix::perspective_matrix(1.0 / 2.0, 1700.0 / 900.0, 0.1, 200.0);
    projection.as_mut()[1].as_mut()[1] *= -1.0;

    // Rotate model.
    let model: [[f32; 4]; 4] = matrix::rotate_matrix(
      matrix::filled_matrix(4, 4, 1.0),
      [0.0, 1.0, 0.0],
      0.4 * self.frame_number as f32,
    );

    // Create render matrix.
    let mesh_matrix = projection.multiply(&view).multiply(&model);

    // Create viewport.
    let viewport = viewport::ViewportBuilder::new().build(800, 600);

    let render_pipeline = self
      .render_pipeline
      .expect("No render pipeline actively set for rendering.");

    let mut commands = vec![
      RenderCommand::SetViewports {
        start_at: 0,
        viewports: vec![viewport.clone()],
      },
      RenderCommand::SetScissors {
        start_at: 0,
        viewports: vec![viewport.clone()],
      },
      RenderCommand::SetPipeline {
        pipeline: render_pipeline.clone(),
      },
      RenderCommand::BeginRenderPass {
        render_pass: self
          .render_pass
          .expect("Cannot begin the render pass when it doesn't exist.")
          .clone(),
        viewport: viewport.clone(),
      },
      RenderCommand::PushConstants {
        pipeline: render_pipeline.clone(),
        stage: PipelineStage::VERTEX,
        offset: 0,
        bytes: Vec::from(push_constants_to_bytes(&PushConstant {
          data: [0.0, 0.0, 0.0, 0.0],
          render_matrix: mesh_matrix,
        })),
      },
      RenderCommand::Draw { vertices: 0..3 },
    ];
    commands.push(RenderCommand::EndRenderPass);

    return commands;
  }
}

impl Default for PushConstantsExample {
  fn default() -> Self {
    let triangle_in_3d = VirtualShader::Source {
      source: VERTEX_SHADER_SOURCE.to_string(),
      kind: ShaderKind::Vertex,
      entry_point: "main".to_string(),
      name: "push_constants".to_string(),
    };

    let mut builder = ShaderBuilder::new();
    let shader = builder.build(triangle_in_3d);

    return Self {
      frame_number: 0,
      shader,
      render_pipeline: None,
      render_pass: None,
    };
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
