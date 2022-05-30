use gfx_hal::{
  device::Device,
  pass::Subpass,
  pso::{
    ColorBlendDesc,
    EntryPoint,
    Face,
    GraphicsPipelineDesc,
    PrimitiveAssemblerDesc,
    Rasterizer,
  },
  Backend,
};

use super::gpu::Gpu;

/// Graphical pipeline for use in the lambda renderer.
pub struct GraphicsPipeline<'a, B: Backend> {
  pipeline_desc: GraphicsPipelineDesc<'a, B>,
}

pub struct RenderPipelineBuilder {}

impl RenderPipelineBuilder {
  pub fn build<RenderBackend: gfx_hal::Backend>(
    gpu: &Gpu<RenderBackend>,
  ) -> RenderPipeline<RenderBackend> {
    let pipeline_layout = unsafe {
      super::internal::logical_device_for(gpu)
        .create_pipeline_layout(vec![].into_iter(), vec![].into_iter())
        .expect(
          "The GPU does not have enough memory to allocate a pipeline layout",
        )
    };
    return RenderPipeline { pipeline_layout };
  }
}

pub struct RenderPipeline<RenderBackend: gfx_hal::Backend> {
  pipeline_layout: RenderBackend::PipelineLayout,
}

impl<'a, B: Backend> GraphicsPipeline<'a, B> {
  pub fn get_pipeline(&mut self) -> &GraphicsPipelineDesc<'a, B> {
    return &self.pipeline_desc;
  }
}

/// Create a Graphical Pipeline to use for rendering.
pub fn create_graphics_pipeline<'a, B: Backend>(
  primitive_assembler: PrimitiveAssemblerDesc<'a, B>,
  pipeline_layout: &'a B::PipelineLayout,
  render_pass: &'a B::RenderPass,
  fragment_shader: Option<EntryPoint<'a, B>>,
) -> GraphicsPipeline<'a, B> {
  let mut pipeline_desc = GraphicsPipelineDesc::new(
    primitive_assembler,
    Rasterizer {
      cull_face: Face::BACK,
      ..Rasterizer::FILL
    },
    fragment_shader,
    pipeline_layout,
    Subpass {
      index: 0,
      main_pass: render_pass,
    },
  );

  pipeline_desc.blender.targets.push(ColorBlendDesc {
    mask: gfx_hal::pso::ColorMask::ALL,
    blend: Some(gfx_hal::pso::BlendState::ALPHA),
  });

  return GraphicsPipeline::<'a> { pipeline_desc };
}
