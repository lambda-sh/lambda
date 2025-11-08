#![allow(clippy::needless_return)]

//! Example: Draw a quad sampling a 3D texture slice with a push constant.

use lambda::{
  component::Component,
  events::Events,
  logging,
  render::{
    bind::{
      BindGroupBuilder,
      BindGroupLayoutBuilder,
      BindingVisibility,
    },
    buffer::BufferBuilder,
    command::RenderCommand,
    mesh::{
      Mesh,
      MeshBuilder,
    },
    pipeline::{
      CullingMode,
      PipelineStage,
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
      ViewDimension,
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

layout (set = 0, binding = 1) uniform texture3D tex3d;
layout (set = 0, binding = 2) uniform sampler samp;

layout (push_constant) uniform Push {
  float pitch; // rotation around X
  float yaw;   // rotation around Y
} pc;

mat3 rot_x(float a) {
  float c = cos(a);
  float s = sin(a);
  return mat3(
    1.0, 0.0, 0.0,
    0.0,   c,  -s,
    0.0,   s,   c
  );
}

mat3 rot_y(float a) {
  float c = cos(a);
  float s = sin(a);
  return mat3(
      c, 0.0,   s,
    0.0, 1.0, 0.0,
     -s, 0.0,   c
  );
}

bool intersect_box(
  vec3 ro,
  vec3 rd,
  vec3 bmin,
  vec3 bmax,
  out float t0,
  out float t1
) {
  vec3 inv = 1.0 / rd;
  vec3 tminTemp = (bmin - ro) * inv;
  vec3 tmaxTemp = (bmax - ro) * inv;
  vec3 tsmaller = min(tminTemp, tmaxTemp);
  vec3 tbigger = max(tminTemp, tmaxTemp);
  t0 = max(max(tsmaller.x, tsmaller.y), tsmaller.z);
  t1 = min(min(tbigger.x, tbigger.y), tbigger.z);
  return t1 >= max(t0, 0.0);
}

void main() {
  // Screen to ray setup
  vec2 uv_ndc = v_uv * 2.0 - 1.0; // [-1,1]
  float fov = 1.2;
  vec3 ro = vec3(0.5, 0.5, -2.0);
  // Camera ray through pixel; keep camera stable and rotate the volume instead
  vec3 rd = normalize(vec3(uv_ndc.x * fov, uv_ndc.y * fov, 1.5));

  float t0, t1;
  bool hit = intersect_box(ro, rd, vec3(0.0), vec3(1.0), t0, t1);
  if (!hit) { fragment_color = vec4(0.02, 0.02, 0.03, 1.0); return; }

  t0 = max(t0, 0.0);
  int STEPS = 64;
  float dt = (t1 - t0) / float(STEPS);
  vec3 color = vec3(0.0);
  float alpha = 0.0;

  mat3 R = rot_y(pc.yaw) * rot_x(pc.pitch);
  vec3 center = vec3(0.5);
  for (int i = 0; i < STEPS; ++i) {
    float t = t0 + (float(i) + 0.5) * dt;
    vec3 pos = ro + rd * t;          // position in [0,1]^3
    // Rotate the volume about its center; sample rotated coords
    vec3 pos_rot = R * (pos - center) + center;
    vec3 vox = texture(sampler3D(tex3d, samp), pos_rot).rgb;
    float density = clamp((vox.r + vox.g + vox.b) * 0.3333, 0.0, 1.0);
    // Convert density to opacity; small scale for pleasant appearance
    float a = 1.0 - exp(-density * 6.0 * dt * 64.0);
    color += (1.0 - alpha) * vox * a;
    alpha += (1.0 - alpha) * a;
    if (alpha > 0.98) break;
  }

  color = mix(vec3(0.02, 0.02, 0.03), color, clamp(alpha, 0.0, 1.0));
  fragment_color = vec4(color, 1.0);
}

"#;

// ------------------------------- PUSH CONSTANT -------------------------------

#[repr(C)]
#[derive(Copy, Clone)]
struct PushAngles {
  pitch: f32,
  yaw: f32,
}

fn push_constant_to_words(pc: &PushAngles) -> &[u32] {
  unsafe {
    std::slice::from_raw_parts(
      (pc as *const PushAngles) as *const u32,
      std::mem::size_of::<PushAngles>() / 4,
    )
  }
}

// --------------------------------- COMPONENT ---------------------------------

pub struct TexturedVolumeExample {
  shader_vs: Shader,
  shader_fs: Shader,
  mesh: Option<Mesh>,
  render_pipeline: Option<ResourceId>,
  render_pass: Option<ResourceId>,
  bind_group: Option<ResourceId>,
  width: u32,
  height: u32,
  pitch: f32,
  yaw: f32,
  pitch_speed: f32,
  yaw_speed: f32,
}

impl Component<ComponentResult, String> for TexturedVolumeExample {
  fn on_attach(
    &mut self,
    render_context: &mut lambda::render::RenderContext,
  ) -> Result<ComponentResult, String> {
    logging::info!("Attaching TexturedVolumeExample");

    // Build render pass and shaders
    let render_pass = RenderPassBuilder::new()
      .with_label("textured-volume-pass")
      .build(render_context);

    let mut shader_builder = ShaderBuilder::new();
    let shader_vs = shader_builder.build(VirtualShader::Source {
      source: VERTEX_SHADER_SOURCE.to_string(),
      kind: ShaderKind::Vertex,
      entry_point: "main".to_string(),
      name: "textured-volume".to_string(),
    });
    let shader_fs = shader_builder.build(VirtualShader::Source {
      source: FRAGMENT_SHADER_SOURCE.to_string(),
      kind: ShaderKind::Fragment,
      entry_point: "main".to_string(),
      name: "textured-volume".to_string(),
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

    // Create a small 3D volume (gradient)
    let (w, h, d) = (32u32, 32u32, 32u32);
    let mut voxels = vec![0u8; (w * h * d * 4) as usize];
    for zz in 0..d {
      for yy in 0..h {
        for xx in 0..w {
          let idx = ((zz * h * w + yy * w + xx) * 4) as usize;
          let r = ((xx as f32) / (w as f32 - 1.0) * 255.0) as u8;
          let g = ((yy as f32) / (h as f32 - 1.0) * 255.0) as u8;
          let b = ((zz as f32) / (d as f32 - 1.0) * 255.0) as u8;
          voxels[idx + 0] = r;
          voxels[idx + 1] = g;
          voxels[idx + 2] = b;
          voxels[idx + 3] = 255;
        }
      }
    }

    let texture3d = TextureBuilder::new_3d(TextureFormat::Rgba8UnormSrgb)
      .with_size_3d(w, h, d)
      .with_data(&voxels)
      .with_label("volume-gradient")
      .build(render_context)
      .expect("Failed to create 3D texture");

    let sampler = SamplerBuilder::new()
      .linear_clamp()
      .with_label("linear-clamp")
      .build(render_context);

    // Layout: binding(1) texture3D, binding(2) sampler
    let layout = BindGroupLayoutBuilder::new()
      .with_sampled_texture_dim(
        1,
        ViewDimension::D3,
        BindingVisibility::Fragment,
      )
      .with_sampler(2)
      .build(render_context);

    let bind_group = BindGroupBuilder::new()
      .with_layout(&layout)
      .with_texture(1, &texture3d)
      .with_sampler(2, &sampler)
      .build(render_context);

    let push_constants_size = std::mem::size_of::<PushAngles>() as u32;
    let pipeline = RenderPipelineBuilder::new()
      .with_culling(CullingMode::None)
      .with_layouts(&[&layout])
      .with_buffer(
        BufferBuilder::build_from_mesh(&mesh, render_context)
          .expect("Failed to create vertex buffer"),
        mesh.attributes().to_vec(),
      )
      .with_push_constant(PipelineStage::FRAGMENT, push_constants_size)
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
    match event {
      Events::Window {
        event: lambda::events::WindowEvent::Resize { width, height },
        ..
      } => {
        self.width = width;
        self.height = height;
      }
      Events::Keyboard { event, .. } => match event {
        lambda::events::Key::Pressed { virtual_key, .. } => match virtual_key {
          Some(lambda::events::VirtualKey::KeyW) => {
            self.pitch_speed += 0.1;
          }
          Some(lambda::events::VirtualKey::KeyS) => {
            self.pitch_speed -= 0.1;
          }
          Some(lambda::events::VirtualKey::KeyA) => {
            self.yaw_speed -= 0.1;
          }
          Some(lambda::events::VirtualKey::KeyD) => {
            self.yaw_speed += 0.1;
          }
          Some(lambda::events::VirtualKey::Space) => {
            self.pitch_speed = 0.0;
            self.yaw_speed = 0.0;
          }
          _ => {}
        },
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
    // Auto-rotate slice orientation over time.
    let dt = (last_frame.as_micros() as f32) / 1_000_000.0;
    self.pitch += self.pitch_speed * dt;
    self.yaw += self.yaw_speed * dt;
    // Wrap angles to [-pi, pi] for numerical stability
    let pi = std::f32::consts::PI;
    if self.pitch > pi {
      self.pitch -= 2.0 * pi;
    }
    if self.pitch < -pi {
      self.pitch += 2.0 * pi;
    }
    if self.yaw > pi {
      self.yaw -= 2.0 * pi;
    }
    if self.yaw < -pi {
      self.yaw += 2.0 * pi;
    }
    return Ok(ComponentResult::Success);
  }

  fn on_render(
    &mut self,
    _render_context: &mut lambda::render::RenderContext,
  ) -> Vec<RenderCommand> {
    let mut commands = vec![];
    // Render to full window dimensions.
    let viewport = ViewportBuilder::new().build(self.width, self.height);

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

    let pc = PushAngles {
      pitch: self.pitch,
      yaw: self.yaw,
    };
    commands.push(RenderCommand::PushConstants {
      pipeline: self.render_pipeline.expect("pipeline not set"),
      stage: PipelineStage::FRAGMENT,
      offset: 0,
      bytes: Vec::from(push_constant_to_words(&pc)),
    });

    commands.push(RenderCommand::Draw { vertices: 0..6 });
    commands.push(RenderCommand::EndRenderPass);
    return commands;
  }
}

impl Default for TexturedVolumeExample {
  fn default() -> Self {
    let mut shader_builder = ShaderBuilder::new();
    let shader_vs = shader_builder.build(VirtualShader::Source {
      source: VERTEX_SHADER_SOURCE.to_string(),
      kind: ShaderKind::Vertex,
      entry_point: "main".to_string(),
      name: "textured-volume".to_string(),
    });
    let shader_fs = shader_builder.build(VirtualShader::Source {
      source: FRAGMENT_SHADER_SOURCE.to_string(),
      kind: ShaderKind::Fragment,
      entry_point: "main".to_string(),
      name: "textured-volume".to_string(),
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
      pitch: 0.0,
      yaw: 0.0,
      pitch_speed: 0.7,
      yaw_speed: 0.45,
    };
  }
}

fn main() {
  let runtime: ApplicationRuntime =
    ApplicationRuntimeBuilder::new("3D Texture Slice Example")
      .with_window_configured_as(|builder| {
        builder
          .with_dimensions(800, 600)
          .with_name("Texture 3D Slice")
      })
      .with_component(|runtime, example: TexturedVolumeExample| {
        (runtime, example)
      })
      .build();

  start_runtime(runtime);
}
