//! Pipeline and shader module wrappers/builders for the platform layer.

use std::ops::Range;

use wgpu;

pub use crate::wgpu::vertex::VertexStepMode;
use crate::wgpu::{
  bind,
  gpu::Gpu,
  texture::{
    DepthFormat,
    TextureFormat,
  },
  vertex::ColorFormat,
};

/// Shader stage flags for immediate data and visibility.
#[derive(Clone, Copy, Debug)]
///
/// This wrapper avoids exposing `wgpu` directly to higher layers while still
/// allowing flexible combinations when needed.
pub struct PipelineStage(wgpu::ShaderStages);

impl PipelineStage {
  /// Vertex stage only.
  pub const VERTEX: PipelineStage = PipelineStage(wgpu::ShaderStages::VERTEX);
  /// Fragment stage only.
  pub const FRAGMENT: PipelineStage =
    PipelineStage(wgpu::ShaderStages::FRAGMENT);
  /// Compute stage only.
  pub const COMPUTE: PipelineStage = PipelineStage(wgpu::ShaderStages::COMPUTE);

  /// Internal mapping to the underlying graphics API.
  pub fn to_wgpu(self) -> wgpu::ShaderStages {
    return self.0;
  }
}

impl std::ops::BitOr for PipelineStage {
  type Output = PipelineStage;

  fn bitor(self, rhs: PipelineStage) -> PipelineStage {
    return PipelineStage(self.0 | rhs.0);
  }
}

impl std::ops::BitOrAssign for PipelineStage {
  fn bitor_assign(&mut self, rhs: PipelineStage) {
    self.0 |= rhs.0;
  }
}

/// Immediate data declaration for a stage and byte range.
#[derive(Clone, Debug)]
pub struct ImmediateDataRange {
  pub stages: PipelineStage,
  pub range: Range<u32>,
}

/// Face culling mode for graphics pipelines.
#[derive(Clone, Copy, Debug)]
pub enum CullingMode {
  None,
  Front,
  Back,
}

impl CullingMode {
  fn to_wgpu(self) -> Option<wgpu::Face> {
    return match self {
      CullingMode::None => None,
      CullingMode::Front => Some(wgpu::Face::Front),
      CullingMode::Back => Some(wgpu::Face::Back),
    };
  }
}

/// Description of a single vertex attribute used by a pipeline.
#[derive(Clone, Copy, Debug)]
pub struct VertexAttributeDesc {
  pub shader_location: u32,
  pub offset: u64,
  pub format: ColorFormat,
}

/// Description of a single vertex buffer layout used by a pipeline.
#[derive(Clone, Debug)]
struct VertexBufferLayoutDesc {
  array_stride: u64,
  step_mode: VertexStepMode,
  attributes: Vec<VertexAttributeDesc>,
}

/// Compare function used for depth and stencil tests.
#[derive(Clone, Copy, Debug)]
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
  fn to_wgpu(self) -> wgpu::CompareFunction {
    match self {
      CompareFunction::Never => wgpu::CompareFunction::Never,
      CompareFunction::Less => wgpu::CompareFunction::Less,
      CompareFunction::LessEqual => wgpu::CompareFunction::LessEqual,
      CompareFunction::Greater => wgpu::CompareFunction::Greater,
      CompareFunction::GreaterEqual => wgpu::CompareFunction::GreaterEqual,
      CompareFunction::Equal => wgpu::CompareFunction::Equal,
      CompareFunction::NotEqual => wgpu::CompareFunction::NotEqual,
      CompareFunction::Always => wgpu::CompareFunction::Always,
    }
  }
}

/// Stencil operation applied when the stencil test or depth test passes/fails.
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
  fn to_wgpu(self) -> wgpu::StencilOperation {
    match self {
      StencilOperation::Keep => wgpu::StencilOperation::Keep,
      StencilOperation::Zero => wgpu::StencilOperation::Zero,
      StencilOperation::Replace => wgpu::StencilOperation::Replace,
      StencilOperation::Invert => wgpu::StencilOperation::Invert,
      StencilOperation::IncrementClamp => {
        wgpu::StencilOperation::IncrementClamp
      }
      StencilOperation::DecrementClamp => {
        wgpu::StencilOperation::DecrementClamp
      }
      StencilOperation::IncrementWrap => wgpu::StencilOperation::IncrementWrap,
      StencilOperation::DecrementWrap => wgpu::StencilOperation::DecrementWrap,
    }
  }
}

