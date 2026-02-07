//! High-level command encoding for GPU work submission.
//!
//! This module provides `CommandEncoder` for recording GPU commands and
//! `RenderPassEncoder` for recording render pass commands. These types wrap
//! the platform layer and provide validation, high-level type safety, and a
//! clean API for the engine.
//!
//! Command encoders are created per-frame and not reused across frames, which
//! matches wgpu best practices and avoids stale state issues.
//!
//! # Usage
//!
//! The encoder uses a callback-based API for render passes to ensure proper
//! lifetime management (wgpu's `RenderPass` borrows from the encoder):
//!
//! ```ignore
//! let mut encoder = CommandEncoder::new(&render_context, "frame-encoder");
//! encoder.with_render_pass(
//!   &pass,
//!   RenderPassDestinationInfo { color_format: None, depth_format: None },
//!   &mut attachments,
//!   depth,
//!   |rp_encoder| {
//!   rp_encoder.set_pipeline(&pipeline)?;
//!   rp_encoder.draw(0..3, 0..1)?;
//!   Ok(())
//! })?;
//! encoder.finish(&render_context);
//! ```

use std::{
  collections::HashSet,
  ops::Range,
};

use lambda_platform::wgpu as platform;
use logging;

use super::{
  bind::BindGroup,
  buffer::{
    Buffer,
    BufferType,
  },
  color_attachments::RenderColorAttachments,
  command::IndexFormat,
  pipeline::RenderPipeline,
  render_pass::RenderPass,
  texture::{
    DepthFormat,
    DepthTexture,
    TextureFormat,
  },
  validation,
  viewport::Viewport,
  RenderContext,
};
use crate::util;

// ---------------------------------------------------------------------------
// CommandEncoder
// ---------------------------------------------------------------------------

/// Destination metadata needed for render-target compatibility validation.
#[derive(Clone, Copy, Debug)]
pub(crate) struct RenderPassDestinationInfo {
  pub(crate) color_format: Option<TextureFormat>,
  pub(crate) depth_format: Option<DepthFormat>,
}

/// High-level command encoder for recording GPU work.
///
/// Created per-frame via `CommandEncoder::new()`. Commands are recorded by
/// beginning render passes (and in the future, compute passes and copy ops).
/// Call `finish()` to submit the recorded work.
///
/// The encoder owns the underlying platform encoder and manages its lifetime.
pub struct CommandEncoder {
  inner: platform::command::CommandEncoder,
}

impl CommandEncoder {
  /// Create a new command encoder for recording GPU work.
  ///
  /// The encoder is tied to the current frame and should not be reused across
  /// frames.
  pub fn new(render_context: &RenderContext, label: &str) -> Self {
    let inner = platform::command::CommandEncoder::new(
      render_context.gpu().platform(),
      Some(label),
    );
    return CommandEncoder { inner };
  }

  /// Execute a render pass with the provided configuration.
  ///
  /// This method begins a render pass, executes the provided closure with a
  /// `RenderPassEncoder`, and automatically ends the pass when the closure
  /// returns. This ensures proper resource cleanup and lifetime management.
  ///
  /// # Arguments
  /// * `pass` - The high-level render pass configuration.
  /// * `color_attachments` - Color attachment views for the pass.
  /// * `depth_texture` - Optional depth texture for depth/stencil operations.
  /// * `func` - Closure that records commands to the render pass encoder.
  ///
  /// # Type Parameters
  /// * `'pass` - Lifetime of resources borrowed during the render pass.
  /// * `PassFn` - The closure type that records commands to the pass.
  /// * `Output` - The return type of the closure.
  ///
  /// # Returns
  /// The result of the closure, or any render pass error encountered.
  pub(crate) fn with_render_pass<'pass, PassFn, Output>(
    &'pass mut self,
    pass: &'pass RenderPass,
    destination_info: RenderPassDestinationInfo,
    color_attachments: &'pass mut RenderColorAttachments<'pass>,
    depth_texture: Option<&'pass DepthTexture>,
    func: PassFn,
  ) -> Result<Output, RenderPassError>
  where
    PassFn:
      FnOnce(&mut RenderPassEncoder<'_>) -> Result<Output, RenderPassError>,
  {
    let pass_encoder = RenderPassEncoder::new(
      &mut self.inner,
      pass,
      destination_info,
      color_attachments,
      depth_texture,
    );

    return func(&mut { pass_encoder });
  }

  /// Finish recording and submit the command buffer to the GPU.
  ///
  /// This consumes the encoder and submits the recorded commands for
  /// execution.
  pub fn finish(self, render_context: &RenderContext) {
    let buffer = self.inner.finish();
    render_context
      .gpu()
      .platform()
      .submit(std::iter::once(buffer));
  }
}

