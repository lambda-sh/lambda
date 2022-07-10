use lambda_platform::gfx::render_pass;

use super::{
  internal::gpu_from_context,
  RenderContext,
};
pub struct RenderPass {
  render_pass: render_pass::RenderPass<super::internal::RenderBackend>,
}

pub struct RenderPassBuilder {}

impl RenderPassBuilder {
  pub fn new() -> Self {
    return Self {};
  }

  /// Builds a render pass that can be used for defining
  pub fn build(self, render_context: &RenderContext) -> RenderPass {
    let render_pass =
      lambda_platform::gfx::render_pass::RenderPassBuilder::new()
        .build(gpu_from_context(render_context));
    return RenderPass { render_pass };
  }
}
