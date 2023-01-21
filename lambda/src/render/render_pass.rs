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

  /// Retrieve a reference to the lower level render pass.
  pub(super) fn internal_render_pass(
    &self,
  ) -> &Rc<render_pass::RenderPass<super::internal::RenderBackend>> {
    return &self.render_pass;
  }

  /// Converts
  pub(super) fn into_gfx_render_pass(
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
