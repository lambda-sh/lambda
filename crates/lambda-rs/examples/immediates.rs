#![allow(clippy::needless_return)]

use lambda::{
  component::Component,
  events::{
    EventMask,
    WindowEvent,
  },
  logging,
  math::matrix::Matrix,
  render::{
    buffer::BufferBuilder,
    command::RenderCommand,
    mesh::{
      Mesh,
      MeshBuilder,
    },
    pipeline::RenderPipelineBuilder,
    render_pass::RenderPassBuilder,
    scene_math::{
      compute_model_view_projection_matrix_about_pivot,
      SimpleCamera,
    },
    shader::{
      Shader,
      ShaderBuilder,
      ShaderKind,
      VirtualShader,
    },
    vertex::{
      ColorFormat,
      VertexAttribute,
      VertexBuilder,
      VertexElement,
    },
    viewport,
    ResourceId,
  },
  runtime::start_runtime,
  runtimes::{
    application::ComponentResult,
    ApplicationRuntimeBuilder,
  },
};

// ------------------------------ SHADER SOURCE --------------------------------

const VERTEX_SHADER_SOURCE: &str = r#"
#version 450

layout (location = 0) in vec3 vertex_position;
layout (location = 1) in vec3 vertex_normal;
layout (location = 2) in vec3 vertex_color;

layout (location = 0) out vec3 frag_color;

layout ( push_constant ) uniform PushConstant {
  vec4 data;
  mat4 render_matrix;
} push_constants;

void main() {
  gl_Position = push_constants.render_matrix * vec4(vertex_position, 1.0);
  frag_color = vertex_color;
}

"#;

const FRAGMENT_SHADER_SOURCE: &str = r#"
#version 450

layout (location = 0) in vec3 frag_color;

layout (location = 0) out vec4 fragment_color;

void main() {
  fragment_color = vec4(frag_color, 1.0);
}

"#;

// ------------------------------ IMMEDIATES ----------------------------------

/// Immediate data structure passed to shaders via wgpu's immediates feature.
/// In GLSL shaders, this is still declared as `push_constant` uniform block.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ImmediateData {
  data: [f32; 4],
  render_matrix: [[f32; 4]; 4],
}

pub fn immediate_data_to_bytes(immediate: &ImmediateData) -> &[u32] {
  let bytes = unsafe {
    let size_in_bytes = std::mem::size_of::<ImmediateData>();
    let size_in_u32 = size_in_bytes / std::mem::size_of::<u32>();
    let ptr = immediate as *const ImmediateData as *const u32;
    std::slice::from_raw_parts(ptr, size_in_u32)
  };

  return bytes;
}

// Model, view, and projection matrix computations are handled by `scene_math`.

// --------------------------------- COMPONENT ---------------------------------

const ROTATION_TURNS_PER_SECOND: f32 = 0.12;

pub struct ImmediatesExample {
  elapsed_seconds: f32,
  shader: Shader,
  fs: Shader,
  mesh: Option<Mesh>,
  render_pipeline: Option<ResourceId>,
  render_pass: Option<ResourceId>,
  width: u32,
  height: u32,
}

impl Component<ComponentResult, String> for ImmediatesExample {
  fn on_attach(
    &mut self,
    render_context: &mut lambda::render::RenderContext,
  ) -> Result<ComponentResult, String> {
    let render_pass = RenderPassBuilder::new().build(
      render_context.gpu(),
      render_context.surface_format(),
      render_context.depth_format(),
    );
    let immediate_data_size = std::mem::size_of::<ImmediateData>() as u32;

    // Create triangle mesh.
    let vertices = [
      VertexBuilder::new()
        .with_position([1.0, 1.0, 0.0])
        .with_normal([0.0, 0.0, 0.0])
        .with_color([1.0, 0.0, 0.0])
        .build(),
      VertexBuilder::new()
        .with_position([-1.0, 1.0, 0.0])
        .with_normal([0.0, 0.0, 0.0])
        .with_color([0.0, 1.0, 0.0])
        .build(),
      VertexBuilder::new()
        .with_position([0.0, -1.0, 0.0])
        .with_normal([0.0, 0.0, 0.0])
        .with_color([0.0, 0.0, 1.0])
        .build(),
    ];

    let mut mesh_builder = MeshBuilder::new();
    vertices.iter().for_each(|vertex| {
      mesh_builder.with_vertex(*vertex);
    });

    let mesh = mesh_builder
      .with_attributes(vec![
        VertexAttribute {
          location: 0,
          offset: 0,
          element: VertexElement {
            format: ColorFormat::Rgb32Sfloat,
            offset: 0,
          },
        },
        VertexAttribute {
          location: 1,
          offset: 0,
          element: VertexElement {
            format: ColorFormat::Rgb32Sfloat,
            offset: 12,
          },
        },
        VertexAttribute {
          location: 2,
          offset: 0,
          element: VertexElement {
            format: ColorFormat::Rgb32Sfloat,
            offset: 24,
          },
        },
      ])
      .build();

    logging::trace!("mesh: {:?}", mesh);

    let pipeline = RenderPipelineBuilder::new()
      .with_culling(lambda::render::pipeline::CullingMode::None)
      .with_immediate_data(immediate_data_size)
      .with_buffer(
        BufferBuilder::build_from_mesh(&mesh, render_context.gpu())
          .expect("Failed to create buffer"),
        mesh.attributes().to_vec(),
      )
      .build(
        render_context.gpu(),
        render_context.surface_format(),
        render_context.depth_format(),
        &render_pass,
        &self.shader,
        Some(&self.fs),
      );

    self.render_pass = Some(render_context.attach_render_pass(render_pass));
    self.render_pipeline = Some(render_context.attach_pipeline(pipeline));
    self.mesh = Some(mesh);

    return Ok(ComponentResult::Success);
  }

