//! Pipeline and shader module wrappers/builders for the platform layer.

use crate::wgpu::types as wgpu;

#[derive(Debug)]
/// Wrapper around `wgpu::ShaderModule` that preserves a label.
pub struct ShaderModule {
  raw: wgpu::ShaderModule,
  label: Option<String>,
}

impl ShaderModule {
  /// Create a shader module from SPIR-V words.
  pub fn from_spirv(
    device: &wgpu::Device,
    words: &[u32],
    label: Option<&str>,
  ) -> Self {
    let raw = device.create_shader_module(wgpu::ShaderModuleDescriptor {
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

#[derive(Debug)]
/// Wrapper around `wgpu::PipelineLayout`.
pub struct PipelineLayout {
  raw: wgpu::PipelineLayout,
  label: Option<String>,
}

impl PipelineLayout {
  /// Borrow the raw pipeline layout.
  pub fn raw(&self) -> &wgpu::PipelineLayout {
    &self.raw
  }
}

/// Builder for creating a `PipelineLayout`.
pub struct PipelineLayoutBuilder<'a> {
  label: Option<String>,
  layouts: Vec<&'a wgpu::BindGroupLayout>,
  push_constant_ranges: Vec<wgpu::PushConstantRange>,
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
    self
  }

  /// Provide bind group layouts.
  pub fn with_layouts(mut self, layouts: &'a [&wgpu::BindGroupLayout]) -> Self {
    self.layouts = layouts.to_vec();
    self
  }

  /// Provide push constant ranges.
  pub fn with_push_constants(
    mut self,
    ranges: Vec<wgpu::PushConstantRange>,
  ) -> Self {
    self.push_constant_ranges = ranges;
    self
  }

  /// Build the layout.
  pub fn build(self, device: &wgpu::Device) -> PipelineLayout {
    let raw = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: self.label.as_deref(),
      bind_group_layouts: &self.layouts,
      push_constant_ranges: &self.push_constant_ranges,
    });
    return PipelineLayout {
      raw,
      label: self.label,
    };
  }
}

#[derive(Debug)]
/// Wrapper around `wgpu::RenderPipeline`.
pub struct RenderPipeline {
  raw: wgpu::RenderPipeline,
  label: Option<String>,
}

impl RenderPipeline {
  /// Borrow the raw pipeline.
  pub fn raw(&self) -> &wgpu::RenderPipeline {
    &self.raw
  }
  /// Consume and return the raw pipeline.
  pub fn into_raw(self) -> wgpu::RenderPipeline {
    self.raw
  }
}

/// Builder for creating a graphics render pipeline.
pub struct RenderPipelineBuilder<'a> {
  label: Option<String>,
  layout: Option<&'a wgpu::PipelineLayout>,
  vertex_buffers: Vec<(u64, Vec<wgpu::VertexAttribute>)>,
  cull_mode: Option<wgpu::Face>,
  color_target: Option<wgpu::ColorTargetState>,
}

impl<'a> RenderPipelineBuilder<'a> {
  /// New builder with defaults.
  pub fn new() -> Self {
    return Self {
      label: None,
      layout: None,
      vertex_buffers: Vec::new(),
      cull_mode: Some(wgpu::Face::Back),
      color_target: None,
    };
  }

  /// Attach a label.
  pub fn with_label(mut self, label: &str) -> Self {
    self.label = Some(label.to_string());
    self
  }

  /// Use the provided pipeline layout.
  pub fn with_layout(mut self, layout: &'a PipelineLayout) -> Self {
    self.layout = Some(layout.raw());
    self
  }

  /// Add a vertex buffer layout with attributes.
  pub fn with_vertex_buffer(
    mut self,
    array_stride: u64,
    attributes: Vec<wgpu::VertexAttribute>,
  ) -> Self {
    self.vertex_buffers.push((array_stride, attributes));
    self
  }

  /// Set cull mode (None disables culling).
  pub fn with_cull_mode(mut self, face: Option<wgpu::Face>) -> Self {
    self.cull_mode = face;
    self
  }

  /// Set single color target for fragment stage.
  pub fn with_color_target(mut self, target: wgpu::ColorTargetState) -> Self {
    self.color_target = Some(target);
    self
  }

  /// Build the render pipeline from provided shader modules.
  pub fn build(
    self,
    device: &wgpu::Device,
    vertex_shader: &ShaderModule,
    fragment_shader: Option<&ShaderModule>,
  ) -> RenderPipeline {
    let mut attr_storage: Vec<Box<[wgpu::VertexAttribute]>> = Vec::new();
    let mut strides: Vec<u64> = Vec::new();
    for (stride, attrs) in &self.vertex_buffers {
      let boxed: Box<[wgpu::VertexAttribute]> =
        attrs.clone().into_boxed_slice();
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
      match &self.color_target {
        Some(ct) => vec![Some(ct.clone())],
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
      cull_mode: self.cull_mode,
      ..wgpu::PrimitiveState::default()
    };

    let layout_ref = self.layout;
    let raw = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
      label: self.label.as_deref(),
      layout: layout_ref,
      vertex: vertex_state,
      primitive: primitive_state,
      depth_stencil: None,
      multisample: wgpu::MultisampleState::default(),
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
