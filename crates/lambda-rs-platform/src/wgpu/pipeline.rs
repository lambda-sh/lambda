//! Pipeline and shader module wrappers/builders for the platform layer.

use std::ops::Range;

use wgpu;

use crate::wgpu::{
  bind,
  gpu::Gpu,
  surface::SurfaceFormat,
  texture::DepthFormat,
  vertex::ColorFormat,
};

/// Shader stage flags for push constants and visibility.
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

/// Push constant declaration for a stage and byte range.
#[derive(Clone, Debug)]
pub struct PushConstantRange {
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
  push_constant_ranges: Vec<PushConstantRange>,
}

impl<'a> PipelineLayoutBuilder<'a> {
  /// New builder with no layouts or push constants.
  pub fn new() -> Self {
    return Self {
      label: None,
      layouts: Vec::new(),
      push_constant_ranges: Vec::new(),
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

  /// Provide push constant ranges.
  pub fn with_push_constants(mut self, ranges: Vec<PushConstantRange>) -> Self {
    self.push_constant_ranges = ranges;
    return self;
  }

  /// Build the layout.
  pub fn build(self, gpu: &Gpu) -> PipelineLayout {
    let layouts_raw: Vec<&wgpu::BindGroupLayout> =
      self.layouts.iter().map(|l| l.raw()).collect();
    let push_constants_raw: Vec<wgpu::PushConstantRange> = self
      .push_constant_ranges
      .iter()
      .map(|pcr| wgpu::PushConstantRange {
        stages: pcr.stages.to_wgpu(),
        range: pcr.range.clone(),
      })
      .collect();

    let raw =
      gpu
        .device()
        .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
          label: self.label.as_deref(),
          bind_group_layouts: &layouts_raw,
          push_constant_ranges: &push_constants_raw,
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
}

/// Builder for creating a graphics render pipeline.
pub struct RenderPipelineBuilder<'a> {
  label: Option<String>,
  layout: Option<&'a wgpu::PipelineLayout>,
  vertex_buffers: Vec<(u64, Vec<VertexAttributeDesc>)>,
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
    self.vertex_buffers.push((array_stride, attributes));
    return self;
  }

  /// Set cull mode (None disables culling).
  pub fn with_cull_mode(mut self, mode: CullingMode) -> Self {
    self.cull_mode = mode;
    return self;
  }

  /// Set single color target for fragment stage from a surface format.
  pub fn with_surface_color_target(mut self, format: SurfaceFormat) -> Self {
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
    for (stride, attrs) in &self.vertex_buffers {
      let mut raw_attrs: Vec<wgpu::VertexAttribute> =
        Vec::with_capacity(attrs.len());

      for attribute in attrs.iter() {
        raw_attrs.push(wgpu::VertexAttribute {
          shader_location: attribute.shader_location,
          offset: attribute.offset,
          format: attribute.format.to_vertex_format(),
        });
      }
      let boxed: Box<[wgpu::VertexAttribute]> = raw_attrs.into_boxed_slice();
      attr_storage.push(boxed);
      strides.push(*stride);
    }
    // Now build layouts referencing the stable storage in `attr_storage`.
    let mut vbl: Vec<wgpu::VertexBufferLayout<'_>> = Vec::new();
    for (i, boxed) in attr_storage.iter().enumerate() {
      let slice = boxed.as_ref();
      vbl.push(wgpu::VertexBufferLayout {
        array_stride: strides[i],
        step_mode: wgpu::VertexStepMode::Vertex,
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
          multiview: None,
          cache: None,
        });

    return RenderPipeline {
      raw,
      label: self.label,
    };
  }
}