impl std::fmt::Debug for CommandEncoder {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    return f.debug_struct("CommandEncoder").finish_non_exhaustive();
  }
}

// ---------------------------------------------------------------------------
// RenderPassEncoder
// ---------------------------------------------------------------------------

/// High-level render pass encoder for recording render commands.
///
/// Created by `CommandEncoder::with_render_pass()`. Records draw commands,
/// pipeline bindings, and resource bindings within the closure scope.
///
/// The encoder borrows the command encoder for the duration of the pass and
/// performs validation on all operations.
///
/// # Type Parameters
/// * `'pass` - The lifetime of the render pass, tied to the borrowed encoder
///   and attachments.
pub struct RenderPassEncoder<'pass> {
  /// Platform render pass for issuing GPU commands.
  pass: platform::render_pass::RenderPass<'pass>,
  /// Whether the pass uses color attachments.
  uses_color: bool,
  /// Whether the pass has a depth attachment.
  has_depth_attachment: bool,
  /// Whether the pass has stencil operations.
  has_stencil: bool,
  /// Sample count for MSAA validation.
  sample_count: u32,
  /// Destination color format when the pass has color output.
  destination_color_format: Option<TextureFormat>,
  /// Destination depth format when a depth attachment is present.
  destination_depth_format: Option<DepthFormat>,

  // Validation state (compiled out in release without features)
  #[cfg(any(debug_assertions, feature = "render-validation-encoder"))]
  current_pipeline: Option<CurrentPipeline>,
  #[cfg(any(debug_assertions, feature = "render-validation-encoder"))]
  bound_index_buffer: Option<BoundIndexBuffer>,
  #[cfg(any(debug_assertions, feature = "render-validation-instancing"))]
  bound_vertex_slots: HashSet<u32>,
  #[cfg(any(
    debug_assertions,
    feature = "render-validation-depth",
    feature = "render-validation-stencil"
  ))]
  warned_no_stencil_for_pipeline: HashSet<usize>,
  #[cfg(any(
    debug_assertions,
    feature = "render-validation-depth",
    feature = "render-validation-stencil"
  ))]
  warned_no_depth_for_pipeline: HashSet<usize>,
}

/// Tracks the currently bound pipeline for validation.
#[cfg(any(debug_assertions, feature = "render-validation-encoder"))]
#[derive(Clone)]
struct CurrentPipeline {
  label: String,
  per_instance_slots: Vec<bool>,
}

/// Tracks the currently bound index buffer for validation.
#[cfg(any(debug_assertions, feature = "render-validation-encoder"))]
struct BoundIndexBuffer {
  max_indices: u32,
}

