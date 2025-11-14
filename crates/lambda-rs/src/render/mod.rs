//! High‑level rendering API for cross‑platform windowed applications.
//!
//! The rendering module provides a small set of stable, engine‑facing types
//! that assemble a frame using explicit commands. It hides lower‑level
//! platform details (the `wgpu` device, queue, surfaces, and raw descriptors)
//! behind builders and handles while keeping configuration visible and
//! predictable.
//!
//! Concepts
//! - `RenderContext`: owns the graphics instance, presentation surface, and
//!   GPU device/queue for a single window. It is the submit point for per‑frame
//!   command encoding.
//! - `RenderPass` and `RenderPipeline`: immutable descriptions used when
//!   beginning a pass and binding a pipeline. Pipelines declare their vertex
//!   inputs, push constants, and layout (bind group layouts).
//! - `Buffer`, `BindGroupLayout`, and `BindGroup`: GPU resources created via
//!   builders and attached to the context, then referenced by small integer
//!   handles when encoding commands.
//! - `RenderCommand`: an explicit, validated sequence that begins with
//!   `BeginRenderPass`, binds state, draws, and ends with `EndRenderPass`.
//!
//! Minimal flow
//! 1) Create a window and a `RenderContext` with `RenderContextBuilder`.
//! 2) Build resources (buffers, bind group layouts, shaders, pipelines).
//! 3) Record a `Vec<RenderCommand>` each frame and pass it to
//!    `RenderContext::render`.
//!
//! See workspace examples under `crates/lambda-rs/examples/` for runnable
//! end‑to‑end snippets.

// Module Exports
pub mod bind;
pub mod buffer;
pub mod command;
pub mod mesh;
pub mod pipeline;
pub mod render_pass;
pub mod scene_math;
pub mod shader;
pub mod texture;
pub mod validation;
pub mod vertex;
pub mod viewport;
pub mod window;

use std::{
  collections::HashSet,
  iter,
  rc::Rc,
};

use lambda_platform::wgpu as platform;
use logging;

use self::{
  command::RenderCommand,
  pipeline::RenderPipeline,
  render_pass::RenderPass as RenderPassDesc,
};
use crate::util;

/// Builder for configuring a `RenderContext` tied to one window.
///
/// Purpose
/// - Construct the graphics `Instance`, presentation `Surface`, and logical
///   `Gpu` using the platform layer.
/// - Configure the surface with sane defaults (sRGB when available,
///   `Fifo`/vsync‑compatible present mode, `RENDER_ATTACHMENT` usage).
///
/// Usage
/// - Create with a human‑readable name used in debug labels.
/// - Optionally adjust timeouts, then `build(window)` to obtain a
///   `RenderContext`.
///
/// Typical use is in an application runtime immediately after creating a
/// window. The returned `RenderContext` owns all GPU objects required to render
/// to that window.
pub struct RenderContextBuilder {
  name: String,
  /// Reserved for future timeout handling during rendering (nanoseconds).
  /// Not currently enforced; kept for forward compatibility with runtime controls.
  _render_timeout: u64,
}

impl RenderContextBuilder {
  /// Create a new builder tagged with a human‑readable `name` used in labels.
  pub fn new(name: &str) -> Self {
    Self {
      name: name.to_string(),
      _render_timeout: 1_000_000_000,
    }
  }

  /// Set how long rendering may take before timing out (nanoseconds).
  pub fn with_render_timeout(mut self, render_timeout: u64) -> Self {
    self._render_timeout = render_timeout;
    self
  }

