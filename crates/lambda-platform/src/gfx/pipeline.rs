pub mod internal {

  /// gfx-hal imports for pipeline.rs
  pub use gfx_hal::{
    device::Device,
    pass::Subpass,
    pso::{
      BlendState,
      ColorBlendDesc,
      ColorMask,
      EntryPoint,
      Face,
      GraphicsPipelineDesc,
      PrimitiveAssemblerDesc,
      Rasterizer,
    },
    Backend,
  };

  /// Retrieves the underlying gfx_hal pipeline for internal use.
  pub fn pipeline_for<RenderBackend: gfx_hal::Backend>(
    pipeline: &super::RenderPipeline<RenderBackend>,
  ) -> &RenderBackend::GraphicsPipeline {
    return &pipeline.pipeline;
  }

  pub fn pipeline_layout_for<RenderBackend: gfx_hal::Backend>(
    pipeline: &super::RenderPipeline<RenderBackend>,
  ) -> &RenderBackend::PipelineLayout {
    return &pipeline.pipeline_layout;
  }
}

use std::ops::Range;

use gfx_hal::device::Device;

use super::{
  assembler::VertexAttribute,
  buffer::Buffer,
  gpu::Gpu,
  shader::ShaderModule,
};

/// Builder for a gfx-hal backed render pipeline.
pub struct RenderPipelineBuilder<RenderBackend: internal::Backend> {
  pipeline_layout: Option<RenderBackend::PipelineLayout>,
  push_constants: Vec<PushConstantUpload>,
  buffers: Vec<Buffer<RenderBackend>>,
  attributes: Vec<VertexAttribute>,
}

pub type PipelineStage = gfx_hal::pso::ShaderStageFlags;

pub type PushConstantUpload = (PipelineStage, Range<u32>);

impl<RenderBackend: internal::Backend> RenderPipelineBuilder<RenderBackend> {
  pub fn new() -> Self {
    return Self {
      pipeline_layout: None,
      push_constants: Vec::new(),
      buffers: Vec::new(),
      attributes: Vec::new(),
    };
  }

  pub fn with_buffer(
    &mut self,
    buffer: Buffer<RenderBackend>,
    attributes: Vec<VertexAttribute>,
  ) -> &mut Self {
    self.buffers.push(buffer);
    self.attributes.extend(attributes);
    return self;
  }

  /// Adds a push constant to the render pipeline at the set PipelineStage(s)
  pub fn with_push_constant(
    &mut self,
    stage: PipelineStage,
    bytes: u32,
  ) -> &mut Self {
    self.push_constants.push((stage, 0..bytes));
    return self;
  }

  /// Adds multiple push constants to the render pipeline at their
  /// set PipelineStage(s)
  pub fn with_push_constants(
    mut self,
    push_constants: Vec<PushConstantUpload>,
  ) -> Self {
    self.push_constants.extend(push_constants);
    return self;
  }

  /// Builds a render pipeline based on your builder configuration. You can
  /// configure a render pipeline to be however you'd like it to be.
  pub fn build(
    self,
    gpu: &Gpu<RenderBackend>,
    render_pass: &super::render_pass::RenderPass<RenderBackend>,
    vertex_shader: &ShaderModule<RenderBackend>,
    fragment_shader: Option<&ShaderModule<RenderBackend>>,
    buffers: &Vec<&Buffer<RenderBackend>>,
    attributes: &[VertexAttribute],
  ) -> RenderPipeline<RenderBackend> {
    // TODO(vmarcella): The pipeline layout should be configurable through the
    // RenderPipelineBuilder.
    let push_constants = self.push_constants.into_iter();

    let pipeline_layout = unsafe {
      use internal::Device;

      super::internal::logical_device_for(gpu)
        .create_pipeline_layout(vec![].into_iter(), push_constants)
        .expect(
          "The GPU does not have enough memory to allocate a pipeline layout",
        )
    };

    // TODO(vmarcella): The primitive assembler should be configurable through
    // the RenderPipelineBuilder so that buffers & attributes can be bound.
    let mut builder = super::assembler::PrimitiveAssemblerBuilder::new();
    let primitive_assembler =
      builder.build(vertex_shader, Some(buffers), Some(attributes));

    let fragment_entry = match fragment_shader {
      Some(shader) => Some(internal::EntryPoint::<RenderBackend> {
        entry: shader.entry(),
        module: super::internal::module_for(shader),
        specialization: gfx_hal::pso::Specialization::default(),
      }),
      None => None,
    };

    let mut pipeline_desc = internal::GraphicsPipelineDesc::new(
      super::internal::into_primitive_assembler(primitive_assembler),
      internal::Rasterizer {
        cull_face: internal::Face::BACK,
        ..internal::Rasterizer::FILL
      },
      fragment_entry,
      &pipeline_layout,
      internal::Subpass {
        index: 0,
        main_pass: super::internal::render_pass_for(render_pass),
      },
    );

    pipeline_desc
      .blender
      .targets
      .push(internal::ColorBlendDesc {
        mask: internal::ColorMask::ALL,
        blend: Some(internal::BlendState::ALPHA),
      });

    let pipeline = unsafe {
      gpu
        .internal_logical_device()
        .create_graphics_pipeline(&pipeline_desc, None)
        .expect("Failed to create graphics pipeline")
    };

    return RenderPipeline {
      pipeline_layout,
      pipeline,
      buffers: self.buffers,
    };
  }
}

/// Represents a render capable pipeline for graphical
#[derive(Debug)]
pub struct RenderPipeline<RenderBackend: internal::Backend> {
  pipeline_layout: RenderBackend::PipelineLayout,
  pipeline: RenderBackend::GraphicsPipeline,
  buffers: Vec<Buffer<RenderBackend>>,
}

impl<RenderBackend: internal::Backend> RenderPipeline<RenderBackend> {
  /// Destroys the pipeline layout and graphical pipeline
  pub fn destroy(self, gpu: &super::gpu::Gpu<RenderBackend>) {
    unsafe {
      gpu
        .internal_logical_device()
        .destroy_pipeline_layout(self.pipeline_layout);

      gpu
        .internal_logical_device()
        .destroy_graphics_pipeline(self.pipeline);
    }
  }
}