impl<'pass> RenderPassEncoder<'pass> {
  /// Create a new render pass encoder (internal).
  fn new(
    encoder: &'pass mut platform::command::CommandEncoder,
    pass: &'pass RenderPass,
    destination_info: RenderPassDestinationInfo,
    color_attachments: &'pass mut RenderColorAttachments<'pass>,
    depth_texture: Option<&'pass DepthTexture>,
  ) -> Self {
    // Build the platform render pass
    let mut rp_builder = platform::render_pass::RenderPassBuilder::new();

    // Map color operations from the high-level RenderPass
    let (color_load_op, color_store_op) = pass.color_operations().to_platform();
    rp_builder = rp_builder
      .with_color_load_op(color_load_op)
      .with_store_op(color_store_op);

    // Map depth and stencil operations from the high-level RenderPass
    let platform_depth_ops =
      pass.depth_operations().map(|dop| dop.to_platform());
    let platform_stencil_ops =
      pass.stencil_operations().map(|sop| sop.to_platform());

    let depth_view = depth_texture.map(|dt| dt.platform_view_ref());
    let has_depth_attachment = depth_texture.is_some();
    let has_stencil = pass.stencil_operations().is_some();

    let platform_pass = rp_builder.build(
      encoder,
      color_attachments.as_platform_attachments_mut(),
      depth_view,
      platform_depth_ops,
      platform_stencil_ops,
      pass.label(),
    );

    return RenderPassEncoder {
      pass: platform_pass,
      uses_color: pass.uses_color(),
      has_depth_attachment,
      has_stencil,
      sample_count: pass.sample_count(),
      destination_color_format: destination_info.color_format,
      destination_depth_format: destination_info.depth_format,
      #[cfg(any(debug_assertions, feature = "render-validation-encoder"))]
      current_pipeline: None,
      #[cfg(any(debug_assertions, feature = "render-validation-encoder"))]
      bound_index_buffer: None,
      #[cfg(any(debug_assertions, feature = "render-validation-instancing"))]
      bound_vertex_slots: HashSet::new(),
      #[cfg(any(
        debug_assertions,
        feature = "render-validation-depth",
        feature = "render-validation-stencil"
      ))]
      warned_no_stencil_for_pipeline: HashSet::new(),
      #[cfg(any(
        debug_assertions,
        feature = "render-validation-depth",
        feature = "render-validation-stencil"
      ))]
      warned_no_depth_for_pipeline: HashSet::new(),
    };
  }

  /// Set the active render pipeline.
  ///
  /// Returns an error if the pipeline is incompatible with the current pass
  /// configuration (e.g., color target mismatch).
  pub fn set_pipeline(
    &mut self,
    pipeline: &RenderPipeline,
  ) -> Result<(), RenderPassError> {
    // Validation
    #[cfg(any(
      debug_assertions,
      feature = "render-validation-pass-compat",
      feature = "render-validation-encoder"
    ))]
    {
      let label = pipeline.pipeline().label().unwrap_or("unnamed");

      if !self.uses_color && pipeline.has_color_targets() {
        return Err(RenderPassError::PipelineIncompatible(format!(
          "Render pipeline '{}' declares color targets but the current pass \
           has no color attachments",
          label
        )));
      }
      if self.uses_color && !pipeline.has_color_targets() {
        return Err(RenderPassError::PipelineIncompatible(format!(
          "Render pipeline '{}' has no color targets but the current pass \
           declares color attachments",
          label
        )));
      }
      if !self.has_depth_attachment && pipeline.expects_depth_stencil() {
        return Err(RenderPassError::PipelineIncompatible(format!(
          "Render pipeline '{}' expects a depth/stencil attachment but the \
           current pass has none",
          label
        )));
      }
    }

    #[cfg(any(debug_assertions, feature = "render-validation-render-targets",))]
    {
      let label = pipeline.pipeline().label().unwrap_or("unnamed");

      if pipeline.sample_count() != self.sample_count {
        return Err(RenderPassError::PipelineIncompatible(format!(
          "Render pipeline '{}' has sample_count={} but pass sample_count={}",
          label,
          pipeline.sample_count(),
          self.sample_count
        )));
      }

      if self.uses_color {
        if let Some(dest_format) = self.destination_color_format {
          if pipeline.color_target_format() != Some(dest_format) {
            return Err(RenderPassError::PipelineIncompatible(format!(
              "Render pipeline '{}' color format {:?} does not match destination color format {:?}",
              label,
              pipeline.color_target_format(),
              dest_format
            )));
          }
        }
      }

      if self.has_depth_attachment && pipeline.expects_depth_stencil() {
        if let Some(dest_depth_format) = self.destination_depth_format {
          if pipeline.depth_format() != Some(dest_depth_format) {
            return Err(RenderPassError::PipelineIncompatible(format!(
              "Render pipeline '{}' depth format {:?} does not match destination depth format {:?}",
              label,
              pipeline.depth_format(),
              dest_depth_format
            )));
          }
        }
      }
    }

    // Track current pipeline for draw validation
    #[cfg(any(debug_assertions, feature = "render-validation-encoder"))]
    {
      let label = pipeline.pipeline().label().unwrap_or("unnamed").to_string();
      self.current_pipeline = Some(CurrentPipeline {
        label,
        per_instance_slots: pipeline.per_instance_slots().clone(),
      });
    }

    // Advisory warnings
    #[cfg(any(
      debug_assertions,
      feature = "render-validation-depth",
      feature = "render-validation-stencil"
    ))]
    {
      let label = pipeline.pipeline().label().unwrap_or("unnamed");
      let pipeline_id = pipeline as *const _ as usize;

      if self.has_stencil
        && !pipeline.uses_stencil()
        && self.warned_no_stencil_for_pipeline.insert(pipeline_id)
      {
        let key = format!("stencil:no_test:{}", label);
        let msg = format!(
          "Pass provides stencil ops but pipeline '{}' has no stencil test; \
           stencil will not affect rendering",
          label
        );
        util::warn_once(&key, &msg);
      }

      if !self.has_stencil && pipeline.uses_stencil() {
        let key = format!("stencil:pass_no_operations:{}", label);
        let msg = format!(
          "Pipeline '{}' enables stencil but pass has no stencil ops \
           configured; stencil reference/tests may be ineffective",
          label
        );
        util::warn_once(&key, &msg);
      }

      if self.has_depth_attachment
        && !pipeline.expects_depth_stencil()
        && self.warned_no_depth_for_pipeline.insert(pipeline_id)
      {
        let key = format!("depth:no_test:{}", label);
        let msg = format!(
          "Pass has depth attachment but pipeline '{}' does not enable depth \
           testing; depth values will not be tested/written",
          label
        );
        util::warn_once(&key, &msg);
      }
    }

    self.pass.set_pipeline(pipeline.pipeline());
    return Ok(());
  }

  /// Set the viewport for subsequent draw commands.
  pub fn set_viewport(&mut self, viewport: &Viewport) {
    let (x, y, width, height, min_depth, max_depth) = viewport.viewport_f32();
    self
      .pass
      .set_viewport(x, y, width, height, min_depth, max_depth);

    let (sx, sy, sw, sh) = viewport.scissor_u32();
    self.pass.set_scissor_rect(sx, sy, sw, sh);
  }

  /// Set only the scissor rectangle.
  pub fn set_scissor(&mut self, viewport: &Viewport) {
    let (x, y, width, height) = viewport.scissor_u32();
    self.pass.set_scissor_rect(x, y, width, height);
  }

  /// Set the stencil reference value.
  pub fn set_stencil_reference(&mut self, reference: u32) {
    self.pass.set_stencil_reference(reference);
  }

  /// Bind a bind group with optional dynamic offsets.
  pub fn set_bind_group(
    &mut self,
    set: u32,
    group: &BindGroup,
    dynamic_offsets: &[u32],
    min_uniform_buffer_offset_alignment: u32,
  ) -> Result<(), RenderPassError> {
    validation::validate_dynamic_offsets(
      group.dynamic_binding_count(),
      dynamic_offsets,
      min_uniform_buffer_offset_alignment,
      set,
    )
    .map_err(RenderPassError::Validation)?;

    self
      .pass
      .set_bind_group(set, group.platform_group(), dynamic_offsets);
    return Ok(());
  }

  /// Bind a vertex buffer to a slot.
  pub fn set_vertex_buffer(&mut self, slot: u32, buffer: &Buffer) {
    #[cfg(any(debug_assertions, feature = "render-validation-instancing"))]
    {
      self.bound_vertex_slots.insert(slot);
    }

    self.pass.set_vertex_buffer(slot, buffer.raw());
  }

  /// Bind an index buffer with the specified format.
  pub fn set_index_buffer(
    &mut self,
    buffer: &Buffer,
    format: IndexFormat,
  ) -> Result<(), RenderPassError> {
    #[cfg(any(debug_assertions, feature = "render-validation-encoder"))]
    {
      if buffer.buffer_type() != BufferType::Index {
        return Err(RenderPassError::Validation(format!(
          "Binding buffer as index but logical type is {:?}; expected \
           BufferType::Index",
          buffer.buffer_type()
        )));
      }

      let element_size = match format {
        IndexFormat::Uint16 => 2u64,
        IndexFormat::Uint32 => 4u64,
      };
      let stride = buffer.stride();
      if stride != element_size {
        return Err(RenderPassError::Validation(format!(
          "Index buffer has element stride {} bytes but format {:?} requires \
           {} bytes",
          stride, format, element_size
        )));
      }

      let buffer_size = buffer.raw().size();
      if !buffer_size.is_multiple_of(element_size) {
        return Err(RenderPassError::Validation(format!(
          "Index buffer size {} bytes is not a multiple of element size {} \
           for format {:?}",
          buffer_size, element_size, format
        )));
      }

      let max_indices =
        (buffer_size / element_size).min(u32::MAX as u64) as u32;
      self.bound_index_buffer = Some(BoundIndexBuffer { max_indices });
    }

    self
      .pass
      .set_index_buffer(buffer.raw(), format.to_platform());
    return Ok(());
  }

  /// Issue a non-indexed draw call.
  pub fn draw(
    &mut self,
    vertices: Range<u32>,
    instances: Range<u32>,
  ) -> Result<(), RenderPassError> {
    #[cfg(any(debug_assertions, feature = "render-validation-encoder"))]
    {
      if self.current_pipeline.is_none() {
        return Err(RenderPassError::NoPipeline(
          "Draw command encountered before any pipeline was set".to_string(),
        ));
      }
    }

    #[cfg(any(debug_assertions, feature = "render-validation-instancing"))]
    {
      if let Some(ref pipeline) = self.current_pipeline {
        validation::validate_instance_bindings(
          &pipeline.label,
          &pipeline.per_instance_slots,
          &self.bound_vertex_slots,
        )
        .map_err(RenderPassError::Validation)?;

        validation::validate_instance_range("Draw", &instances)
          .map_err(RenderPassError::Validation)?;
      }

      if instances.start == instances.end {
        logging::debug!(
          "Skipping Draw with empty instance range {}..{}",
          instances.start,
          instances.end
        );
        return Ok(());
      }
    }

    self.pass.draw(vertices, instances);
    return Ok(());
  }

  /// Issue an indexed draw call.
  pub fn draw_indexed(
    &mut self,
    indices: Range<u32>,
    base_vertex: i32,
    instances: Range<u32>,
  ) -> Result<(), RenderPassError> {
    #[cfg(any(debug_assertions, feature = "render-validation-encoder"))]
    {
      if self.current_pipeline.is_none() {
        return Err(RenderPassError::NoPipeline(
          "DrawIndexed command encountered before any pipeline was set"
            .to_string(),
        ));
      }

      match &self.bound_index_buffer {
        None => {
          return Err(RenderPassError::NoIndexBuffer(
            "DrawIndexed command encountered without a bound index buffer"
              .to_string(),
          ));
        }
        Some(bound) => {
          if indices.start > indices.end {
            return Err(RenderPassError::Validation(format!(
              "DrawIndexed index range start {} is greater than end {}",
              indices.start, indices.end
            )));
          }
          if indices.end > bound.max_indices {
            return Err(RenderPassError::Validation(format!(
              "DrawIndexed index range {}..{} exceeds index buffer capacity {}",
              indices.start, indices.end, bound.max_indices
            )));
          }
        }
      }
    }

    #[cfg(any(debug_assertions, feature = "render-validation-instancing"))]
    {
      if let Some(ref pipeline) = self.current_pipeline {
        validation::validate_instance_bindings(
          &pipeline.label,
          &pipeline.per_instance_slots,
          &self.bound_vertex_slots,
        )
        .map_err(RenderPassError::Validation)?;

        validation::validate_instance_range("DrawIndexed", &instances)
          .map_err(RenderPassError::Validation)?;
      }

      if instances.start == instances.end {
        logging::debug!(
          "Skipping DrawIndexed with empty instance range {}..{}",
          instances.start,
          instances.end
        );
        return Ok(());
      }
    }

    self.pass.draw_indexed(indices, base_vertex, instances);
    return Ok(());
  }

  /// Set immediate data for subsequent draw calls.
  ///
  /// The `offset` and `data` length MUST be multiples of 4 bytes
  /// (IMMEDIATE_DATA_ALIGNMENT). The data bytes are passed directly to the
  /// GPU for use by shaders that declare `push_constant` uniform blocks
  /// (the GLSL syntax remains unchanged).
  pub fn set_immediates(&mut self, offset: u32, data: &[u8]) {
    self.pass.set_immediates(offset, data);
  }
}

