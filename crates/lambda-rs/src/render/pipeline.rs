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
//! // Single vertex buffer with position/color; one immediate data range for the vertex stage
//! use lambda::render::pipeline::{RenderPipelineBuilder, CullingMode};
//! let pipeline = RenderPipelineBuilder::new()
//!   .with_buffer(vertex_buffer, attributes)
//!   .with_immediate_data(64)
//!   .with_layouts(&[&globals_bgl])
//!   .with_culling(CullingMode::Back)
//!   .build(&mut render_context, &render_pass, &vs, Some(&fs));
//! ```

use std::rc::Rc;

use lambda_platform::wgpu::pipeline as platform_pipeline;
use logging;

use super::{
  bind,
  buffer::{
    Buffer,
    BufferType,
  },
  gpu::Gpu,
  render_pass::RenderPass,
  shader::Shader,
  texture,
  vertex::{
    VertexAttribute,
    VertexBufferLayout,
    VertexStepMode,
  },
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
  color_target_format: Option<texture::TextureFormat>,
  expects_depth_stencil: bool,
  depth_format: Option<texture::DepthFormat>,
  uses_stencil: bool,
  per_instance_slots: Vec<bool>,
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

  pub(super) fn color_target_format(&self) -> Option<texture::TextureFormat> {
    return self.color_target_format;
  }

  /// Whether the pipeline expects a depth-stencil attachment.
  pub(super) fn expects_depth_stencil(&self) -> bool {
    return self.expects_depth_stencil;
  }

  pub(super) fn depth_format(&self) -> Option<texture::DepthFormat> {
    return self.depth_format;
  }

  /// Whether the pipeline configured a stencil test/state.
  pub(super) fn uses_stencil(&self) -> bool {
    return self.uses_stencil;
  }

  /// Per-vertex-buffer flags indicating which slots advance per instance.
  pub(super) fn per_instance_slots(&self) -> &Vec<bool> {
    return &self.per_instance_slots;
  }
}

/// Public alias for platform shader stage flags.
///
/// Stage flags remain useful for APIs such as bind group visibility, even
/// though wgpu v28 immediates no longer use stage-scoped updates.
pub use platform_pipeline::PipelineStage;

/// Blend mode for the pipeline's (single) color target.
///
/// Notes
/// - Defaults to `BlendMode::None` (opaque/replace). Opt in to blending for
///   transparent geometry.
/// - This is currently a single blend state for a single color attachment.
///   Per-attachment blending for MRT is future work.
#[derive(Clone, Copy, Debug)]
pub enum BlendMode {
  /// No blending; replace destination (default).
  None,
  /// Standard alpha blending.
  AlphaBlending,
  /// Premultiplied alpha blending.
  PremultipliedAlpha,
  /// Additive blending.
  Additive,
}

impl BlendMode {
  fn to_platform(self) -> platform_pipeline::BlendMode {
    return match self {
      BlendMode::None => platform_pipeline::BlendMode::None,
      BlendMode::AlphaBlending => platform_pipeline::BlendMode::AlphaBlending,
      BlendMode::PremultipliedAlpha => {
        platform_pipeline::BlendMode::PremultipliedAlpha
      }
      BlendMode::Additive => platform_pipeline::BlendMode::Additive,
    };
  }
}

