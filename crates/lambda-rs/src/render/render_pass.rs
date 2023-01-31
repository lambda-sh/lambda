//! Render pass builders and definitions for lambda runtimes and applications.
use std::rc::Rc;

use lambda_platform::gfx::render_pass;

use super::RenderContext;

#[derive(Debug)]
pub struct RenderPass {
  render_pass: Rc<render_pass::RenderPass<super::internal::RenderBackend>>,
}

impl RenderPass {
  /// Destroy the render pass with the render context that created it.
  pub fn destroy(self, render_context: &RenderContext) {
    Rc::try_unwrap(self.render_pass)
      .expect("Failed to destroy render pass. Is something holding a reference to it?")
      .destroy(render_context.internal_gpu());
    logging::debug!("Render pass destroyed");
  }
}

/// Internal Renderpass functions for lambda.
impl RenderPass {
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
  /// Creates a new render pass builder.
  pub fn new() -> Self {
    return Self {};
  }

  /// Builds a render pass that can be used for defining
  pub fn build(self, render_context: &RenderContext) -> RenderPass {
    let render_pass =
      lambda_platform::gfx::render_pass::RenderPassBuilder::new()
        .build(render_context.internal_gpu());
    return RenderPass {
      render_pass: Rc::new(render_pass),
    };
  }
}
