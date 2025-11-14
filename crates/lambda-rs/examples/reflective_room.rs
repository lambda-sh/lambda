#![allow(clippy::needless_return)]

//! Example: Reflective floor using the stencil buffer with MSAA.
//!
//! - Phase 1: Depth/stencil-only pass to write a stencil mask where the floor
//!   geometry exists. Depth writes are disabled for the mask.
//! - Phase 2: Render a mirrored (reflected) cube only where stencil == 1.
//!   Disable culling to avoid backface issues due to the mirrored transform.
//! - Phase 3 (optional visual): Draw the floor surface with alpha so the
//!   reflection appears as if seen in a mirror.
//! - Phase 4: Render the normal, unreflected cube above the floor.
//!
//! The pass enables depth testing/clears and 4x MSAA for smoother edges.

use lambda::{
  component::Component,
  logging,
  math::matrix::Matrix,
  render::{
    buffer::BufferBuilder,
    command::RenderCommand,
    mesh::{
      Mesh,
      MeshBuilder,
    },
    pipeline::{
      CompareFunction,
      CullingMode,
      PipelineStage,
      RenderPipelineBuilder,
      StencilFaceState,
      StencilOperation,
      StencilState,
    },
    render_pass::RenderPassBuilder,
    scene_math::{
      compute_perspective_projection,
      compute_view_matrix,
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
use lambda_platform::wgpu::texture::DepthFormat;

// ------------------------------ SHADER SOURCE --------------------------------

const VERTEX_SHADER_SOURCE: &str = r#"
#version 450

layout (location = 0) in vec3 vertex_position;
layout (location = 1) in vec3 vertex_normal;

layout (location = 0) out vec3 v_world_normal;

layout ( push_constant ) uniform Push {
  mat4 mvp;
  mat4 model;
} pc;

void main() {
  gl_Position = pc.mvp * vec4(vertex_position, 1.0);
  // Rotate normals into world space using the model matrix (no scale/shear needed for this demo).
  v_world_normal = mat3(pc.model) * vertex_normal;
}
"#;

const FRAGMENT_LIT_COLOR_SOURCE: &str = r#"
#version 450

layout (location = 0) in vec3 v_world_normal;
layout (location = 0) out vec4 fragment_color;

void main() {
  vec3 N = normalize(v_world_normal);
  vec3 L = normalize(vec3(0.4, 0.7, 1.0));
  float diff = max(dot(N, L), 0.0);
  vec3 base = vec3(0.2, 0.6, 0.9);
  vec3 color = base * (0.25 + 0.75 * diff);
  fragment_color = vec4(color, 1.0);
}
"#;

const FRAGMENT_FLOOR_TINT_SOURCE: &str = r#"
#version 450

layout (location = 0) out vec4 fragment_color;

void main() {
  // Slightly tint with alpha so the reflection appears through the floor.
  fragment_color = vec4(0.1, 0.1, 0.12, 0.5);
}
"#;

// (No extra fragment shaders needed; the floor mask uses a vertex-only pipeline.)

// ------------------------------ PUSH CONSTANTS -------------------------------

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PushConstant {
  mvp: [[f32; 4]; 4],
  model: [[f32; 4]; 4],
}

pub fn push_constants_to_words(push_constants: &PushConstant) -> &[u32] {
  unsafe {
    let size_in_bytes = std::mem::size_of::<PushConstant>();
    let size_in_u32 = size_in_bytes / std::mem::size_of::<u32>();
    let ptr = push_constants as *const PushConstant as *const u32;
    return std::slice::from_raw_parts(ptr, size_in_u32);
  }
}

// --------------------------------- COMPONENT ---------------------------------

pub struct ReflectiveRoomExample {
  shader_vs: Shader,
  shader_fs_lit: Shader,
  shader_fs_floor: Shader,
  cube_mesh: Option<Mesh>,
  floor_mesh: Option<Mesh>,
  pass_id_mask: Option<ResourceId>,
  pass_id_color: Option<ResourceId>,
  pipe_floor_mask: Option<ResourceId>,
  pipe_reflected: Option<ResourceId>,
  pipe_floor_visual: Option<ResourceId>,
  pipe_normal: Option<ResourceId>,
  width: u32,
  height: u32,
  elapsed: f32,
}

impl Component<ComponentResult, String> for ReflectiveRoomExample {
  fn on_attach(
    &mut self,
    render_context: &mut lambda::render::RenderContext,
  ) -> Result<ComponentResult, String> {
    logging::info!("Attaching ReflectiveRoomExample");

    // Pass 1 (mask): depth clear, stencil clear, MSAA 4x, without color.
    let render_pass_mask = RenderPassBuilder::new()
      .with_label("reflective-room-pass-mask")
      .with_depth_clear(1.0)
      .with_stencil_clear(0)
      .with_multi_sample(4)
      .without_color()
      .build(render_context);

    // Pass 2 (color): color + depth clear, stencil LOAD (use mask), MSAA 4x.
    let render_pass_color = RenderPassBuilder::new()
      .with_label("reflective-room-pass-color")
      .with_depth_clear(1.0)
      .with_stencil_load()
      .with_multi_sample(4)
      .build(render_context);

    // Shaders
    let mut shader_builder = ShaderBuilder::new();
    let shader_vs = shader_builder.build(VirtualShader::Source {
      source: VERTEX_SHADER_SOURCE.to_string(),
      kind: ShaderKind::Vertex,
      entry_point: "main".to_string(),
      name: "reflective-room-vs".to_string(),
    });
    let shader_fs_lit = shader_builder.build(VirtualShader::Source {
      source: FRAGMENT_LIT_COLOR_SOURCE.to_string(),
      kind: ShaderKind::Fragment,
      entry_point: "main".to_string(),
      name: "reflective-room-fs-lit".to_string(),
    });
    let shader_fs_floor = shader_builder.build(VirtualShader::Source {
      source: FRAGMENT_FLOOR_TINT_SOURCE.to_string(),
      kind: ShaderKind::Fragment,
      entry_point: "main".to_string(),
      name: "reflective-room-fs-floor".to_string(),
    });
    // Note: the mask pipeline is vertex-only and will be used in a pass
    // without color attachments.

    // Geometry: cube (unit) and floor quad at y = 0.
    let cube_mesh = build_unit_cube_mesh();
    let floor_mesh = build_floor_quad_mesh(5.0);

    let push_constants_size = std::mem::size_of::<PushConstant>() as u32;

    // Stencil mask pipeline. Writes stencil=1 where the floor exists.
    let pipe_floor_mask = RenderPipelineBuilder::new()
      .with_label("floor-mask")
      .with_culling(CullingMode::Back)
      .with_depth_format(DepthFormat::Depth24PlusStencil8)
      .with_depth_write(false)
      .with_depth_compare(CompareFunction::Always)
      .with_push_constant(PipelineStage::VERTEX, push_constants_size)
      .with_buffer(
        BufferBuilder::build_from_mesh(&floor_mesh, render_context)
          .expect("Failed to create floor vertex buffer"),
        floor_mesh.attributes().to_vec(),
      )
      .with_stencil(StencilState {
        front: StencilFaceState {
          compare: CompareFunction::Always,
          fail_op: StencilOperation::Keep,
          depth_fail_op: StencilOperation::Keep,
          pass_op: StencilOperation::Replace,
        },
        back: StencilFaceState {
          compare: CompareFunction::Always,
          fail_op: StencilOperation::Keep,
          depth_fail_op: StencilOperation::Keep,
          pass_op: StencilOperation::Replace,
        },
        read_mask: 0xFF,
        write_mask: 0xFF,
      })
      .with_multi_sample(4)
      .build(render_context, &render_pass_mask, &shader_vs, None);

    // Reflected cube pipeline: stencil test Equal, depth test enabled, no culling.
    let pipe_reflected = RenderPipelineBuilder::new()
      .with_label("reflected-cube")
      .with_culling(CullingMode::None)
      .with_depth_format(DepthFormat::Depth24PlusStencil8)
      .with_depth_write(true)
      .with_depth_compare(CompareFunction::LessEqual)
      .with_push_constant(PipelineStage::VERTEX, push_constants_size)
      .with_buffer(
        BufferBuilder::build_from_mesh(&cube_mesh, render_context)
          .expect("Failed to create cube vertex buffer"),
        cube_mesh.attributes().to_vec(),
      )
      .with_stencil(StencilState {
        front: StencilFaceState {
          compare: CompareFunction::Equal,
          fail_op: StencilOperation::Keep,
          depth_fail_op: StencilOperation::Keep,
          pass_op: StencilOperation::Keep,
        },
        back: StencilFaceState {
          compare: CompareFunction::Equal,
          fail_op: StencilOperation::Keep,
          depth_fail_op: StencilOperation::Keep,
          pass_op: StencilOperation::Keep,
        },
        read_mask: 0xFF,
        write_mask: 0x00,
      })
      .with_multi_sample(4)
      .build(
        render_context,
        &render_pass_color,
        &shader_vs,
        Some(&shader_fs_lit),
      );

    // Floor visual pipeline: draw a tinted surface above the reflection.
    let pipe_floor_visual = RenderPipelineBuilder::new()
      .with_label("floor-visual")
      .with_culling(CullingMode::Back)
      .with_depth_format(DepthFormat::Depth24PlusStencil8)
      .with_depth_write(false)
      .with_depth_compare(CompareFunction::LessEqual)
      .with_push_constant(PipelineStage::VERTEX, push_constants_size)
      .with_buffer(
        BufferBuilder::build_from_mesh(&floor_mesh, render_context)
          .expect("Failed to create floor vertex buffer"),
        floor_mesh.attributes().to_vec(),
      )
      .with_multi_sample(4)
      .build(
        render_context,
        &render_pass_color,
        &shader_vs,
        Some(&shader_fs_floor),
      );

    // Normal (unreflected) cube pipeline: standard depth test.
    let pipe_normal = RenderPipelineBuilder::new()
      .with_label("cube-normal")
      .with_culling(CullingMode::Back)
      .with_depth_format(DepthFormat::Depth24PlusStencil8)
      .with_depth_write(true)
      .with_depth_compare(CompareFunction::Less)
      .with_push_constant(PipelineStage::VERTEX, push_constants_size)
      .with_buffer(
        BufferBuilder::build_from_mesh(&cube_mesh, render_context)
          .expect("Failed to create cube vertex buffer"),
        cube_mesh.attributes().to_vec(),
      )
      .with_multi_sample(4)
      .build(
        render_context,
        &render_pass_color,
        &shader_vs,
        Some(&shader_fs_lit),
      );

    self.pass_id_mask =
      Some(render_context.attach_render_pass(render_pass_mask));
    self.pass_id_color =
      Some(render_context.attach_render_pass(render_pass_color));
    self.pipe_floor_mask =
      Some(render_context.attach_pipeline(pipe_floor_mask));
    self.pipe_reflected = Some(render_context.attach_pipeline(pipe_reflected));
    self.pipe_floor_visual =
      Some(render_context.attach_pipeline(pipe_floor_visual));
    self.pipe_normal = Some(render_context.attach_pipeline(pipe_normal));
    self.cube_mesh = Some(cube_mesh);
    self.floor_mesh = Some(floor_mesh);
    self.shader_vs = shader_vs;
    self.shader_fs_lit = shader_fs_lit;
    self.shader_fs_floor = shader_fs_floor;

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
    // Camera
    let camera = SimpleCamera {
      position: [0.0, 1.2, 3.5],
      field_of_view_in_turns: 0.24,
      near_clipping_plane: 0.1,
      far_clipping_plane: 100.0,
    };

    // Cube animation
    let angle_y_turns = 0.12 * self.elapsed;
    let mut model: [[f32; 4]; 4] = lambda::math::matrix::identity_matrix(4, 4);
    model = lambda::math::matrix::rotate_matrix(
      model,
      [0.0, 1.0, 0.0],
      angle_y_turns,
    );
    // Translate cube upward by 0.5 on Y
    let t_up: [[f32; 4]; 4] =
      lambda::math::matrix::translation_matrix([0.0, 0.5, 0.0]);
    model = model.multiply(&t_up);

    let view = compute_view_matrix(camera.position);
    let projection = compute_perspective_projection(
      camera.field_of_view_in_turns,
      self.width.max(1),
      self.height.max(1),
      camera.near_clipping_plane,
      camera.far_clipping_plane,
    );
    let mvp = projection.multiply(&view).multiply(&model);

    // Reflected model: mirror across the floor (y=0) by scaling Y by -1.
    let mut model_reflect: [[f32; 4]; 4] =
      lambda::math::matrix::identity_matrix(4, 4);
    // Mirror across the floor plane by scaling Y by -1.
    let s_mirror: [[f32; 4]; 4] = [
      [1.0, 0.0, 0.0, 0.0],
      [0.0, -1.0, 0.0, 0.0],
      [0.0, 0.0, 1.0, 0.0],
      [0.0, 0.0, 0.0, 1.0],
    ];
    model_reflect = model_reflect.multiply(&s_mirror);
    // Apply the same rotation and translation as the normal cube but mirrored.
    model_reflect = lambda::math::matrix::rotate_matrix(
      model_reflect,
      [0.0, 1.0, 0.0],
      angle_y_turns,
    );
    let t_down: [[f32; 4]; 4] =
      lambda::math::matrix::translation_matrix([0.0, -0.5, 0.0]);
    model_reflect = model_reflect.multiply(&t_down);
    let mvp_reflect = projection.multiply(&view).multiply(&model_reflect);

    // Floor model: at y = 0 plane
    let mut model_floor: [[f32; 4]; 4] =
      lambda::math::matrix::identity_matrix(4, 4);
    let mvp_floor = projection.multiply(&view).multiply(&model_floor);

    let viewport = ViewportBuilder::new().build(self.width, self.height);

    let pass_id_mask = self.pass_id_mask.expect("mask pass not set");
    let pass_id_color = self.pass_id_color.expect("color pass not set");
    let pipe_floor_mask = self.pipe_floor_mask.expect("floor mask pipeline");
    let pipe_reflected = self.pipe_reflected.expect("reflected pipeline");
    let pipe_floor_visual =
      self.pipe_floor_visual.expect("floor visual pipeline");
    let pipe_normal = self.pipe_normal.expect("normal pipeline");

    return vec![
      // Pass 1: depth/stencil-only to write the floor mask.
      RenderCommand::BeginRenderPass {
        render_pass: pass_id_mask,
        viewport: viewport.clone(),
      },
      // Phase 1: write stencil where the floor geometry exists (stencil = 1).
      RenderCommand::SetPipeline {
        pipeline: pipe_floor_mask,
      },
      RenderCommand::SetViewports {
        start_at: 0,
        viewports: vec![viewport.clone()],
      },
      RenderCommand::SetScissors {
        start_at: 0,
        viewports: vec![viewport.clone()],
      },
      RenderCommand::SetStencilReference { reference: 1 },
      RenderCommand::BindVertexBuffer {
        pipeline: pipe_floor_mask,
        buffer: 0,
      },
      RenderCommand::PushConstants {
        pipeline: pipe_floor_mask,
        stage: PipelineStage::VERTEX,
        offset: 0,
        bytes: Vec::from(push_constants_to_words(&PushConstant {
          mvp: mvp_floor.transpose(),
          model: model_floor.transpose(),
        })),
      },
      RenderCommand::Draw {
        vertices: 0..self.floor_mesh.as_ref().unwrap().vertices().len() as u32,
      },
      RenderCommand::EndRenderPass,
      // Pass 2: color + depth, stencil loaded from mask.
      RenderCommand::BeginRenderPass {
        render_pass: pass_id_color,
        viewport: viewport.clone(),
      },
      // Phase 2: draw the reflected cube where stencil == 1.
      RenderCommand::SetPipeline {
        pipeline: pipe_reflected,
      },
      RenderCommand::SetStencilReference { reference: 1 },
      RenderCommand::BindVertexBuffer {
        pipeline: pipe_reflected,
        buffer: 0,
      },
      RenderCommand::PushConstants {
        pipeline: pipe_reflected,
        stage: PipelineStage::VERTEX,
        offset: 0,
        bytes: Vec::from(push_constants_to_words(&PushConstant {
          mvp: mvp_reflect.transpose(),
          model: model_reflect.transpose(),
        })),
      },
      RenderCommand::Draw {
        vertices: 0..self.cube_mesh.as_ref().unwrap().vertices().len() as u32,
      },
      RenderCommand::SetPipeline {
        pipeline: pipe_floor_visual,
      },
      RenderCommand::BindVertexBuffer {
        pipeline: pipe_floor_visual,
        buffer: 0,
      },
      RenderCommand::PushConstants {
        pipeline: pipe_floor_visual,
        stage: PipelineStage::VERTEX,
        offset: 0,
        bytes: Vec::from(push_constants_to_words(&PushConstant {
          mvp: mvp_floor.transpose(),
          model: model_floor.transpose(),
        })),
      },
      RenderCommand::Draw {
        vertices: 0..self.floor_mesh.as_ref().unwrap().vertices().len() as u32,
      },
      // Phase 4: draw the normal cube above the floor.
      RenderCommand::SetPipeline {
        pipeline: pipe_normal,
      },
      RenderCommand::BindVertexBuffer {
        pipeline: pipe_normal,
        buffer: 0,
      },
      RenderCommand::PushConstants {
        pipeline: pipe_normal,
        stage: PipelineStage::VERTEX,
        offset: 0,
        bytes: Vec::from(push_constants_to_words(&PushConstant {
          mvp: mvp.transpose(),
          model: model.transpose(),
        })),
      },
      RenderCommand::Draw {
        vertices: 0..self.cube_mesh.as_ref().unwrap().vertices().len() as u32,
      },
      RenderCommand::EndRenderPass,
    ];
  }
}

impl Default for ReflectiveRoomExample {
  fn default() -> Self {
    let mut shader_builder = ShaderBuilder::new();
    let shader_vs = shader_builder.build(VirtualShader::Source {
      source: VERTEX_SHADER_SOURCE.to_string(),
      kind: ShaderKind::Vertex,
      entry_point: "main".to_string(),
      name: "reflective-room-vs".to_string(),
    });
    let shader_fs_lit = shader_builder.build(VirtualShader::Source {
      source: FRAGMENT_LIT_COLOR_SOURCE.to_string(),
      kind: ShaderKind::Fragment,
      entry_point: "main".to_string(),
      name: "reflective-room-fs-lit".to_string(),
    });
    let shader_fs_floor = shader_builder.build(VirtualShader::Source {
      source: FRAGMENT_FLOOR_TINT_SOURCE.to_string(),
      kind: ShaderKind::Fragment,
      entry_point: "main".to_string(),
      name: "reflective-room-fs-floor".to_string(),
    });

    return Self {
      shader_vs,
      shader_fs_lit,
      shader_fs_floor,
      cube_mesh: None,
      floor_mesh: None,
      pass_id_mask: None,
      pass_id_color: None,
      pipe_floor_mask: None,
      pipe_reflected: None,
      pipe_floor_visual: None,
      pipe_normal: None,
      width: 800,
      height: 600,
      elapsed: 0.0,
    };
  }
}

fn build_unit_cube_mesh() -> Mesh {
  let mut verts: Vec<Vertex> = Vec::new();
  let mut add_face =
    |nx: f32, ny: f32, nz: f32, corners: [(f32, f32, f32); 4]| {
      let n = [nx, ny, nz];
      let v = |p: (f32, f32, f32)| {
        return VertexBuilder::new()
          .with_position([p.0, p.1, p.2])
          .with_normal(n)
          .with_color([0.0, 0.0, 0.0])
          .build();
      };
      // Two triangles per face: (0,1,2) and (0,2,3)
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
  add_face(
    1.0,
    0.0,
    0.0,
    [(h, -h, -h), (h, h, -h), (h, h, h), (h, -h, h)],
  );
  add_face(
    -1.0,
    0.0,
    0.0,
    [(-h, -h, -h), (-h, -h, h), (-h, h, h), (-h, h, -h)],
  );
  add_face(
    0.0,
    1.0,
    0.0,
    [(-h, h, h), (h, h, h), (h, h, -h), (-h, h, -h)],
  );
  add_face(
    0.0,
    -1.0,
    0.0,
    [(-h, -h, -h), (h, -h, -h), (h, -h, h), (-h, -h, h)],
  );
  add_face(
    0.0,
    0.0,
    1.0,
    [(-h, -h, h), (h, -h, h), (h, h, h), (-h, h, h)],
  );
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
    ])
    .build();
  return mesh;
}

fn build_floor_quad_mesh(extent: f32) -> Mesh {
  // Large quad on XZ plane at Y=0
  let y = 0.0f32;
  let h = extent * 0.5;
  let normal = [0.0, 1.0, 0.0];
  let v = |x: f32, z: f32| {
    return VertexBuilder::new()
      .with_position([x, y, z])
      .with_normal(normal)
      .with_color([0.0, 0.0, 0.0])
      .build();
  };
  let p0 = v(-h, -h);
  let p1 = v(h, -h);
  let p2 = v(h, h);
  let p3 = v(-h, h);

  let mut mesh_builder = MeshBuilder::new();
  // Tri 1
  mesh_builder.with_vertex(p0);
  mesh_builder.with_vertex(p1);
  mesh_builder.with_vertex(p2);
  // Tri 2
  mesh_builder.with_vertex(p0);
  mesh_builder.with_vertex(p2);
  mesh_builder.with_vertex(p3);

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
    ])
    .build();
  return mesh;
}

fn main() {
  let runtime: ApplicationRuntime =
    ApplicationRuntimeBuilder::new("Reflective Room Example")
      .with_window_configured_as(|builder| {
        builder
          .with_dimensions(960, 600)
          .with_name("Reflective Room")
      })
      .with_component(|runtime, example: ReflectiveRoomExample| {
        (runtime, example)
      })
      .build();

  start_runtime(runtime);
}
