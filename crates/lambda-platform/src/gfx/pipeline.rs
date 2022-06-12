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
}

use super::{
  gpu::Gpu,
  shader::ShaderModule,
};

pub struct RenderPipelineBuilder<RenderBackend: internal::Backend> {
  pipeline_layout: Option<RenderBackend::PipelineLayout>,
}

impl<RenderBackend: internal::Backend> RenderPipelineBuilder<RenderBackend> {
  pub fn new() -> Self {
    return Self {
      pipeline_layout: None,
    };
  }
  /// Builds a render pipeline based on your builder configuration. You can
  /// configure a render pipeline to be however you'd like it to be.
  pub fn build(
    self,
    gpu: &Gpu<RenderBackend>,
    vertex_shader: &ShaderModule<RenderBackend>,
    fragment_shader: &ShaderModule<RenderBackend>,
    render_pass: &super::render_pass::RenderPass<RenderBackend>,
  ) -> RenderPipeline<RenderBackend> {
    // TODO(vmarcella): The pipeline layout should be configurable through the
    // RenderPipelineBuilder.
    let pipeline_layout = unsafe {
      use internal::Device;
      super::internal::logical_device_for(gpu)
        .create_pipeline_layout(vec![].into_iter(), vec![].into_iter())
        .expect(
          "The GPU does not have enough memory to allocate a pipeline layout",
        )
    };

    let primitive_assembler =
      super::assembler::PrimitiveAssemblerBuilder::new().build(vertex_shader);

    let fragment_entry = internal::EntryPoint {
      entry: fragment_shader.entry(),
      module: super::internal::module_for(fragment_shader),
      specialization: fragment_shader.specializations().clone(),
    };

    let mut pipeline_desc = internal::GraphicsPipelineDesc::new(
      super::internal::into_primitive_assembler(primitive_assembler),
      internal::Rasterizer {
        cull_face: internal::Face::BACK,
        ..internal::Rasterizer::FILL
      },
      Some(fragment_entry),
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

    return RenderPipeline { pipeline_layout };
  }
}

pub struct RenderPipeline<RenderBackend: internal::Backend> {
  pipeline_layout: RenderBackend::PipelineLayout,
}
