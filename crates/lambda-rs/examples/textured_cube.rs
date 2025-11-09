#![allow(clippy::needless_return)]

//! Example: Spinning 3D cube sampled with a 3D texture.
//! - Uses MVP push constants (vertex stage) for classic camera + rotation.
//! - Colors come from a 2D checkerboard texture sampled in the fragment
//!   shader. Each face projects model-space coordinates to UVs.

use lambda::{
  component::Component,
  logging,
  math::matrix::Matrix,
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
      PipelineStage,
      RenderPipelineBuilder,
    },
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
layout (location = 1) in vec3 vertex_normal;
layout (location = 2) in vec3 vertex_color; // unused

layout (location = 0) out vec3 v_model_pos;
layout (location = 1) out vec3 v_normal;

layout ( push_constant ) uniform Push {
  mat4 mvp;
} pc;

void main() {
  gl_Position = pc.mvp * vec4(vertex_position, 1.0);
  v_model_pos = vertex_position;
  v_normal = vertex_normal;
}

"#;

const FRAGMENT_SHADER_SOURCE: &str = r#"
#version 450

layout (location = 0) in vec3 v_model_pos;
layout (location = 1) in vec3 v_normal;

layout (location = 0) out vec4 fragment_color;

layout (set = 0, binding = 1) uniform texture2D tex;
layout (set = 0, binding = 2) uniform sampler samp;

// Project model-space position to 2D UVs based on the dominant normal axis.
vec2 project_uv(vec3 p, vec3 n) {
  vec3 a = abs(n);
  if (a.x > a.y && a.x > a.z) {
    // +/-X faces: map Z,Y
    return p.zy * 0.5 + 0.5;
  } else if (a.y > a.z) {
    // +/-Y faces: map X,Z
    return p.xz * 0.5 + 0.5;
  } else {
    // +/-Z faces: map X,Y
    return p.xy * 0.5 + 0.5;
  }
}

void main() {
  // Sample color from 2D checkerboard using projected UVs in [0,1]
  vec3 N = normalize(v_normal);
  vec2 uv = clamp(project_uv(v_model_pos, N), 0.0, 1.0);
  vec3 base = texture(sampler2D(tex, samp), uv).rgb;

  // Simple lambert lighting to emphasize shape
  vec3 L = normalize(vec3(0.4, 0.7, 1.0));
  float diff = max(dot(N, L), 0.0);
  vec3 color = base * (0.25 + 0.75 * diff);
  fragment_color = vec4(color, 1.0);
}

"#;

// ------------------------------ PUSH CONSTANTS -------------------------------

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PushConstant {
  mvp: [[f32; 4]; 4],
}

pub fn push_constants_to_bytes(push_constants: &PushConstant) -> &[u32] {
  unsafe {
    let size_in_bytes = std::mem::size_of::<PushConstant>();
    let size_in_u32 = size_in_bytes / std::mem::size_of::<u32>();
    let ptr = push_constants as *const PushConstant as *const u32;
    std::slice::from_raw_parts(ptr, size_in_u32)
  }
}

// --------------------------------- COMPONENT ---------------------------------

pub struct TexturedCubeExample {
  shader_vs: Shader,
  shader_fs: Shader,
  mesh: Option<Mesh>,
  render_pipeline: Option<ResourceId>,
  render_pass: Option<ResourceId>,
  bind_group: Option<ResourceId>,
  width: u32,
  height: u32,
  elapsed: f32,
}