  /// Build a `RenderContext` for the provided `window` and configure the
  /// presentation surface.
  ///
  /// Errors are returned instead of panicking to allow callers to surface
  /// actionable initialization failures.
  pub fn build(
    self,
    window: &window::Window,
  ) -> Result<RenderContext, RenderContextError> {
    let RenderContextBuilder { name, .. } = self;

    let instance = platform::instance::InstanceBuilder::new()
      .with_label(&format!("{} Instance", name))
      .build();

    let mut surface = platform::surface::SurfaceBuilder::new()
      .with_label(&format!("{} Surface", name))
      .build(&instance, window.window_handle())
      .map_err(|e| {
        RenderContextError::SurfaceCreate(format!(
          "Failed to create rendering surface: {:?}",
          e
        ))
      })?;

    let gpu = platform::gpu::GpuBuilder::new()
      .with_label(&format!("{} Device", name))
      .build(&instance, Some(&surface))
      .map_err(|e| {
        RenderContextError::GpuCreate(format!(
          "Failed to create GPU device: {:?}",
          e
        ))
      })?;

    let size = window.dimensions();
    surface
      .configure_with_defaults(
        &gpu,
        size,
        platform::surface::PresentMode::Fifo,
        platform::surface::TextureUsages::RENDER_ATTACHMENT,
      )
      .map_err(|e| {
        RenderContextError::SurfaceConfig(format!(
          "Failed to configure surface: {:?}",
          e
        ))
      })?;

    let config = surface.configuration().cloned().ok_or_else(|| {
      RenderContextError::SurfaceConfig(
        "Surface was not configured".to_string(),
      )
    })?;
    let present_mode = config.present_mode;
    let texture_usage = config.usage;

    // Initialize a depth texture matching the surface size.
    let depth_format = platform::texture::DepthFormat::Depth32Float;
    let depth_texture = Some(
      platform::texture::DepthTextureBuilder::new()
        .with_size(size.0.max(1), size.1.max(1))
        .with_format(depth_format)
        .with_label("lambda-depth")
        .build(&gpu),
    );

    return Ok(RenderContext {
      label: name,
      instance,
      surface,
      gpu,
      config,
      present_mode,
      texture_usage,
      size,
      depth_texture,
      depth_format,
      depth_sample_count: 1,
      msaa_color: None,
      msaa_sample_count: 1,
      render_passes: vec![],
      render_pipelines: vec![],
      bind_group_layouts: vec![],
      bind_groups: vec![],
      buffers: vec![],
      seen_error_messages: HashSet::new(),
    });
  }
}

/// High‑level rendering context for a single window.
///
/// Purpose
/// - Own the platform `Instance`, presentation `Surface`, and logical `Gpu`
///   objects bound to one window.
/// - Host immutable resources (`RenderPass`, `RenderPipeline`, bind layouts,
///   bind groups, and buffers) and expose small integer handles to reference
///   them when recording commands.
/// - Encode and submit per‑frame work based on an explicit `RenderCommand`
///   list.
///
/// Behavior
/// - All methods avoid panics unless explicitly documented; recoverable errors
///   are logged and dropped to keep the app running where possible.
/// - Surface loss or outdated configuration triggers transparent
///   reconfiguration with preserved present mode and usage.
pub struct RenderContext {
  label: String,
  instance: platform::instance::Instance,
  surface: platform::surface::Surface<'static>,
  gpu: platform::gpu::Gpu,
  config: platform::surface::SurfaceConfig,
  present_mode: platform::surface::PresentMode,
  texture_usage: platform::surface::TextureUsages,
  size: (u32, u32),
  depth_texture: Option<platform::texture::DepthTexture>,
  depth_format: platform::texture::DepthFormat,
  depth_sample_count: u32,
  msaa_color: Option<platform::texture::ColorAttachmentTexture>,
  msaa_sample_count: u32,
  render_passes: Vec<RenderPassDesc>,
  render_pipelines: Vec<RenderPipeline>,
  bind_group_layouts: Vec<bind::BindGroupLayout>,
  bind_groups: Vec<bind::BindGroup>,
  buffers: Vec<Rc<buffer::Buffer>>,
  seen_error_messages: HashSet<String>,
}

/// Opaque handle used to refer to resources attached to a `RenderContext`.
pub type ResourceId = usize;

impl RenderContext {
  /// Attach a render pipeline and return a handle for use in commands.
  pub fn attach_pipeline(&mut self, pipeline: RenderPipeline) -> ResourceId {
    let id = self.render_pipelines.len();
    self.render_pipelines.push(pipeline);
    return id;
  }

