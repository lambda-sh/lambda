#![allow(clippy::needless_return)]

//! Example: Spinning triangle in 3D using a uniform buffer and a bind group.
//!
//! This example mirrors the push constants demo but uses a uniform buffer
//! bound at group(0) binding(0) and a bind group layout declared in Rust.
//! The model, view, and projection matrices are computed via the shared
//! `scene_math` helpers so the example does not hand-roll the math.

use lambda::{
  component::Component,
  events::WindowEvent,
  logging,
  math::matrix::Matrix,
  render::{
    bind::{
      BindGroupBuilder,
      BindGroupLayoutBuilder,
      BindingVisibility,
    },
    buffer::{
      BufferBuilder,
      Properties,
      Usage,
    },
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
    ApplicationRuntime,
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

layout (set = 0, binding = 0) uniform Globals {
  mat4 render_matrix;
} globals;

void main() {
  gl_Position = globals.render_matrix * vec4(vertex_position, 1.0);
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

// ---------------------------- UNIFORM STRUCTURE ------------------------------

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct GlobalsUniform {
  pub render_matrix: [[f32; 4]; 4],
}

// --------------------------------- COMPONENT ---------------------------------

pub struct UniformBufferExample {
  elapsed_seconds: f32,
  shader: Shader,
  fragment_shader: Shader,
  mesh: Option<Mesh>,
  render_pipeline: Option<ResourceId>,
  render_pass: Option<ResourceId>,
  uniform_buffer: Option<lambda::render::buffer::Buffer>,
  bind_group: Option<ResourceId>,
  width: u32,
  height: u32,
}

impl Component<ComponentResult, String> for UniformBufferExample {
  fn on_attach(
    &mut self,
    render_context: &mut lambda::render::RenderContext,
  ) -> Result<ComponentResult, String> {
    let render_pass = RenderPassBuilder::new().build(
      render_context.gpu(),
      render_context.surface_format(),
      render_context.depth_format(),
    );

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
      mesh_builder.with_vertex(vertex.clone());
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

    // Create a bind group layout with a single uniform buffer at binding 0.
    let layout = BindGroupLayoutBuilder::new()
      .with_uniform(0, BindingVisibility::Vertex)
      .build(render_context.gpu());

    // Create the uniform buffer with an initial matrix.
    let camera = SimpleCamera {
      position: [0.0, 0.0, 3.0],
      field_of_view_in_turns: 0.25,
      near_clipping_plane: 0.1,
      far_clipping_plane: 100.0,
    };
    let initial_matrix = compute_model_view_projection_matrix_about_pivot(
      &camera,
      self.width.max(1),
      self.height.max(1),
      [0.0, -1.0 / 3.0, 0.0],
      [0.0, 1.0, 0.0],
      0.0,
      0.5,
      [0.0, 1.0 / 3.0, 0.0],
    );

    let initial_uniform = GlobalsUniform {
      // Transpose to match GPU column‑major layout.
      render_matrix: initial_matrix.transpose(),
    };

    let uniform_buffer = BufferBuilder::new()
      .with_length(std::mem::size_of::<GlobalsUniform>())
      .with_usage(Usage::UNIFORM)
      .with_properties(Properties::CPU_VISIBLE)
      .with_label("globals-uniform")
      .build(render_context.gpu(), vec![initial_uniform])
      .expect("Failed to create uniform buffer");

    // Create the bind group using the layout and uniform buffer.
    let bind_group = BindGroupBuilder::new()
      .with_layout(&layout)
      .with_uniform(0, &uniform_buffer, 0, None)
      .build(render_context.gpu());

    let pipeline = RenderPipelineBuilder::new()
      .with_culling(lambda::render::pipeline::CullingMode::None)
      .with_layouts(&[&layout])
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
        Some(&self.fragment_shader),
      );

    self.render_pass = Some(render_context.attach_render_pass(render_pass));
    self.render_pipeline = Some(render_context.attach_pipeline(pipeline));
    self.bind_group = Some(render_context.attach_bind_group(bind_group));
    self.uniform_buffer = Some(uniform_buffer);
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

  fn on_event(
    &mut self,
    event: lambda::events::Events,
  ) -> Result<ComponentResult, String> {
    match event {
      lambda::events::Events::Window {
        event,
        issued_at: _,
      } => match event {
        WindowEvent::Resize { width, height } => {
          self.width = width;
          self.height = height;
          logging::info!("Window resized to {}x{}", width, height);
        }
        _ => {}
      },
      _ => {}
    };
    return Ok(ComponentResult::Success);
  }

  fn on_update(
    &mut self,
    last_frame: &std::time::Duration,
  ) -> Result<ComponentResult, String> {
    self.elapsed_seconds += last_frame.as_secs_f32();
    return Ok(ComponentResult::Success);
  }

  fn on_render(
    &mut self,
    render_context: &mut lambda::render::RenderContext,
  ) -> Vec<lambda::render::command::RenderCommand> {
    const ROTATION_TURNS_PER_SECOND: f32 = 0.12;

    // Compute the model, view, projection matrix for this frame.
    let camera = SimpleCamera {
      position: [0.0, 0.0, 3.0],
      field_of_view_in_turns: 0.25,
      near_clipping_plane: 0.1,
      far_clipping_plane: 100.0,
    };
    let angle_in_turns = ROTATION_TURNS_PER_SECOND * self.elapsed_seconds;
    let render_matrix = compute_model_view_projection_matrix_about_pivot(
      &camera,
      self.width.max(1),
      self.height.max(1),
      [0.0, -1.0 / 3.0, 0.0],
      [0.0, 1.0, 0.0],
      angle_in_turns,
      0.5,
      [0.0, 1.0 / 3.0, 0.0],
    );

    // Update the uniform buffer with the new matrix.
    if let Some(ref uniform_buffer) = self.uniform_buffer {
      // Transpose to match GPU column‑major layout.
      let value = GlobalsUniform {
        render_matrix: render_matrix.transpose(),
      };
      uniform_buffer.write_value(render_context.gpu(), 0, &value);
    }

    // Create viewport.
    let viewport =
      viewport::ViewportBuilder::new().build(self.width, self.height);

    let render_pipeline = self
      .render_pipeline
      .expect("No render pipeline actively set for rendering.");
    let group_id = self.bind_group.expect("Bind group must exist");

    return vec![
      RenderCommand::BeginRenderPass {
        render_pass: self
          .render_pass
          .expect("Cannot begin the render pass when it does not exist.")
          .clone(),
        viewport: viewport.clone(),
      },
      RenderCommand::SetPipeline {
        pipeline: render_pipeline.clone(),
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
        pipeline: render_pipeline.clone(),
        buffer: 0,
      },
      RenderCommand::SetBindGroup {
        set: 0,
        group: group_id,
        dynamic_offsets: vec![],
      },
      RenderCommand::Draw {
        vertices: 0..self.mesh.as_ref().unwrap().vertices().len() as u32,
        instances: 0..1,
      },
      RenderCommand::EndRenderPass,
    ];
  }
}

impl Default for UniformBufferExample {
  fn default() -> Self {
    let vertex_virtual_shader = VirtualShader::Source {
      source: VERTEX_SHADER_SOURCE.to_string(),
      kind: ShaderKind::Vertex,
      entry_point: "main".to_string(),
      name: "uniform_buffer_triangle".to_string(),
    };

    let fragment_virtual_shader = VirtualShader::Source {
      source: FRAGMENT_SHADER_SOURCE.to_string(),
      kind: ShaderKind::Fragment,
      entry_point: "main".to_string(),
      name: "uniform_buffer_triangle".to_string(),
    };

    let mut builder = ShaderBuilder::new();
    let shader = builder.build(vertex_virtual_shader);
    let fragment_shader = builder.build(fragment_virtual_shader);

    return Self {
      elapsed_seconds: 0.0,
      shader,
      fragment_shader,
      mesh: None,
      render_pipeline: None,
      render_pass: None,
      uniform_buffer: None,
      bind_group: None,
      width: 800,
      height: 600,
    };
  }
}

fn main() {
  let runtime = ApplicationRuntimeBuilder::new("3D Uniform Buffer Example")
    .with_window_configured_as(move |window_builder| {
      return window_builder
        .with_dimensions(800, 600)
        .with_name("3D Uniform Buffer Example");
    })
    .with_renderer_configured_as(|renderer_builder| {
      return renderer_builder.with_render_timeout(1_000_000_000);
    })
    .with_component(move |runtime, example: UniformBufferExample| {
      return (runtime, example);
    })
    .build();

  start_runtime(runtime);
}
