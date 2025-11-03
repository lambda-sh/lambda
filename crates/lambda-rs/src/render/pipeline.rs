//! Graphics render pipelines and builders.
//!
//! Purpose
//! - Define how vertex data flows into the vertex shader (buffer layouts and
//!   attributes) and how fragments are produced (optional fragment stage and
//!   color target).
//! - Compose a pipeline layout from bind group layouts and optional push
//!   constant ranges.
//!
//! Usage
//! - Accumulate vertex buffers and `VertexAttribute` lists matching shader
//!   `location`s.
//! - Provide one or more `BindGroupLayout`s used by the shaders.
//! - Supply a vertex shader and optional fragment shader compiled to SPIR‑V.
//!
//! Example
//! ```rust
//! // Single vertex buffer with position/color; one push constant range for the vertex stage
//! use lambda::render::pipeline::{RenderPipelineBuilder, PipelineStage, CullingMode};
//! let pipeline = RenderPipelineBuilder::new()
//!   .with_buffer(vertex_buffer, attributes)
//!   .with_push_constant(PipelineStage::VERTEX, 64)
//!   .with_layouts(&[&globals_bgl])
//!   .with_culling(CullingMode::Back)
//!   .build(&mut render_context, &render_pass, &vs, Some(&fs));
//! ```

use std::{
  ops::Range,
  rc::Rc,
};

use lambda_platform::wgpu::pipeline as platform_pipeline;

use super::{
  bind,
  buffer::Buffer,
  render_pass::RenderPass,
  shader::Shader,
  vertex::VertexAttribute,
  RenderContext,
};

#[derive(Debug)]
/// A created graphics pipeline and the vertex buffers it expects.
///
/// Pipelines are immutable; destroy them with the context when no longer needed.
pub struct RenderPipeline {
  pipeline: Rc<platform_pipeline::RenderPipeline>,
  buffers: Vec<Rc<Buffer>>,
}

impl RenderPipeline {
  /// Destroy the render pipeline with the render context that created it.
  pub fn destroy(self, _render_context: &RenderContext) {}

  /// Access the vertex buffers associated with this pipeline.
  pub(super) fn buffers(&self) -> &Vec<Rc<Buffer>> {
    return &self.buffers;
  }

  /// Access the underlying platform render pipeline.
  pub(super) fn pipeline(&self) -> &platform_pipeline::RenderPipeline {
    return self.pipeline.as_ref();
  }
}

/// Public alias for platform shader stage flags used by push constants.
pub use platform_pipeline::PipelineStage;

/// Convenience alias for uploading push constants: stage and byte range.
pub type PushConstantUpload = (PipelineStage, Range<u32>);

struct BufferBinding {
  buffer: Rc<Buffer>,
  attributes: Vec<VertexAttribute>,
}

/// Public alias for platform culling mode used by pipeline builders.
pub use platform_pipeline::CullingMode;

/// Builder for creating a graphics `RenderPipeline`.
///
/// Notes
/// - The number of bind group layouts MUST NOT exceed the device limit; the
///   builder asserts this against the current device.
/// - If a fragment shader is omitted, no color target is attached and the
///   pipeline can still be used for vertex‑only workloads.
pub struct RenderPipelineBuilder {
  push_constants: Vec<PushConstantUpload>,
  bindings: Vec<BufferBinding>,
  culling: CullingMode,
  bind_group_layouts: Vec<bind::BindGroupLayout>,
  label: Option<String>,
}

impl RenderPipelineBuilder {
  /// Creates a new render pipeline builder.
  pub fn new() -> Self {
    Self {
      push_constants: Vec::new(),
      bindings: Vec::new(),
      culling: CullingMode::Back,
      bind_group_layouts: Vec::new(),
      label: None,
    }
  }

  /// Declare a vertex buffer and the vertex attributes consumed by the shader.
  pub fn with_buffer(
    mut self,
    buffer: Buffer,
    attributes: Vec<VertexAttribute>,
  ) -> Self {
    self.bindings.push(BufferBinding {
      buffer: Rc::new(buffer),
      attributes,
    });
    return self;
  }