/// Per-face stencil state.
#[derive(Clone, Copy, Debug)]
pub struct StencilFaceState {
  pub compare: CompareFunction,
  pub fail_op: StencilOperation,
  pub depth_fail_op: StencilOperation,
  pub pass_op: StencilOperation,
}

impl StencilFaceState {
  fn to_wgpu(self) -> wgpu::StencilFaceState {
    wgpu::StencilFaceState {
      compare: self.compare.to_wgpu(),
      fail_op: self.fail_op.to_wgpu(),
      depth_fail_op: self.depth_fail_op.to_wgpu(),
      pass_op: self.pass_op.to_wgpu(),
    }
  }
}

/// Full stencil state (front/back + masks).
#[derive(Clone, Copy, Debug)]
pub struct StencilState {
  pub front: StencilFaceState,
  pub back: StencilFaceState,
  pub read_mask: u32,
  pub write_mask: u32,
}

/// Wrapper around `wgpu::ShaderModule` that preserves a label.
#[derive(Debug)]
pub struct ShaderModule {
  raw: wgpu::ShaderModule,
  label: Option<String>,
}

impl ShaderModule {
  /// Create a shader module from SPIR-V words.
  pub fn from_spirv(gpu: &Gpu, words: &[u32], label: Option<&str>) -> Self {
    let raw = gpu
      .device()
      .create_shader_module(wgpu::ShaderModuleDescriptor {
        label,
        source: wgpu::ShaderSource::SpirV(std::borrow::Cow::Borrowed(words)),
      });
    return Self {
      raw,
      label: label.map(|s| s.to_string()),
    };
  }

  /// Borrow the raw shader module.
  pub fn raw(&self) -> &wgpu::ShaderModule {
    &self.raw
  }
}

/// Wrapper around `wgpu::PipelineLayout`.
#[derive(Debug)]
pub struct PipelineLayout {
  raw: wgpu::PipelineLayout,
  label: Option<String>,
}

impl PipelineLayout {
  /// Borrow the raw pipeline layout.
  pub fn raw(&self) -> &wgpu::PipelineLayout {
    return &self.raw;
  }
}

/// Builder for creating a `PipelineLayout`.
pub struct PipelineLayoutBuilder<'a> {
  label: Option<String>,
  layouts: Vec<&'a bind::BindGroupLayout>,
  immediate_data_ranges: Vec<ImmediateDataRange>,
}

impl<'a> PipelineLayoutBuilder<'a> {
  /// New builder with no layouts or immediate data.
  pub fn new() -> Self {
    return Self {
      label: None,
      layouts: Vec::new(),
      immediate_data_ranges: Vec::new(),
    };
  }

  /// Attach a label.
  pub fn with_label(mut self, label: &str) -> Self {
    self.label = Some(label.to_string());
    return self;
  }

  /// Provide bind group layouts.
  pub fn with_layouts(mut self, layouts: &'a [&bind::BindGroupLayout]) -> Self {
    self.layouts = layouts.to_vec();
    return self;
  }

  /// Provide immediate data ranges for shader stages.
  pub fn with_immediate_data_ranges(
    mut self,
    ranges: Vec<ImmediateDataRange>,
  ) -> Self {
    self.immediate_data_ranges = ranges;
    return self;
  }

  /// Build the layout.
  pub fn build(self, gpu: &Gpu) -> PipelineLayout {
    let layouts_raw: Vec<&wgpu::BindGroupLayout> =
      self.layouts.iter().map(|l| l.raw()).collect();

    // Calculate the total immediate size from immediate data ranges.
    // The immediate_size is the maximum end offset across all ranges.
    let immediate_size = self
      .immediate_data_ranges
      .iter()
      .map(|r| r.range.end)
      .max()
      .unwrap_or(0);

    let raw =
      gpu
        .device()
        .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
          label: self.label.as_deref(),
          bind_group_layouts: &layouts_raw,
          immediate_size,
        });
    return PipelineLayout {
      raw,
      label: self.label,
    };
  }
}

/// Wrapper around `wgpu::RenderPipeline`.
#[derive(Debug)]
pub struct RenderPipeline {
  raw: wgpu::RenderPipeline,
  label: Option<String>,
}

impl RenderPipeline {
  /// Borrow the raw pipeline.
  pub(crate) fn raw(&self) -> &wgpu::RenderPipeline {
    return &self.raw;
  }
  /// Consume and return the raw pipeline.
  pub(crate) fn into_raw(self) -> wgpu::RenderPipeline {
    return self.raw;
  }
  /// Pipeline label if provided.
  pub fn label(&self) -> Option<&str> {
    return self.label.as_deref();
  }
}