  /// Attach a render pass and return a handle for use in commands.
  pub fn attach_render_pass(
    &mut self,
    render_pass: RenderPassDesc,
  ) -> ResourceId {
    let id = self.render_passes.len();
    self.render_passes.push(render_pass);
    return id;
  }

  /// Attach a bind group layout and return a handle for use in pipeline layout composition.
  pub fn attach_bind_group_layout(
    &mut self,
    layout: bind::BindGroupLayout,
  ) -> ResourceId {
    let id = self.bind_group_layouts.len();
    self.bind_group_layouts.push(layout);
    return id;
  }

  /// Attach a bind group and return a handle for use in render commands.
  pub fn attach_bind_group(&mut self, group: bind::BindGroup) -> ResourceId {
    let id = self.bind_groups.len();
    self.bind_groups.push(group);
    return id;
  }

  /// Attach a generic GPU buffer and return a handle for render commands.
  pub fn attach_buffer(&mut self, buffer: buffer::Buffer) -> ResourceId {
    let id = self.buffers.len();
    self.buffers.push(Rc::new(buffer));
    return id;
  }

  /// Explicitly destroy the context. Dropping also releases resources.
  pub fn destroy(self) {
    drop(self);
  }

  /// Render a list of commands. No‑ops when the list is empty.
  ///
  /// Expectations
  /// - The sequence MUST begin a render pass before issuing draw‑related
  ///   commands and MUST terminate that pass with `EndRenderPass`.
  /// - Referenced resource handles (passes, pipelines, buffers, bind groups)
  ///   MUST have been attached to this context.
  ///
  /// Error handling
  /// - Errors are logged and do not panic (e.g., lost/outdated surface,
  ///   missing resources, invalid dynamic offsets). See `RenderError`.
  pub fn render(&mut self, commands: Vec<RenderCommand>) {
    if commands.is_empty() {
      return;
    }

    if let Err(err) = self.render_internal(commands) {
      let key = format!("{:?}", err);
      if self.seen_error_messages.insert(key) {
        logging::error!("Render error: {:?}", err);
      }
    }
  }

  /// Resize the surface and update surface configuration.
  pub fn resize(&mut self, width: u32, height: u32) {
    if width == 0 || height == 0 {
      return;
    }

    self.size = (width, height);
    if let Err(err) = self.reconfigure_surface(self.size) {
      logging::error!("Failed to resize surface: {:?}", err);
    }

    // Recreate depth texture to match new size.
    self.depth_texture = Some(
      platform::texture::DepthTextureBuilder::new()
        .with_size(self.size.0.max(1), self.size.1.max(1))
        .with_format(self.depth_format)
        .with_sample_count(self.depth_sample_count)
        .with_label("lambda-depth")
        .build(self.gpu()),
    );
    // Drop MSAA color target so it is rebuilt on demand with the new size.
    self.msaa_color = None;
  }

  /// Borrow a previously attached render pass by id.
  ///
  /// Panics if `id` does not refer to an attached pass.
  pub fn get_render_pass(&self, id: ResourceId) -> &RenderPassDesc {
    return &self.render_passes[id];
  }

  /// Borrow a previously attached render pipeline by id.
  ///
  /// Panics if `id` does not refer to an attached pipeline.
  pub fn get_render_pipeline(&self, id: ResourceId) -> &RenderPipeline {
    return &self.render_pipelines[id];
  }

  pub(crate) fn gpu(&self) -> &platform::gpu::Gpu {
    return &self.gpu;
  }

  pub(crate) fn surface_format(&self) -> platform::surface::SurfaceFormat {
    return self.config.format;
  }

  /// Device limit: maximum bytes that can be bound for a single uniform buffer binding.
  pub fn limit_max_uniform_buffer_binding_size(&self) -> u64 {
    return self.gpu.limits().max_uniform_buffer_binding_size;
  }

  /// Device limit: number of bind groups that can be used by a pipeline layout.
  pub fn limit_max_bind_groups(&self) -> u32 {
    return self.gpu.limits().max_bind_groups;
  }

  /// Device limit: required alignment in bytes for dynamic uniform buffer offsets.
  pub fn limit_min_uniform_buffer_offset_alignment(&self) -> u32 {
    return self.gpu.limits().min_uniform_buffer_offset_alignment;
  }