  /// Declare a push constant range for a shader stage in bytes.
  pub fn with_push_constant(
    mut self,
    stage: PipelineStage,
    bytes: u32,
  ) -> Self {
    self.push_constants.push((stage, 0..bytes));
    return self;
  }

  /// Attach a debug label to the pipeline.
  pub fn with_label(mut self, label: &str) -> Self {
    self.label = Some(label.to_string());
    return self;
  }

  /// Configure triangle face culling. Defaults to culling back faces.
  pub fn with_culling(mut self, mode: CullingMode) -> Self {
    self.culling = mode;
    return self;
  }

  /// Provide one or more bind group layouts used to create the pipeline layout.
  pub fn with_layouts(mut self, layouts: &[&bind::BindGroupLayout]) -> Self {
    self.bind_group_layouts = layouts.iter().map(|l| (*l).clone()).collect();
    return self;
  }

  /// Build a graphics pipeline using the provided shader modules and
  /// previously registered vertex inputs and push constants.
  pub fn build(
    self,
    render_context: &mut RenderContext,
    _render_pass: &RenderPass,
    vertex_shader: &Shader,
    fragment_shader: Option<&Shader>,
  ) -> RenderPipeline {
    let surface_format = render_context.surface_format();

    // Shader modules
    let vertex_module = platform_pipeline::ShaderModule::from_spirv(
      render_context.gpu(),
      vertex_shader.binary(),
      Some("lambda-vertex-shader"),
    );
    let fragment_module = fragment_shader.map(|shader| {
      platform_pipeline::ShaderModule::from_spirv(
        render_context.gpu(),
        shader.binary(),
        Some("lambda-fragment-shader"),
      )
    });

    // Push constant ranges
    let push_constant_ranges: Vec<platform_pipeline::PushConstantRange> = self
      .push_constants
      .iter()
      .map(|(stage, range)| platform_pipeline::PushConstantRange {
        stages: *stage,
        range: range.clone(),
      })
      .collect();

    // Bind group layouts limit check
    let max_bind_groups = render_context.limit_max_bind_groups() as usize;
    assert!(
      self.bind_group_layouts.len() <= max_bind_groups,
      "Pipeline declares {} bind group layouts, exceeds device max {}",
      self.bind_group_layouts.len(),
      max_bind_groups
    );

    // Pipeline layout via platform
    let bgl_platform: Vec<&lambda_platform::wgpu::bind::BindGroupLayout> = self
      .bind_group_layouts
      .iter()
      .map(|l| l.platform_layout())
      .collect();
    let pipeline_layout = platform_pipeline::PipelineLayoutBuilder::new()
      .with_label("lambda-pipeline-layout")
      .with_layouts(&bgl_platform)
      .with_push_constants(push_constant_ranges)
      .build(render_context.gpu());

    // Vertex buffers and attributes
    let mut buffers = Vec::with_capacity(self.bindings.len());
    let mut rp_builder = platform_pipeline::RenderPipelineBuilder::new()
      .with_label(self.label.as_deref().unwrap_or("lambda-render-pipeline"))
      .with_layout(&pipeline_layout)
      .with_cull_mode(self.culling);

    for binding in &self.bindings {
      let attributes: Vec<platform_pipeline::VertexAttributeDesc> = binding
        .attributes
        .iter()
        .map(|attribute| platform_pipeline::VertexAttributeDesc {
          shader_location: attribute.location,
          offset: (attribute.offset + attribute.element.offset) as u64,
          format: attribute.element.format.to_platform(),
        })
        .collect();

      rp_builder =
        rp_builder.with_vertex_buffer(binding.buffer.stride(), attributes);
      buffers.push(binding.buffer.clone());
    }

    if fragment_module.is_some() {
      rp_builder = rp_builder.with_surface_color_target(surface_format);
    }

    let pipeline = rp_builder.build(
      render_context.gpu(),
      &vertex_module,
      fragment_module.as_ref(),
    );

    return RenderPipeline {
      pipeline: Rc::new(pipeline),
      buffers,
    };
  }
}
