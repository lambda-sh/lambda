//! Render pipeline builders and definitions for lambda runtimes and
//! applications.

use std::{
  borrow::Cow,
  ops::Range,
  rc::Rc,
};

use lambda_platform::wgpu::types as wgpu;

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

  pub(super) fn buffers(&self) -> &Vec<Rc<Buffer>> {
    &self.buffers
  }

  pub(super) fn pipeline(&self) -> &wgpu::RenderPipeline {
    self.pipeline.as_ref()
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
    self.0
  }
}

impl std::ops::BitOr for PipelineStage {
  type Output = PipelineStage;

  fn bitor(self, rhs: PipelineStage) -> PipelineStage {
    PipelineStage(self.0 | rhs.0)
  }
}

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
    match self {
      CullingMode::None => None,
      CullingMode::Front => Some(wgpu::Face::Front),
      CullingMode::Back => Some(wgpu::Face::Back),
    }
  }
}

/// Builder for creating a graphics `RenderPipeline`.
pub struct RenderPipelineBuilder {
  push_constants: Vec<PushConstantUpload>,
  bindings: Vec<BufferBinding>,
  culling: CullingMode,
  bind_group_layouts: Vec<std::rc::Rc<wgpu::BindGroupLayout>>,
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
    self
  }

  /// Declare a push constant range for a shader stage in bytes.
  pub fn with_push_constant(
    mut self,
    stage: PipelineStage,
    bytes: u32,
  ) -> Self {
    self.push_constants.push((stage, 0..bytes));
    self
  }

  /// Attach a debug label to the pipeline.
  pub fn with_label(mut self, label: &str) -> Self {
    self.label = Some(label.to_string());
    self
  }

  /// Configure triangle face culling. Defaults to culling back faces.
  pub fn with_culling(mut self, mode: CullingMode) -> Self {
    self.culling = mode;
    self
  }

  /// Provide one or more bind group layouts used to create the pipeline layout.
  pub fn with_layouts(mut self, layouts: &[&bind::BindGroupLayout]) -> Self {
    self.bind_group_layouts = layouts
      .iter()
      .map(|l| std::rc::Rc::new(l.raw().clone()))
      .collect();
    self
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

    let vertex_shader_module =
      device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("lambda-vertex-shader"),
        source: wgpu::ShaderSource::SpirV(Cow::Owned(
          vertex_shader.as_binary(),
        )),
      });

    let fragment_shader_module = fragment_shader.map(|shader| {
      device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("lambda-fragment-shader"),
        source: wgpu::ShaderSource::SpirV(Cow::Owned(shader.as_binary())),
      })
    });

    let push_constant_ranges: Vec<wgpu::PushConstantRange> = self
      .push_constants
      .iter()
      .map(|(stage, range)| wgpu::PushConstantRange {
        stages: stage.to_wgpu(),
        range: range.clone(),
      })
      .collect();

    let bind_group_layout_refs: Vec<&wgpu::BindGroupLayout> = self
      .bind_group_layouts
      .iter()
      .map(|rc| rc.as_ref())
      .collect();
    let pipeline_layout =
      device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("lambda-pipeline-layout"),
        bind_group_layouts: &bind_group_layout_refs,
        push_constant_ranges: &push_constant_ranges,
      });

    let mut attribute_storage: Vec<Box<[wgpu::VertexAttribute]>> =
      Vec::with_capacity(self.bindings.len());
    let mut vertex_buffer_layouts: Vec<wgpu::VertexBufferLayout<'_>> =
      Vec::with_capacity(self.bindings.len());
    let mut buffers = Vec::with_capacity(self.bindings.len());

    // First, collect attributes and buffers
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
      attribute_storage.push(attributes.into_boxed_slice());
      buffers.push(binding.buffer.clone());
    }

    // Then, build layouts referencing the stable storage
    for (i, binding) in self.bindings.iter().enumerate() {
      let attributes_slice = attribute_storage[i].as_ref();
      vertex_buffer_layouts.push(wgpu::VertexBufferLayout {
        array_stride: binding.buffer.stride(),
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: attributes_slice,
      });
    }

    // Stable storage for color targets to satisfy borrow checker
    let mut color_targets: Vec<Option<wgpu::ColorTargetState>> = Vec::new();
    if fragment_shader_module.is_some() {
      color_targets.push(Some(wgpu::ColorTargetState {
        format: surface_format,
        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
        write_mask: wgpu::ColorWrites::ALL,
      }));
    }

    let fragment =
      fragment_shader_module
        .as_ref()
        .map(|module| wgpu::FragmentState {
          module,
          entry_point: Some("main"),
          compilation_options: wgpu::PipelineCompilationOptions::default(),
          targets: color_targets.as_slice(),
        });

    let vertex_state = wgpu::VertexState {
      module: &vertex_shader_module,
      entry_point: Some("main"),
      compilation_options: wgpu::PipelineCompilationOptions::default(),
      buffers: vertex_buffer_layouts.as_slice(),
    };

    let primitive_state = wgpu::PrimitiveState {
      cull_mode: self.culling.to_wgpu(),
      ..wgpu::PrimitiveState::default()
    };

    let pipeline_descriptor = wgpu::RenderPipelineDescriptor {
      label: self.label.as_deref(),
      layout: Some(&pipeline_layout),
      vertex: vertex_state,
      primitive: primitive_state,
      depth_stencil: None,
      multisample: wgpu::MultisampleState::default(),
      fragment,
      multiview: None,
      cache: None,
    };

    let pipeline = device.create_render_pipeline(&pipeline_descriptor);

    RenderPipeline {
      pipeline: Rc::new(pipeline),
      buffers,
    }
  }
}
