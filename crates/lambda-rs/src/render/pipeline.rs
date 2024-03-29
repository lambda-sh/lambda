//! Render pipeline builders and definitions for lambda runtimes and
//! applications.
use std::rc::Rc;

use lambda_platform::gfx::shader::{
  ShaderModuleBuilder,
  ShaderModuleType,
};

use super::{
  buffer::Buffer,
  internal::RenderBackend,
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
  buffers: Vec<Rc<Buffer>>,
}

impl RenderPipeline {
  /// Destroy the render pipeline with the render context that created it.
  pub fn destroy(self, render_context: &RenderContext) {
    logging::trace!("Destroying render pipeline");
    Rc::try_unwrap(self.pipeline)
      .expect("Failed to destroy render pipeline")
      .destroy(render_context.internal_gpu());

    logging::trace!("Destroying buffers");
    for buffer in self.buffers {
      Rc::try_unwrap(buffer)
        .expect("Failed to get high level buffer.")
        .destroy(render_context);
    }

    logging::info!("Render pipeline & all attached buffers destroyed");
  }
}

impl RenderPipeline {
  pub(super) fn buffers(&self) -> &Vec<Rc<Buffer>> {
    return &self.buffers;
  }

  pub(super) fn into_platform_render_pipeline(
    &self,
  ) -> Rc<lambda_platform::gfx::pipeline::RenderPipeline<RenderBackend>> {
    return self.pipeline.clone();
  }
}

use lambda_platform::gfx::pipeline::PushConstantUpload;
pub use lambda_platform::gfx::{
  assembler::VertexAttribute,
  pipeline::PipelineStage,
};

pub struct RenderPipelineBuilder {
  push_constants: Vec<PushConstantUpload>,
  buffers: Vec<Rc<Buffer>>,
  attributes: Vec<VertexAttribute>,
}

impl RenderPipelineBuilder {
  /// Creates a new render pipeline builder.
  pub fn new() -> Self {
    return Self {
      push_constants: Vec::new(),
      buffers: Vec::new(),
      attributes: Vec::new(),
    };
  }

  /// Adds a buffer to the render pipeline.
  pub fn with_buffer(
    mut self,
    buffer: Buffer,
    attributes: Vec<VertexAttribute>,
  ) -> Self {
    self.buffers.push(Rc::new(buffer));
    self.attributes.extend(attributes);
    return self;
  }

  /// Adds a push constant to the render pipeline at the given stage
  /// with the given size in bytes.
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
    logging::debug!("Building render pipeline");

    logging::debug!("Building vertex shader... ");
    let vertex_shader_module = ShaderModuleBuilder::new().build(
      render_context.internal_mutable_gpu(),
      &vertex_shader.as_binary(),
      ShaderModuleType::Vertex,
    );

    logging::debug!(
      "\tDone. (Vertex shader: {} bytes)",
      vertex_shader.as_binary().len()
    );

    logging::debug!("Building fragment shader... ");
    let fragment_shader_module = match fragment_shader {
      Some(shader) => Some(ShaderModuleBuilder::new().build(
        render_context.internal_mutable_gpu(),
        &shader.as_binary(),
        ShaderModuleType::Fragment,
      )),
      None => None,
    };

    logging::debug!(
      "\tDone. (Fragment shader: {} bytes)",
      fragment_shader.map(|s| s.as_binary().len()).unwrap_or(0)
    );

    let builder = lambda_platform::gfx::pipeline::RenderPipelineBuilder::new();

    let buffers = self.buffers;
    let internal_buffers = buffers
      .iter()
      .map(|b| b.internal_buffer())
      .collect::<Vec<_>>();

    let render_pipeline = builder
      .with_push_constants(self.push_constants.clone())
      .build(
        render_context.internal_gpu(),
        render_pass.internal_render_pass(),
        &vertex_shader_module,
        fragment_shader_module.as_ref(),
        &internal_buffers,
        self.attributes.as_slice(),
      );

    // Clean up shader modules.
    vertex_shader_module.destroy(render_context.internal_mutable_gpu());
    if let Some(fragment_shader_module) = fragment_shader_module {
      fragment_shader_module.destroy(render_context.internal_mutable_gpu());
    }

    return RenderPipeline {
      pipeline: Rc::new(render_pipeline),
      buffers,
    };
  }
}