struct BufferBinding {
  buffer: Rc<Buffer>,
  layout: VertexBufferLayout,
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

fn to_platform_step_mode(
  step_mode: VertexStepMode,
) -> platform_pipeline::VertexStepMode {
  return match step_mode {
    VertexStepMode::PerVertex => platform_pipeline::VertexStepMode::Vertex,
    VertexStepMode::PerInstance => platform_pipeline::VertexStepMode::Instance,
  };
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
    return platform_pipeline::StencilFaceState {
      compare: self.compare.to_platform(),
      fail_op: self.fail_op.to_platform(),
      depth_fail_op: self.depth_fail_op.to_platform(),
      pass_op: self.pass_op.to_platform(),
    };
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
  immediate_data: Vec<std::ops::Range<u32>>,
  bindings: Vec<BufferBinding>,
  culling: CullingMode,
  blend_mode: BlendMode,
  bind_group_layouts: Vec<bind::BindGroupLayout>,
  label: Option<String>,
  use_depth: bool,
  depth_format: Option<texture::DepthFormat>,
  sample_count: u32,
  depth_compare: Option<CompareFunction>,
  stencil: Option<platform_pipeline::StencilState>,
  depth_write_enabled: Option<bool>,
}

impl Default for RenderPipelineBuilder {
  fn default() -> Self {
    return Self::new();
  }
}

impl RenderPipelineBuilder {
  /// Creates a new render pipeline builder.
  pub fn new() -> Self {
    Self {
      immediate_data: Vec::new(),
      bindings: Vec::new(),
      culling: CullingMode::Back,
      blend_mode: BlendMode::None,
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
    self,
    buffer: Buffer,
    attributes: Vec<VertexAttribute>,
  ) -> Self {
    return self.with_buffer_step_mode(
      buffer,
      attributes,
      VertexStepMode::PerVertex,
    );
  }

  /// Declare a vertex buffer with an explicit step mode.
  pub fn with_buffer_step_mode(
    mut self,
    buffer: Buffer,
    attributes: Vec<VertexAttribute>,
    step_mode: VertexStepMode,
  ) -> Self {
    #[cfg(any(debug_assertions, feature = "render-validation-encoder",))]
    {
      if buffer.buffer_type() != BufferType::Vertex {
        logging::error!(
          "RenderPipelineBuilder::with_buffer called with a non-vertex buffer type {:?}; expected BufferType::Vertex",
          buffer.buffer_type()
        );
      }
    }

    let layout = VertexBufferLayout {
      stride: buffer.stride(),
      step_mode,
    };
    self.bindings.push(BufferBinding {
      buffer: Rc::new(buffer),
      layout,
      attributes,
    });
    return self;
  }

  /// Declare a per-instance vertex buffer.
  pub fn with_instance_buffer(
    self,
    buffer: Buffer,
    attributes: Vec<VertexAttribute>,
  ) -> Self {
    return self.with_buffer_step_mode(
      buffer,
      attributes,
      VertexStepMode::PerInstance,
    );
  }

  /// Declare an immediate data byte range size.
  ///
  /// wgpu v28 uses a single immediate data region sized by the pipeline
  /// layout. This method records a range starting at 0 whose end defines the
  /// required allocation size. Multiple calls are allowed; the final
  /// allocation is derived from the union of ranges.
  pub fn with_immediate_data(mut self, bytes: u32) -> Self {
    self.immediate_data.push(0..bytes);
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

  /// Configure blending for the pipeline's color target.
  ///
  /// Defaults to `BlendMode::None` (opaque).
  pub fn with_blend(mut self, mode: BlendMode) -> Self {
    self.blend_mode = mode;
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
  /// previously registered vertex inputs and immediate data.
  ///
  /// # Arguments
  /// * `gpu` - The GPU device to create the pipeline on.
  /// * `surface_format` - The texture format of the render target surface.
  /// * `depth_format` - The depth format for depth/stencil operations.
  /// * `render_pass` - The render pass this pipeline will be used with.
  /// * `vertex_shader` - The vertex shader module.
  /// * `fragment_shader` - Optional fragment shader module.
  pub fn build(
    self,
    gpu: &Gpu,
    surface_format: texture::TextureFormat,
    depth_format: texture::DepthFormat,
    render_pass: &RenderPass,
    vertex_shader: &Shader,
    fragment_shader: Option<&Shader>,
  ) -> RenderPipeline {
    // Shader modules
    let vertex_module = platform_pipeline::ShaderModule::from_spirv(
      gpu.platform(),
      vertex_shader.binary(),
      Some("lambda-vertex-shader"),
    );
    let fragment_module = fragment_shader.map(|shader| {
      platform_pipeline::ShaderModule::from_spirv(
        gpu.platform(),
        shader.binary(),
        Some("lambda-fragment-shader"),
      )
    });

    // Immediate data ranges
    let immediate_data_ranges: Vec<platform_pipeline::ImmediateDataRange> =
      self
        .immediate_data
        .iter()
        .map(|range| platform_pipeline::ImmediateDataRange {
          range: range.clone(),
        })
        .collect();

    // Bind group layouts limit check
    let max_bind_groups = gpu.limit_max_bind_groups() as usize;
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

    // Vertex buffer slot and attribute count limit checks.
    let max_vertex_buffers = gpu.limit_max_vertex_buffers() as usize;
    if self.bindings.len() > max_vertex_buffers {
      logging::error!(
        "Pipeline declares {} vertex buffers, exceeds device max {}",
        self.bindings.len(),
        max_vertex_buffers
      );
    }
    debug_assert!(
      self.bindings.len() <= max_vertex_buffers,
      "Pipeline declares {} vertex buffers, exceeds device max {}",
      self.bindings.len(),
      max_vertex_buffers
    );

    let total_vertex_attributes: usize = self
      .bindings
      .iter()
      .map(|binding| binding.attributes.len())
      .sum();
    let max_vertex_attributes = gpu.limit_max_vertex_attributes() as usize;
    if total_vertex_attributes > max_vertex_attributes {
      logging::error!(
        "Pipeline declares {} vertex attributes across all vertex buffers, exceeds device max {}",
        total_vertex_attributes,
        max_vertex_attributes
      );
    }
    debug_assert!(
      total_vertex_attributes <= max_vertex_attributes,
      "Pipeline declares {} vertex attributes across all vertex buffers, exceeds device max {}",
      total_vertex_attributes,
      max_vertex_attributes
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
      .with_immediate_data_ranges(immediate_data_ranges)
      .build(gpu.platform());

    // Vertex buffers and attributes
    let mut buffers = Vec::with_capacity(self.bindings.len());
    let mut per_instance_slots = Vec::with_capacity(self.bindings.len());
    let mut rp_builder = platform_pipeline::RenderPipelineBuilder::new()
      .with_label(self.label.as_deref().unwrap_or("lambda-render-pipeline"))
      .with_layout(&pipeline_layout)
      .with_cull_mode(self.culling.to_platform())
      .with_blend(self.blend_mode.to_platform());

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

      rp_builder = rp_builder.with_vertex_buffer_step_mode(
        binding.layout.stride,
        to_platform_step_mode(binding.layout.step_mode),
        attributes,
      );
      buffers.push(binding.buffer.clone());
      per_instance_slots.push(matches!(
        binding.layout.step_mode,
        VertexStepMode::PerInstance
      ));
    }

    if fragment_module.is_some() {
      rp_builder = rp_builder.with_color_target(surface_format.to_platform());
    }

    let pipeline_color_target_format = if fragment_module.is_some() {
      Some(surface_format)
    } else {
      None
    };

    let mut pipeline_depth_format: Option<texture::DepthFormat> = None;
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

      let requested_depth_format = dfmt.to_platform();

      // Derive the pass attachment depth format from pass configuration.
      let pass_has_stencil = render_pass.stencil_operations().is_some();
      let pass_depth_format = if pass_has_stencil {
        texture::DepthFormat::Depth24PlusStencil8
      } else {
        depth_format
      };
      pipeline_depth_format = Some(pass_depth_format);

      // Align the pipeline depth format with the pass attachment format to
      // avoid hidden global state on the render context. When formats differ,
      // prefer the pass attachment format and log for easier debugging.
      let final_depth_format = if requested_depth_format
        != pass_depth_format.to_platform()
      {
        #[cfg(any(
          debug_assertions,
          feature = "render-validation-depth",
          feature = "render-validation-stencil",
        ))]
        logging::error!(
            "Render pipeline depth format {:?} does not match pass depth attachment format {:?}; aligning pipeline to pass format",
            requested_depth_format,
            pass_depth_format
          );
        pass_depth_format.to_platform()
      } else {
        pass_depth_format.to_platform()
      };

      rp_builder = rp_builder.with_depth_stencil(final_depth_format);
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
    let pass_samples = render_pass.sample_count();
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
      gpu.platform(),
      &vertex_module,
      fragment_module.as_ref(),
    );

