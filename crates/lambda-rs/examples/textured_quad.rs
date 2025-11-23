#![allow(clippy::needless_return)]

//! Example: Draw a textured quad using a sampled 2D texture and sampler.

use lambda::{
  component::Component,
  events::Events,
  logging,
  render::{
    bind::{
      BindGroupBuilder,
      BindGroupLayoutBuilder,
    },
    buffer::BufferBuilder,
    command::RenderCommand,
    mesh::{
      Mesh,
      MeshBuilder,
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
    texture::{
      SamplerBuilder,
      TextureBuilder,
      TextureFormat,
    },
    vertex::{
      ColorFormat,
      Vertex,
      VertexAttribute,
      VertexBuilder,
      VertexElement,
    },
    viewport::ViewportBuilder,
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
layout (location = 2) in vec3 vertex_color; // uv packed into .xy

layout (location = 0) out vec2 v_uv;

void main() {
  gl_Position = vec4(vertex_position, 1.0);
  v_uv = vertex_color.xy;
}

"#;

const FRAGMENT_SHADER_SOURCE: &str = r#"
#version 450

layout (location = 0) in vec2 v_uv;
layout (location = 0) out vec4 fragment_color;

layout (set = 0, binding = 1) uniform texture2D tex;
layout (set = 0, binding = 2) uniform sampler samp;

void main() {
  fragment_color = texture(sampler2D(tex, samp), v_uv);
}

"#;

// --------------------------------- COMPONENT ---------------------------------

pub struct TexturedQuadExample {
  shader_vs: Shader,
  shader_fs: Shader,
  mesh: Option<Mesh>,
  render_pipeline: Option<ResourceId>,
  render_pass: Option<ResourceId>,
  bind_group: Option<ResourceId>,
  width: u32,
  height: u32,
}

impl Component<ComponentResult, String> for TexturedQuadExample {
  fn on_attach(
    &mut self,
    render_context: &mut lambda::render::RenderContext,
  ) -> Result<ComponentResult, String> {
    logging::info!("Attaching TexturedQuadExample");

    // Build render pass and shaders
    let render_pass = RenderPassBuilder::new()
      .with_label("textured-quad-pass")
      .build(render_context);

    let mut shader_builder = ShaderBuilder::new();
    let shader_vs = shader_builder.build(VirtualShader::Source {
      source: VERTEX_SHADER_SOURCE.to_string(),
      kind: ShaderKind::Vertex,
      entry_point: "main".to_string(),
      name: "textured-quad".to_string(),
    });
    let shader_fs = shader_builder.build(VirtualShader::Source {
      source: FRAGMENT_SHADER_SOURCE.to_string(),
      kind: ShaderKind::Fragment,
      entry_point: "main".to_string(),
      name: "textured-quad".to_string(),
    });

    // Quad vertices (two triangles), uv packed into vertex.color.xy
    let vertices: [Vertex; 6] = [
      VertexBuilder::new()
        .with_position([-0.5, -0.5, 0.0])
        .with_normal([0.0, 0.0, 1.0])
        .with_color([0.0, 0.0, 0.0])
        .build(), // uv (0,0)
      VertexBuilder::new()
        .with_position([0.5, -0.5, 0.0])
        .with_normal([0.0, 0.0, 1.0])
        .with_color([1.0, 0.0, 0.0])
        .build(), // uv (1,0)
      VertexBuilder::new()
        .with_position([0.5, 0.5, 0.0])
        .with_normal([0.0, 0.0, 1.0])
        .with_color([1.0, 1.0, 0.0])
        .build(), // uv (1,1)
      VertexBuilder::new()
        .with_position([-0.5, -0.5, 0.0])
        .with_normal([0.0, 0.0, 1.0])
        .with_color([0.0, 0.0, 0.0])
        .build(), // uv (0,0)
      VertexBuilder::new()
        .with_position([0.5, 0.5, 0.0])
        .with_normal([0.0, 0.0, 1.0])
        .with_color([1.0, 1.0, 0.0])
        .build(), // uv (1,1)
      VertexBuilder::new()
        .with_position([-0.5, 0.5, 0.0])
        .with_normal([0.0, 0.0, 1.0])
        .with_color([0.0, 1.0, 0.0])
        .build(), // uv (0,1)
    ];

    let mut mesh_builder = MeshBuilder::new();
    vertices.iter().for_each(|v| {
      mesh_builder.with_vertex(*v);
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

    // Create a small checkerboard texture
    let tex_w = 64u32;
    let tex_h = 64u32;
    let mut pixels = vec![0u8; (tex_w * tex_h * 4) as usize];
    for y in 0..tex_h {
      for x in 0..tex_w {
        let i = ((y * tex_w + x) * 4) as usize;
        let checker = ((x / 8) % 2) ^ ((y / 8) % 2);
        let c = if checker == 0 { 40 } else { 220 };
        pixels[i + 0] = c; // R
        pixels[i + 1] = c; // G
        pixels[i + 2] = c; // B
        pixels[i + 3] = 255; // A
      }
    }

    let texture = TextureBuilder::new_2d(TextureFormat::Rgba8UnormSrgb)
      .with_size(tex_w, tex_h)
      .with_data(&pixels)
      .with_label("checkerboard")
      .build(render_context)
      .expect("Failed to create texture");

    let sampler = SamplerBuilder::new()
      .linear_clamp()
      .with_label("linear-clamp")
      .build(render_context);

    // Layout: binding(1) texture2D, binding(2) sampler
    let layout = BindGroupLayoutBuilder::new()
      .with_sampled_texture(1)
      .with_sampler(2)
      .build(render_context);

    let bind_group = BindGroupBuilder::new()
      .with_layout(&layout)
      .with_texture(1, &texture)
      .with_sampler(2, &sampler)
      .build(render_context);

    let pipeline = RenderPipelineBuilder::new()
      .with_culling(CullingMode::None)
      .with_layouts(&[&layout])
      .with_buffer(
        BufferBuilder::build_from_mesh(&mesh, render_context)
          .expect("Failed to create vertex buffer"),
        mesh.attributes().to_vec(),
      )
      .build(render_context, &render_pass, &shader_vs, Some(&shader_fs));

    self.render_pass = Some(render_context.attach_render_pass(render_pass));
    self.render_pipeline = Some(render_context.attach_pipeline(pipeline));
    self.bind_group = Some(render_context.attach_bind_group(bind_group));
    self.mesh = Some(mesh);
    self.shader_vs = shader_vs;
    self.shader_fs = shader_fs;

    return Ok(ComponentResult::Success);
  }

  fn on_detach(
    &mut self,
    _render_context: &mut lambda::render::RenderContext,
  ) -> Result<ComponentResult, String> {
    return Ok(ComponentResult::Success);
  }

  fn on_event(&mut self, event: Events) -> Result<ComponentResult, String> {
    if let Events::Window {
      event: lambda::events::WindowEvent::Resize { width, height },
      ..
    } = event
    {
      self.width = width;
      self.height = height;
    }
    return Ok(ComponentResult::Success);
  }

  fn on_update(
    &mut self,
    _last_frame: &std::time::Duration,
  ) -> Result<ComponentResult, String> {
    return Ok(ComponentResult::Success);
  }

  fn on_render(
    &mut self,
    _render_context: &mut lambda::render::RenderContext,
  ) -> Vec<RenderCommand> {
    let mut commands = vec![];
    // Center a square viewport to keep aspect ratio and center the quad.
    let win_w = self.width.max(1);
    let win_h = self.height.max(1);
    let side = u32::min(win_w, win_h);
    let x = ((win_w - side) / 2) as i32;
    let y = ((win_h - side) / 2) as i32;
    let viewport = ViewportBuilder::new()
      .with_coordinates(x, y)
      .build(side, side);
    commands.push(RenderCommand::BeginRenderPass {
      render_pass: self.render_pass.expect("render pass not set"),
      viewport,
    });
    commands.push(RenderCommand::SetPipeline {
      pipeline: self.render_pipeline.expect("pipeline not set"),
    });
    commands.push(RenderCommand::SetBindGroup {
      set: 0,
      group: self.bind_group.expect("bind group not set"),
      dynamic_offsets: vec![],
    });
    commands.push(RenderCommand::BindVertexBuffer {
      pipeline: self.render_pipeline.expect("pipeline not set"),
      buffer: 0,
    });
    commands.push(RenderCommand::Draw {
      vertices: 0..6,
      instances: 0..1,
    });
    commands.push(RenderCommand::EndRenderPass);
    return commands;
  }
}

impl Default for TexturedQuadExample {
  fn default() -> Self {
    let mut shader_builder = ShaderBuilder::new();
    let shader_vs = shader_builder.build(VirtualShader::Source {
      source: VERTEX_SHADER_SOURCE.to_string(),
      kind: ShaderKind::Vertex,
      entry_point: "main".to_string(),
      name: "textured-quad".to_string(),
    });
    let shader_fs = shader_builder.build(VirtualShader::Source {
      source: FRAGMENT_SHADER_SOURCE.to_string(),
      kind: ShaderKind::Fragment,
      entry_point: "main".to_string(),
      name: "textured-quad".to_string(),
    });

    return Self {
      shader_vs,
      shader_fs,
      mesh: None,
      render_pipeline: None,
      render_pass: None,
      bind_group: None,
      width: 800,
      height: 600,
    };
  }
}

fn main() {
  let runtime: ApplicationRuntime =
    ApplicationRuntimeBuilder::new("Textured Quad Example")
      .with_window_configured_as(|builder| {
        builder.with_dimensions(800, 600).with_name("Textured Quad")
      })
      .with_component(|runtime, example: TexturedQuadExample| {
        (runtime, example)
      })
      .build();

  start_runtime(runtime);
}
