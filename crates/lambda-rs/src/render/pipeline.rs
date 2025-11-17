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
//! ```rust,ignore
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

use lambda_platform::wgpu::{
  pipeline as platform_pipeline,
  texture as platform_texture,
};
use logging;

use super::{
  bind,
  buffer::Buffer,
  render_pass::RenderPass,
  shader::Shader,
  texture,
  vertex::VertexAttribute,
  RenderContext,
};
use crate::render::validation;

/// A created graphics pipeline and the vertex buffers it expects.
#[derive(Debug)]
///
/// Pipelines are immutable; destroy them with the context when no longer needed.
pub struct RenderPipeline {
  pipeline: Rc<platform_pipeline::RenderPipeline>,
  buffers: Vec<Rc<Buffer>>,
  sample_count: u32,
  color_target_count: u32,
  expects_depth_stencil: bool,
  uses_stencil: bool,
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

  /// Multisample count configured on this pipeline.
  pub fn sample_count(&self) -> u32 {
    return self.sample_count.max(1);
  }

  /// Whether the pipeline declares one or more color targets.
  pub(super) fn has_color_targets(&self) -> bool {
    return self.color_target_count > 0;
  }

  /// Whether the pipeline expects a depth-stencil attachment.
  pub(super) fn expects_depth_stencil(&self) -> bool {
    return self.expects_depth_stencil;
  }

