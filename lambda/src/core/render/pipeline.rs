use gfx_hal::{
  pass::Subpass,
  pso::{
    Face,
    GraphicsPipelineDesc,
    PrimitiveAssemblerDesc,
    Rasterizer,
  },
};

struct GraphicsPipeline<'a> {
  pipeline_desc: GraphicsPipelineDesc<'a, backend::Backend>,
}

pub fn create_graphics_pipeline<'a>(
  primitive_assembler: PrimitiveAssemblerDesc<backend::Backend>,
) -> GraphicsPipeline<'a> {
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
