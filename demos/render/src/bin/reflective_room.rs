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
  events::{
    EventMask,
    Key,
    VirtualKey,
    WindowEvent,
  },
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
      BlendMode,
      CompareFunction,
      CullingMode,
      RenderPipelineBuilder,
      StencilFaceState,
      StencilOperation,
      StencilState,
    },
    render_pass::RenderPassBuilder,
    scene_math::{
      compute_model_matrix,
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
  // Transform normals into world space using the model matrix.
  // Note: This demo uses only rigid transforms and a Y-mirror; `mat3(model)`
  // remains adequate and avoids unsupported `inverse` on some backends (MSL).
  v_world_normal = normalize(mat3(pc.model) * vertex_normal);
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

layout (location = 0) in vec3 v_world_normal;
layout (location = 0) out vec4 fragment_color;

void main() {
  // Lit floor with partial transparency so the reflection shows through.
  vec3 N = normalize(v_world_normal);
  vec3 L = normalize(vec3(0.4, 0.7, 1.0));
  float diff = max(dot(N, L), 0.0);
  // Subtle base tint to suggest a surface, keep alpha low so reflection reads.
  vec3 base = vec3(0.10, 0.10, 0.11);
  vec3 color = base * (0.35 + 0.65 * diff);
  fragment_color = vec4(color, 0.15);
}
"#;

// (No extra fragment shaders needed; the floor mask uses a vertex-only pipeline.)

// -------------------------------- IMMEDIATES ---------------------------------

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ImmediateData {
  mvp: [[f32; 4]; 4],
  model: [[f32; 4]; 4],
}

pub fn immediate_data_to_words(immediate_data: &ImmediateData) -> &[u32] {
  unsafe {
    let size_in_bytes = std::mem::size_of::<ImmediateData>();
    let size_in_u32 = size_in_bytes / std::mem::size_of::<u32>();
    let ptr = immediate_data as *const ImmediateData as *const u32;
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
  // Visual tuning
  floor_tilt_turns: f32,
  camera_distance: f32,
  camera_height: f32,
  camera_pitch_turns: f32,
  // When true, do not draw the floor surface; leaves a clean mirror.
  mirror_mode: bool,
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

  fn event_mask(&self) -> EventMask {
    return EventMask::WINDOW | EventMask::KEYBOARD;
  }

  fn on_window_event(&mut self, event: &WindowEvent) -> Result<(), String> {
    if let WindowEvent::Resize { width, height } = event {
      self.width = *width;
      self.height = *height;
    }
    return Ok(());
  }

  fn on_keyboard_event(&mut self, event: &Key) -> Result<(), String> {
    if let Key::Pressed {
      scan_code: _,
      virtual_key,
    } = event
    {
      match virtual_key {
        Some(VirtualKey::KeyM) => {
          self.msaa_samples = if self.msaa_samples > 1 { 1 } else { 4 };
          self.needs_rebuild = true;
          logging::info!("Toggled MSAA → {}x (key: M)", self.msaa_samples);
        }
        Some(VirtualKey::KeyS) => {
          self.stencil_enabled = !self.stencil_enabled;
          self.needs_rebuild = true;
          logging::info!("Toggled Stencil → {} (key: S)", self.stencil_enabled);
        }
        Some(VirtualKey::KeyD) => {
          self.depth_test_enabled = !self.depth_test_enabled;
          self.needs_rebuild = true;
          logging::info!(
            "Toggled Depth Test → {} (key: D)",
            self.depth_test_enabled
          );
        }
        Some(VirtualKey::KeyF) => {
          self.mirror_mode = !self.mirror_mode;
          logging::info!(
            "Toggled Mirror Mode (hide floor overlay) → {} (key: F)",
            self.mirror_mode
          );
        }
        Some(VirtualKey::KeyI) => {
          self.camera_pitch_turns =
            (self.camera_pitch_turns - 0.01).clamp(0.0, 0.25);
          logging::info!(
            "Camera pitch (turns) → {:.3}",
            self.camera_pitch_turns
          );
        }
        Some(VirtualKey::KeyK) => {
          self.camera_pitch_turns =
            (self.camera_pitch_turns + 0.01).clamp(0.0, 0.25);
          logging::info!(
            "Camera pitch (turns) → {:.3}",
            self.camera_pitch_turns
          );
        }
        _ => {}
      }
    }
    return Ok(());
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
      position: [0.0, self.camera_height, self.camera_distance],
      field_of_view_in_turns: 0.24,
      near_clipping_plane: 0.1,
      far_clipping_plane: 100.0,
    };

    // Cube animation
    let angle_y_turns = 0.12 * self.elapsed;
    // Build model with canonical order using the scene helpers:
    // world = T(0, +0.5, 0) * R_y(angle) * S(1)
    let model: [[f32; 4]; 4] = compute_model_matrix(
      [0.0, 0.5, 0.0],
      [0.0, 1.0, 0.0],
      angle_y_turns,
      1.0,
    );

    // View: pitch downward, then translate by camera position (R * T)
    let rot_x: [[f32; 4]; 4] = lambda::math::matrix::rotate_matrix(
      lambda::math::matrix::identity_matrix(4, 4),
      [1.0, 0.0, 0.0],
      -self.camera_pitch_turns,
    )
    .expect("rotation axis must be a unit axis vector");
    let view = rot_x.multiply(&compute_view_matrix(camera.position));
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
      // Reflection across the (possibly tilted) floor plane that passes
      // through the origin. Build the plane normal by rotating +Y by the
      // configured floor tilt around X.
      let angle = self.floor_tilt_turns * std::f32::consts::PI * 2.0;
      let nx = 0.0f32;
      let ny = angle.cos();
      let nz = -angle.sin();
      // Reflection matrix R = I - 2*n*n^T for a plane through the origin.
      let (nx2, ny2, nz2) = (nx * nx, ny * ny, nz * nz);
      let s_mirror: [[f32; 4]; 4] = [
        [1.0 - 2.0 * nx2, -2.0 * nx * ny, -2.0 * nx * nz, 0.0],
        [-2.0 * ny * nx, 1.0 - 2.0 * ny2, -2.0 * ny * nz, 0.0],
        [-2.0 * nz * nx, -2.0 * nz * ny, 1.0 - 2.0 * nz2, 0.0],
        [0.0, 0.0, 0.0, 1.0],
      ];
      let mr = s_mirror.multiply(&model);
      let mvp_r = projection.multiply(&view).multiply(&mr);
      (mr, mvp_r)
    } else {
      // Unused in subsequent commands when stencil is disabled.
      (lambda::math::matrix::identity_matrix(4, 4), mvp)
    };

    // Floor model: plane through origin, tilted slightly around X for clarity
    let mut model_floor: [[f32; 4]; 4] =
      lambda::math::matrix::identity_matrix(4, 4);
    model_floor = lambda::math::matrix::rotate_matrix(
      model_floor,
      [1.0, 0.0, 0.0],
      self.floor_tilt_turns,
    )
    .expect("rotation axis must be a unit axis vector");
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
        cmds.push(RenderCommand::Immediates {
          pipeline: pipe_floor_mask,
          offset: 0,
          bytes: Vec::from(immediate_data_to_words(&ImmediateData {
            mvp: mvp_floor.transpose(),
            model: model_floor.transpose(),
          })),
        });
        cmds.push(RenderCommand::Draw {
          vertices: 0..floor_vertex_count,
          instances: 0..1,
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
        cmds.push(RenderCommand::Immediates {
          pipeline: pipe_reflected,
          offset: 0,
          bytes: Vec::from(immediate_data_to_words(&ImmediateData {
            mvp: mvp_reflect.transpose(),
            model: model_reflect.transpose(),
          })),
        });
        cmds.push(RenderCommand::Draw {
          vertices: 0..cube_vertex_count,
          instances: 0..1,
        });
      }
    }

    // Floor surface (tinted)
    if !self.mirror_mode {
      let pipe_floor_visual =
        self.pipe_floor_visual.expect("floor visual pipeline");
      cmds.push(RenderCommand::SetPipeline {
        pipeline: pipe_floor_visual,
      });
      cmds.push(RenderCommand::BindVertexBuffer {
        pipeline: pipe_floor_visual,
        buffer: 0,
      });
      cmds.push(RenderCommand::Immediates {
        pipeline: pipe_floor_visual,
        offset: 0,
        bytes: Vec::from(immediate_data_to_words(&ImmediateData {
          mvp: mvp_floor.transpose(),
          model: model_floor.transpose(),
        })),
      });
      cmds.push(RenderCommand::Draw {
        vertices: 0..floor_vertex_count,
        instances: 0..1,
      });
    }

    // Normal cube
    let pipe_normal = self.pipe_normal.expect("normal pipeline");
    cmds.push(RenderCommand::SetPipeline {
      pipeline: pipe_normal,
    });
    cmds.push(RenderCommand::BindVertexBuffer {
      pipeline: pipe_normal,
      buffer: 0,
    });
    cmds.push(RenderCommand::Immediates {
      pipeline: pipe_normal,
      offset: 0,
      bytes: Vec::from(immediate_data_to_words(&ImmediateData {
        mvp: mvp.transpose(),
        model: model.transpose(),
      })),
    });
    cmds.push(RenderCommand::Draw {
      vertices: 0..cube_vertex_count,
      instances: 0..1,
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
      floor_tilt_turns: 0.0, // Keep plane flat; angle comes from camera
      camera_distance: 4.0,
      camera_height: 3.0,
      camera_pitch_turns: 0.10, // ~36 degrees downward
      mirror_mode: false,
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
    let immediate_data_size = std::mem::size_of::<ImmediateData>() as u32;

    // Build pass descriptions locally first
    let rp_mask_desc = if self.stencil_enabled {
      Some(
        RenderPassBuilder::new()
          .with_label("reflective-room-pass-mask")
          .with_depth_clear(1.0)
          .with_stencil_clear(0)
          .with_multi_sample(self.msaa_samples)
          .without_color()
          .build(
            render_context.gpu(),
            render_context.surface_format(),
            render_context.depth_format(),
          ),
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
    let rp_color_desc = rp_color_builder.build(
      render_context.gpu(),
      render_context.surface_format(),
      render_context.depth_format(),
    );

    // Floor mask pipeline (stencil write)
    self.pipe_floor_mask = if self.stencil_enabled {
      let p = RenderPipelineBuilder::new()
        .with_label("floor-mask")
        // Disable culling to guarantee stencil writes regardless of winding.
        .with_culling(CullingMode::None)
        .with_depth_format(DepthFormat::Depth24PlusStencil8)
        .with_depth_write(false)
        .with_depth_compare(CompareFunction::Always)
        .with_immediate_data(immediate_data_size)
        .with_buffer(
          BufferBuilder::new()
            .with_length(std::mem::size_of_val(floor_mesh.vertices()))
            .with_usage(Usage::VERTEX)
            .with_properties(Properties::DEVICE_LOCAL)
            .with_buffer_type(BufferType::Vertex)
            .build(render_context.gpu(), floor_mesh.vertices().to_vec())
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
          render_context.gpu(),
          render_context.surface_format(),
          render_context.depth_format(),
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
      let builder = RenderPipelineBuilder::new()
        .with_label("reflected-cube")
        // Mirrored transform reverses winding; cull front to keep visible faces.
        .with_culling(CullingMode::Front)
        .with_depth_format(DepthFormat::Depth24PlusStencil8)
        .with_immediate_data(immediate_data_size)
        .with_buffer(
          BufferBuilder::new()
            .with_length(std::mem::size_of_val(cube_mesh.vertices()))
            .with_usage(Usage::VERTEX)
            .with_properties(Properties::DEVICE_LOCAL)
            .with_buffer_type(BufferType::Vertex)
            .build(render_context.gpu(), cube_mesh.vertices().to_vec())
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
        .with_multi_sample(self.msaa_samples)
        // Render reflection regardless of depth to ensure visibility;
        // the floor overlay and stencil confine and visually place it.
        .with_depth_write(false)
        .with_depth_compare(CompareFunction::Always);
      let p = builder.build(
        render_context.gpu(),
        render_context.surface_format(),
        render_context.depth_format(),
        &rp_color_desc,
        &self.shader_vs,
        Some(&self.shader_fs_lit),
      );
      Some(render_context.attach_pipeline(p))
    } else {
      None
    };

    // No unmasked reflection pipeline in production example.

    // Floor visual pipeline
    let mut floor_builder = RenderPipelineBuilder::new()
      .with_label("floor-visual")
      .with_culling(CullingMode::Back)
      .with_blend(BlendMode::AlphaBlending)
      .with_immediate_data(immediate_data_size)
      .with_buffer(
        BufferBuilder::new()
          .with_length(std::mem::size_of_val(floor_mesh.vertices()))
          .with_usage(Usage::VERTEX)
          .with_properties(Properties::DEVICE_LOCAL)
          .with_buffer_type(BufferType::Vertex)
          .build(render_context.gpu(), floor_mesh.vertices().to_vec())
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
      render_context.gpu(),
      render_context.surface_format(),
      render_context.depth_format(),
      &rp_color_desc,
      &self.shader_vs,
      Some(&self.shader_fs_floor),
    );
    self.pipe_floor_visual = Some(render_context.attach_pipeline(floor_pipe));

    // Normal cube pipeline
    let mut normal_builder = RenderPipelineBuilder::new()
      .with_label("cube-normal")
      .with_culling(CullingMode::Back)
      .with_immediate_data(immediate_data_size)
      .with_buffer(
        BufferBuilder::new()
          .with_length(std::mem::size_of_val(cube_mesh.vertices()))
          .with_usage(Usage::VERTEX)
          .with_properties(Properties::DEVICE_LOCAL)
          .with_buffer_type(BufferType::Vertex)
          .build(render_context.gpu(), cube_mesh.vertices().to_vec())
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
      render_context.gpu(),
      render_context.surface_format(),
      render_context.depth_format(),
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
  // Tri winding flipped to face +Y (avoid back-face cull)
  // Tri 1
  mesh_builder.with_vertex(p0);
  mesh_builder.with_vertex(p2);
  mesh_builder.with_vertex(p1);
  // Tri 2
  mesh_builder.with_vertex(p0);
  mesh_builder.with_vertex(p3);
  mesh_builder.with_vertex(p2);

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
