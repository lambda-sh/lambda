use lambda_platform::gfx::{
  render_pass,
  shader::{
    ShaderModuleBuilder,
    ShaderModuleType,
  },
};

use super::{
  internal::{
    gpu_from_context,
    mut_gpu_from_context,
  },
  render_pass::internal::platform_render_pass_from_render_pass,
  shader::Shader,
  RenderContext,
};

pub struct RenderPipelineBuilder {}
impl RenderPipeline {
  pub fn destroy(self) {}
}

impl RenderPipelineBuilder {
  pub fn new() -> Self {
    return Self {};
  }

  /// Builds a render pipeline based on your builder configuration.
  pub fn build(
    self,
    render_context: &mut RenderContext,
    render_pass: &super::render_pass::RenderPass,
    vertex_shader: &Shader,
    fragment_shader: &Shader,
  ) -> RenderPipeline {
    let vertex_shader_module = ShaderModuleBuilder::new().build(
      mut_gpu_from_context(render_context),
      &vertex_shader.as_binary(),
      ShaderModuleType::Vertex,
    );

    let fragment_shader_module = ShaderModuleBuilder::new().build(
      mut_gpu_from_context(render_context),
      &fragment_shader.as_binary(),
      ShaderModuleType::Fragment,
    );

    let render_pipeline =
      lambda_platform::gfx::pipeline::RenderPipelineBuilder::new().build(
        gpu_from_context(render_context),
        &platform_render_pass_from_render_pass(render_pass),
        &vertex_shader_module,
        &fragment_shader_module,
      );

    return RenderPipeline {
      pipeline: render_pipeline,
    };
  }
}

pub struct RenderPipeline {
  pipeline: lambda_platform::gfx::pipeline::RenderPipeline<
    super::internal::RenderBackend,
  >,
}