    return RenderPipeline {
      pipeline: Rc::new(pipeline),
      buffers,
      sample_count: pipeline_samples,
      color_target_count: if fragment_module.is_some() { 1 } else { 0 },
      color_target_format: pipeline_color_target_format,
      // Depth/stencil is enabled when `with_depth*` was called on the builder.
      expects_depth_stencil: self.use_depth,
      depth_format: pipeline_depth_format,
      uses_stencil: self.stencil.is_some(),
      per_instance_slots,
    };
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  /// Ensures vertex step modes map to the platform vertex step modes.
  #[test]
  fn engine_step_mode_maps_to_platform_step_mode() {
    let per_vertex = to_platform_step_mode(VertexStepMode::PerVertex);
    let per_instance = to_platform_step_mode(VertexStepMode::PerInstance);

    assert!(matches!(
      per_vertex,
      platform_pipeline::VertexStepMode::Vertex
    ));
    assert!(matches!(
      per_instance,
      platform_pipeline::VertexStepMode::Instance
    ));
  }

  /// Ensures depth compare functions map to the platform compare functions.
  #[test]
  fn compare_function_maps_to_platform() {
    assert!(matches!(
      CompareFunction::Less.to_platform(),
      platform_pipeline::CompareFunction::Less
    ));
    assert!(matches!(
      CompareFunction::Always.to_platform(),
      platform_pipeline::CompareFunction::Always
    ));
  }