  /// Encode and submit GPU work for a single frame.
  fn render_internal(
    &mut self,
    commands: Vec<RenderCommand>,
  ) -> Result<(), RenderError> {
    if self.size.0 == 0 || self.size.1 == 0 {
      return Ok(());
    }

    let mut frame = match self.surface.acquire_next_frame() {
      Ok(frame) => frame,
      Err(platform::surface::SurfaceError::Lost)
      | Err(platform::surface::SurfaceError::Outdated) => {
        self.reconfigure_surface(self.size)?;
        self
          .surface
          .acquire_next_frame()
          .map_err(RenderError::Surface)?
      }
      Err(err) => return Err(RenderError::Surface(err)),
    };

    let view = frame.texture_view();
    let mut encoder = platform::command::CommandEncoder::new(
      self.gpu(),
      Some("lambda-render-command-encoder"),
    );

    let mut command_iter = commands.into_iter();
    while let Some(command) = command_iter.next() {
      match command {
        RenderCommand::BeginRenderPass {
          render_pass,
          viewport,
        } => {
          let pass = self.render_passes.get(render_pass).ok_or_else(|| {
            RenderError::Configuration(format!(
              "Unknown render pass {render_pass}"
            ))
          })?;

          // Build (begin) the platform render pass using the builder API.
          let mut rp_builder = platform::render_pass::RenderPassBuilder::new();
          if let Some(label) = pass.label() {
            rp_builder = rp_builder.with_label(label);
          }
          let ops = pass.color_operations();
          rp_builder = match ops.load {
            self::render_pass::ColorLoadOp::Load => rp_builder
              .with_color_load_op(platform::render_pass::ColorLoadOp::Load),
            self::render_pass::ColorLoadOp::Clear(color) => rp_builder
              .with_color_load_op(platform::render_pass::ColorLoadOp::Clear(
                color,
              )),
          };
          rp_builder = match ops.store {
            self::render_pass::StoreOp::Store => {
              rp_builder.with_store_op(platform::render_pass::StoreOp::Store)
            }
            self::render_pass::StoreOp::Discard => {
              rp_builder.with_store_op(platform::render_pass::StoreOp::Discard)
            }
          };
          // Create variably sized color attachments and begin the pass.
          let mut color_attachments =
            platform::render_pass::RenderColorAttachments::new();
          let sample_count = pass.sample_count();
          if pass.uses_color() {
            if sample_count > 1 {
              let need_recreate = match &self.msaa_color {
                Some(_) => self.msaa_sample_count != sample_count,
                None => true,
              };
              if need_recreate {
                self.msaa_color = Some(
                  platform::texture::ColorAttachmentTextureBuilder::new(
                    self.config.format,
                  )
                  .with_size(self.size.0.max(1), self.size.1.max(1))
                  .with_sample_count(sample_count)
                  .with_label("lambda-msaa-color")
                  .build(self.gpu()),
                );
                self.msaa_sample_count = sample_count;
              }
              let msaa_view = self
                .msaa_color
                .as_ref()
                .expect("MSAA color attachment should be created")
                .view_ref();
              color_attachments.push_msaa_color(msaa_view, view);
            } else {
              color_attachments.push_color(view);
            }
          }

          // Depth/stencil attachment when either depth or stencil requested.
          let want_depth_attachment = pass.depth_operations().is_some()
            || pass.stencil_operations().is_some();

          let (depth_view, depth_ops) = if want_depth_attachment {
            // Ensure depth texture exists, with proper sample count and format.
            let desired_samples = sample_count.max(1);

            // If stencil is requested on the pass, ensure we use a stencil-capable format.
            if pass.stencil_operations().is_some()
              && self.depth_format
                != platform::texture::DepthFormat::Depth24PlusStencil8
            {
              logging::error!(
                "Render pass has stencil ops but depth format {:?} lacks stencil; upgrading to Depth24PlusStencil8",
                self.depth_format
              );
              self.depth_format =
                platform::texture::DepthFormat::Depth24PlusStencil8;
            }

            let format_mismatch = self
              .depth_texture
              .as_ref()
              .map(|dt| dt.format() != self.depth_format)
              .unwrap_or(true);

            if self.depth_texture.is_none()
              || self.depth_sample_count != desired_samples
              || format_mismatch
            {
              self.depth_texture = Some(
                platform::texture::DepthTextureBuilder::new()
                  .with_size(self.size.0.max(1), self.size.1.max(1))
                  .with_format(self.depth_format)
                  .with_sample_count(desired_samples)
                  .with_label("lambda-depth")
                  .build(self.gpu()),
              );
              self.depth_sample_count = desired_samples;
            }

            let view_ref = self
              .depth_texture
              .as_ref()
              .expect("depth texture should be present")
              .view_ref();

            // Map depth ops; default when not explicitly provided.
            let mapped = match pass.depth_operations() {
              Some(dops) => platform::render_pass::DepthOperations {
                load: match dops.load {
                  render_pass::DepthLoadOp::Load => {
                    platform::render_pass::DepthLoadOp::Load
                  }
                  render_pass::DepthLoadOp::Clear(v) => {
                    platform::render_pass::DepthLoadOp::Clear(v as f32)
                  }
                },
                store: match dops.store {
                  render_pass::StoreOp::Store => {
                    platform::render_pass::StoreOp::Store
                  }
                  render_pass::StoreOp::Discard => {
                    platform::render_pass::StoreOp::Discard
                  }
                },
              },
              None => platform::render_pass::DepthOperations::default(),
            };
            (Some(view_ref), Some(mapped))
          } else {
            (None, None)
          };

          // Optional stencil operations
          let stencil_ops = pass.stencil_operations().map(|sop| {
            platform::render_pass::StencilOperations {
              load: match sop.load {
                render_pass::StencilLoadOp::Load => {
                  platform::render_pass::StencilLoadOp::Load
                }
                render_pass::StencilLoadOp::Clear(v) => {
                  platform::render_pass::StencilLoadOp::Clear(v)
                }
              },
              store: match sop.store {
                render_pass::StoreOp::Store => {
                  platform::render_pass::StoreOp::Store
                }
                render_pass::StoreOp::Discard => {
                  platform::render_pass::StoreOp::Discard
                }
              },
            }
          });

          let mut pass_encoder = rp_builder.build(
            &mut encoder,
            &mut color_attachments,
            depth_view,
            depth_ops,
            stencil_ops,
          );

          self.encode_pass(
            &mut pass_encoder,
            pass.uses_color(),
            pass.depth_operations().is_some(),
            pass.stencil_operations().is_some(),
            viewport,
            &mut command_iter,
          )?;
        }
        other => {
          logging::warn!(
            "Ignoring render command outside of a render pass: {:?}",
            other
          );
        }
      }
    }

    self.gpu.submit(iter::once(encoder.finish()));
    frame.present();
    return Ok(());
  }