/// Builder for creating a graphics render pipeline.
pub struct RenderPipelineBuilder<'a> {
  label: Option<String>,
  layout: Option<&'a wgpu::PipelineLayout>,
  vertex_buffers: Vec<VertexBufferLayoutDesc>,
  cull_mode: CullingMode,
  color_target_format: Option<wgpu::TextureFormat>,
  depth_stencil: Option<wgpu::DepthStencilState>,
  sample_count: u32,
}

impl<'a> RenderPipelineBuilder<'a> {
  /// New builder with defaults.
  pub fn new() -> Self {
    return Self {
      label: None,
      layout: None,
      vertex_buffers: Vec::new(),
      cull_mode: CullingMode::Back,
      color_target_format: None,
      depth_stencil: None,
      sample_count: 1,
    };
  }

  /// Attach a label.
  pub fn with_label(mut self, label: &str) -> Self {
    self.label = Some(label.to_string());
    return self;
  }

  /// Use the provided pipeline layout.
  pub fn with_layout(mut self, layout: &'a PipelineLayout) -> Self {
    self.layout = Some(layout.raw());
    return self;
  }

  /// Add a vertex buffer layout with attributes.
  pub fn with_vertex_buffer(
    mut self,
    array_stride: u64,
    attributes: Vec<VertexAttributeDesc>,
  ) -> Self {
    self = self.with_vertex_buffer_step_mode(
      array_stride,
      VertexStepMode::Vertex,
      attributes,
    );
    return self;
  }

  /// Add a vertex buffer layout with attributes and an explicit step mode.
  pub fn with_vertex_buffer_step_mode(
    mut self,
    array_stride: u64,
    step_mode: VertexStepMode,
    attributes: Vec<VertexAttributeDesc>,
  ) -> Self {
    self.vertex_buffers.push(VertexBufferLayoutDesc {
      array_stride,
      step_mode,
      attributes,
    });
    return self;
  }

  /// Set cull mode (None disables culling).
  pub fn with_cull_mode(mut self, mode: CullingMode) -> Self {
    self.cull_mode = mode;
    return self;
  }

  /// Set single color target for fragment stage from a texture format.
  pub fn with_color_target(mut self, format: TextureFormat) -> Self {
    self.color_target_format = Some(format.to_wgpu());
    return self;
  }

  /// Enable depth testing/writes using the provided depth format and default compare/write settings.
  ///
  /// Defaults: compare Less, depth writes enabled, no stencil.
  pub fn with_depth_stencil(mut self, format: DepthFormat) -> Self {
    self.depth_stencil = Some(wgpu::DepthStencilState {
      format: format.to_wgpu(),
      depth_write_enabled: true,
      depth_compare: wgpu::CompareFunction::Less,
      stencil: wgpu::StencilState::default(),
      bias: wgpu::DepthBiasState::default(),
    });
    return self;
  }

  /// Set the depth compare function. Requires depth to be enabled.
  pub fn with_depth_compare(mut self, compare: CompareFunction) -> Self {
    let ds = self.depth_stencil.get_or_insert(wgpu::DepthStencilState {
      format: wgpu::TextureFormat::Depth32Float,
      depth_write_enabled: true,
      depth_compare: wgpu::CompareFunction::Less,
      stencil: wgpu::StencilState::default(),
      bias: wgpu::DepthBiasState::default(),
    });
    ds.depth_compare = compare.to_wgpu();
    return self;
  }

  /// Enable or disable depth writes. Requires depth-stencil enabled.
  pub fn with_depth_write_enabled(mut self, enabled: bool) -> Self {
    let ds = self.depth_stencil.get_or_insert(wgpu::DepthStencilState {
      format: wgpu::TextureFormat::Depth32Float,
      depth_write_enabled: true,
      depth_compare: wgpu::CompareFunction::Less,
      stencil: wgpu::StencilState::default(),
      bias: wgpu::DepthBiasState::default(),
    });
    ds.depth_write_enabled = enabled;
    return self;
  }

  /// Configure stencil state (front/back ops and masks). Requires depth-stencil enabled.
  pub fn with_stencil(mut self, stencil: StencilState) -> Self {
    let ds = self.depth_stencil.get_or_insert(wgpu::DepthStencilState {
      format: wgpu::TextureFormat::Depth24PlusStencil8,
      depth_write_enabled: true,
      depth_compare: wgpu::CompareFunction::Less,
      stencil: wgpu::StencilState::default(),
      bias: wgpu::DepthBiasState::default(),
    });
    ds.stencil = wgpu::StencilState {
      front: stencil.front.to_wgpu(),
      back: stencil.back.to_wgpu(),
      read_mask: stencil.read_mask,
      write_mask: stencil.write_mask,
    };
    return self;
  }