  /// Ensures culling mode configuration maps to the platform culling modes.
  #[test]
  fn culling_mode_maps_to_platform() {
    assert!(matches!(
      CullingMode::None.to_platform(),
      platform_pipeline::CullingMode::None
    ));
    assert!(matches!(
      CullingMode::Back.to_platform(),
      platform_pipeline::CullingMode::Back
    ));
  }

  /// Ensures blend modes default to `None` and map to platform blend modes.
  #[test]
  fn blend_mode_defaults_and_maps_to_platform() {
    let builder = RenderPipelineBuilder::new();
    assert!(matches!(builder.blend_mode, BlendMode::None));

    assert!(matches!(
      BlendMode::None.to_platform(),
      platform_pipeline::BlendMode::None
    ));
    assert!(matches!(
      BlendMode::AlphaBlending.to_platform(),
      platform_pipeline::BlendMode::AlphaBlending
    ));
    assert!(matches!(
      BlendMode::PremultipliedAlpha.to_platform(),
      platform_pipeline::BlendMode::PremultipliedAlpha
    ));
    assert!(matches!(
      BlendMode::Additive.to_platform(),
      platform_pipeline::BlendMode::Additive
    ));
  }

  /// Ensures invalid MSAA sample counts are clamped/fallen back to `1`.
  #[test]
  fn pipeline_builder_invalid_sample_count_falls_back_to_one() {
    let builder = RenderPipelineBuilder::new().with_multi_sample(3);
    assert_eq!(builder.sample_count, 1);
  }

  /// Builds a pipeline with depth+stencil enabled and both per-vertex and
  /// per-instance buffers, covering format upgrade and instance slot tracking.
  #[test]
  fn pipeline_builds_with_depth_stencil_and_instance_layout() {
    use crate::render::{
      bind::{
        BindGroupLayoutBuilder,
        BindingVisibility,
      },
      buffer::{
        BufferBuilder,
        BufferType,
        Properties,
        Usage,
      },
      gpu::create_test_gpu,
      render_pass::RenderPassBuilder,
      shader::{
        ShaderBuilder,
        ShaderKind,
        VirtualShader,
      },
      texture::{
        DepthFormat,
        TextureFormat,
      },
      vertex::{
        ColorFormat,
        VertexAttribute,
        VertexElement,
      },
    };

    let Some(gpu) = create_test_gpu("lambda-pipeline-depth-test") else {
      return;
    };

    let mut shaders = ShaderBuilder::new();
    let vs = shaders.build(VirtualShader::Source {
      source: r#"
        #version 450
        #extension GL_ARB_separate_shader_objects : enable
        layout(location = 0) in vec3 a_pos;
        layout(location = 1) in vec3 a_inst;
        void main() { gl_Position = vec4(a_pos + a_inst * 0.0, 1.0); }
      "#
      .to_string(),
      kind: ShaderKind::Vertex,
      name: "lambda-pipeline-depth-vs".to_string(),
      entry_point: "main".to_string(),
    });
    let fs = shaders.build(VirtualShader::Source {
      source: r#"
        #version 450
        #extension GL_ARB_separate_shader_objects : enable
        layout(location = 0) out vec4 fragment_color;
        void main() { fragment_color = vec4(1.0); }
      "#
      .to_string(),
      kind: ShaderKind::Fragment,
      name: "lambda-pipeline-depth-fs".to_string(),
      entry_point: "main".to_string(),
    });

    let pass = RenderPassBuilder::new()
      .with_label("lambda-pipeline-depth-pass")
      .with_depth()
      .with_stencil()
      .build(&gpu, TextureFormat::Rgba8Unorm, DepthFormat::Depth24Plus);

    let layout = BindGroupLayoutBuilder::new()
      .with_uniform(0, BindingVisibility::VertexAndFragment)
      .build(&gpu);

    let vertex_buffer = BufferBuilder::new()
      .with_label("lambda-pipeline-depth-vertex")
      .with_usage(Usage::VERTEX)
      .with_properties(Properties::CPU_VISIBLE)
      .with_buffer_type(BufferType::Vertex)
      .build(&gpu, vec![[0.0_f32; 3]; 3])
      .expect("build vertex buffer");

    let instance_buffer = BufferBuilder::new()
      .with_label("lambda-pipeline-depth-instance")
      .with_usage(Usage::VERTEX)
      .with_properties(Properties::CPU_VISIBLE)
      .with_buffer_type(BufferType::Vertex)
      .build(&gpu, vec![[0.0_f32; 3]; 1])
      .expect("build instance buffer");

    let attrs_pos = vec![VertexAttribute {
      location: 0,
      offset: 0,
      element: VertexElement {
        format: ColorFormat::Rgb32Sfloat,
        offset: 0,
      },
    }];
    let attrs_inst = vec![VertexAttribute {
      location: 1,
      offset: 0,
      element: VertexElement {
        format: ColorFormat::Rgb32Sfloat,
        offset: 0,
      },
    }];

    let stencil = StencilState {
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
      read_mask: 0xff,
      write_mask: 0xff,
    };

    // Intentionally request a mismatched depth format; build should align to the pass.
    let pipeline = RenderPipelineBuilder::new()
      .with_label("lambda-pipeline-depth-pipeline")
      .with_layouts(&[&layout])
      .with_buffer(vertex_buffer, attrs_pos)
      .with_instance_buffer(instance_buffer, attrs_inst)
      .with_depth_format(DepthFormat::Depth32Float)
      .with_depth_compare(CompareFunction::Less)
      .with_depth_write(true)
      .with_stencil(stencil)
      .build(
        &gpu,
        TextureFormat::Rgba8Unorm,
        DepthFormat::Depth24Plus,
        &pass,
        &vs,
        Some(&fs),
      );

    assert!(pipeline.expects_depth_stencil());
    assert!(pipeline.uses_stencil());
    assert_eq!(
      pipeline.depth_format(),
      Some(DepthFormat::Depth24PlusStencil8)
    );
    assert_eq!(pipeline.per_instance_slots().len(), 2);
  }

