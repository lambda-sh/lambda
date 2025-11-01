//! High level Rendering API designed for cross platform rendering and
//! windowing.

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

use std::iter;

use lambda_platform::wgpu::{
  types as wgpu,
  Gpu,
  GpuBuilder,
  Instance,
  InstanceBuilder,
  Surface,
  SurfaceBuilder,
};
use logging;
pub use vertex::ColorFormat;

use self::{
  command::RenderCommand,
  pipeline::RenderPipeline,
  render_pass::RenderPass,
};

/// Builder for configuring a `RenderContext` tied to a single window.
///
/// The builder wires up a `wgpu::Instance`, `Surface`, and `Gpu` using the
/// cross‑platform platform layer, then configures the surface with reasonable
/// defaults. Use this when setting up rendering for an application window.
pub struct RenderContextBuilder {
  name: String,
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
  pub fn build(self, window: &window::Window) -> RenderContext {
    let RenderContextBuilder { name, .. } = self;

    let instance = InstanceBuilder::new()
      .with_label(&format!("{} Instance", name))
      .build();

    let mut surface = SurfaceBuilder::new()
      .with_label(&format!("{} Surface", name))
      .build(&instance, window.window_handle())
      .expect("Failed to create rendering surface");

    let gpu = GpuBuilder::new()
      .with_label(&format!("{} Device", name))
      .build(&instance, Some(&surface))
      .expect("Failed to create GPU device");

    let size = window.dimensions();
    let config = surface
      .configure_with_defaults(
        gpu.adapter(),
        gpu.device(),
        size,
        wgpu::PresentMode::Fifo,
        wgpu::TextureUsages::RENDER_ATTACHMENT,
      )
      .expect("Failed to configure surface");

    let depth = Some(
      lambda_platform::wgpu::texture::DepthTextureBuilder::new()
        .with_label("lambda-depth")
        .with_size(size.0, size.1)
        .with_format(lambda_platform::wgpu::texture::DepthFormat::Depth32Float)
        .build(gpu.device()),
    );

    return RenderContext {
      label: name,
      instance,
      surface,
      gpu,
      config,
      present_mode: wgpu::PresentMode::Fifo,
      texture_usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
      size,
      render_passes: vec![],
      render_pipelines: vec![],
      bind_group_layouts: vec![],
      bind_groups: vec![],
      depth,
    };
  }
}

/// High‑level rendering context backed by `wgpu` for a single window.
///
/// The context owns the `Instance`, presentation `Surface`, and `Gpu` device
/// objects and maintains a set of attached render passes and pipelines used
/// while encoding command streams for each frame.
pub struct RenderContext {
  label: String,
  instance: Instance,
  surface: Surface<'static>,
  gpu: Gpu,
  config: wgpu::SurfaceConfiguration,
  present_mode: wgpu::PresentMode,
  texture_usage: wgpu::TextureUsages,
  size: (u32, u32),
  render_passes: Vec<RenderPass>,
  render_pipelines: Vec<RenderPipeline>,
  bind_group_layouts: Vec<bind::BindGroupLayout>,
  bind_groups: Vec<bind::BindGroup>,
  depth: Option<lambda_platform::wgpu::texture::DepthTexture>,
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
  pub fn attach_render_pass(&mut self, render_pass: RenderPass) -> ResourceId {
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

  /// Explicitly destroy the context. Dropping also releases resources.
  pub fn destroy(self) {
    drop(self);
  }

  /// Render a list of commands. No‑ops when the list is empty.
  ///
  /// Errors are logged and do not panic; see `RenderError` for cases.
  pub fn render(&mut self, commands: Vec<RenderCommand>) {
    if commands.is_empty() {
      return;
    }

    if let Err(err) = self.render_internal(commands) {
      logging::error!("Render error: {:?}", err);
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
  }

  /// Borrow a previously attached render pass by id.
  pub fn get_render_pass(&self, id: ResourceId) -> &RenderPass {
    return &self.render_passes[id];
  }

  /// Borrow a previously attached render pipeline by id.
  pub fn get_render_pipeline(&self, id: ResourceId) -> &RenderPipeline {
    return &self.render_pipelines[id];
  }

  pub(crate) fn device(&self) -> &wgpu::Device {
    return self.gpu.device();
  }

  pub(crate) fn queue(&self) -> &wgpu::Queue {
    return self.gpu.queue();
  }

  pub(crate) fn surface_format(&self) -> wgpu::TextureFormat {
    return self.config.format;
  }

  /// Depth format used for pipelines and attachments.
  pub(crate) fn depth_format(&self) -> wgpu::TextureFormat {
    return wgpu::TextureFormat::Depth32Float;
  }

  /// Device limit: maximum bytes that can be bound for a single uniform buffer binding.
  pub fn limit_max_uniform_buffer_binding_size(&self) -> u64 {
    return self.gpu.limits().max_uniform_buffer_binding_size.into();
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
      Err(wgpu::SurfaceError::Lost) | Err(wgpu::SurfaceError::Outdated) => {
        self.reconfigure_surface(self.size)?;
        self
          .surface
          .acquire_next_frame()
          .map_err(RenderError::Surface)?
      }
      Err(err) => return Err(RenderError::Surface(err)),
    };

    let view = frame.texture_view();
    let mut encoder =
      self
        .device()
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
          label: Some("lambda-render-command-encoder"),
        });

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

