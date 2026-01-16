#![allow(clippy::needless_return)]

//! Example: Instanced 2D quads with per-instance vertex data.
//!
//! This example renders a grid of quads that all share the same geometry
//! but use per-instance offsets and colors supplied from a second vertex
//! buffer. It exercises `RenderPipelineBuilder::with_instance_buffer` and
//! `RenderCommand::DrawIndexed` with a non-trivial instance range.

use lambda::{
  component::Component,
  events::{
    EventMask,
    WindowEvent,
  },
  logging,
  render::{
    buffer::{
      BufferBuilder,
      BufferType,
      Properties,
      Usage,
    },
    command::{
      IndexFormat,
      RenderCommand,
    },
    pipeline::{
      CullingMode,
      RenderPipelineBuilder,
    },
    render_pass::RenderPassBuilder,
    shader::{
      Shader,
      ShaderBuilder,
      ShaderKind,
      VirtualShader,
    },
    vertex::{
      ColorFormat,
      VertexAttribute,
      VertexElement,
    },
    viewport,
    RenderContext,
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
layout (location = 1) in vec3 instance_offset;
layout (location = 2) in vec3 instance_color;

layout (location = 0) out vec3 frag_color;

void main() {
  vec3 position = vertex_position + instance_offset;
  gl_Position = vec4(position, 1.0);
  frag_color = instance_color;
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

// ------------------------------- VERTEX TYPES --------------------------------

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct QuadVertex {
  position: [f32; 3],
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct InstanceData {
  offset: [f32; 3],
  color: [f32; 3],
}

// --------------------------------- COMPONENT ---------------------------------

/// Component that renders a grid of instanced quads.
pub struct InstancedQuadsExample {
  vertex_shader: Shader,
  fragment_shader: Shader,
  render_pass_id: Option<ResourceId>,
  render_pipeline_id: Option<ResourceId>,
  index_buffer_id: Option<ResourceId>,
  index_count: u32,
  instance_count: u32,
  width: u32,
  height: u32,
}

impl Component<ComponentResult, String> for InstancedQuadsExample {
  fn on_attach(
    &mut self,
    render_context: &mut RenderContext,
  ) -> Result<ComponentResult, String> {
    let render_pass = RenderPassBuilder::new().build(
      render_context.gpu(),
      render_context.surface_format(),
      render_context.depth_format(),
    );

    // Quad geometry in clip space centered at the origin.
    let quad_vertices: Vec<QuadVertex> = vec![
      QuadVertex {
        position: [-0.05, -0.05, 0.0],
      },
      QuadVertex {
        position: [0.05, -0.05, 0.0],
      },
      QuadVertex {
        position: [0.05, 0.05, 0.0],
      },
      QuadVertex {
        position: [-0.05, 0.05, 0.0],
      },
    ];

    // Two triangles forming a quad.
    let indices: Vec<u16> = vec![0, 1, 2, 2, 3, 0];
    let index_count = indices.len() as u32;

    // Build a grid of instance offsets and colors.
    let grid_size: u32 = 10;
    let spacing: f32 = 0.2;
    let start: f32 = -0.9;

    let mut instances: Vec<InstanceData> = Vec::new();
    for y in 0..grid_size {
      for x in 0..grid_size {
        let offset_x = start + (x as f32) * spacing;
        let offset_y = start + (y as f32) * spacing;

        // Simple color gradient across the grid.
        let color_r = (x as f32) / ((grid_size - 1) as f32);
        let color_g = (y as f32) / ((grid_size - 1) as f32);
        let color_b = 0.5;

        instances.push(InstanceData {
          offset: [offset_x, offset_y, 0.0],
          color: [color_r, color_g, color_b],
        });
      }
    }
    let instance_count = instances.len() as u32;

    // Build vertex, instance, and index buffers.
    let vertex_buffer = BufferBuilder::new()
      .with_usage(Usage::VERTEX)
      .with_properties(Properties::DEVICE_LOCAL)
      .with_buffer_type(BufferType::Vertex)
      .with_label("instanced-quads-vertices")
      .build(render_context.gpu(), quad_vertices)
      .map_err(|error| error.to_string())?;

    let instance_buffer = BufferBuilder::new()
      .with_usage(Usage::VERTEX)
      .with_properties(Properties::DEVICE_LOCAL)
      .with_buffer_type(BufferType::Vertex)
      .with_label("instanced-quads-instances")
      .build(render_context.gpu(), instances)
      .map_err(|error| error.to_string())?;

    let index_buffer = BufferBuilder::new()
      .with_usage(Usage::INDEX)
      .with_properties(Properties::DEVICE_LOCAL)
      .with_buffer_type(BufferType::Index)
      .with_label("instanced-quads-indices")
      .build(render_context.gpu(), indices)
      .map_err(|error| error.to_string())?;

    // Vertex attributes for per-vertex positions in slot 0.
    let vertex_attributes = vec![VertexAttribute {
      location: 0,
      offset: 0,
      element: VertexElement {
        format: ColorFormat::Rgb32Sfloat,
        offset: 0,
      },
    }];

    // Instance attributes in slot 1: offset and color.
    let instance_attributes = vec![
      VertexAttribute {
        location: 1,
        offset: 0,
        element: VertexElement {
          format: ColorFormat::Rgb32Sfloat,
          offset: 0,
        },
      },
      VertexAttribute {
        location: 2,
        offset: 0,
        element: VertexElement {
          format: ColorFormat::Rgb32Sfloat,
          offset: 12,
        },
      },
    ];

    let pipeline = RenderPipelineBuilder::new()
      .with_culling(CullingMode::Back)
      .with_buffer(vertex_buffer, vertex_attributes)
      .with_instance_buffer(instance_buffer, instance_attributes)
      .build(
        render_context.gpu(),
        render_context.surface_format(),
        render_context.depth_format(),
        &render_pass,
        &self.vertex_shader,
        Some(&self.fragment_shader),
      );

    self.render_pass_id = Some(render_context.attach_render_pass(render_pass));
    self.render_pipeline_id = Some(render_context.attach_pipeline(pipeline));
    self.index_buffer_id = Some(render_context.attach_buffer(index_buffer));
    self.index_count = index_count;
    self.instance_count = instance_count;

    logging::info!(
      "Instanced quads example attached with {} instances",
      self.instance_count
    );
    return Ok(ComponentResult::Success);
  }

  fn on_detach(
    &mut self,
    _render_context: &mut RenderContext,
  ) -> Result<ComponentResult, String> {
    logging::info!("Instanced quads example detached");
    return Ok(ComponentResult::Success);
  }

  fn event_mask(&self) -> EventMask {
    return EventMask::WINDOW;
  }

  fn on_window_event(&mut self, event: &WindowEvent) -> Result<(), String> {
    match event {
      WindowEvent::Resize { width, height } => {
        self.width = *width;
        self.height = *height;
        logging::info!("Window resized to {}x{}", width, height);
      }
      _ => {}
    }
    return Ok(());
  }

  fn on_update(
    &mut self,
    _last_frame: &std::time::Duration,
  ) -> Result<ComponentResult, String> {
    // This example uses static instance data; no per-frame updates required.
    return Ok(ComponentResult::Success);
  }

  fn on_render(
    &mut self,
    _render_context: &mut RenderContext,
  ) -> Vec<RenderCommand> {
    let viewport =
      viewport::ViewportBuilder::new().build(self.width, self.height);

    let render_pass_id = self
      .render_pass_id
      .expect("Render pass must be attached before rendering");
    let pipeline_id = self
      .render_pipeline_id
      .expect("Pipeline must be attached before rendering");
    let index_buffer_id = self
      .index_buffer_id
      .expect("Index buffer must be attached before rendering");

    return vec![
      RenderCommand::BeginRenderPass {
        render_pass: render_pass_id,
        viewport: viewport.clone(),
      },
      RenderCommand::SetPipeline {
        pipeline: pipeline_id,
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
        pipeline: pipeline_id,
        buffer: 0,
      },
      RenderCommand::BindVertexBuffer {
        pipeline: pipeline_id,
        buffer: 1,
      },
      RenderCommand::BindIndexBuffer {
        buffer: index_buffer_id,
        format: IndexFormat::Uint16,
      },
      RenderCommand::DrawIndexed {
        indices: 0..self.index_count,
        base_vertex: 0,
        instances: 0..self.instance_count,
      },
      RenderCommand::EndRenderPass,
    ];
  }
}

impl Default for InstancedQuadsExample {
  fn default() -> Self {
    let vertex_virtual_shader = VirtualShader::Source {
      source: VERTEX_SHADER_SOURCE.to_string(),
      kind: ShaderKind::Vertex,
      entry_point: "main".to_string(),
      name: "instanced_quads".to_string(),
    };

    let fragment_virtual_shader = VirtualShader::Source {
      source: FRAGMENT_SHADER_SOURCE.to_string(),
      kind: ShaderKind::Fragment,
      entry_point: "main".to_string(),
      name: "instanced_quads".to_string(),
    };

    let mut shader_builder = ShaderBuilder::new();
    let vertex_shader = shader_builder.build(vertex_virtual_shader);
    let fragment_shader = shader_builder.build(fragment_virtual_shader);

    return Self {
      vertex_shader,
      fragment_shader,
      render_pass_id: None,
      render_pipeline_id: None,
      index_buffer_id: None,
      index_count: 0,
      instance_count: 0,
      width: 800,
      height: 600,
    };
  }
}

fn main() {
  let runtime: ApplicationRuntime =
    ApplicationRuntimeBuilder::new("Instanced Quads Example")
      .with_window_configured_as(|window_builder| {
        return window_builder
          .with_dimensions(800, 600)
          .with_name("Instanced Quads Example");
      })
      .with_renderer_configured_as(|render_builder| {
        return render_builder.with_render_timeout(1_000_000_000);
      })
      .with_component(|runtime, example: InstancedQuadsExample| {
        return (runtime, example);
      })
      .build();

  start_runtime(runtime);
}
