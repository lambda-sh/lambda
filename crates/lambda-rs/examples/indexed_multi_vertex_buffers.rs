#![allow(clippy::needless_return)]

//! Example: Indexed draw with multiple vertex buffers.
//!
//! This example renders a simple quad composed from two triangles using
//! separate vertex buffers for positions and colors plus a 16-bit index
//! buffer. It exercises `BindVertexBuffer` for multiple slots and
//! `BindIndexBuffer`/`DrawIndexed` in the render command stream.

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
    ApplicationRuntimeBuilder,
  },
};

// ------------------------------ SHADER SOURCE --------------------------------

const VERTEX_SHADER_SOURCE: &str = r#"
#version 450

layout (location = 0) in vec3 vertex_position;
layout (location = 1) in vec3 vertex_color;

layout (location = 0) out vec3 frag_color;

void main() {
  gl_Position = vec4(vertex_position, 1.0);
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

// ------------------------------- VERTEX TYPES --------------------------------

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct PositionVertex {
  position: [f32; 3],
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct ColorVertex {
  color: [f32; 3],
}

// --------------------------------- COMPONENT ---------------------------------

pub struct IndexedMultiBufferExample {
  vertex_shader: Shader,
  fragment_shader: Shader,
  render_pass_id: Option<ResourceId>,
  render_pipeline_id: Option<ResourceId>,
  index_buffer_id: Option<ResourceId>,
  index_count: u32,
  width: u32,
  height: u32,
}

impl Component<ComponentResult, String> for IndexedMultiBufferExample {
  fn on_attach(
    &mut self,
    render_context: &mut RenderContext,
  ) -> Result<ComponentResult, String> {
    let render_pass = RenderPassBuilder::new().build(
      render_context.gpu(),
      render_context.surface_format(),
      render_context.depth_format(),
    );

    // Quad composed from two triangles in clip space.
    let positions: Vec<PositionVertex> = vec![
      PositionVertex {
        position: [-0.5, -0.5, 0.0],
      },
      PositionVertex {
        position: [0.5, -0.5, 0.0],
      },
      PositionVertex {
        position: [0.5, 0.5, 0.0],
      },
      PositionVertex {
        position: [-0.5, 0.5, 0.0],
      },
    ];

    let colors: Vec<ColorVertex> = vec![
      ColorVertex {
        color: [1.0, 0.0, 0.0],
      },
      ColorVertex {
        color: [0.0, 1.0, 0.0],
      },
      ColorVertex {
        color: [0.0, 0.0, 1.0],
      },
      ColorVertex {
        color: [1.0, 1.0, 1.0],
      },
    ];

    let indices: Vec<u16> = vec![0, 1, 2, 2, 3, 0];
    let index_count = indices.len() as u32;

    // Build vertex buffers for positions and colors in separate slots.
    let position_buffer = BufferBuilder::new()
      .with_usage(Usage::VERTEX)
      .with_properties(Properties::DEVICE_LOCAL)
      .with_buffer_type(BufferType::Vertex)
      .with_label("indexed-positions")
      .build(render_context.gpu(), positions)
      .map_err(|e| e.to_string())?;

    let color_buffer = BufferBuilder::new()
      .with_usage(Usage::VERTEX)
      .with_properties(Properties::DEVICE_LOCAL)
      .with_buffer_type(BufferType::Vertex)
      .with_label("indexed-colors")
      .build(render_context.gpu(), colors)
      .map_err(|e| e.to_string())?;

    // Build a 16-bit index buffer.
    let index_buffer = BufferBuilder::new()
      .with_usage(Usage::INDEX)
      .with_properties(Properties::DEVICE_LOCAL)
      .with_buffer_type(BufferType::Index)
      .with_label("indexed-indices")
      .build(render_context.gpu(), indices)
      .map_err(|e| e.to_string())?;

    let pipeline = RenderPipelineBuilder::new()
      .with_culling(CullingMode::Back)
      .with_buffer(
        position_buffer,
        vec![VertexAttribute {
          location: 0,
          offset: 0,
          element: VertexElement {
            format: ColorFormat::Rgb32Sfloat,
            offset: 0,
          },
        }],
      )
      .with_buffer(
        color_buffer,
        vec![VertexAttribute {
          location: 1,
          offset: 0,
          element: VertexElement {
            format: ColorFormat::Rgb32Sfloat,
            offset: 0,
          },
        }],
      )
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

    logging::info!("Indexed multi-vertex-buffer example attached");
    return Ok(ComponentResult::Success);
  }

  fn on_detach(
    &mut self,
    _render_context: &mut RenderContext,
  ) -> Result<ComponentResult, String> {
    logging::info!("Indexed multi-vertex-buffer example detached");
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

  fn on_update(
    &mut self,
    _last_frame: &std::time::Duration,
  ) -> Result<ComponentResult, String> {
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
        instances: 0..1,
      },
      RenderCommand::EndRenderPass,
    ];
  }
}

impl Default for IndexedMultiBufferExample {
  fn default() -> Self {
    let vertex_virtual_shader = VirtualShader::Source {
      source: VERTEX_SHADER_SOURCE.to_string(),
      kind: ShaderKind::Vertex,
      entry_point: "main".to_string(),
      name: "indexed_multi_vertex_buffers".to_string(),
    };

    let fragment_virtual_shader = VirtualShader::Source {
      source: FRAGMENT_SHADER_SOURCE.to_string(),
      kind: ShaderKind::Fragment,
      entry_point: "main".to_string(),
      name: "indexed_multi_vertex_buffers".to_string(),
    };

    let mut builder = ShaderBuilder::new();
    let vertex_shader = builder.build(vertex_virtual_shader);
    let fragment_shader = builder.build(fragment_virtual_shader);

    return Self {
      vertex_shader,
      fragment_shader,
      render_pass_id: None,
      render_pipeline_id: None,
      index_buffer_id: None,
      index_count: 0,
      width: 800,
      height: 600,
    };
  }
}

fn main() {
  let runtime =
    ApplicationRuntimeBuilder::new("Indexed Multi-Vertex-Buffer Example")
      .with_window_configured_as(move |window_builder| {
        return window_builder
          .with_dimensions(800, 600)
          .with_name("Indexed Multi-Vertex-Buffer Example");
      })
      .with_renderer_configured_as(|renderer_builder| {
        return renderer_builder.with_render_timeout(1_000_000_000);
      })
      .with_component(move |runtime, example: IndexedMultiBufferExample| {
        return (runtime, example);
      })
      .build();

  start_runtime(runtime);
}