impl std::fmt::Debug for RenderPassEncoder<'_> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    return f
      .debug_struct("RenderPassEncoder")
      .field("uses_color", &self.uses_color)
      .field("has_depth_attachment", &self.has_depth_attachment)
      .field("has_stencil", &self.has_stencil)
      .field("sample_count", &self.sample_count)
      .finish_non_exhaustive();
  }
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Errors that can occur during render pass encoding.
#[derive(Debug, Clone)]
pub enum RenderPassError {
  /// Pipeline is incompatible with the current pass configuration.
  PipelineIncompatible(String),
  /// No pipeline has been set before a draw call.
  NoPipeline(String),
  /// No index buffer has been bound before an indexed draw call.
  NoIndexBuffer(String),
  /// Validation error (offsets, ranges, types, etc.).
  Validation(String),
}

impl std::fmt::Display for RenderPassError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    return match self {
      RenderPassError::PipelineIncompatible(s) => write!(f, "{}", s),
      RenderPassError::NoPipeline(s) => write!(f, "{}", s),
      RenderPassError::NoIndexBuffer(s) => write!(f, "{}", s),
      RenderPassError::Validation(s) => write!(f, "{}", s),
    };
  }
}

impl std::error::Error for RenderPassError {}

#[cfg(test)]
pub(crate) fn new_render_pass_encoder_for_tests<'pass>(
  encoder: &'pass mut platform::command::CommandEncoder,
  pass: &'pass RenderPass,
  destination_info: RenderPassDestinationInfo,
  color_attachments: &'pass mut RenderColorAttachments<'pass>,
  depth_texture: Option<&'pass DepthTexture>,
) -> RenderPassEncoder<'pass> {
  return RenderPassEncoder::new(
    encoder,
    pass,
    destination_info,
    color_attachments,
    depth_texture,
  );
}

