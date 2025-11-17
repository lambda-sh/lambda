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
    buffer::{
      BufferBuilder,
      BufferType,
      Properties,
      Usage,
    },
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
    texture::DepthFormat,
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
  // Toggleable demo settings
  msaa_samples: u32,
  stencil_enabled: bool,
  depth_test_enabled: bool,
  needs_rebuild: bool,
}

impl Component<ComponentResult, String> for ReflectiveRoomExample {
  fn on_attach(
    &mut self,
    render_context: &mut lambda::render::RenderContext,
  ) -> Result<ComponentResult, String> {
    logging::info!("Attaching ReflectiveRoomExample");

    // Build resources according to current toggles via the shared path.
    match self.rebuild_resources(render_context) {
      Ok(()) => {
        return Ok(ComponentResult::Success);
      }
      Err(err) => {
        logging::error!("Initial resource build failed: {}", err);
        return Err(err);
      }
    }
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
      lambda::events::Events::Keyboard { event, .. } => match event {
        lambda::events::Key::Pressed {
          scan_code: _,
          virtual_key,
        } => match virtual_key {
          Some(lambda::events::VirtualKey::KeyM) => {
            self.msaa_samples = if self.msaa_samples > 1 { 1 } else { 4 };
            self.needs_rebuild = true;
            logging::info!("Toggled MSAA → {}x (key: M)", self.msaa_samples);
          }
          Some(lambda::events::VirtualKey::KeyS) => {
            self.stencil_enabled = !self.stencil_enabled;
            self.needs_rebuild = true;
            logging::info!(
              "Toggled Stencil → {} (key: S)",
              self.stencil_enabled
            );
          }
          Some(lambda::events::VirtualKey::KeyD) => {
            self.depth_test_enabled = !self.depth_test_enabled;
            self.needs_rebuild = true;
            logging::info!(
              "Toggled Depth Test → {} (key: D)",
              self.depth_test_enabled
            );
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
    self.elapsed += last_frame.as_secs_f32();
    return Ok(ComponentResult::Success);
  }

  fn on_render(
    &mut self,
    render_context: &mut lambda::render::RenderContext,
  ) -> Vec<RenderCommand> {
    if self.needs_rebuild {
      // Attempt to rebuild resources according to current toggles
      if let Err(err) = self.rebuild_resources(render_context) {
        logging::error!("Failed to rebuild resources: {}", err);
      }
    }
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

    // Compute reflected transform only if stencil/reflection is enabled.
    let (model_reflect, mvp_reflect) = if self.stencil_enabled {
      let mut mr: [[f32; 4]; 4] = lambda::math::matrix::identity_matrix(4, 4);
      let s_mirror: [[f32; 4]; 4] = [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, -1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
      ];
      mr = mr.multiply(&s_mirror);
      mr =
        lambda::math::matrix::rotate_matrix(mr, [0.0, 1.0, 0.0], angle_y_turns);
      let t_down: [[f32; 4]; 4] =
        lambda::math::matrix::translation_matrix([0.0, -0.5, 0.0]);
      mr = mr.multiply(&t_down);
      let mvp_r = projection.multiply(&view).multiply(&mr);
      (mr, mvp_r)
    } else {
      // Unused in subsequent commands when stencil is disabled.
      (lambda::math::matrix::identity_matrix(4, 4), mvp)
    };

    // Floor model: at y = 0 plane
    let mut model_floor: [[f32; 4]; 4] =
      lambda::math::matrix::identity_matrix(4, 4);
    let mvp_floor = projection.multiply(&view).multiply(&model_floor);

    let viewport = ViewportBuilder::new().build(self.width, self.height);

    let mut cmds: Vec<RenderCommand> = Vec::new();

    // Cache vertex counts locally to avoid repeated lookups.
    let cube_vertex_count: u32 = self
      .cube_mesh
      .as_ref()
      .map(|m| m.vertices().len() as u32)
      .unwrap_or(0);
    let floor_vertex_count: u32 = self
      .floor_mesh
      .as_ref()
      .map(|m| m.vertices().len() as u32)
      .unwrap_or(0);

    if self.stencil_enabled {
      // Optional Pass 1: write floor stencil mask
      if let (Some(pass_id_mask), Some(pipe_floor_mask)) =
        (self.pass_id_mask, self.pipe_floor_mask)
      {
        cmds.push(RenderCommand::BeginRenderPass {
          render_pass: pass_id_mask,
          viewport: viewport.clone(),
        });
        cmds.push(RenderCommand::SetPipeline {
          pipeline: pipe_floor_mask,
        });
        cmds.push(RenderCommand::SetViewports {
          start_at: 0,
          viewports: vec![viewport.clone()],
        });
        cmds.push(RenderCommand::SetScissors {
          start_at: 0,
          viewports: vec![viewport.clone()],
        });
        cmds.push(RenderCommand::SetStencilReference { reference: 1 });
        cmds.push(RenderCommand::BindVertexBuffer {
          pipeline: pipe_floor_mask,
          buffer: 0,
        });
        cmds.push(RenderCommand::PushConstants {
          pipeline: pipe_floor_mask,
          stage: PipelineStage::VERTEX,
          offset: 0,
          bytes: Vec::from(push_constants_to_words(&PushConstant {
            mvp: mvp_floor.transpose(),
            model: model_floor.transpose(),
          })),
        });
        cmds.push(RenderCommand::Draw {
          vertices: 0..floor_vertex_count,
        });
        cmds.push(RenderCommand::EndRenderPass);
      }
    }

    // Color pass (with optional depth/stencil configured on the pass itself)
    let pass_id_color = self.pass_id_color.expect("color pass not set");
    cmds.push(RenderCommand::BeginRenderPass {
      render_pass: pass_id_color,
      viewport: viewport.clone(),
    });

    if self.stencil_enabled {
      if let Some(pipe_reflected) = self.pipe_reflected {
        cmds.push(RenderCommand::SetPipeline {
          pipeline: pipe_reflected,
        });
        cmds.push(RenderCommand::SetStencilReference { reference: 1 });
        cmds.push(RenderCommand::BindVertexBuffer {
          pipeline: pipe_reflected,
          buffer: 0,
        });
        cmds.push(RenderCommand::PushConstants {
          pipeline: pipe_reflected,
          stage: PipelineStage::VERTEX,
          offset: 0,
          bytes: Vec::from(push_constants_to_words(&PushConstant {
            mvp: mvp_reflect.transpose(),
            model: model_reflect.transpose(),
          })),
        });
        cmds.push(RenderCommand::Draw {
          vertices: 0..cube_vertex_count,
        });
      }
    }

    // Floor surface (tinted)
    let pipe_floor_visual =
      self.pipe_floor_visual.expect("floor visual pipeline");
    cmds.push(RenderCommand::SetPipeline {
      pipeline: pipe_floor_visual,
    });
    cmds.push(RenderCommand::BindVertexBuffer {
      pipeline: pipe_floor_visual,
      buffer: 0,
    });
    cmds.push(RenderCommand::PushConstants {
      pipeline: pipe_floor_visual,
      stage: PipelineStage::VERTEX,
      offset: 0,
      bytes: Vec::from(push_constants_to_words(&PushConstant {
        mvp: mvp_floor.transpose(),
        model: model_floor.transpose(),
      })),
    });
    cmds.push(RenderCommand::Draw {
      vertices: 0..floor_vertex_count,
    });

    // Normal cube
    let pipe_normal = self.pipe_normal.expect("normal pipeline");
    cmds.push(RenderCommand::SetPipeline {
      pipeline: pipe_normal,
    });
    cmds.push(RenderCommand::BindVertexBuffer {
      pipeline: pipe_normal,
      buffer: 0,
    });
    cmds.push(RenderCommand::PushConstants {
      pipeline: pipe_normal,
      stage: PipelineStage::VERTEX,
      offset: 0,
      bytes: Vec::from(push_constants_to_words(&PushConstant {
        mvp: mvp.transpose(),
        model: model.transpose(),
      })),
    });
    cmds.push(RenderCommand::Draw {
      vertices: 0..cube_vertex_count,
    });
    cmds.push(RenderCommand::EndRenderPass);

    return cmds;
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
      msaa_samples: 4,
      stencil_enabled: true,
      depth_test_enabled: true,
      needs_rebuild: false,
    };
  }
}

impl ReflectiveRoomExample {
  fn rebuild_resources(
    &mut self,
    render_context: &mut lambda::render::RenderContext,
  ) -> Result<(), String> {
    self.needs_rebuild = false;

    // Ensure meshes exist (reuse existing buffers on context attach).
    if self.cube_mesh.is_none() {
      self.cube_mesh = Some(build_unit_cube_mesh());
    }
    if self.floor_mesh.is_none() {
      self.floor_mesh = Some(build_floor_quad_mesh(5.0));
    }
    let cube_mesh = self.cube_mesh.as_ref().unwrap();
    let floor_mesh = self.floor_mesh.as_ref().unwrap();
    let push_constants_size = std::mem::size_of::<PushConstant>() as u32;

    // Build pass descriptions locally first
    let rp_mask_desc = if self.stencil_enabled {
      Some(
        RenderPassBuilder::new()
          .with_label("reflective-room-pass-mask")
          .with_depth_clear(1.0)
          .with_stencil_clear(0)
          .with_multi_sample(self.msaa_samples)
          .without_color()
          .build(render_context),
      )
    } else {
      None
    };

    let mut rp_color_builder = RenderPassBuilder::new()
      .with_label("reflective-room-pass-color")
      .with_multi_sample(self.msaa_samples);
    if self.depth_test_enabled {
      rp_color_builder = rp_color_builder.with_depth_clear(1.0);
    } else if self.stencil_enabled {
      // Ensure a depth-stencil attachment exists even if we are not depth-testing,
      // because pipelines with stencil state expect a depth/stencil attachment.
      rp_color_builder = rp_color_builder.with_depth_load();
    }
    if self.stencil_enabled {
      rp_color_builder = rp_color_builder.with_stencil_load();
    }
    let rp_color_desc = rp_color_builder.build(render_context);

    // Floor mask pipeline (stencil write)
    self.pipe_floor_mask = if self.stencil_enabled {
      let p = RenderPipelineBuilder::new()
        .with_label("floor-mask")
        .with_culling(CullingMode::Back)
        .with_depth_format(DepthFormat::Depth24PlusStencil8)
        .with_depth_write(false)
        .with_depth_compare(CompareFunction::Always)
        .with_push_constant(PipelineStage::VERTEX, push_constants_size)
        .with_buffer(
          BufferBuilder::new()
            .with_length(
              floor_mesh.vertices().len() * std::mem::size_of::<Vertex>(),
            )
            .with_usage(Usage::VERTEX)
            .with_properties(Properties::DEVICE_LOCAL)
            .with_buffer_type(BufferType::Vertex)
            .build(render_context, floor_mesh.vertices().to_vec())
            .map_err(|e| format!("Failed to create floor buffer: {}", e))?,
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
        .with_multi_sample(self.msaa_samples)
        .build(
          render_context,
          rp_mask_desc
            .as_ref()
            .expect("mask pass missing for stencil"),
          &self.shader_vs,
          None,
        );
      Some(render_context.attach_pipeline(p))
    } else {
      None
    };

    // Reflected cube pipeline
    self.pipe_reflected = if self.stencil_enabled {
      let mut builder = RenderPipelineBuilder::new()
        .with_label("reflected-cube")
        .with_culling(CullingMode::None)
        .with_depth_format(DepthFormat::Depth24PlusStencil8)
        .with_push_constant(PipelineStage::VERTEX, push_constants_size)
        .with_buffer(
          BufferBuilder::new()
            .with_length(
              cube_mesh.vertices().len() * std::mem::size_of::<Vertex>(),
            )
            .with_usage(Usage::VERTEX)
            .with_properties(Properties::DEVICE_LOCAL)
            .with_buffer_type(BufferType::Vertex)
            .build(render_context, cube_mesh.vertices().to_vec())
            .map_err(|e| format!("Failed to create cube buffer: {}", e))?,
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
        .with_multi_sample(self.msaa_samples);
      if self.depth_test_enabled {
        builder = builder
          .with_depth_write(true)
          .with_depth_compare(CompareFunction::LessEqual);
      } else {
        builder = builder
          .with_depth_write(false)
          .with_depth_compare(CompareFunction::Always);
      }
      let p = builder.build(
        render_context,
        &rp_color_desc,
        &self.shader_vs,
        Some(&self.shader_fs_lit),
      );
      Some(render_context.attach_pipeline(p))
    } else {
      None
    };

    // Floor visual pipeline
    let mut floor_builder = RenderPipelineBuilder::new()
      .with_label("floor-visual")
      .with_culling(CullingMode::Back)
      .with_push_constant(PipelineStage::VERTEX, push_constants_size)
      .with_buffer(
        BufferBuilder::new()
          .with_length(
            floor_mesh.vertices().len() * std::mem::size_of::<Vertex>(),
          )
          .with_usage(Usage::VERTEX)
          .with_properties(Properties::DEVICE_LOCAL)
          .with_buffer_type(BufferType::Vertex)
          .build(render_context, floor_mesh.vertices().to_vec())
          .map_err(|e| format!("Failed to create floor buffer: {}", e))?,
        floor_mesh.attributes().to_vec(),
      )
      .with_multi_sample(self.msaa_samples);
    if self.depth_test_enabled || self.stencil_enabled {
      floor_builder = floor_builder
        .with_depth_format(DepthFormat::Depth24PlusStencil8)
        .with_depth_write(false)
        .with_depth_compare(if self.depth_test_enabled {
          CompareFunction::LessEqual
        } else {
          CompareFunction::Always
        });
    }
    let floor_pipe = floor_builder.build(
      render_context,
      &rp_color_desc,
      &self.shader_vs,
      Some(&self.shader_fs_floor),
    );
    self.pipe_floor_visual = Some(render_context.attach_pipeline(floor_pipe));

    // Normal cube pipeline
    let mut normal_builder = RenderPipelineBuilder::new()
      .with_label("cube-normal")
      .with_culling(CullingMode::Back)
      .with_push_constant(PipelineStage::VERTEX, push_constants_size)
      .with_buffer(
        BufferBuilder::new()
          .with_length(
            cube_mesh.vertices().len() * std::mem::size_of::<Vertex>(),
          )
          .with_usage(Usage::VERTEX)
          .with_properties(Properties::DEVICE_LOCAL)
          .with_buffer_type(BufferType::Vertex)
          .build(render_context, cube_mesh.vertices().to_vec())
          .map_err(|e| format!("Failed to create cube buffer: {}", e))?,
        cube_mesh.attributes().to_vec(),
      )
      .with_multi_sample(self.msaa_samples);
    if self.depth_test_enabled || self.stencil_enabled {
      normal_builder = normal_builder
        .with_depth_format(DepthFormat::Depth24PlusStencil8)
        .with_depth_write(self.depth_test_enabled)
        .with_depth_compare(if self.depth_test_enabled {
          CompareFunction::Less
        } else {
          CompareFunction::Always
        });
    }
    let normal_pipe = normal_builder.build(
      render_context,
      &rp_color_desc,
      &self.shader_vs,
      Some(&self.shader_fs_lit),
    );
    self.pipe_normal = Some(render_context.attach_pipeline(normal_pipe));

    // Finally attach the passes and record their handles
    self.pass_id_mask =
      rp_mask_desc.map(|rp| render_context.attach_render_pass(rp));
    self.pass_id_color = Some(render_context.attach_render_pass(rp_color_desc));

    logging::info!(
      "Rebuilt — MSAA: {}x, Stencil: {}, Depth Test: {}",
      self.msaa_samples,
      self.stencil_enabled,
      self.depth_test_enabled
    );
    return Ok(());
  }
}

fn build_unit_cube_mesh() -> Mesh {
  // 6 faces * 2 triangles * 3 vertices = 36
  let mut verts: Vec<Vertex> = Vec::with_capacity(36);
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
