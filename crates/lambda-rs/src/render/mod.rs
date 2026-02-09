//! High‑level rendering API for cross‑platform windowed applications.
//!
//! The rendering module provides a small set of stable, engine‑facing types
//! that assemble a frame using explicit commands.
//!
//! Concepts
//! - `RenderContext`: owns the graphics instance, presentation surface, and
//!   GPU device/queue for a single window. It is the submit point for per‑frame
//!   command encoding.
//! - `RenderPass` and `RenderPipeline`: immutable descriptions used when
//!   beginning a pass and binding a pipeline. Pipelines declare their vertex
//!   inputs, immediate data, and layout (bind group layouts).
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
//! See runnable demos under `demos/render/src/bin/` for end‑to‑end snippets.

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
pub mod scene_math;
pub mod shader;
pub mod targets;
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
  command::{
    RenderCommand,
    RenderDestination,
  },
  encoder::{
    CommandEncoder,
    RenderPassDestinationInfo,
    RenderPassError,
  },
  pipeline::RenderPipeline,
  render_pass::RenderPass as RenderPassDesc,
  targets::surface::RenderTarget,
};

/// High-level presentation mode selection for window surfaces.
///
/// The selected mode is validated against the adapter's surface capabilities
/// during `RenderContextBuilder::build`. If the requested mode is not
/// supported, Lambda selects a supported fallback.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PresentMode {
  /// VSync enabled, capped to display refresh rate (FIFO).
  Vsync,
  /// VSync disabled, immediate presentation (may tear).
  Immediate,
  /// Triple buffering, low latency without tearing if supported.
  Mailbox,
}

impl Default for PresentMode {
  fn default() -> Self {
    return PresentMode::Vsync;
  }
}

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
  present_mode: Option<PresentMode>,
}

impl RenderContextBuilder {
  /// Create a new builder tagged with a human‑readable `name` used in labels.
  pub fn new(name: &str) -> Self {
    Self {
      name: name.to_string(),
      _render_timeout: 1_000_000_000,
      present_mode: None,
    }
  }

  /// Set how long rendering may take before timing out (nanoseconds).
  pub fn with_render_timeout(mut self, render_timeout: u64) -> Self {
    self._render_timeout = render_timeout;
    self
  }

  /// Enable or disable vertical sync.
  ///
  /// When enabled, the builder requests `PresentMode::Vsync` (FIFO).
  ///
  /// When disabled, the builder requests a non‑vsync mode (immediate
  /// presentation) and falls back to a supported low-latency mode if needed.
  pub fn with_vsync(mut self, enabled: bool) -> Self {
    self.present_mode = Some(if enabled {
      PresentMode::Vsync
    } else {
      PresentMode::Immediate
    });
    return self;
  }

