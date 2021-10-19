use gfx_hal::{
  image::Layout,
  pass::Subpass,
  pso::{
    Face,
    GraphicsPipelineDesc,
    PrimitiveAssemblerDesc,
    Rasterizer,
  },
  Backend,
};

/// Graphical pipeline for use in the lambda renderer.
pub struct GraphicsPipeline<'a, B: Backend> {
  pipeline_desc: GraphicsPipelineDesc<'a, B>,
}

impl<'a, B: Backend> GraphicsPipeline<'a, B> {
  pub fn get_pipeline(&mut self) -> &GraphicsPipelineDesc<'a, B> {
    return &self.pipeline_desc;
  }
}

pub fn create_graphics_pipeline<'a, B: Backend>(
  primitive_assembler: PrimitiveAssemblerDesc<'a, B>,
  pipeline_layout: &'a B::PipelineLayout,
  render_pass: &'a B::RenderPass,
) -> GraphicsPipeline<'a, B> {
  let mut pipeline_desc = GraphicsPipelineDesc::new(
    primitive_assembler,
    Rasterizer {
      cull_face: Face::BACK,
      ..Rasterizer::FILL
    },
    None,
    pipeline_layout,
    Subpass {
      index: 0,
      main_pass: render_pass,
    },
  );

  return GraphicsPipeline::<'a> { pipeline_desc };
}