  /// Whether the pipeline configured a stencil test/state.
  pub(super) fn uses_stencil(&self) -> bool {
    return self.uses_stencil;
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

#[derive(Clone, Copy, Debug)]
/// Engine-level compare function for depth/stencil tests.
pub enum CompareFunction {
  Never,
  Less,
  LessEqual,
  Greater,
  GreaterEqual,
  Equal,
  NotEqual,
  Always,
}

impl CompareFunction {
  fn to_platform(self) -> platform_pipeline::CompareFunction {
    return match self {
      CompareFunction::Never => platform_pipeline::CompareFunction::Never,
      CompareFunction::Less => platform_pipeline::CompareFunction::Less,
      CompareFunction::LessEqual => {
        platform_pipeline::CompareFunction::LessEqual
      }
      CompareFunction::Greater => platform_pipeline::CompareFunction::Greater,
      CompareFunction::GreaterEqual => {
        platform_pipeline::CompareFunction::GreaterEqual
      }
      CompareFunction::Equal => platform_pipeline::CompareFunction::Equal,
      CompareFunction::NotEqual => platform_pipeline::CompareFunction::NotEqual,
      CompareFunction::Always => platform_pipeline::CompareFunction::Always,
    };
  }
}

#[derive(Clone, Copy, Debug)]
/// Engine-level face culling mode for graphics pipelines.
pub enum CullingMode {
  None,
  Front,
  Back,
}

impl CullingMode {
  fn to_platform(self) -> platform_pipeline::CullingMode {
    return match self {
      CullingMode::None => platform_pipeline::CullingMode::None,
      CullingMode::Front => platform_pipeline::CullingMode::Front,
      CullingMode::Back => platform_pipeline::CullingMode::Back,
    };
  }
}

/// Engine-level stencil operation.
#[derive(Clone, Copy, Debug)]
pub enum StencilOperation {
  Keep,
  Zero,
  Replace,
  Invert,
  IncrementClamp,
  DecrementClamp,
  IncrementWrap,
  DecrementWrap,
}

impl StencilOperation {
  fn to_platform(self) -> platform_pipeline::StencilOperation {
    return match self {
      StencilOperation::Keep => platform_pipeline::StencilOperation::Keep,
      StencilOperation::Zero => platform_pipeline::StencilOperation::Zero,
      StencilOperation::Replace => platform_pipeline::StencilOperation::Replace,
      StencilOperation::Invert => platform_pipeline::StencilOperation::Invert,
      StencilOperation::IncrementClamp => {
        platform_pipeline::StencilOperation::IncrementClamp
      }
      StencilOperation::DecrementClamp => {
        platform_pipeline::StencilOperation::DecrementClamp
      }
      StencilOperation::IncrementWrap => {
        platform_pipeline::StencilOperation::IncrementWrap
      }
      StencilOperation::DecrementWrap => {
        platform_pipeline::StencilOperation::DecrementWrap
      }
    };
  }
}

/// Engine-level per-face stencil state.
#[derive(Clone, Copy, Debug)]
pub struct StencilFaceState {
  pub compare: CompareFunction,
  pub fail_op: StencilOperation,
  pub depth_fail_op: StencilOperation,
  pub pass_op: StencilOperation,
}

impl StencilFaceState {
  fn to_platform(self) -> platform_pipeline::StencilFaceState {
    platform_pipeline::StencilFaceState {
      compare: self.compare.to_platform(),
      fail_op: self.fail_op.to_platform(),
      depth_fail_op: self.depth_fail_op.to_platform(),
      pass_op: self.pass_op.to_platform(),
    }
  }
}

/// Engine-level full stencil state.
#[derive(Clone, Copy, Debug)]
pub struct StencilState {
  pub front: StencilFaceState,
  pub back: StencilFaceState,
  pub read_mask: u32,
  pub write_mask: u32,
}

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
  use_depth: bool,
  depth_format: Option<texture::DepthFormat>,
  sample_count: u32,
  depth_compare: Option<CompareFunction>,
  stencil: Option<platform_pipeline::StencilState>,
  depth_write_enabled: Option<bool>,
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
      use_depth: false,
      depth_format: None,
      sample_count: 1,
      depth_compare: None,
      stencil: None,
      depth_write_enabled: None,
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

  /// Enable depth testing/writes using the render context's depth format.
  pub fn with_depth(mut self) -> Self {
    self.use_depth = true;
    return self;
  }

  /// Enable depth with an explicit depth format.
  pub fn with_depth_format(mut self, format: texture::DepthFormat) -> Self {
    self.use_depth = true;
    self.depth_format = Some(format);
    return self;
  }

  /// Configure multi-sampling for this pipeline.
  pub fn with_multi_sample(mut self, samples: u32) -> Self {
    // Always apply a cheap validity check; log under feature/debug gates.
    if matches!(samples, 1 | 2 | 4 | 8) {
      self.sample_count = samples;
    } else {
      #[cfg(any(debug_assertions, feature = "render-validation-msaa",))]
      {
        if let Err(msg) = validation::validate_sample_count(samples) {
          logging::error!(
            "{}; falling back to sample_count=1 for pipeline",
            msg
          );
        }
      }
      self.sample_count = 1;
    }
    return self;
  }

  /// Set a non-default depth compare function.
  pub fn with_depth_compare(mut self, compare: CompareFunction) -> Self {
    self.depth_compare = Some(compare);
    return self;
  }

  /// Configure stencil state for the pipeline using engine types.
  pub fn with_stencil(mut self, stencil: StencilState) -> Self {
    let mapped = platform_pipeline::StencilState {
      front: stencil.front.to_platform(),
      back: stencil.back.to_platform(),
      read_mask: stencil.read_mask,
      write_mask: stencil.write_mask,
    };
    self.stencil = Some(mapped);
    return self;
  }

  /// Enable or disable depth writes for this pipeline.
  pub fn with_depth_write(mut self, enabled: bool) -> Self {
    self.depth_write_enabled = Some(enabled);
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
    if self.bind_group_layouts.len() > max_bind_groups {
      logging::error!(
        "Pipeline declares {} bind group layouts, exceeds device max {}",
        self.bind_group_layouts.len(),
        max_bind_groups
      );
    }
    debug_assert!(
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
      .with_cull_mode(self.culling.to_platform());

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

    if self.use_depth {
      // Engine-level depth format with default
      let mut dfmt = self
        .depth_format
        .unwrap_or(texture::DepthFormat::Depth32Float);
      // If stencil state is configured, ensure a stencil-capable depth format.
      if self.stencil.is_some()
        && dfmt != texture::DepthFormat::Depth24PlusStencil8
      {
        #[cfg(any(debug_assertions, feature = "render-validation-stencil",))]
        logging::error!(
          "Stencil configured but depth format {:?} lacks stencil; upgrading to Depth24PlusStencil8",
          dfmt
        );
        dfmt = texture::DepthFormat::Depth24PlusStencil8;
      }
      // Map to platform and keep context depth format in sync for attachment creation.
      let dfmt_platform = dfmt.to_platform();
      render_context.depth_format = dfmt_platform;
      rp_builder = rp_builder.with_depth_stencil(dfmt_platform);
      if let Some(compare) = self.depth_compare {
        rp_builder = rp_builder.with_depth_compare(compare.to_platform());
      }
      if let Some(stencil) = self.stencil {
        rp_builder = rp_builder.with_stencil(stencil);
      }
      if let Some(enabled) = self.depth_write_enabled {
        rp_builder = rp_builder.with_depth_write_enabled(enabled);
      }
    }

    // Apply multi-sampling to the pipeline.
    // Always align to the pass sample count; gate logs.
    let mut pipeline_samples = self.sample_count;
    let pass_samples = _render_pass.sample_count();
    if pipeline_samples != pass_samples {
      #[cfg(any(debug_assertions, feature = "render-validation-msaa",))]
      logging::error!(
        "Pipeline sample_count={} does not match pass sample_count={}; aligning to pass",
        pipeline_samples,
        pass_samples
      );
      pipeline_samples = pass_samples;
    }
    if !matches!(pipeline_samples, 1 | 2 | 4 | 8) {
      #[cfg(any(debug_assertions, feature = "render-validation-msaa",))]
      {
        let _ = validation::validate_sample_count(pipeline_samples);
      }
      pipeline_samples = 1;
    }
    rp_builder = rp_builder.with_sample_count(pipeline_samples);

    let pipeline = rp_builder.build(
      render_context.gpu(),
      &vertex_module,
      fragment_module.as_ref(),
    );

    return RenderPipeline {
      pipeline: Rc::new(pipeline),
      buffers,
      sample_count: pipeline_samples,
      color_target_count: if fragment_module.is_some() { 1 } else { 0 },
      // Depth/stencil is enabled when `with_depth*` was called on the builder.
      expects_depth_stencil: self.use_depth,
      uses_stencil: self.stencil.is_some(),
    };
  }
}