  fn on_detach(
    &mut self,
    _render_context: &mut lambda::render::RenderContext,
  ) -> Result<ComponentResult, String> {
    logging::info!("Detaching component");
    return Ok(ComponentResult::Success);
  }

  fn event_mask(&self) -> EventMask {
    return EventMask::WINDOW;
  }

  fn on_window_event(&mut self, event: &WindowEvent) -> Result<(), String> {
    if let WindowEvent::Resize { width, height } = event {
      self.width = *width;
      self.height = *height;
      logging::info!("Window resized to {}x{}", width, height);
    }
    return Ok(());
  }

  /// Update elapsed time every frame.
  fn on_update(
    &mut self,
    last_frame: &std::time::Duration,
  ) -> Result<ComponentResult, String> {
    self.elapsed_seconds += last_frame.as_secs_f32();
    return Ok(ComponentResult::Success);
  }

  fn on_render(
    &mut self,
    _render_context: &mut lambda::render::RenderContext,
  ) -> Vec<lambda::render::command::RenderCommand> {
    let camera = SimpleCamera {
      position: [0.0, 0.0, 3.0],
      field_of_view_in_turns: 0.25,
      near_clipping_plane: 0.1,
      far_clipping_plane: 100.0,
    };
    let angle_in_turns = ROTATION_TURNS_PER_SECOND * self.elapsed_seconds;
    let mesh_matrix = compute_model_view_projection_matrix_about_pivot(
      &camera,
      self.width.max(1),
      self.height.max(1),
      [0.0, -1.0 / 3.0, 0.0],
      [0.0, 1.0, 0.0],
      angle_in_turns,
      0.5,
      [0.0, 1.0 / 3.0, 0.0],
    );

    // Create viewport.
    let viewport =
      viewport::ViewportBuilder::new().build(self.width, self.height);

    let render_pipeline = self
      .render_pipeline
      .expect("No render pipeline actively set for rendering.");

    return vec![
      RenderCommand::BeginRenderPass {
        render_pass: self
          .render_pass
          .expect("Cannot begin the render pass when it doesn't exist."),
        viewport: viewport.clone(),
      },
      RenderCommand::SetPipeline {
        pipeline: render_pipeline,
      },
      RenderCommand::SetViewports {
        start_at: 0,
        viewports: vec![viewport.clone()],
      },
      RenderCommand::SetScissors {
        start_at: 0,
        viewports: vec![viewport.clone()],
      },
      RenderCommand::BindVertexBuffer {
        pipeline: render_pipeline,
        buffer: 0,
      },
      RenderCommand::Immediates {
        pipeline: render_pipeline,
        offset: 0,
        bytes: Vec::from(immediate_data_to_bytes(&ImmediateData {
          data: [0.0, 0.0, 0.0, 0.0],
          // Transpose to match GPU's column‑major expectation.
          render_matrix: mesh_matrix.transpose(),
        })),
      },
      RenderCommand::Draw {
        vertices: 0..self.mesh.as_ref().unwrap().vertices().len() as u32,
        instances: 0..1,
      },
      RenderCommand::EndRenderPass,
    ];
  }
}

impl Default for ImmediatesExample {
  fn default() -> Self {
    let triangle_in_3d = VirtualShader::Source {
      source: VERTEX_SHADER_SOURCE.to_string(),
      kind: ShaderKind::Vertex,
      entry_point: "main".to_string(),
      name: "immediates".to_string(),
    };

    let triangle_fragment_shader = VirtualShader::Source {
      source: FRAGMENT_SHADER_SOURCE.to_string(),
      kind: ShaderKind::Fragment,
      entry_point: "main".to_string(),
      name: "immediates".to_string(),
    };

    let mut builder = ShaderBuilder::new();
    let shader = builder.build(triangle_in_3d);
    let fs = builder.build(triangle_fragment_shader);

    return Self {
      elapsed_seconds: 0.0,
      shader,
      fs,
      mesh: None,
      render_pipeline: None,
      render_pass: None,
      width: 800,
      height: 600,
    };
  }
}

fn main() {
  let runtime = ApplicationRuntimeBuilder::new("3D Immediates Example")
    .with_window_configured_as(move |window_builder| {
      return window_builder
        .with_dimensions(800, 600)
        .with_name("3D Immediates Example");
    })
    .with_renderer_configured_as(|renderer_builder| {
      return renderer_builder.with_render_timeout(1_000_000_000);
    })
    .with_component(move |runtime, triangles: ImmediatesExample| {
      return (runtime, triangles);
    })
    .build();

  start_runtime(runtime);
}