  /// Explicitly select a presentation mode.
  ///
  /// The requested mode is validated against the adapter's surface
  /// capabilities. If unsupported, the renderer falls back to a supported
  /// mode with similar behavior.
  pub fn with_present_mode(mut self, mode: PresentMode) -> Self {
    self.present_mode = Some(mode);
    return self;
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
    let RenderContextBuilder {
      name, present_mode, ..
    } = self;

    let instance = instance::InstanceBuilder::new()
      .with_label(&format!("{} Instance", name))
      .build();

    let mut surface = targets::surface::WindowSurface::new(&instance, window)
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
    let requested_present_mode = present_mode.unwrap_or_else(|| {
      if window.vsync_requested() {
        return PresentMode::Vsync;
      }
      return PresentMode::Immediate;
    });
    let platform_present_mode = match requested_present_mode {
      PresentMode::Vsync => targets::surface::PresentMode::Fifo,
      PresentMode::Immediate => targets::surface::PresentMode::Immediate,
      PresentMode::Mailbox => targets::surface::PresentMode::Mailbox,
    };
    surface
      .configure_with_defaults(
        &gpu,
        size,
        platform_present_mode,
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
      surface: Some(surface),
      gpu,
      config,
      texture_usage,
      size,
      depth_texture: None,
      depth_format,
      depth_sample_count: 1,
      msaa_color: None,
      msaa_sample_count: 1,
      offscreen_targets: vec![],
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
  #[allow(dead_code)]
  instance: instance::Instance,
  surface: Option<targets::surface::WindowSurface>,
  gpu: gpu::Gpu,
  config: targets::surface::SurfaceConfig,
  texture_usage: texture::TextureUsages,
  size: (u32, u32),
  depth_texture: Option<texture::DepthTexture>,
  depth_format: texture::DepthFormat,
  depth_sample_count: u32,
  msaa_color: Option<texture::ColorAttachmentTexture>,
  msaa_sample_count: u32,
  offscreen_targets: Vec<targets::offscreen::OffscreenTarget>,
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
  /// Optional label assigned when constructing the context.
  pub fn label(&self) -> &str {
    return self.label.as_str();
  }

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

  /// Attach an offscreen target and return a handle for use in destinations.
  pub fn attach_offscreen_target(
    &mut self,
    target: targets::offscreen::OffscreenTarget,
  ) -> ResourceId {
    let id = self.offscreen_targets.len();
    self.offscreen_targets.push(target);
    return id;
  }

  /// Replace an attached offscreen target in-place.
  ///
  /// Returns an error when `id` does not refer to an attached offscreen
  /// target.
  pub fn replace_offscreen_target(
    &mut self,
    id: ResourceId,
    target: targets::offscreen::OffscreenTarget,
  ) -> Result<(), String> {
    let slot = match self.offscreen_targets.get_mut(id) {
      Some(slot) => slot,
      None => return Err(format!("Unknown offscreen target id {}", id)),
    };
    *slot = target;
    return Ok(());
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

  /// Replace an attached bind group in-place.
  pub fn replace_bind_group(
    &mut self,
    id: ResourceId,
    group: bind::BindGroup,
  ) -> Result<(), String> {
    let slot = match self.bind_groups.get_mut(id) {
      Some(slot) => slot,
      None => return Err(format!("Unknown bind group id {}", id)),
    };
    *slot = group;
    return Ok(());
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
    if self.surface.is_some() {
      if let Err(err) = self.reconfigure_surface(self.size) {
        logging::error!("Failed to resize surface: {:?}", err);
      }
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

  /// Borrow a previously attached offscreen target by id.
  ///
  /// Panics if `id` does not refer to an attached offscreen target.
  pub fn get_offscreen_target(
    &self,
    id: ResourceId,
  ) -> &targets::offscreen::OffscreenTarget {
    return &self.offscreen_targets[id];
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
  ) -> targets::surface::TextureView<'_> {
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

    // Determine whether this command list needs access to the presentation
    // surface. We only acquire a surface frame when a surface-backed pass is
    // requested; offscreen-only command lists can render without a window.
    let requires_surface = commands.iter().any(|cmd| {
      return matches!(
        cmd,
        RenderCommand::BeginRenderPass { .. }
          | RenderCommand::BeginRenderPassTo {
            destination: RenderDestination::Surface,
            ..
          }
      );
    });

    let mut frame = if requires_surface {
      // Acquire exactly one surface frame up-front and reuse its `TextureView`
      // for all surface-backed render passes in this command list. The acquired
      // frame is presented after encoding completes.
      //
      // If acquisition fails due to surface loss/outdated config, attempt to
      // reconfigure the surface to the current context size and retry once.
      let acquired = {
        let surface = self.surface.as_mut().ok_or_else(|| {
          RenderError::Configuration(
            "No surface attached to RenderContext".to_string(),
          )
        })?;
        surface.acquire_frame()
      };

      Some(match acquired {
        Ok(frame) => frame,
        Err(err) => match err {
          targets::surface::SurfaceError::Lost
          | targets::surface::SurfaceError::Outdated => {
            self.reconfigure_surface(self.size)?;
            let surface = self.surface.as_mut().ok_or_else(|| {
              RenderError::Configuration(
                "No surface attached to RenderContext".to_string(),
              )
            })?;
            surface.acquire_frame().map_err(RenderError::Surface)?
          }
          _ => return Err(RenderError::Surface(err)),
        },
      })
    } else {
      None
    };

    let mut encoder =
      CommandEncoder::new(self, "lambda-render-command-encoder");

    let mut command_iter = commands.into_iter();
    while let Some(command) = command_iter.next() {
      match command {
        RenderCommand::BeginRenderPass {
          render_pass,
          viewport,
        } => {
          let view = frame
            .as_ref()
            .ok_or_else(|| {
              RenderError::Configuration(
                "Surface render pass requested but no surface is attached"
                  .to_string(),
              )
            })?
            .texture_view();
          self.encode_surface_render_pass(
            &mut encoder,
            &mut command_iter,
            render_pass,
            viewport,
            view,
          )?;
        }
        RenderCommand::BeginRenderPassTo {
          render_pass,
          viewport,
          destination,
        } => match destination {
          RenderDestination::Surface => {
            let view = frame
              .as_ref()
              .ok_or_else(|| {
                RenderError::Configuration(
                  "Surface render pass requested but no surface is attached"
                    .to_string(),
                )
              })?
              .texture_view();
            self.encode_surface_render_pass(
              &mut encoder,
              &mut command_iter,
              render_pass,
              viewport,
              view,
            )?;
          }
          RenderDestination::Offscreen(target_id) => {
            self.encode_offscreen_render_pass(
              &mut encoder,
              &mut command_iter,
              render_pass,
              viewport,
              target_id,
            )?;
          }
        },
        other => {
          logging::warn!(
            "Ignoring render command outside of a render pass: {:?}",
            other
          );
        }
      }
    }

    encoder.finish(self);
    if let Some(frame) = frame.take() {
      frame.present();
    }
    return Ok(());
  }

  fn encode_surface_render_pass<'view>(
    &mut self,
    encoder: &mut CommandEncoder,
    command_iter: &mut std::vec::IntoIter<RenderCommand>,
    render_pass: ResourceId,
    viewport: viewport::Viewport,
    surface_view: targets::surface::TextureView<'view>,
  ) -> Result<(), RenderError> {
    // Clone the render pass descriptor to avoid borrowing self while we need
    // mutable access for MSAA texture creation.
    let pass = self
      .render_passes
      .get(render_pass)
      .ok_or_else(|| {
        RenderError::Configuration(format!("Unknown render pass {render_pass}"))
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
        surface_view,
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

      // If stencil is requested on the pass, ensure we use a stencil-capable format.
      if pass.stencil_operations().is_some()
        && self.depth_format != texture::DepthFormat::Depth24PlusStencil8
      {
        #[cfg(any(debug_assertions, feature = "render-validation-stencil",))]
        logging::error!(
          "Render pass has stencil ops but depth format {:?} lacks stencil; upgrading to Depth24PlusStencil8",
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

    let min_uniform_buffer_offset_alignment =
      self.limit_min_uniform_buffer_offset_alignment();
    let render_pipelines = &self.render_pipelines;
    let bind_groups = &self.bind_groups;
    let buffers = &self.buffers;

    encoder.with_render_pass(
      &pass,
      RenderPassDestinationInfo {
        color_format: if uses_color {
          Some(self.surface_format())
        } else {
          None
        },
        depth_format: if want_depth_attachment {
          Some(self.depth_format())
        } else {
          None
        },
      },
      &mut color_attachments,
      depth_texture_ref,
      |rp_encoder| {
        return Self::encode_active_render_pass_commands(
          command_iter,
          rp_encoder,
          &viewport,
          render_pipelines,
          bind_groups,
          buffers,
          min_uniform_buffer_offset_alignment,
        );
      },
    )?;

    return Ok(());
  }

  fn encode_offscreen_render_pass(
    &mut self,
    encoder: &mut CommandEncoder,
    command_iter: &mut std::vec::IntoIter<RenderCommand>,
    render_pass: ResourceId,
    viewport: viewport::Viewport,
    target_id: ResourceId,
  ) -> Result<(), RenderError> {
    let pass = self
      .render_passes
      .get(render_pass)
      .ok_or_else(|| {
        RenderError::Configuration(format!("Unknown render pass {render_pass}"))
      })?
      .clone();

    let target = self.offscreen_targets.get(target_id).ok_or_else(|| {
      RenderError::Configuration(format!(
        "Unknown offscreen target {target_id}"
      ))
    })?;

    let pass_samples = pass.sample_count();
    let target_samples = target.sample_count();
    if pass_samples != target_samples {
      return Err(RenderError::Configuration(format!(
        "Pass sample_count={} does not match offscreen target sample_count={}",
        pass_samples, target_samples
      )));
    }

    let uses_color = pass.uses_color();
    let mut color_attachments =
      color_attachments::RenderColorAttachments::for_offscreen_pass(
        uses_color,
        target_samples,
        target.msaa_view(),
        target.resolve_view(),
      );

    let want_depth_attachment = Self::has_depth_attachment(
      pass.depth_operations(),
      pass.stencil_operations(),
    );

    if want_depth_attachment && target.depth_texture().is_none() {
      return Err(RenderError::Configuration(
        "Render pass requests depth/stencil operations but the selected offscreen target has no depth attachment"
          .to_string(),
      ));
    }

    if pass.stencil_operations().is_some()
      && target.depth_format()
        != Some(texture::DepthFormat::Depth24PlusStencil8)
    {
      return Err(RenderError::Configuration(
        "Render pass requests stencil operations but the selected offscreen target depth format lacks stencil"
          .to_string(),
      ));
    }

    let depth_texture_ref = if want_depth_attachment {
      target.depth_texture()
    } else {
      None
    };

    let min_uniform_buffer_offset_alignment =
      self.limit_min_uniform_buffer_offset_alignment();
    let render_pipelines = &self.render_pipelines;
    let bind_groups = &self.bind_groups;
    let buffers = &self.buffers;

    encoder.with_render_pass(
      &pass,
      RenderPassDestinationInfo {
        color_format: if uses_color {
          Some(target.color_format())
        } else {
          None
        },
        depth_format: if want_depth_attachment {
          target.depth_format()
        } else {
          None
        },
      },
      &mut color_attachments,
      depth_texture_ref,
      |rp_encoder| {
        return Self::encode_active_render_pass_commands(
          command_iter,
          rp_encoder,
          &viewport,
          render_pipelines,
          bind_groups,
          buffers,
          min_uniform_buffer_offset_alignment,
        );
      },
    )?;

    return Ok(());
  }

  fn validate_pipeline_exists(
    render_pipelines: &[RenderPipeline],
    pipeline: usize,
  ) -> Result<(), RenderPassError> {
    if render_pipelines.get(pipeline).is_none() {
      return Err(RenderPassError::Validation(format!(
        "Unknown pipeline {pipeline}"
      )));
    }
    return Ok(());
  }

  fn encode_active_render_pass_commands(
    command_iter: &mut std::vec::IntoIter<RenderCommand>,
    rp_encoder: &mut encoder::RenderPassEncoder<'_>,
    initial_viewport: &viewport::Viewport,
    render_pipelines: &[RenderPipeline],
    bind_groups: &[bind::BindGroup],
    buffers: &[Rc<buffer::Buffer>],
    min_uniform_buffer_offset_alignment: u32,
  ) -> Result<(), RenderPassError> {
    rp_encoder.set_viewport(initial_viewport);

    for cmd in command_iter.by_ref() {
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
          let group_ref = bind_groups.get(group).ok_or_else(|| {
            RenderPassError::Validation(format!("Unknown bind group {group}"))
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
          let buffer_ref =
            pipeline_ref.buffers().get(buffer as usize).ok_or_else(|| {
              RenderPassError::Validation(format!(
                "Vertex buffer index {buffer} not found for pipeline {pipeline}"
              ))
            })?;
          rp_encoder.set_vertex_buffer(buffer, buffer_ref);
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
        RenderCommand::Immediates {
          pipeline,
          offset,
          bytes,
        } => {
          Self::validate_pipeline_exists(render_pipelines, pipeline)?;

          // Convert the u32 words to a byte slice for set_immediates.
          let byte_slice = unsafe {
            std::slice::from_raw_parts(
              bytes.as_ptr() as *const u8,
              bytes.len() * std::mem::size_of::<u32>(),
            )
          };
          rp_encoder.set_immediates(offset, byte_slice);
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
        RenderCommand::BeginRenderPass { .. }
        | RenderCommand::BeginRenderPassTo { .. } => {
          return Err(RenderPassError::Validation(
            "Nested render passes are not supported.".to_string(),
          ));
        }
      }
    }

    return Err(RenderPassError::Validation(
      "Render pass did not terminate with EndRenderPass".to_string(),
    ));
  }

  /// Reconfigure the presentation surface using current present mode/usage.
  fn reconfigure_surface(
    &mut self,
    size: (u32, u32),
  ) -> Result<(), RenderError> {
    let surface = self.surface.as_mut().ok_or_else(|| {
      RenderError::Configuration(
        "No surface attached to RenderContext".to_string(),
      )
    })?;

    surface
      .resize(&self.gpu, size)
      .map_err(RenderError::Configuration)?;

    let config = surface.configuration().ok_or_else(|| {
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
  Surface(targets::surface::SurfaceError),
  Configuration(String),
}

impl From<targets::surface::SurfaceError> for RenderError {
  fn from(error: targets::surface::SurfaceError) -> Self {
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

  /// Ensures the internal attachment predicate returns false when neither depth
  /// nor stencil operations are configured.
  #[test]
  fn has_depth_attachment_false_when_no_depth_or_stencil() {
    let has_attachment = RenderContext::has_depth_attachment(None, None);
    assert!(!has_attachment);
  }

  /// Ensures depth-only passes are recognized as having a depth/stencil
  /// attachment.
  #[test]
  fn has_depth_attachment_true_for_depth_only() {
    let depth_ops = Some(render_pass::DepthOperations::default());
    let has_attachment = RenderContext::has_depth_attachment(depth_ops, None);
    assert!(has_attachment);
  }

  /// Ensures stencil-only passes are recognized as having a depth/stencil
  /// attachment.
  #[test]
  fn has_depth_attachment_true_for_stencil_only() {
    let stencil_ops = Some(render_pass::StencilOperations::default());
    let has_attachment = RenderContext::has_depth_attachment(None, stencil_ops);
    assert!(has_attachment);
  }

  /// Ensures render context validation rejects references to missing pipelines
  /// with an actionable error.
  #[test]
  fn immediates_validate_pipeline_exists_rejects_unknown_pipeline() {
    let pipelines: Vec<RenderPipeline> = vec![];
    let err = RenderContext::validate_pipeline_exists(&pipelines, 7)
      .expect_err("must error");
    assert!(err.to_string().contains("Unknown pipeline 7"));
  }

  /// Exercises the core command encoding loop against a real device, covering
  /// common command variants (viewport/scissor/pipeline/bind group/draw).
  #[test]
  fn encode_active_render_pass_commands_executes_common_commands() {
    use lambda_platform::wgpu as platform;

    let instance = instance::InstanceBuilder::new()
      .with_label("lambda-render-mod-test-instance")
      .build();
    let Some(gpu) =
      gpu::create_test_gpu_with_instance(&instance, "lambda-render-mod-test")
    else {
      return;
    };

    let (vs, fs) = {
      let vert_path = format!(
        "{}/assets/shaders/triangle.vert",
        env!("CARGO_MANIFEST_DIR")
      );
      let frag_path = format!(
        "{}/assets/shaders/triangle.frag",
        env!("CARGO_MANIFEST_DIR")
      );
      let mut builder = shader::ShaderBuilder::new();
      let vs = builder.build(shader::VirtualShader::File {
        path: vert_path,
        kind: shader::ShaderKind::Vertex,
        name: "triangle-vert".to_string(),
        entry_point: "main".to_string(),
      });
      let fs = builder.build(shader::VirtualShader::File {
        path: frag_path,
        kind: shader::ShaderKind::Fragment,
        name: "triangle-frag".to_string(),
        entry_point: "main".to_string(),
      });
      (vs, fs)
    };

    let pass = render_pass::RenderPassBuilder::new()
      .with_label("lambda-mod-encode-pass")
      .build(
        &gpu,
        texture::TextureFormat::Rgba8Unorm,
        texture::DepthFormat::Depth24Plus,
      );

    let pipeline = pipeline::RenderPipelineBuilder::new()
      .with_label("lambda-mod-encode-pipeline")
      .build(
        &gpu,
        texture::TextureFormat::Rgba8Unorm,
        texture::DepthFormat::Depth24Plus,
        &pass,
        &vs,
        Some(&fs),
      );

    let uniform = buffer::BufferBuilder::new()
      .with_label("lambda-mod-uniform")
      .with_usage(buffer::Usage::UNIFORM)
      .with_properties(buffer::Properties::CPU_VISIBLE)
      .with_buffer_type(buffer::BufferType::Uniform)
      .build(&gpu, vec![0u32; 4])
      .expect("build uniform buffer");

    let layout = bind::BindGroupLayoutBuilder::new()
      .with_uniform(0, bind::BindingVisibility::VertexAndFragment)
      .build(&gpu);
    let group = bind::BindGroupBuilder::new()
      .with_label("lambda-mod-bind-group")
      .with_layout(&layout)
      .with_uniform(0, &uniform, 0, None)
      .build(&gpu);

    let index_buffer = buffer::BufferBuilder::new()
      .with_label("lambda-mod-index")
      .with_usage(buffer::Usage::INDEX)
      .with_properties(buffer::Properties::CPU_VISIBLE)
      .with_buffer_type(buffer::BufferType::Index)
      .build(&gpu, vec![0u16, 1u16, 2u16])
      .expect("build index buffer");

    let resolve =
      texture::TextureBuilder::new_2d(texture::TextureFormat::Rgba8Unorm)
        .with_size(4, 4)
        .for_render_target()
        .build(&gpu)
        .expect("build resolve texture");

    let mut platform_encoder = platform::command::CommandEncoder::new(
      gpu.platform(),
      Some("lambda-mod-command-encoder"),
    );

    let mut attachments =
      color_attachments::RenderColorAttachments::for_offscreen_pass(
        pass.uses_color(),
        pass.sample_count(),
        None,
        resolve.view_ref(),
      );

    let mut rp_encoder = encoder::new_render_pass_encoder_for_tests(
      &mut platform_encoder,
      &pass,
      encoder::RenderPassDestinationInfo {
        color_format: Some(texture::TextureFormat::Rgba8Unorm),
        depth_format: None,
      },
      &mut attachments,
      None,
    );

    let initial_viewport = viewport::Viewport {
      x: 0,
      y: 0,
      width: 4,
      height: 4,
      min_depth: 0.0,
      max_depth: 1.0,
    };

    let render_pipelines = vec![pipeline];
    let bind_groups = vec![group];
    let buffers = vec![Rc::new(index_buffer)];
    let min_align = gpu.limit_min_uniform_buffer_offset_alignment();

    let commands = vec![
      RenderCommand::SetViewports {
        start_at: 0,
        viewports: vec![initial_viewport.clone()],
      },
      RenderCommand::SetScissors {
        start_at: 0,
        viewports: vec![initial_viewport.clone()],
      },
      RenderCommand::SetPipeline { pipeline: 0 },
      RenderCommand::SetBindGroup {
        set: 0,
        group: 0,
        dynamic_offsets: vec![],
      },
      RenderCommand::BindIndexBuffer {
        buffer: 0,
        format: command::IndexFormat::Uint16,
      },
      RenderCommand::Draw {
        vertices: 0..3,
        instances: 0..1,
      },
      RenderCommand::DrawIndexed {
        indices: 0..3,
        base_vertex: 0,
        instances: 0..1,
      },
      RenderCommand::EndRenderPass,
    ];

    let mut iter = commands.into_iter();
    RenderContext::encode_active_render_pass_commands(
      &mut iter,
      &mut rp_encoder,
      &initial_viewport,
      &render_pipelines,
      &bind_groups,
      &buffers,
      min_align,
    )
    .expect("encode commands");

    drop(rp_encoder);
    let buffer = platform_encoder.finish();
    gpu.submit(std::iter::once(buffer));
  }

  /// Ensures the command encoding loop rejects frames that omit an
  /// `EndRenderPass` terminator.
  #[test]
  fn encode_active_render_pass_commands_requires_end_render_pass() {
    use lambda_platform::wgpu as platform;

    let instance = instance::InstanceBuilder::new()
      .with_label("lambda-render-mod-test-instance-2")
      .build();
    let Some(gpu) =
      gpu::create_test_gpu_with_instance(&instance, "lambda-render-mod-test-2")
    else {
      return;
    };

    let pass = render_pass::RenderPassBuilder::new()
      .with_label("lambda-mod-missing-end-pass")
      .build(
        &gpu,
        texture::TextureFormat::Rgba8Unorm,
        texture::DepthFormat::Depth24Plus,
      );

    let resolve =
      texture::TextureBuilder::new_2d(texture::TextureFormat::Rgba8Unorm)
        .with_size(1, 1)
        .for_render_target()
        .build(&gpu)
        .expect("build resolve texture");

    let mut platform_encoder = platform::command::CommandEncoder::new(
      gpu.platform(),
      Some("lambda-mod-missing-end-encoder"),
    );

    let mut attachments =
      color_attachments::RenderColorAttachments::for_offscreen_pass(
        pass.uses_color(),
        pass.sample_count(),
        None,
        resolve.view_ref(),
      );

    let mut rp_encoder = encoder::new_render_pass_encoder_for_tests(
      &mut platform_encoder,
      &pass,
      encoder::RenderPassDestinationInfo {
        color_format: Some(texture::TextureFormat::Rgba8Unorm),
        depth_format: None,
      },
      &mut attachments,
      None,
    );

    let initial_viewport = viewport::Viewport {
      x: 0,
      y: 0,
      width: 1,
      height: 1,
      min_depth: 0.0,
      max_depth: 1.0,
    };

    let mut iter =
      vec![RenderCommand::SetStencilReference { reference: 1 }].into_iter();
    let err = RenderContext::encode_active_render_pass_commands(
      &mut iter,
      &mut rp_encoder,
      &initial_viewport,
      &[],
      &[],
      &[],
      gpu.limit_min_uniform_buffer_offset_alignment(),
    )
    .expect_err("must require EndRenderPass");

    assert!(err.to_string().contains("EndRenderPass"));
  }

  /// End-to-end GPU test that renders into both "surface-style" and offscreen
  /// passes without requiring an actual window/surface.
  #[test]
  fn render_context_builder_renders_surface_and_offscreen_passes() {
    use std::num::NonZeroU64;

    use crate::render::{
      bind::{
        BindGroupBuilder,
        BindGroupLayoutBuilder,
        BindingVisibility,
      },
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
      encoder::CommandEncoder,
      instance::InstanceBuilder,
      pipeline::RenderPipelineBuilder,
      shader::{
        ShaderBuilder,
        ShaderKind,
        VirtualShader,
      },
      targets::offscreen::OffscreenTargetBuilder,
      texture::{
        DepthFormat,
        DepthTextureBuilder,
        TextureBuilder,
        TextureFormat,
        TextureUsages,
      },
      vertex::{
        ColorFormat,
        VertexAttribute,
        VertexElement,
      },
      viewport::ViewportBuilder,
    };

    fn compile_shaders(
    ) -> (crate::render::shader::Shader, crate::render::shader::Shader) {
      let vs_source = r#"
        #version 450
        #extension GL_ARB_separate_shader_objects : enable

        layout(location = 0) in vec3 a_pos;

        layout(push_constant) uniform Immediates {
          vec4 v;
        } imms;

        void main() {
          // Reference immediates to keep push constants alive.
          gl_Position = vec4(a_pos, 1.0) + imms.v * 0.0;
        }
      "#;

      let fs_source = r#"
        #version 450
        #extension GL_ARB_separate_shader_objects : enable

        layout(set = 0, binding = 0) uniform ColorData {
          vec4 color;
        } u_color;

        layout(location = 0) out vec4 fragment_color;

        void main() {
          fragment_color = u_color.color;
        }
      "#;

      let mut builder = ShaderBuilder::new();
      let vs = builder.build(VirtualShader::Source {
        source: vs_source.to_string(),
        kind: ShaderKind::Vertex,
        name: "lambda-e2e-vert".to_string(),
        entry_point: "main".to_string(),
      });
      let fs = builder.build(VirtualShader::Source {
        source: fs_source.to_string(),
        kind: ShaderKind::Fragment,
        name: "lambda-e2e-frag".to_string(),
        entry_point: "main".to_string(),
      });
      return (vs, fs);
    }

    let instance = InstanceBuilder::new()
      .with_label("lambda-render-context-e2e-instance")
      .build();
    let Some(gpu) = gpu::create_test_gpu_with_instance(
      &instance,
      "lambda-render-context-e2e",
    ) else {
      return;
    };

    let config = targets::surface::SurfaceConfig {
      width: 64,
      height: 64,
      format: TextureFormat::Rgba8Unorm,
      present_mode: targets::surface::PresentMode::Fifo,
      usage: TextureUsages::RENDER_ATTACHMENT,
    };

    let depth_texture = DepthTextureBuilder::new()
      .with_size(64, 64)
      .with_format(DepthFormat::Depth32Float)
      .with_label("lambda-depth")
      .build(&gpu);

    let mut render_context = RenderContext {
      label: "lambda-render-context-e2e".to_string(),
      instance,
      surface: None,
      gpu,
      config: config.clone(),
      texture_usage: config.usage,
      size: (64, 64),
      depth_texture: Some(depth_texture),
      depth_format: DepthFormat::Depth32Float,
      depth_sample_count: 1,
      msaa_color: None,
      msaa_sample_count: 1,
      offscreen_targets: vec![],
      render_passes: vec![],
      render_pipelines: vec![],
      bind_group_layouts: vec![],
      bind_groups: vec![],
      buffers: vec![],
      seen_error_messages: Default::default(),
    };

    assert_eq!(render_context.label(), "lambda-render-context-e2e");
    assert_eq!(render_context.surface_size(), (64, 64));
    assert_eq!(render_context.surface_format(), TextureFormat::Rgba8Unorm);

    let msaa_samples = [4_u32, 2, 1]
      .into_iter()
      .find(|&count| {
        render_context.gpu().supports_sample_count_for_format(
          render_context.surface_format(),
          count,
        ) && render_context
          .gpu()
          .supports_sample_count_for_depth(DepthFormat::Depth32Float, count)
      })
      .unwrap_or(1);

    // Build an offscreen destination matching the headless surface config.
    let offscreen = OffscreenTargetBuilder::new()
      .with_label("lambda-e2e-offscreen")
      .with_color(TextureFormat::Rgba8Unorm, 64, 64)
      .with_depth(DepthFormat::Depth24PlusStencil8)
      .with_multi_sample(1)
      .build(render_context.gpu())
      .expect("build offscreen target");
    let offscreen_id = render_context.attach_offscreen_target(offscreen);

    // Exercise error path for unknown ids.
    assert!(render_context
      .replace_offscreen_target(
        999,
        OffscreenTargetBuilder::new()
          .with_color(TextureFormat::Rgba8Unorm, 1, 1)
          .build(render_context.gpu())
          .expect("build replacement target"),
      )
      .is_err());

    // Create a pass that requests depth + stencil (single-sample for offscreen compatibility).
    let supported_samples = 1_u32;
    let pass = render_pass::RenderPassBuilder::new()
      .with_label("lambda-e2e-pass")
      .with_multi_sample(supported_samples)
      .with_depth()
      .with_stencil()
      .build(
        render_context.gpu(),
        render_context.surface_format(),
        DepthFormat::Depth24PlusStencil8,
      );
    let pass_id = render_context.attach_render_pass(pass);

    // One dynamic uniform at set=0,binding=0.
    let layout = BindGroupLayoutBuilder::new()
      .with_label("lambda-e2e-bgl")
      .with_uniform_dynamic(0, BindingVisibility::Fragment)
      .build(render_context.gpu());
    let layout_id = render_context.attach_bind_group_layout(layout.clone());
    assert_eq!(layout_id, 0);

    let min_alignment =
      render_context.limit_min_uniform_buffer_offset_alignment() as usize;
    let ubo_byte_len = (min_alignment * 2).max(256);
    let ubo_u32_len = ubo_byte_len / std::mem::size_of::<u32>();
    let uniform = BufferBuilder::new()
      .with_label("lambda-e2e-uniform")
      .with_usage(Usage::UNIFORM)
      .with_properties(Properties::CPU_VISIBLE)
      .with_buffer_type(BufferType::Uniform)
      .build(render_context.gpu(), vec![0_u32; ubo_u32_len])
      .expect("build uniform buffer");

    let group = BindGroupBuilder::new()
      .with_label("lambda-e2e-bg")
      .with_layout(&layout)
      .with_uniform(0, &uniform, 0, Some(NonZeroU64::new(16).unwrap()))
      .build(render_context.gpu());
    let group_id = render_context.attach_bind_group(group.clone());
    assert_eq!(group_id, 0);

    assert!(render_context.replace_bind_group(999, group).is_err());

    // Vertex + index buffers for a simple triangle.
    let vertices: Vec<[f32; 3]> =
      vec![[0.0, -0.5, 0.0], [-0.5, 0.5, 0.0], [0.5, 0.5, 0.0]];
    let vertex_buffer = BufferBuilder::new()
      .with_label("lambda-e2e-vertex")
      .with_usage(Usage::VERTEX)
      .with_properties(Properties::CPU_VISIBLE)
      .with_buffer_type(BufferType::Vertex)
      .build(render_context.gpu(), vertices)
      .expect("build vertex buffer");

    let vertices_msaa: Vec<[f32; 3]> =
      vec![[0.0, -0.5, 0.0], [-0.5, 0.5, 0.0], [0.5, 0.5, 0.0]];
    let vertex_buffer_msaa = BufferBuilder::new()
      .with_label("lambda-e2e-vertex-msaa")
      .with_usage(Usage::VERTEX)
      .with_properties(Properties::CPU_VISIBLE)
      .with_buffer_type(BufferType::Vertex)
      .build(render_context.gpu(), vertices_msaa)
      .expect("build msaa vertex buffer");

    let indices: Vec<u16> = vec![0, 1, 2];
    let index_buffer = BufferBuilder::new()
      .with_label("lambda-e2e-index")
      .with_usage(Usage::INDEX)
      .with_properties(Properties::CPU_VISIBLE)
      .with_buffer_type(BufferType::Index)
      .build(render_context.gpu(), indices)
      .expect("build index buffer");
    let index_id = render_context.attach_buffer(index_buffer);

    let (vs, fs) = compile_shaders();

    let attributes = vec![VertexAttribute {
      location: 0,
      offset: 0,
      element: VertexElement {
        format: ColorFormat::Rgb32Sfloat,
        offset: 0,
      },
    }];

    let pipeline = RenderPipelineBuilder::new()
      .with_label("lambda-e2e-pipeline")
      .with_layouts(&[&layout])
      .with_immediate_data(16)
      .with_buffer(vertex_buffer, attributes.clone())
      .with_multi_sample(supported_samples)
      .with_depth_format(DepthFormat::Depth24PlusStencil8)
      .build(
        render_context.gpu(),
        render_context.surface_format(),
        DepthFormat::Depth24PlusStencil8,
        render_context.get_render_pass(pass_id),
        &vs,
        Some(&fs),
      );
    let pipeline_id = render_context.attach_pipeline(pipeline);

    let viewport = ViewportBuilder::new().build(64, 64);
    let viewport_small = ViewportBuilder::new().build(16, 16);
    let viewport_offset =
      ViewportBuilder::new().with_coordinates(8, 8).build(8, 8);

    // Exercise encoding for a "surface" pass using an offscreen texture view.
    let resolve = TextureBuilder::new_2d(TextureFormat::Rgba8Unorm)
      .with_size(64, 64)
      .for_render_target()
      .build(render_context.gpu())
      .expect("build resolve texture");
    let surface_view = resolve.view_ref();

    let dynamic_offset = min_alignment as u32;
    let mut encoder =
      CommandEncoder::new(&render_context, "lambda-e2e-encoder");

    let surface_pass_commands = vec![
      RenderCommand::SetViewports {
        start_at: 0,
        viewports: vec![viewport_small.clone(), viewport_offset.clone()],
      },
      RenderCommand::SetScissors {
        start_at: 0,
        viewports: vec![viewport_small.clone(), viewport_offset.clone()],
      },
      RenderCommand::SetStencilReference { reference: 1 },
      RenderCommand::SetPipeline {
        pipeline: pipeline_id,
      },
      RenderCommand::SetBindGroup {
        set: 0,
        group: group_id,
        dynamic_offsets: vec![dynamic_offset],
      },
      RenderCommand::BindVertexBuffer {
        pipeline: pipeline_id,
        buffer: 0,
      },
      RenderCommand::BindIndexBuffer {
        buffer: index_id,
        format: IndexFormat::Uint16,
      },
      RenderCommand::Immediates {
        pipeline: pipeline_id,
        offset: 0,
        bytes: vec![0_u32; 4],
      },
      RenderCommand::DrawIndexed {
        indices: 0..3,
        base_vertex: 0,
        instances: 0..1,
      },
      RenderCommand::EndRenderPass,
    ];
    let mut surface_pass_iter = surface_pass_commands.into_iter();
    render_context
      .encode_surface_render_pass(
        &mut encoder,
        &mut surface_pass_iter,
        pass_id,
        viewport.clone(),
        surface_view,
      )
      .expect("encode headless surface pass");

    // Encode an offscreen pass as well.
    let offscreen_commands = vec![
      RenderCommand::SetPipeline {
        pipeline: pipeline_id,
      },
      RenderCommand::SetBindGroup {
        set: 0,
        group: group_id,
        dynamic_offsets: vec![0],
      },
      RenderCommand::BindVertexBuffer {
        pipeline: pipeline_id,
        buffer: 0,
      },
      RenderCommand::BindIndexBuffer {
        buffer: index_id,
        format: IndexFormat::Uint16,
      },
      RenderCommand::DrawIndexed {
        indices: 0..3,
        base_vertex: 0,
        instances: 0..1,
      },
      RenderCommand::EndRenderPass,
    ];
    let mut offscreen_iter = offscreen_commands.into_iter();
    render_context
      .encode_offscreen_render_pass(
        &mut encoder,
        &mut offscreen_iter,
        pass_id,
        viewport.clone(),
        offscreen_id,
      )
      .expect("encode offscreen pass");

    if msaa_samples > 1 {
      let pass_msaa = render_pass::RenderPassBuilder::new()
        .with_label("lambda-e2e-pass-msaa")
        .with_multi_sample(msaa_samples)
        .with_depth()
        .with_stencil()
        .build(
          render_context.gpu(),
          render_context.surface_format(),
          DepthFormat::Depth24PlusStencil8,
        );
      let pass_msaa_id = render_context.attach_render_pass(pass_msaa);

      let pipeline_msaa = RenderPipelineBuilder::new()
        .with_label("lambda-e2e-pipeline-msaa")
        .with_layouts(&[&layout])
        .with_immediate_data(16)
        .with_buffer(vertex_buffer_msaa, attributes)
        .with_multi_sample(msaa_samples)
        .with_depth_format(DepthFormat::Depth24PlusStencil8)
        .build(
          render_context.gpu(),
          render_context.surface_format(),
          DepthFormat::Depth24PlusStencil8,
          render_context.get_render_pass(pass_msaa_id),
          &vs,
          Some(&fs),
        );
      let pipeline_msaa_id = render_context.attach_pipeline(pipeline_msaa);

      let msaa_pass_commands = vec![
        RenderCommand::SetPipeline {
          pipeline: pipeline_msaa_id,
        },
        RenderCommand::SetBindGroup {
          set: 0,
          group: group_id,
          dynamic_offsets: vec![0],
        },
        RenderCommand::BindVertexBuffer {
          pipeline: pipeline_msaa_id,
          buffer: 0,
        },
        RenderCommand::BindIndexBuffer {
          buffer: index_id,
          format: IndexFormat::Uint16,
        },
        RenderCommand::DrawIndexed {
          indices: 0..3,
          base_vertex: 0,
          instances: 0..1,
        },
        RenderCommand::EndRenderPass,
      ];
      let mut msaa_iter = msaa_pass_commands.into_iter();
      render_context
        .encode_surface_render_pass(
          &mut encoder,
          &mut msaa_iter,
          pass_msaa_id,
          viewport.clone(),
          surface_view,
        )
        .expect("encode msaa surface pass");
    }

    encoder.finish(&render_context);

    // Cover headless `render_internal` (offscreen-only) as well.
    render_context
      .render_internal(vec![
        RenderCommand::SetPipeline {
          pipeline: pipeline_id,
        },
        RenderCommand::BeginRenderPassTo {
          render_pass: pass_id,
          viewport: viewport.clone(),
          destination: command::RenderDestination::Offscreen(offscreen_id),
        },
        RenderCommand::SetPipeline {
          pipeline: pipeline_id,
        },
        RenderCommand::SetBindGroup {
          set: 0,
          group: group_id,
          dynamic_offsets: vec![0],
        },
        RenderCommand::BindVertexBuffer {
          pipeline: pipeline_id,
          buffer: 0,
        },
        RenderCommand::BindIndexBuffer {
          buffer: index_id,
          format: IndexFormat::Uint16,
        },
        RenderCommand::DrawIndexed {
          indices: 0..3,
          base_vertex: 0,
          instances: 0..1,
        },
        RenderCommand::EndRenderPass,
      ])
      .expect("headless render_internal should support offscreen passes");

    let err = render_context
      .render_internal(vec![RenderCommand::BeginRenderPassTo {
        render_pass: pass_id,
        viewport: viewport.clone(),
        destination: command::RenderDestination::Surface,
      }])
      .expect_err("surface passes require an attached surface");
    assert!(matches!(
      err,
      RenderError::Configuration(msg) if msg.contains("No surface")
    ));

    // Cover offscreen configuration error paths.
    let mismatch_samples = [4_u32, 2]
      .into_iter()
      .find(|&count| {
        render_context
          .gpu()
          .supports_sample_count_for_format(TextureFormat::Rgba8Unorm, count)
      })
      .unwrap_or(1);
    if mismatch_samples != 1 {
      let mismatch_pass = render_pass::RenderPassBuilder::new()
        .with_label("lambda-e2e-mismatch-pass")
        .with_multi_sample(mismatch_samples)
        .build(
          render_context.gpu(),
          TextureFormat::Rgba8Unorm,
          DepthFormat::Depth24Plus,
        );
      let mismatch_pass_id = render_context.attach_render_pass(mismatch_pass);
      let mut mismatch_iter = vec![RenderCommand::EndRenderPass].into_iter();
      let mut mismatch_encoder =
        CommandEncoder::new(&render_context, "lambda-e2e-mismatch-encoder");
      let mismatch_err = render_context
        .encode_offscreen_render_pass(
          &mut mismatch_encoder,
          &mut mismatch_iter,
          mismatch_pass_id,
          viewport.clone(),
          offscreen_id,
        )
        .expect_err("mismatched pass/target sample counts must error");
      assert!(matches!(
        mismatch_err,
        RenderError::Configuration(msg) if msg.contains("sample_count")
      ));
    }

    let target_no_depth = OffscreenTargetBuilder::new()
      .with_label("lambda-e2e-offscreen-no-depth")
      .with_color(TextureFormat::Rgba8Unorm, 8, 8)
      .build(render_context.gpu())
      .expect("build offscreen target without depth");
    let target_no_depth_id =
      render_context.attach_offscreen_target(target_no_depth);
    let mut no_depth_iter = vec![RenderCommand::EndRenderPass].into_iter();
    let mut no_depth_encoder =
      CommandEncoder::new(&render_context, "lambda-e2e-no-depth-encoder");
    let no_depth_err = render_context
      .encode_offscreen_render_pass(
        &mut no_depth_encoder,
        &mut no_depth_iter,
        pass_id,
        viewport.clone(),
        target_no_depth_id,
      )
      .expect_err(
        "pass with depth/stencil must require a target depth attachment",
      );
    assert!(matches!(
      no_depth_err,
      RenderError::Configuration(msg) if msg.contains("no depth attachment")
    ));

    let target_no_stencil = OffscreenTargetBuilder::new()
      .with_label("lambda-e2e-offscreen-no-stencil")
      .with_color(TextureFormat::Rgba8Unorm, 8, 8)
      .with_depth(DepthFormat::Depth24Plus)
      .build(render_context.gpu())
      .expect("build offscreen target without stencil");
    let target_no_stencil_id =
      render_context.attach_offscreen_target(target_no_stencil);
    let stencil_pass = render_pass::RenderPassBuilder::new()
      .with_label("lambda-e2e-stencil-pass")
      .with_stencil()
      .build(
        render_context.gpu(),
        TextureFormat::Rgba8Unorm,
        DepthFormat::Depth24Plus,
      );
    let stencil_pass_id = render_context.attach_render_pass(stencil_pass);
    let mut stencil_iter = vec![RenderCommand::EndRenderPass].into_iter();
    let mut stencil_encoder =
      CommandEncoder::new(&render_context, "lambda-e2e-stencil-encoder");
    let stencil_err = render_context
      .encode_offscreen_render_pass(
        &mut stencil_encoder,
        &mut stencil_iter,
        stencil_pass_id,
        viewport.clone(),
        target_no_stencil_id,
      )
      .expect_err("stencil pass must require stencil-capable depth format");
    assert!(matches!(
      stencil_err,
      RenderError::Configuration(msg) if msg.contains("stencil")
    ));

    // Resize exercises headless depth/MSAA rebuild paths without touching a surface.
    render_context.resize(32, 32);
    assert_eq!(render_context.surface_size(), (32, 32));
  }
}
