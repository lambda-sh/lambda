use lambda_platform::gfx;

use super::shader::Shader;

pub struct RenderPipelineBuilder {}

impl RenderPipelineBuilder {
  pub fn new() -> Self {
    return Self {};
  }

  pub fn build(
    self,
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