  /// Encode a single render pass and consume commands until `EndRenderPass`.
  fn encode_pass<I>(
    &self,
    pass: &mut platform::render_pass::RenderPass<'_>,
    uses_color: bool,
    pass_has_depth: bool,
    pass_has_stencil: bool,
    initial_viewport: viewport::Viewport,
    commands: &mut I,
  ) -> Result<(), RenderError>
  where
    I: Iterator<Item = RenderCommand>,
  {
    Self::apply_viewport(pass, &initial_viewport);
    // De-duplicate advisories within this pass
    let mut warned_no_stencil_for_pipeline: HashSet<usize> = HashSet::new();
    let mut warned_no_depth_for_pipeline: HashSet<usize> = HashSet::new();

    while let Some(command) = commands.next() {
      match command {
        RenderCommand::EndRenderPass => return Ok(()),
        RenderCommand::SetStencilReference { reference } => {
          pass.set_stencil_reference(reference);
        }
        RenderCommand::SetPipeline { pipeline } => {
          let pipeline_ref =
            self.render_pipelines.get(pipeline).ok_or_else(|| {
              return RenderError::Configuration(format!(
                "Unknown pipeline {pipeline}"
              ));
            })?;
          // Validate pass/pipeline compatibility before deferring to wgpu.
          if !uses_color && pipeline_ref.has_color_targets() {
            let label = pipeline_ref.pipeline().label().unwrap_or("unnamed");
            return Err(RenderError::Configuration(format!(
              "Render pipeline '{}' declares color targets but the current pass has no color attachments",
              label
            )));
          }
          if uses_color && !pipeline_ref.has_color_targets() {
            let label = pipeline_ref.pipeline().label().unwrap_or("unnamed");
            return Err(RenderError::Configuration(format!(
              "Render pipeline '{}' has no color targets but the current pass declares color attachments",
              label
            )));
          }
          if !pass_has_depth && pipeline_ref.expects_depth_stencil() {
            let label = pipeline_ref.pipeline().label().unwrap_or("unnamed");
            return Err(RenderError::Configuration(format!(
              "Render pipeline '{}' expects a depth/stencil attachment but the current pass has none",
              label
            )));
          }
          // Advisory checks to help users reason about stencil/depth behavior.
          if pass_has_stencil
            && !pipeline_ref.uses_stencil()
            && warned_no_stencil_for_pipeline.insert(pipeline)
          {
            let label = pipeline_ref.pipeline().label().unwrap_or("unnamed");
            let key = format!("stencil:no_test:{}", label);
            let msg = format!(
              "Pass provides stencil ops but pipeline '{}' has no stencil test; stencil will not affect rendering",
              label
            );
            util::warn_once(&key, &msg);
          }
          if !pass_has_stencil && pipeline_ref.uses_stencil() {
            let label = pipeline_ref.pipeline().label().unwrap_or("unnamed");
            let key = format!("stencil:pass_no_operations:{}", label);
            let msg = format!(
              "Pipeline '{}' enables stencil but pass has no stencil ops configured; stencil reference/tests may be ineffective",
              label
            );
            util::warn_once(&key, &msg);
          }
          if pass_has_depth
            && !pipeline_ref.expects_depth_stencil()
            && warned_no_depth_for_pipeline.insert(pipeline)
          {
            let label = pipeline_ref.pipeline().label().unwrap_or("unnamed");
            let key = format!("depth:no_test:{}", label);
            let msg = format!(
              "Pass has depth attachment but pipeline '{}' does not enable depth testing; depth values will not be tested/written",
              label
            );
            util::warn_once(&key, &msg);
          }
          pass.set_pipeline(pipeline_ref.pipeline());
        }
        RenderCommand::SetViewports { viewports, .. } => {
          for viewport in viewports {
            Self::apply_viewport(pass, &viewport);
          }
        }
        RenderCommand::SetScissors { viewports, .. } => {
          for viewport in viewports {
            let (x, y, width, height) = viewport.scissor_u32();
            pass.set_scissor_rect(x, y, width, height);
          }
        }
        RenderCommand::SetBindGroup {
          set,
          group,
          dynamic_offsets,
        } => {
          let group_ref = self.bind_groups.get(group).ok_or_else(|| {
            return RenderError::Configuration(format!(
              "Unknown bind group {group}"
            ));
          })?;
          // Validate dynamic offsets count and alignment before binding.
          validation::validate_dynamic_offsets(
            group_ref.dynamic_binding_count(),
            &dynamic_offsets,
            self.limit_min_uniform_buffer_offset_alignment(),
            set,
          )
          .map_err(RenderError::Configuration)?;
          pass.set_bind_group(
            set,
            group_ref.platform_group(),
            &dynamic_offsets,
          );
        }
        RenderCommand::BindVertexBuffer { pipeline, buffer } => {
          let pipeline_ref =
            self.render_pipelines.get(pipeline).ok_or_else(|| {
              return RenderError::Configuration(format!(
                "Unknown pipeline {pipeline}"
              ));
            })?;
          let buffer_ref =
            pipeline_ref.buffers().get(buffer as usize).ok_or_else(|| {
              return RenderError::Configuration(format!(
                "Vertex buffer index {buffer} not found for pipeline {pipeline}"
              ));
            })?;

          pass.set_vertex_buffer(buffer as u32, buffer_ref.raw());
        }
        RenderCommand::BindIndexBuffer { buffer, format } => {
          let buffer_ref = self.buffers.get(buffer).ok_or_else(|| {
            return RenderError::Configuration(format!(
              "Index buffer id {} not found",
              buffer
            ));
          })?;
          // Soft validation: encourage correct logical type.
          if buffer_ref.buffer_type() != buffer::BufferType::Index {
            logging::warn!(
              "Binding buffer id {} as index but logical type is {:?}",
              buffer,
              buffer_ref.buffer_type()
            );
          }
          pass.set_index_buffer(buffer_ref.raw(), format);
        }
        RenderCommand::PushConstants {
          pipeline,
          stage,
          offset,
          bytes,
        } => {
          let _ = self.render_pipelines.get(pipeline).ok_or_else(|| {
            return RenderError::Configuration(format!(
              "Unknown pipeline {pipeline}"
            ));
          })?;
          let slice = unsafe {
            std::slice::from_raw_parts(
              bytes.as_ptr() as *const u8,
              bytes.len() * std::mem::size_of::<u32>(),
            )
          };
          pass.set_push_constants(stage, offset, slice);
        }
        RenderCommand::Draw { vertices } => {
          pass.draw(vertices);
        }
        RenderCommand::DrawIndexed {
          indices,
          base_vertex,
        } => {
          pass.draw_indexed(indices, base_vertex);
        }
        RenderCommand::BeginRenderPass { .. } => {
          return Err(RenderError::Configuration(
            "Nested render passes are not supported.".to_string(),
          ));
        }
      }
    }

    return Err(RenderError::Configuration(
      "Render pass did not terminate with EndRenderPass".to_string(),
    ));
  }

