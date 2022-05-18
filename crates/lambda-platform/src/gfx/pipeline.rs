use gfx_hal::{
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

/// Graphical pipeline for use in the lambda renderer.
pub struct GraphicsPipeline<'a, B: Backend> {
  pipeline_desc: GraphicsPipelineDesc<'a, B>,
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