  /// Ensures pipeline construction aligns its MSAA sample count to the render
  /// pass sample count to avoid target incompatibility.
  #[test]
  fn pipeline_build_aligns_sample_count_to_render_pass() {
    use crate::render::{
      buffer::{
        BufferBuilder,
        BufferType,
        Properties,
        Usage,
      },
      gpu::create_test_gpu,
      render_pass::RenderPassBuilder,
      shader::{
        ShaderBuilder,
        ShaderKind,
        VirtualShader,
      },
      texture::{
        DepthFormat,
        TextureFormat,
      },
      vertex::{
        ColorFormat,
        VertexAttribute,
        VertexElement,
      },
    };

    let Some(gpu) = create_test_gpu("lambda-pipeline-test") else {
      return;
    };

    let vert_path = format!(
      "{}/assets/shaders/triangle.vert",
      env!("CARGO_MANIFEST_DIR")
    );
    let frag_path = format!(
      "{}/assets/shaders/triangle.frag",
      env!("CARGO_MANIFEST_DIR")
    );
    let mut shaders = ShaderBuilder::new();
    let vs = shaders.build(VirtualShader::File {
      path: vert_path,
      kind: ShaderKind::Vertex,
      name: "triangle-vert".to_string(),
      entry_point: "main".to_string(),
    });
    let fs = shaders.build(VirtualShader::File {
      path: frag_path,
      kind: ShaderKind::Fragment,
      name: "triangle-frag".to_string(),
      entry_point: "main".to_string(),
    });

    let pass = RenderPassBuilder::new()
      .with_label("pipeline-sample-align-pass")
      .with_multi_sample(4)
      .build(&gpu, TextureFormat::Rgba8Unorm, DepthFormat::Depth24Plus);

    let vertex_buffer = BufferBuilder::new()
      .with_label("pipeline-test-vertex-buffer")
      .with_usage(Usage::VERTEX)
      .with_properties(Properties::CPU_VISIBLE)
      .with_buffer_type(BufferType::Vertex)
      .build(&gpu, vec![[0.0_f32; 3]; 4])
      .expect("build vertex buffer");

    let attributes = vec![VertexAttribute {
      location: 0,
      offset: 0,
      element: VertexElement {
        format: ColorFormat::Rgb32Sfloat,
        offset: 0,
      },
    }];

    // Intentionally request a different sample count; build should align to the pass.
    let pipeline = RenderPipelineBuilder::new()
      .with_label("pipeline-sample-align-pipeline")
      .with_multi_sample(1)
      .with_buffer(vertex_buffer, attributes)
      .build(
        &gpu,
        TextureFormat::Rgba8Unorm,
        DepthFormat::Depth24Plus,
        &pass,
        &vs,
        Some(&fs),
      );

    assert_eq!(pipeline.sample_count(), 4);
    assert!(pipeline.has_color_targets());
    assert_eq!(
      pipeline.color_target_format(),
      Some(TextureFormat::Rgba8Unorm)
    );
  }
}
