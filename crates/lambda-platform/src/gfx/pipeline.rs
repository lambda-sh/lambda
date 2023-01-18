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
  buffer::Buffer,
  gpu::Gpu,
  shader::ShaderModule,
};

/// Builder for a gfx-hal backed render pipeline.
pub struct RenderPipelineBuilder<RenderBackend: internal::Backend> {
  pipeline_layout: Option<RenderBackend::PipelineLayout>,
  push_constants: Vec<PushConstantUpload>,
}

pub type PipelineStage = gfx_hal::pso::ShaderStageFlags;

pub type PushConstantUpload = (PipelineStage, Range<u32>);

impl<RenderBackend: internal::Backend> RenderPipelineBuilder<RenderBackend> {
  pub fn new() -> Self {
    return Self {
      pipeline_layout: None,
      push_constants: Vec::new(),
    };
  }

  pub fn with_buffer(&mut self, buffer: &Buffer<RenderBackend>) -> &mut Self {
    todo!()
  }

  /// Adds a push constant to the render pipeline at the set PipelineStage(s)
  pub fn with_push_constant(
    mut self,
    stage: PipelineStage,
    bytes: u32,
  ) -> Self {
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

    let primitive_assembler =
      super::assembler::PrimitiveAssemblerBuilder::new().build(vertex_shader);

    let fragment_entry = match fragment_shader {
      Some(shader) => Some(internal::EntryPoint::<RenderBackend> {
        entry: "main",
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
      super::internal::logical_device_for(gpu)
        .create_graphics_pipeline(&pipeline_desc, None)
        .expect("Failed to create graphics pipeline")
    };

    return RenderPipeline {
      pipeline_layout,
      pipeline,
    };
  }
}

/// Represents a render capable pipeline for graphical
#[derive(Debug)]
pub struct RenderPipeline<RenderBackend: internal::Backend> {
  pipeline_layout: RenderBackend::PipelineLayout,
  pipeline: RenderBackend::GraphicsPipeline,
}

impl<RenderBackend: internal::Backend> RenderPipeline<RenderBackend> {
  /// Destroys the pipeline layout and graphical pipeline
  pub fn destroy(self, gpu: &super::gpu::Gpu<RenderBackend>) {
    unsafe {
      super::gpu::internal::logical_device_for(gpu)
        .destroy_pipeline_layout(self.pipeline_layout);

      super::gpu::internal::logical_device_for(gpu)
        .destroy_graphics_pipeline(self.pipeline);
    }
  }
}