#[cfg(test)]
mod tests {
  use super::*;
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
    gpu::{
      Gpu,
      GpuBuilder,
    },
    instance::InstanceBuilder,
    pipeline::RenderPipelineBuilder,
    render_pass::RenderPassBuilder,
    shader::{
      ShaderBuilder,
      ShaderKind,
      VirtualShader,
    },
    texture::{
      DepthFormat,
      DepthTextureBuilder,
      TextureBuilder,
      TextureFormat,
    },
    viewport::Viewport,
  };

  fn create_test_gpu() -> Option<Gpu> {
    let instance = InstanceBuilder::new()
      .with_label("lambda-encoder-test-instance")
      .build();
    return GpuBuilder::new()
      .with_label("lambda-encoder-test-gpu")
      .build(&instance, None)
      .ok();
  }

  fn compile_triangle_shaders(
  ) -> (crate::render::shader::Shader, crate::render::shader::Shader) {
    let vert_path = format!(
      "{}/assets/shaders/triangle.vert",
      env!("CARGO_MANIFEST_DIR")
    );
    let frag_path = format!(
      "{}/assets/shaders/triangle.frag",
      env!("CARGO_MANIFEST_DIR")
    );

    let mut builder = ShaderBuilder::new();
    let vs = builder.build(VirtualShader::File {
      path: vert_path,
      kind: ShaderKind::Vertex,
      name: "triangle-vert".to_string(),
      entry_point: "main".to_string(),
    });
    let fs = builder.build(VirtualShader::File {
      path: frag_path,
      kind: ShaderKind::Fragment,
      name: "triangle-frag".to_string(),
      entry_point: "main".to_string(),
    });
    return (vs, fs);
  }

  /// Ensures the `Display` implementation for `RenderPassError` forwards the
  /// underlying message without modification.
  #[test]
  fn render_pass_error_display_is_passthrough() {
    let err = RenderPassError::NoPipeline("oops".to_string());
    assert_eq!(err.to_string(), "oops");
  }

  /// Validates the encoder reports an error when a draw is issued before
  /// setting a pipeline (when validation is enabled).
  #[test]
  #[ignore = "requires a real GPU adapter"]
  fn render_pass_encoder_draw_requires_pipeline_when_validation_enabled() {
    let gpu = create_test_gpu().expect("requires a real GPU adapter");

    let resolve = TextureBuilder::new_2d(TextureFormat::Rgba8Unorm)
      .with_size(4, 4)
      .for_render_target()
      .build(&gpu)
      .expect("build resolve texture");

    let pass = RenderPassBuilder::new()
      .with_label("no-pipeline-pass")
      .build(&gpu, TextureFormat::Rgba8Unorm, DepthFormat::Depth24Plus);

    let mut encoder = platform::command::CommandEncoder::new(
      gpu.platform(),
      Some("lambda-no-pipeline-encoder"),
    );

    let mut attachments = RenderColorAttachments::for_offscreen_pass(
      pass.uses_color(),
      pass.sample_count(),
      None,
      resolve.view_ref(),
    );

    let mut rp = RenderPassEncoder::new(
      &mut encoder,
      &pass,
      RenderPassDestinationInfo {
        color_format: Some(TextureFormat::Rgba8Unorm),
        depth_format: None,
      },
      &mut attachments,
      None,
    );

    let result = rp.draw(0..3, 0..1);
    if cfg!(any(debug_assertions, feature = "render-validation-encoder")) {
      let err = result.expect_err("draw must error without a pipeline");
      assert!(matches!(err, RenderPassError::NoPipeline(_)));
    } else {
      result.expect("draw ok without validation");
    }

    drop(rp);
    let _ = encoder.finish();
  }

  /// In debug builds, checks the engine's pipeline/pass compatibility checks
  /// fire before provoking underlying wgpu validation errors.
  #[test]
  #[ignore = "requires a real GPU adapter"]
  fn render_pass_encoder_validates_pipeline_compatibility_in_debug() {
    if !cfg!(debug_assertions) {
      // The explicit pass/pipeline compatibility checks are debug- or
      // feature-gated; don't attempt to provoke wgpu validation in release.
      return;
    }

    let gpu = create_test_gpu().expect("requires a real GPU adapter");

    let (vs, fs) = compile_triangle_shaders();

    let pass = RenderPassBuilder::new()
      .with_label("depth-only-pass")
      .without_color()
      .with_depth()
      .build(&gpu, TextureFormat::Rgba8Unorm, DepthFormat::Depth24Plus);

    let pipeline = RenderPipelineBuilder::new()
      .with_label("color-pipeline")
      .build(
        &gpu,
        TextureFormat::Rgba8Unorm,
        DepthFormat::Depth24Plus,
        &pass,
        &vs,
        Some(&fs),
      );

    let mut encoder = platform::command::CommandEncoder::new(
      gpu.platform(),
      Some("lambda-pass-compat-encoder"),
    );

    let mut attachments = RenderColorAttachments::new();
    let depth_texture = DepthTextureBuilder::new()
      .with_label("lambda-pass-compat-depth")
      .with_size(1, 1)
      .with_format(DepthFormat::Depth24Plus)
      .build(&gpu);

    let mut rp = RenderPassEncoder::new(
      &mut encoder,
      &pass,
      RenderPassDestinationInfo {
        color_format: None,
        depth_format: Some(DepthFormat::Depth24Plus),
      },
      &mut attachments,
      Some(&depth_texture),
    );

    let err = rp
      .set_pipeline(&pipeline)
      .expect_err("pipeline with color targets must be incompatible");
    assert!(matches!(err, RenderPassError::PipelineIncompatible(_)));

    drop(rp);
    let _ = encoder.finish();
  }

  /// Exercises the common command encoding path (viewport/scissor/pipeline),
  /// plus validation branches for bind group dynamic offsets and index buffers.
  #[test]
  #[ignore = "requires a real GPU adapter"]
  fn render_pass_encoder_encodes_commands_and_validates_index_buffers() {
    let gpu = create_test_gpu().expect("requires a real GPU adapter");

    let (vs, fs) = compile_triangle_shaders();

    let pass = RenderPassBuilder::new().with_label("basic-pass").build(
      &gpu,
      TextureFormat::Rgba8Unorm,
      DepthFormat::Depth24Plus,
    );

    let pipeline = RenderPipelineBuilder::new()
      .with_label("basic-pipeline")
      .build(
        &gpu,
        TextureFormat::Rgba8Unorm,
        DepthFormat::Depth24Plus,
        &pass,
        &vs,
        Some(&fs),
      );

    let resolve = TextureBuilder::new_2d(TextureFormat::Rgba8Unorm)
      .with_size(4, 4)
      .for_render_target()
      .build(&gpu)
      .expect("build resolve texture");

    let mut encoder = platform::command::CommandEncoder::new(
      gpu.platform(),
      Some("lambda-encode-commands-encoder"),
    );

    let mut attachments = RenderColorAttachments::for_offscreen_pass(
      pass.uses_color(),
      pass.sample_count(),
      None,
      resolve.view_ref(),
    );

    let mut rp = RenderPassEncoder::new(
      &mut encoder,
      &pass,
      RenderPassDestinationInfo {
        color_format: Some(TextureFormat::Rgba8Unorm),
        depth_format: None,
      },
      &mut attachments,
      None,
    );

    let viewport = Viewport {
      x: 0,
      y: 0,
      width: 4,
      height: 4,
      min_depth: 0.0,
      max_depth: 1.0,
    };
    rp.set_viewport(&viewport);
    rp.set_scissor(&viewport);

    rp.set_pipeline(&pipeline).expect("set pipeline");

    // Bind group validation: dynamic binding count mismatch.
    let layout = BindGroupLayoutBuilder::new()
      .with_uniform_dynamic(0, BindingVisibility::VertexAndFragment)
      .build(&gpu);

    let uniform = BufferBuilder::new()
      .with_label("encoder-test-uniform")
      .with_usage(Usage::UNIFORM)
      .with_properties(Properties::CPU_VISIBLE)
      .with_buffer_type(BufferType::Uniform)
      .build(&gpu, vec![0u32; 4])
      .expect("build uniform buffer");

    let group = BindGroupBuilder::new()
      .with_layout(&layout)
      .with_uniform(0, &uniform, 0, None)
      .build(&gpu);

    let err = rp
      .set_bind_group(
        0,
        &group,
        &[],
        gpu.limit_min_uniform_buffer_offset_alignment(),
      )
      .expect_err("dynamic offsets must be provided");
    assert!(matches!(err, RenderPassError::Validation(_)));

    // Index buffer validation should catch wrong logical type and stride when enabled.
    let vertex_buffer = BufferBuilder::new()
      .with_label("encoder-test-vertex")
      .with_usage(Usage::VERTEX)
      .with_properties(Properties::CPU_VISIBLE)
      .with_buffer_type(BufferType::Vertex)
      .build(&gpu, vec![0u32; 4])
      .expect("build vertex buffer");

    let bad_type = rp.set_index_buffer(&vertex_buffer, IndexFormat::Uint16);
    if cfg!(any(debug_assertions, feature = "render-validation-encoder")) {
      assert!(matches!(bad_type, Err(RenderPassError::Validation(_))));
    } else {
      assert!(bad_type.is_ok());
    }

    let bad_stride = BufferBuilder::new()
      .with_label("encoder-test-index-bad-stride")
      .with_usage(Usage::INDEX)
      .with_properties(Properties::CPU_VISIBLE)
      .with_buffer_type(BufferType::Index)
      .build(&gpu, vec![0u8; 8])
      .expect("build index buffer");

    let bad_stride = rp.set_index_buffer(&bad_stride, IndexFormat::Uint16);
    if cfg!(any(debug_assertions, feature = "render-validation-encoder")) {
      assert!(matches!(bad_stride, Err(RenderPassError::Validation(_))));
    } else {
      assert!(bad_stride.is_ok());
    }

    let index_buffer = BufferBuilder::new()
      .with_label("encoder-test-index")
      .with_usage(Usage::INDEX)
      .with_properties(Properties::CPU_VISIBLE)
      .with_buffer_type(BufferType::Index)
      .build(&gpu, vec![0u16, 1u16, 2u16])
      .expect("build index buffer");

    rp.set_index_buffer(&index_buffer, IndexFormat::Uint16)
      .expect("set index buffer");

    rp.draw(0..3, 0..1).expect("draw");

    let indexed = rp.draw_indexed(0..4, 0, 0..1);
    if cfg!(any(debug_assertions, feature = "render-validation-encoder")) {
      assert!(matches!(indexed, Err(RenderPassError::Validation(_))));
    } else {
      assert!(indexed.is_ok());
    }

    drop(rp);
    let cb = encoder.finish();
    gpu.submit(std::iter::once(cb));
  }
}
