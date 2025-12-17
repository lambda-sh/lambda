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
pub mod encoder;
pub mod gpu;
pub mod instance;
pub mod mesh;
pub mod pipeline;
pub mod render_pass;
pub mod render_target;
pub mod scene_math;
pub mod shader;
pub mod surface;
pub mod target;
pub mod texture;
pub mod validation;
pub mod vertex;
pub mod viewport;
pub mod window;

// Internal modules
mod color_attachments;

use std::{
  collections::HashSet,
  rc::Rc,
};

use logging;

use self::{
  command::RenderCommand,
  encoder::{
    CommandEncoder,
    RenderPassError,
  },
  pipeline::RenderPipeline,
  render_pass::RenderPass as RenderPassDesc,
  render_target::RenderTarget,
};

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

    let instance = instance::InstanceBuilder::new()
      .with_label(&format!("{} Instance", name))
      .build();

    let mut surface = render_target::WindowSurface::new(&instance, window)
      .map_err(|e| {
        RenderContextError::SurfaceCreate(format!(
          "Failed to create rendering surface: {:?}",
          e
        ))
      })?;

    let gpu = gpu::GpuBuilder::new()
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
        surface::PresentMode::default(),
        texture::TextureUsages::RENDER_ATTACHMENT,
      )
      .map_err(|e| {
        RenderContextError::SurfaceConfig(format!(
          "Failed to configure surface: {:?}",
          e
        ))
      })?;

    let config = surface.configuration().ok_or_else(|| {
      RenderContextError::SurfaceConfig(
        "Surface was not configured".to_string(),
      )
    })?;
    let texture_usage = config.usage;
    let config = config.clone();

    // Initialize the render context with an engine-level depth format.
    let depth_format = texture::DepthFormat::Depth32Float;

    let mut render_context = RenderContext {
      label: name,
      instance,
      surface,
      gpu,
      config,
      texture_usage,
      size,
      depth_texture: None,
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
    };

    // Initialize a depth texture matching the surface size using the
    // high-level depth texture builder.
    let depth_texture = texture::DepthTextureBuilder::new()
      .with_size(size.0.max(1), size.1.max(1))
      .with_format(depth_format)
      .with_label("lambda-depth")
      .build(render_context.gpu());
    render_context.depth_texture = Some(depth_texture);

    return Ok(render_context);
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
  instance: instance::Instance,
  surface: render_target::WindowSurface,
  gpu: gpu::Gpu,
  config: surface::SurfaceConfig,
  texture_usage: texture::TextureUsages,
  size: (u32, u32),
  depth_texture: Option<texture::DepthTexture>,
  depth_format: texture::DepthFormat,
  depth_sample_count: u32,
  msaa_color: Option<texture::ColorAttachmentTexture>,
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
  /// Current surface size in pixels.
  ///
  /// This reflects the most recent configured surface dimensions and is used
  /// as a default for render‑target creation and viewport setup.
  pub fn surface_size(&self) -> (u32, u32) {
    return self.size;
  }

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
      texture::DepthTextureBuilder::new()
        .with_size(self.size.0.max(1), self.size.1.max(1))
        .with_format(self.depth_format)
        .with_sample_count(self.depth_sample_count)
        .with_label("lambda-depth")
        .build(&self.gpu),
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

  /// Access the GPU device for resource creation.
  ///
  /// Use this to pass to resource builders (buffers, textures, bind groups,
  /// etc.) when creating GPU resources.
  pub fn gpu(&self) -> &gpu::Gpu {
    return &self.gpu;
  }

  /// The texture format of the render surface.
  pub fn surface_format(&self) -> texture::TextureFormat {
    return self.config.format;
  }

  /// The depth texture format used for depth/stencil operations.
  pub fn depth_format(&self) -> texture::DepthFormat {
    return self.depth_format;
  }

  pub(crate) fn supports_surface_sample_count(
    &self,
    sample_count: u32,
  ) -> bool {
    return self
      .gpu
      .supports_sample_count_for_format(self.config.format, sample_count);
  }

  pub(crate) fn supports_depth_sample_count(
    &self,
    format: texture::DepthFormat,
    sample_count: u32,
  ) -> bool {
    return self
      .gpu
      .supports_sample_count_for_depth(format, sample_count);
  }

  /// Device limit: maximum bytes that can be bound for a single uniform buffer binding.
  pub fn limit_max_uniform_buffer_binding_size(&self) -> u64 {
    return self.gpu.limit_max_uniform_buffer_binding_size();
  }

  /// Device limit: number of bind groups that can be used by a pipeline layout.
  pub fn limit_max_bind_groups(&self) -> u32 {
    return self.gpu.limit_max_bind_groups();
  }

  /// Device limit: maximum number of vertex buffers that can be bound.
  pub fn limit_max_vertex_buffers(&self) -> u32 {
    return self.gpu.limit_max_vertex_buffers();
  }

  /// Device limit: maximum number of vertex attributes that can be declared.
  pub fn limit_max_vertex_attributes(&self) -> u32 {
    return self.gpu.limit_max_vertex_attributes();
  }

  /// Device limit: required alignment in bytes for dynamic uniform buffer offsets.
  pub fn limit_min_uniform_buffer_offset_alignment(&self) -> u32 {
    return self.gpu.limit_min_uniform_buffer_offset_alignment();
  }

  /// Ensure the MSAA color attachment texture exists with the given sample
  /// count, recreating it if necessary. Returns the texture view reference.
  ///
  /// This method manages the lifecycle of the internal MSAA texture, creating
  /// or recreating it when the sample count changes.
  fn ensure_msaa_color_texture(
    &mut self,
    sample_count: u32,
  ) -> surface::TextureView<'_> {
    let need_recreate = match &self.msaa_color {
      Some(_) => self.msaa_sample_count != sample_count,
      None => true,
    };

    if need_recreate {
      self.msaa_color = Some(
        texture::ColorAttachmentTextureBuilder::new(self.config.format)
          .with_size(self.size.0.max(1), self.size.1.max(1))
          .with_sample_count(sample_count)
          .with_label("lambda-msaa-color")
          .build(&self.gpu),
      );
      self.msaa_sample_count = sample_count;
    }

    return self
      .msaa_color
      .as_ref()
      .expect("MSAA color attachment should exist")
      .view_ref();
  }

  /// Encode and submit GPU work for a single frame.
  fn render_internal(
    &mut self,
    commands: Vec<RenderCommand>,
  ) -> Result<(), RenderError> {
    if self.size.0 == 0 || self.size.1 == 0 {
      return Ok(());
    }

    let frame = match self.surface.acquire_frame() {
      Ok(frame) => frame,
      Err(err) => match err {
        surface::SurfaceError::Lost | surface::SurfaceError::Outdated => {
          self.reconfigure_surface(self.size)?;
          self
            .surface
            .acquire_frame()
            .map_err(|e| RenderError::Surface(e))?
        }
        _ => return Err(RenderError::Surface(err)),
      },
    };

    let view = frame.texture_view();
    let mut encoder =
      CommandEncoder::new(self, "lambda-render-command-encoder");

    let mut command_iter = commands.into_iter();
    while let Some(command) = command_iter.next() {
      match command {
        RenderCommand::BeginRenderPass {
          render_pass,
          viewport,
        } => {
          // Clone the render pass descriptor to avoid borrowing self while we
          // need mutable access for MSAA texture creation.
          let pass = self
            .render_passes
            .get(render_pass)
            .ok_or_else(|| {
              RenderError::Configuration(format!(
                "Unknown render pass {render_pass}"
              ))
            })?
            .clone();

          // Ensure MSAA texture exists if needed.
          let sample_count = pass.sample_count();
          let uses_color = pass.uses_color();
          if uses_color && sample_count > 1 {
            self.ensure_msaa_color_texture(sample_count);
          }

          // Create color attachments for the surface pass. The MSAA view is
          // retrieved here after the mutable borrow for texture creation ends.
          let msaa_view = if sample_count > 1 {
            self.msaa_color.as_ref().map(|t| t.view_ref())
          } else {
            None
          };
          let mut color_attachments =
            color_attachments::RenderColorAttachments::for_surface_pass(
              uses_color,
              sample_count,
              msaa_view,
              view,
            );

          // Depth/stencil attachment when either depth or stencil requested.
          let want_depth_attachment = Self::has_depth_attachment(
            pass.depth_operations(),
            pass.stencil_operations(),
          );

          // Prepare depth texture if needed.
          if want_depth_attachment {
            // Ensure depth texture exists, with proper sample count and format.
            let desired_samples = sample_count.max(1);

            // If stencil is requested on the pass, ensure we use a
            // stencil-capable format.
            if pass.stencil_operations().is_some()
              && self.depth_format != texture::DepthFormat::Depth24PlusStencil8
            {
              #[cfg(any(
                debug_assertions,
                feature = "render-validation-stencil",
              ))]
              logging::error!(
                "Render pass has stencil ops but depth format {:?} lacks \
                 stencil; upgrading to Depth24PlusStencil8",
                self.depth_format
              );
              self.depth_format = texture::DepthFormat::Depth24PlusStencil8;
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
                texture::DepthTextureBuilder::new()
                  .with_size(self.size.0.max(1), self.size.1.max(1))
                  .with_format(self.depth_format)
                  .with_sample_count(desired_samples)
                  .with_label("lambda-depth")
                  .build(&self.gpu),
              );
              self.depth_sample_count = desired_samples;
            }
          }

          let depth_texture_ref = if want_depth_attachment {
            self.depth_texture.as_ref()
          } else {
            None
          };

          // Use the high-level encoder's with_render_pass callback API.
          let min_uniform_buffer_offset_alignment =
            self.limit_min_uniform_buffer_offset_alignment();
          let render_pipelines = &self.render_pipelines;
          let bind_groups = &self.bind_groups;
          let buffers = &self.buffers;

          encoder.with_render_pass(
            &pass,
            &mut color_attachments,
            depth_texture_ref,
            |rp_encoder| {
              rp_encoder.set_viewport(&viewport);

              while let Some(cmd) = command_iter.next() {
                match cmd {
                  RenderCommand::EndRenderPass => return Ok(()),
                  RenderCommand::SetStencilReference { reference } => {
                    rp_encoder.set_stencil_reference(reference);
                  }
                  RenderCommand::SetPipeline { pipeline } => {
                    let pipeline_ref =
                      render_pipelines.get(pipeline).ok_or_else(|| {
                        RenderPassError::Validation(format!(
                          "Unknown pipeline {pipeline}"
                        ))
                      })?;
                    rp_encoder.set_pipeline(pipeline_ref)?;
                  }
                  RenderCommand::SetViewports { viewports, .. } => {
                    for vp in viewports {
                      rp_encoder.set_viewport(&vp);
                    }
                  }
                  RenderCommand::SetScissors { viewports, .. } => {
                    for vp in viewports {
                      rp_encoder.set_scissor(&vp);
                    }
                  }
                  RenderCommand::SetBindGroup {
                    set,
                    group,
                    dynamic_offsets,
                  } => {
                    let group_ref =
                      bind_groups.get(group).ok_or_else(|| {
                        RenderPassError::Validation(format!(
                          "Unknown bind group {group}"
                        ))
                      })?;
                    rp_encoder.set_bind_group(
                      set,
                      group_ref,
                      &dynamic_offsets,
                      min_uniform_buffer_offset_alignment,
                    )?;
                  }
                  RenderCommand::BindVertexBuffer { pipeline, buffer } => {
                    let pipeline_ref =
                      render_pipelines.get(pipeline).ok_or_else(|| {
                        RenderPassError::Validation(format!(
                          "Unknown pipeline {pipeline}"
                        ))
                      })?;
                    let buffer_ref = pipeline_ref
                      .buffers()
                      .get(buffer as usize)
                      .ok_or_else(|| {
                        RenderPassError::Validation(format!(
                          "Vertex buffer index {buffer} not found for \
                           pipeline {pipeline}"
                        ))
                      })?;
                    rp_encoder.set_vertex_buffer(buffer as u32, buffer_ref);
                  }
                  RenderCommand::BindIndexBuffer { buffer, format } => {
                    let buffer_ref = buffers.get(buffer).ok_or_else(|| {
                      RenderPassError::Validation(format!(
                        "Index buffer id {} not found",
                        buffer
                      ))
                    })?;
                    rp_encoder.set_index_buffer(buffer_ref, format)?;
                  }
                  RenderCommand::PushConstants {
                    pipeline,
                    stage,
                    offset,
                    bytes,
                  } => {
                    let _ =
                      render_pipelines.get(pipeline).ok_or_else(|| {
                        RenderPassError::Validation(format!(
                          "Unknown pipeline {pipeline}"
                        ))
                      })?;
                    let slice = unsafe {
                      std::slice::from_raw_parts(
                        bytes.as_ptr() as *const u8,
                        bytes.len() * std::mem::size_of::<u32>(),
                      )
                    };
                    rp_encoder.set_push_constants(stage, offset, slice);
                  }
                  RenderCommand::Draw {
                    vertices,
                    instances,
                  } => {
                    rp_encoder.draw(vertices, instances)?;
                  }
                  RenderCommand::DrawIndexed {
                    indices,
                    base_vertex,
                    instances,
                  } => {
                    rp_encoder.draw_indexed(indices, base_vertex, instances)?;
                  }
                  RenderCommand::BeginRenderPass { .. } => {
                    return Err(RenderPassError::Validation(
                      "Nested render passes are not supported.".to_string(),
                    ));
                  }
                }
              }

              return Err(RenderPassError::Validation(
                "Render pass did not terminate with EndRenderPass".to_string(),
              ));
            },
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

    encoder.finish(self);
    frame.present();
    return Ok(());
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

    let config = self.surface.configuration().ok_or_else(|| {
      RenderError::Configuration("Surface was not configured".to_string())
    })?;

    self.texture_usage = config.usage;
    self.config = config.clone();
    return Ok(());
  }

  /// Determine whether a pass requires a depth attachment based on depth or
  /// stencil operations.
  fn has_depth_attachment(
    depth_ops: Option<render_pass::DepthOperations>,
    stencil_ops: Option<render_pass::StencilOperations>,
  ) -> bool {
    return depth_ops.is_some() || stencil_ops.is_some();
  }
}

