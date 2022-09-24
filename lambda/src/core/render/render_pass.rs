use std::rc::Rc;

use lambda_platform::gfx::render_pass;

use super::{
  internal::gpu_from_context,
  RenderContext,
};

#[derive(Debug)]
pub struct RenderPass {
  render_pass: Rc<render_pass::RenderPass<super::internal::RenderBackend>>,
}

impl RenderPass {
  /// Destroy the render pass with the render context that created it.
  pub fn destroy(self, render_context: &RenderContext) {
    Rc::try_unwrap(self.render_pass)
      .expect("Failed to destroy render pass. Is something holding a reference to it?")
      .destroy(gpu_from_context(render_context));
  }

  /// Converts
  pub fn into_gfx_render_pass(
    &self,
  ) -> Rc<render_pass::RenderPass<super::internal::RenderBackend>> {
    return self.render_pass.clone();
  }
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
    return RenderPass {
      render_pass: Rc::new(render_pass),
    };
  }
}

pub mod internal {
  use crate::core::render::internal::RenderBackend;

  /// Converts a render pass into a platform render pass.
  pub fn platform_render_pass_from_render_pass(
    render_pass: &super::RenderPass,
  ) -> &lambda_platform::gfx::render_pass::RenderPass<RenderBackend> {
    return &render_pass.render_pass;
  }
}