  /// Apply both viewport and scissor state to the active pass.
  fn apply_viewport(
    pass: &mut platform::render_pass::RenderPass<'_>,
    viewport: &viewport::Viewport,
  ) {
    let (x, y, width, height, min_depth, max_depth) = viewport.viewport_f32();
    pass.set_viewport(x, y, width, height, min_depth, max_depth);
    let (sx, sy, sw, sh) = viewport.scissor_u32();
    pass.set_scissor_rect(sx, sy, sw, sh);
  }

  /// Reconfigure the presentation surface using current present mode/usage.
  fn reconfigure_surface(
    &mut self,
    size: (u32, u32),
  ) -> Result<(), RenderError> {
    self
      .surface
      .resize(&self.gpu, size)
      .map_err(RenderError::Configuration)?;

    let config = self.surface.configuration().cloned().ok_or_else(|| {
      RenderError::Configuration("Surface was not configured".to_string())
    })?;

    self.present_mode = config.present_mode;
    self.texture_usage = config.usage;
    self.config = config;
    return Ok(());
  }
}

/// Errors reported while preparing or presenting a frame.
#[derive(Debug)]
///
/// Variants summarize recoverable issues that can appear during frame
/// acquisition or command encoding. The renderer logs these and continues when
/// possible; callers SHOULD treat them as warnings unless persistent.
pub enum RenderError {
  Surface(platform::surface::SurfaceError),
  Configuration(String),
}

impl From<platform::surface::SurfaceError> for RenderError {
  fn from(error: platform::surface::SurfaceError) -> Self {
    return RenderError::Surface(error);
  }
}

/// Errors encountered while creating a `RenderContext`.
#[derive(Debug)]
///
/// Returned by `RenderContextBuilder::build` to avoid panics during
/// initialization and provide actionable error messages to callers.
pub enum RenderContextError {
  /// Failure creating the presentation surface for the provided window.
  SurfaceCreate(String),
  /// Failure creating the logical GPU device/queue.
  GpuCreate(String),
  /// Failure configuring or retrieving the surface configuration.
  SurfaceConfig(String),
}

impl core::fmt::Display for RenderContextError {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match self {
      RenderContextError::SurfaceCreate(s) => write!(f, "{}", s),
      RenderContextError::GpuCreate(s) => write!(f, "{}", s),
      RenderContextError::SurfaceConfig(s) => write!(f, "{}", s),
    }
  }
}

impl std::error::Error for RenderContextError {}