/// Errors reported while preparing or presenting a frame.
#[derive(Debug)]
///
/// Variants summarize recoverable issues that can appear during frame
/// acquisition or command encoding. The renderer logs these and continues when
/// possible; callers SHOULD treat them as warnings unless persistent.
pub enum RenderError {
  Surface(surface::SurfaceError),
  Configuration(String),
}

impl From<surface::SurfaceError> for RenderError {
  fn from(error: surface::SurfaceError) -> Self {
    return RenderError::Surface(error);
  }
}

impl From<RenderPassError> for RenderError {
  fn from(error: RenderPassError) -> Self {
    return RenderError::Configuration(error.to_string());
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

#[cfg(test)]
mod tests {
  use super::*;
  use crate::render::render_pass;

  #[test]
  fn has_depth_attachment_false_when_no_depth_or_stencil() {
    let has_attachment = RenderContext::has_depth_attachment(None, None);
    assert!(!has_attachment);
  }

  #[test]
  fn has_depth_attachment_true_for_depth_only() {
    let depth_ops = Some(render_pass::DepthOperations::default());
    let has_attachment = RenderContext::has_depth_attachment(depth_ops, None);
    assert!(has_attachment);
  }

  #[test]
  fn has_depth_attachment_true_for_stencil_only() {
    let stencil_ops = Some(render_pass::StencilOperations::default());
    let has_attachment = RenderContext::has_depth_attachment(None, stencil_ops);
    assert!(has_attachment);
  }
}
