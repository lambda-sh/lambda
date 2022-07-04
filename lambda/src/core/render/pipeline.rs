use lambda_platform::gfx;

use super::{
  shader::Shader,
  RenderAPI,
};

pub struct RenderPipelineBuilder {}
impl RenderPipeline {
  pub fn destroy(self) {}
}

impl RenderPipelineBuilder {
  pub fn new() -> Self {
    return Self {};
  }

  pub fn build(
    self,
    render_context: &RenderAPI,
    vertex_shader: &Shader,
    fragment_shader: &Shader,
  ) -> RenderPipeline {
    return RenderPipeline { pipeline: None };
  }
}

pub struct RenderPipeline {
  pipeline:
    Option<gfx::pipeline::RenderPipeline<super::internal::RenderBackend>>,
}
