//! Render pipeline builders and definitions for lambda runtimes and
//! applications.

use std::{
  ops::Range,
  rc::Rc,
};

use lambda_platform::wgpu::{
  pipeline as platform_pipeline,
  types as wgpu,
};

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
pub struct RenderPipeline {
  pipeline: Rc<wgpu::RenderPipeline>,
  buffers: Vec<Rc<Buffer>>,
}

impl RenderPipeline {
  /// Destroy the render pipeline with the render context that created it.
  pub fn destroy(self, _render_context: &RenderContext) {}

  /// Access the vertex buffers associated with this pipeline.
  pub(super) fn buffers(&self) -> &Vec<Rc<Buffer>> {
    return &self.buffers;
  }

  /// Access the underlying wgpu render pipeline.
  pub(super) fn pipeline(&self) -> &wgpu::RenderPipeline {
    return self.pipeline.as_ref();
  }
}

#[derive(Clone, Copy, Debug)]
/// Bitflag wrapper for shader stages used by push constants.
pub struct PipelineStage(wgpu::ShaderStages);

impl PipelineStage {
  /// Vertex stage.
  pub const VERTEX: PipelineStage = PipelineStage(wgpu::ShaderStages::VERTEX);
  /// Fragment stage.
  pub const FRAGMENT: PipelineStage =
    PipelineStage(wgpu::ShaderStages::FRAGMENT);
  /// Compute stage.
  pub const COMPUTE: PipelineStage = PipelineStage(wgpu::ShaderStages::COMPUTE);

  pub(crate) fn to_wgpu(self) -> wgpu::ShaderStages {
    return self.0;
  }
}

/// Bitwise OR for combining pipeline stages.
impl std::ops::BitOr for PipelineStage {
  type Output = PipelineStage;

  fn bitor(self, rhs: PipelineStage) -> PipelineStage {
    return PipelineStage(self.0 | rhs.0);
  }
}

/// Bitwise OR assignment for combining pipeline stages.
impl std::ops::BitOrAssign for PipelineStage {
  fn bitor_assign(&mut self, rhs: PipelineStage) {
    self.0 |= rhs.0;
  }
}

/// Convenience alias for uploading push constants: stage and byte range.
pub type PushConstantUpload = (PipelineStage, Range<u32>);

struct BufferBinding {
  buffer: Rc<Buffer>,
  attributes: Vec<VertexAttribute>,
}

#[derive(Clone, Copy, Debug)]
/// Controls triangle face culling for the graphics pipeline.
pub enum CullingMode {
  /// Disable face culling; render both triangle faces.
  None,
  /// Cull triangles whose winding is counterclockwise after projection.
  Front,
  /// Cull triangles whose winding is clockwise after projection.
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

/// Builder for creating a graphics `RenderPipeline`.
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
    let device = render_context.device();
    let surface_format = render_context.surface_format();

    // Shader modules
    let vertex_module = platform_pipeline::ShaderModule::from_spirv(
      device,
      &vertex_shader.as_binary(),
      Some("lambda-vertex-shader"),
    );
    let fragment_module = fragment_shader.map(|shader| {
      platform_pipeline::ShaderModule::from_spirv(
        device,
        &shader.as_binary(),
        Some("lambda-fragment-shader"),
      )
    });

    // Push constant ranges
    let push_constant_ranges: Vec<wgpu::PushConstantRange> = self
      .push_constants
      .iter()
      .map(|(stage, range)| wgpu::PushConstantRange {
        stages: stage.to_wgpu(),
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
    let bgl_raw: Vec<&wgpu::BindGroupLayout> =
      self.bind_group_layouts.iter().map(|l| l.raw()).collect();
    let pipeline_layout = platform_pipeline::PipelineLayoutBuilder::new()
      .with_label("lambda-pipeline-layout")
      .with_layouts(&bgl_raw)
      .with_push_constants(push_constant_ranges)
      .build(device);

    // Vertex buffers and attributes
    let mut buffers = Vec::with_capacity(self.bindings.len());
    let mut rp_builder = platform_pipeline::RenderPipelineBuilder::new()
      .with_label(self.label.as_deref().unwrap_or("lambda-render-pipeline"))
      .with_layout(&pipeline_layout)
      .with_cull_mode(self.culling.to_wgpu());

    for binding in &self.bindings {
      let attributes: Vec<wgpu::VertexAttribute> = binding
        .attributes
        .iter()
        .map(|attribute| wgpu::VertexAttribute {
          shader_location: attribute.location,
          offset: (attribute.offset + attribute.element.offset) as u64,
          format: attribute.element.format.to_vertex_format(),
        })
        .collect();
      rp_builder =
        rp_builder.with_vertex_buffer(binding.buffer.stride(), attributes);
      buffers.push(binding.buffer.clone());
    }

    if fragment_module.is_some() {
      rp_builder = rp_builder.with_color_target(wgpu::ColorTargetState {
        format: surface_format,
        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
        write_mask: wgpu::ColorWrites::ALL,
      });
    }

    let rp = rp_builder.build(device, &vertex_module, fragment_module.as_ref());

    return RenderPipeline {
      pipeline: Rc::new(rp.into_raw()),
      buffers,
    };
  }
}