          let color_attachment = wgpu::RenderPassColorAttachment {
            view,
            depth_slice: None,
            resolve_target: None,
            ops: pass.color_ops(),
          };
          let color_attachments = [Some(color_attachment)];
          let depth_attachment = self.depth.as_ref().map(|d| {
            wgpu::RenderPassDepthStencilAttachment {
              view: d.view(),
              depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(1.0),
                store: wgpu::StoreOp::Store,
              }),
              stencil_ops: None,
            }
          });

          let mut pass_encoder =
            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
              label: pass.label(),
              color_attachments: &color_attachments,
              depth_stencil_attachment: depth_attachment,
              timestamp_writes: None,
              occlusion_query_set: None,
            });

          self.encode_pass(&mut pass_encoder, viewport, &mut command_iter)?;
        }
        other => {
          logging::warn!(
            "Ignoring render command outside of a render pass: {:?}",
            other
          );
        }
      }
    }

    self.queue().submit(iter::once(encoder.finish()));
    frame.present();
    return Ok(());
  }

  /// Encode a single render pass and consume commands until `EndRenderPass`.
  fn encode_pass<I>(
    &mut self,
    pass: &mut wgpu::RenderPass<'_>,
    initial_viewport: viewport::Viewport,
    commands: &mut I,
  ) -> Result<(), RenderError>
  where
    I: Iterator<Item = RenderCommand>,
  {
    Self::apply_viewport(pass, &initial_viewport);

    while let Some(command) = commands.next() {
      match command {
        RenderCommand::EndRenderPass => return Ok(()),
        RenderCommand::SetPipeline { pipeline } => {
          let pipeline_ref =
            self.render_pipelines.get(pipeline).ok_or_else(|| {
              return RenderError::Configuration(format!(
                "Unknown pipeline {pipeline}"
              ));
            })?;
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
          pass.set_bind_group(set, group_ref.raw(), &dynamic_offsets);
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

          pass.set_vertex_buffer(buffer as u32, buffer_ref.raw().slice(..));
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
          pass.set_push_constants(stage.to_wgpu(), offset, slice);
        }
        RenderCommand::Draw { vertices } => {
          pass.draw(vertices, 0..1);
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
    pass: &mut wgpu::RenderPass<'_>,
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
    let config = self
      .surface
      .configure_with_defaults(
        self.gpu.adapter(),
        self.gpu.device(),
        size,
        self.present_mode,
        self.texture_usage,
      )
      .map_err(RenderError::Configuration)?;

    self.present_mode = config.present_mode;
    self.texture_usage = config.usage;
    self.config = config;
    // Recreate depth attachment with the new surface size
    self.depth = Some(
      lambda_platform::wgpu::texture::DepthTextureBuilder::new()
        .with_label("lambda-depth")
        .with_size(size.0, size.1)
        .with_format(lambda_platform::wgpu::texture::DepthFormat::Depth32Float)
        .build(self.gpu.device()),
    );
    return Ok(());
  }
}

#[derive(Debug)]
/// Errors that can occur while preparing or presenting a frame.
pub enum RenderError {
  Surface(wgpu::SurfaceError),
  Configuration(String),
}

impl From<wgpu::SurfaceError> for RenderError {
  fn from(error: wgpu::SurfaceError) -> Self {
    return RenderError::Surface(error);
  }
}
