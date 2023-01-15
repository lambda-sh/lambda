use std::rc::Rc;

use lambda_platform::gfx::shader::{
  ShaderModuleBuilder,
  ShaderModuleType,
};

use super::{
  internal::{
    gpu_from_context,
    mut_gpu_from_context,
    RenderBackend,
  },
  render_pass::internal::platform_render_pass_from_render_pass,
  shader::Shader,
  RenderContext,
};

#[derive(Debug)]
pub struct RenderPipeline {
  pipeline: Rc<
    lambda_platform::gfx::pipeline::RenderPipeline<
      super::internal::RenderBackend,
    >,
  >,
}

impl RenderPipeline {
  /// Destroy the render pipeline with the render context that created it.
  pub fn destroy(self, render_context: &RenderContext) {
    Rc::try_unwrap(self.pipeline)
      .expect("Failed to destroy render pipeline")
      .destroy(gpu_from_context(render_context));
  }

  pub fn into_platform_render_pipeline(
    &self,
  ) -> Rc<lambda_platform::gfx::pipeline::RenderPipeline<RenderBackend>> {
    return self.pipeline.clone();
  }
}

pub use lambda_platform::gfx::pipeline::PipelineStage;
use lambda_platform::gfx::pipeline::PushConstantUpload;

pub struct RenderPipelineBuilder {
  push_constants: Vec<PushConstantUpload>,
}

impl RenderPipelineBuilder {
  pub fn new() -> Self {
    return Self {
      push_constants: Vec::new(),
    };
  }

  pub fn with_push_constant(
    mut self,
    stage: PipelineStage,
    bytes: u32,
  ) -> Self {
    self.push_constants.push((stage, 0..bytes));
    return self;
  }

  /// Builds a render pipeline based on your builder configuration.
  pub fn build(
    self,
    render_context: &mut RenderContext,
    render_pass: &super::render_pass::RenderPass,
    vertex_shader: &Shader,
    fragment_shader: Option<&Shader>,
  ) -> RenderPipeline {
    let vertex_shader_module = ShaderModuleBuilder::new().build(
      mut_gpu_from_context(render_context),
      &vertex_shader.as_binary(),
      ShaderModuleType::Vertex,
    );

    let fragment_shader_module = match fragment_shader {
      Some(shader) => Some(ShaderModuleBuilder::new().build(
        mut_gpu_from_context(render_context),
        &shader.as_binary(),
        ShaderModuleType::Fragment,
      )),
      None => None,
    };

    let render_pipeline =
      lambda_platform::gfx::pipeline::RenderPipelineBuilder::new()
        .with_push_constants(self.push_constants)
        .build(
          gpu_from_context(render_context),
          &platform_render_pass_from_render_pass(render_pass),
          &vertex_shader_module,
          fragment_shader_module.as_ref(),
        );

    // Clean up shader modules.
    vertex_shader_module.destroy(mut_gpu_from_context(render_context));
    if let Some(fragment_shader_module) = fragment_shader_module {
      fragment_shader_module.destroy(mut_gpu_from_context(render_context));
    }

    return RenderPipeline {
      pipeline: Rc::new(render_pipeline),
    };
  }
}