  /// Configure multisampling. Count MUST be >= 1 and supported by the device.
  pub fn with_sample_count(mut self, count: u32) -> Self {
    self.sample_count = count.max(1);
    return self;
  }

  /// Build the render pipeline from provided shader modules.
  pub fn build(
    self,
    gpu: &Gpu,
    vertex_shader: &ShaderModule,
    fragment_shader: Option<&ShaderModule>,
  ) -> RenderPipeline {
    // Convert vertex attributes into raw `wgpu` descriptors while keeping
    // storage stable for layout lifetimes.
    let mut attr_storage: Vec<Box<[wgpu::VertexAttribute]>> = Vec::new();
    let mut strides: Vec<u64> = Vec::new();
    let mut step_modes: Vec<VertexStepMode> = Vec::new();
    for buffer_desc in &self.vertex_buffers {
      let mut raw_attrs: Vec<wgpu::VertexAttribute> =
        Vec::with_capacity(buffer_desc.attributes.len());

      for attribute in buffer_desc.attributes.iter() {
        raw_attrs.push(wgpu::VertexAttribute {
          shader_location: attribute.shader_location,
          offset: attribute.offset,
          format: attribute.format.to_vertex_format(),
        });
      }
      let boxed: Box<[wgpu::VertexAttribute]> = raw_attrs.into_boxed_slice();
      attr_storage.push(boxed);
      strides.push(buffer_desc.array_stride);
      step_modes.push(buffer_desc.step_mode);
    }
    // Now build layouts referencing the stable storage in `attr_storage`.
    let mut vbl: Vec<wgpu::VertexBufferLayout<'_>> = Vec::new();
    for (i, boxed) in attr_storage.iter().enumerate() {
      let slice = boxed.as_ref();
      vbl.push(wgpu::VertexBufferLayout {
        array_stride: strides[i],
        step_mode: step_modes[i].to_wgpu(),
        attributes: slice,
      });
    }

    let color_targets: Vec<Option<wgpu::ColorTargetState>> =
      match &self.color_target_format {
        Some(fmt) => vec![Some(wgpu::ColorTargetState {
          format: *fmt,
          blend: Some(wgpu::BlendState::ALPHA_BLENDING),
          write_mask: wgpu::ColorWrites::ALL,
        })],
        None => Vec::new(),
      };

    let fragment = fragment_shader.map(|fs| wgpu::FragmentState {
      module: fs.raw(),
      entry_point: Some("main"),
      compilation_options: wgpu::PipelineCompilationOptions::default(),
      targets: color_targets.as_slice(),
    });

    let vertex_state = wgpu::VertexState {
      module: vertex_shader.raw(),
      entry_point: Some("main"),
      compilation_options: wgpu::PipelineCompilationOptions::default(),
      buffers: vbl.as_slice(),
    };

    let primitive_state = wgpu::PrimitiveState {
      cull_mode: self.cull_mode.to_wgpu(),
      ..wgpu::PrimitiveState::default()
    };

    let layout_ref = self.layout;
    let raw =
      gpu
        .device()
        .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
          label: self.label.as_deref(),
          layout: layout_ref,
          vertex: vertex_state,
          primitive: primitive_state,
          depth_stencil: self.depth_stencil,
          multisample: wgpu::MultisampleState {
            count: self.sample_count,
            ..wgpu::MultisampleState::default()
          },
          fragment,
          multiview_mask: None,
          cache: None,
        });

    return RenderPipeline {
      raw,
      label: self.label,
    };
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn vertex_step_mode_maps_to_wgpu() {
    let vertex_mode = VertexStepMode::Vertex.to_wgpu();
    let instance_mode = VertexStepMode::Instance.to_wgpu();

    assert_eq!(vertex_mode, wgpu::VertexStepMode::Vertex);
    assert_eq!(instance_mode, wgpu::VertexStepMode::Instance);
  }

  #[test]
  fn with_vertex_buffer_defaults_to_per_vertex_step_mode() {
    let builder = RenderPipelineBuilder::new().with_vertex_buffer(
      16,
      vec![VertexAttributeDesc {
        shader_location: 0,
        offset: 0,
        format: ColorFormat::Rgb32Sfloat,
      }],
    );

    let vertex_buffers = &builder.vertex_buffers;

    assert_eq!(vertex_buffers.len(), 1);
    assert!(matches!(
      vertex_buffers[0].step_mode,
      VertexStepMode::Vertex
    ));
  }
}