impl Component<ComponentResult, String> for TexturedCubeExample {
  fn on_attach(
    &mut self,
    render_context: &mut lambda::render::RenderContext,
  ) -> Result<ComponentResult, String> {
    logging::info!("Attaching TexturedCubeExample");
    // Render pass and shaders
    let render_pass = RenderPassBuilder::new()
      .with_label("textured-cube-pass")
      .with_depth()
      .build(render_context);

    let mut shader_builder = ShaderBuilder::new();
    let shader_vs = shader_builder.build(VirtualShader::Source {
      source: VERTEX_SHADER_SOURCE.to_string(),
      kind: ShaderKind::Vertex,
      entry_point: "main".to_string(),
      name: "textured-cube".to_string(),
    });
    let shader_fs = shader_builder.build(VirtualShader::Source {
      source: FRAGMENT_SHADER_SOURCE.to_string(),
      kind: ShaderKind::Fragment,
      entry_point: "main".to_string(),
      name: "textured-cube".to_string(),
    });

    // Build a unit cube centered at origin (6 faces × 2 tris × 3 verts)
    let mut verts: Vec<Vertex> = Vec::new();
    let mut add_face =
      |nx: f32, ny: f32, nz: f32, corners: [(f32, f32, f32); 4]| {
        // corners are 4 corners in CCW order on that face, coords in [-0.5,0.5]
        let n = [nx, ny, nz];
        let v = |p: (f32, f32, f32)| {
          VertexBuilder::new()
            .with_position([p.0, p.1, p.2])
            .with_normal(n)
            .with_color([0.0, 0.0, 0.0])
            .build()
        };
        // Two triangles: (0,1,2) and (0,2,3)
        let p0 = v(corners[0]);
        let p1 = v(corners[1]);
        let p2 = v(corners[2]);
        let p3 = v(corners[3]);
        verts.push(p0);
        verts.push(p1);
        verts.push(p2);
        verts.push(p0);
        verts.push(p2);
        verts.push(p3);
      };
    let h = 0.5f32;
    // +X (corrected CCW winding)
    add_face(
      1.0,
      0.0,
      0.0,
      [(h, -h, -h), (h, h, -h), (h, h, h), (h, -h, h)],
    );
    // -X (corrected CCW winding)
    add_face(
      -1.0,
      0.0,
      0.0,
      [(-h, -h, -h), (-h, -h, h), (-h, h, h), (-h, h, -h)],
    );
    // +Y (original correct winding)
    add_face(
      0.0,
      1.0,
      0.0,
      [(-h, h, -h), (h, h, -h), (h, h, h), (-h, h, h)],
    );
    // -Y (original correct winding)
    add_face(
      0.0,
      -1.0,
      0.0,
      [(-h, -h, h), (h, -h, h), (h, -h, -h), (-h, -h, -h)],
    );
    // +Z
    add_face(
      0.0,
      0.0,
      1.0,
      [(-h, -h, h), (h, -h, h), (h, h, h), (-h, h, h)],
    );
    // -Z
    add_face(
      0.0,
      0.0,
      -1.0,
      [(h, -h, -h), (-h, -h, -h), (-h, h, -h), (h, h, -h)],
    );

    let mut mesh_builder = MeshBuilder::new();
    for v in verts.into_iter() {
      mesh_builder.with_vertex(v);
    }
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

    // 2D checkerboard texture used on all faces
    let tex_w = 64u32;
    let tex_h = 64u32;
    let mut pixels = vec![0u8; (tex_w * tex_h * 4) as usize];
    for y in 0..tex_h {
      for x in 0..tex_w {
        let i = ((y * tex_w + x) * 4) as usize;
        let checker = ((x / 8) % 2) ^ ((y / 8) % 2);
        let c: u8 = if checker == 0 { 40 } else { 220 };
        pixels[i + 0] = c;
        pixels[i + 1] = c;
        pixels[i + 2] = c;
        pixels[i + 3] = 255;
      }
    }

    let texture2d = TextureBuilder::new_2d(TextureFormat::Rgba8UnormSrgb)
      .with_size(tex_w, tex_h)
      .with_data(&pixels)
      .with_label("checkerboard")
      .build(render_context)
      .expect("Failed to create 2D texture");
    let sampler = SamplerBuilder::new().linear_clamp().build(render_context);

    let layout = BindGroupLayoutBuilder::new()
      .with_sampled_texture(1)
      .with_sampler(2)
      .build(render_context);
    let bind_group = BindGroupBuilder::new()
      .with_layout(&layout)
      .with_texture(1, &texture2d)
      .with_sampler(2, &sampler)
      .build(render_context);

    let push_constants_size = std::mem::size_of::<PushConstant>() as u32;
    let pipeline = RenderPipelineBuilder::new()
      .with_culling(lambda::render::pipeline::CullingMode::Back)
      .with_depth()
      .with_push_constant(PipelineStage::VERTEX, push_constants_size)
      .with_buffer(
        BufferBuilder::build_from_mesh(&mesh, render_context)
          .expect("Failed to create vertex buffer"),
        mesh.attributes().to_vec(),
      )
      .with_layouts(&[&layout])
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

  fn on_event(
    &mut self,
    event: lambda::events::Events,
  ) -> Result<ComponentResult, String> {
    match event {
      lambda::events::Events::Window { event, .. } => match event {
        lambda::events::WindowEvent::Resize { width, height } => {
          self.width = width;
          self.height = height;
        }
        _ => {}
      },
      _ => {}
    }
    return Ok(ComponentResult::Success);
  }

  fn on_update(
    &mut self,
    last_frame: &std::time::Duration,
  ) -> Result<ComponentResult, String> {
    self.elapsed += last_frame.as_secs_f32();
    return Ok(ComponentResult::Success);
  }

  fn on_render(
    &mut self,
    _render_context: &mut lambda::render::RenderContext,
  ) -> Vec<RenderCommand> {
    // Camera and rotation
    let camera = SimpleCamera {
      position: [0.0, 0.0, 2.2],
      field_of_view_in_turns: 0.24,
      near_clipping_plane: 0.1,
      far_clipping_plane: 100.0,
    };
    let angle_turns = 0.15 * self.elapsed; // slow rotation
    let mvp = compute_model_view_projection_matrix_about_pivot(
      &camera,
      self.width.max(1),
      self.height.max(1),
      [0.0, 0.0, 0.0], // pivot
      [0.0, 1.0, 0.0], // axis
      angle_turns,
      1.0,             // scale
      [0.0, 0.0, 0.0], // translation
    );

    let viewport = ViewportBuilder::new().build(self.width, self.height);
    let pipeline = self.render_pipeline.expect("pipeline not set");
    let group = self.bind_group.expect("bind group not set");
    let mesh_len = self.mesh.as_ref().unwrap().vertices().len() as u32;

    return vec![
      RenderCommand::BeginRenderPass {
        render_pass: self.render_pass.expect("render pass not set"),
        viewport: viewport.clone(),
      },
      RenderCommand::SetPipeline { pipeline },
      RenderCommand::SetViewports {
        start_at: 0,
        viewports: vec![viewport.clone()],
      },
      RenderCommand::SetScissors {
        start_at: 0,
        viewports: vec![viewport.clone()],
      },
      RenderCommand::SetBindGroup {
        set: 0,
        group,
        dynamic_offsets: vec![],
      },
      RenderCommand::BindVertexBuffer {
        pipeline,
        buffer: 0,
      },
      RenderCommand::PushConstants {
        pipeline,
        stage: PipelineStage::VERTEX,
        offset: 0,
        bytes: Vec::from(push_constants_to_bytes(&PushConstant {
          mvp: mvp.transpose(),
        })),
      },
      RenderCommand::Draw {
        vertices: 0..mesh_len,
      },
      RenderCommand::EndRenderPass,
    ];
  }
}

impl Default for TexturedCubeExample {
  fn default() -> Self {
    let mut shader_builder = ShaderBuilder::new();
    let shader_vs = shader_builder.build(VirtualShader::Source {
      source: VERTEX_SHADER_SOURCE.to_string(),
      kind: ShaderKind::Vertex,
      entry_point: "main".to_string(),
      name: "textured-cube".to_string(),
    });
    let shader_fs = shader_builder.build(VirtualShader::Source {
      source: FRAGMENT_SHADER_SOURCE.to_string(),
      kind: ShaderKind::Fragment,
      entry_point: "main".to_string(),
      name: "textured-cube".to_string(),
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
      elapsed: 0.0,
    };
  }
}

fn main() {
  let runtime: ApplicationRuntime =
    ApplicationRuntimeBuilder::new("Textured Cube Example")
      .with_window_configured_as(|builder| {
        builder.with_dimensions(800, 600).with_name("Textured Cube")
      })
      .with_component(|runtime, example: TexturedCubeExample| {
        (runtime, example)
      })
      .build();

  start_runtime(runtime);
}
