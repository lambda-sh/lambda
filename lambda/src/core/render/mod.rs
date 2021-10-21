use std::time::Duration;

pub(crate) use gfx_hal::pso::{
  EntryPoint,
  Specialization,
};

use self::pipeline::GraphicsPipeline;
use super::{
  event_loop::LambdaEvent,
  window::LambdaWindow,
};
use crate::core::{
  layer::Component,
  render::{
    assembler::create_vertex_assembler,
    shader::ShaderKind,
  },
};

pub mod assembler;
pub mod pipeline;
pub mod shader;

use shader::LambdaShader;

use crate::platform::gfx;

pub trait Renderer {
  fn resize(&mut self, width: u32, height: u32);
}

pub struct LambdaRenderer<B: gfx_hal::Backend> {
  instance: gfx::GfxInstance<B>,
  gpu: gfx::gpu::GfxGpu<B>,

  // Both surface and Window are optional
  surface: Option<B::Surface>,
  shader_library: Vec<LambdaShader>,
  submission_complete_fence: Option<B::Fence>,
  rendering_complete_semaphore: Option<B::Semaphore>,

  // TODO(vmarcella): Isolate pipeline & render pass management away from
  // the Renderer.
  graphic_pipelines: Option<Vec<B::GraphicsPipeline>>,
  pipeline_layouts: Option<Vec<B::PipelineLayout>>,
  render_passes: Option<Vec<B::RenderPass>>,
}

impl<B: gfx_hal::Backend> Default for LambdaRenderer<B> {
  /// The default constructor returns a named renderer that has no
  /// window attached.
  fn default() -> Self {
    return Self::new("LambdaRenderer", None);
  }
}

/// Platform RenderAPI for layers to use for issuing calls to
/// a LambdaRenderer using the default rendering nackend provided
/// by the platform.
pub type RenderAPI = LambdaRenderer<backend::Backend>;

impl Component for RenderAPI {
  /// Allocates resources on the GPU to enable rendering for other
  fn attach(&mut self) {
    println!("The Rendering API has been attached and is being initialized.");

    let command_buffer = self.gpu.allocate_command_buffer();
    let render_pass = self.gpu.create_render_pass(None, None, None);
    let (submission_fence, rendering_semaphore) =
      self.gpu.create_access_fences();

    let shader =
      LambdaShader::from_string("test-shader", "", ShaderKind::Vertex);

    let (module, pipeline_layout, pipeline) =
      self.create_gpu_pipeline(shader, &render_pass);

    self.gpu.destroy_shader_module(module);

    self.render_passes = Some(vec![render_pass]);
    self.pipeline_layouts = Some(vec![pipeline_layout]);
    self.graphic_pipelines = Some(vec![pipeline]);
    self.submission_complete_fence = Some(submission_fence);
    self.rendering_complete_semaphore = Some(rendering_semaphore);
  }

  /// Detaches the renderers resources from t
  fn detach(&mut self) {
    // Destroy access fences now that the renderer has been detached.
    self.gpu.destroy_access_fences(
      self.submission_complete_fence.unwrap(),
      self.rendering_complete_semaphore.unwrap(),
    );

    for pipeline_layout in self.pipeline_layouts {
      self.gpu.destroy_pipeline_layout(pipeline_layout);
    }

    for render_pass in self.render_passes.unwrap() {
      self.gpu.destroy_render_pass(render_pass);
    }

    for pipeline in self.graphic_pipelines.unwrap() {
      self.gpu.destroy_graphics_pipeline(pipeline);
    }
  }

  fn on_event(&mut self, event: &LambdaEvent) {
    todo!()
  }

  fn on_update(&mut self, _: &Duration, _: &mut RenderAPI) {
    todo!()
  }
}

impl<B: gfx_hal::Backend> LambdaRenderer<B> {
  pub fn new(name: &str, window: Option<&LambdaWindow>) -> Self {
    let instance = gfx::GfxInstance::<B>::new(name);

    // Surfaces are only required if the renderer is constructed with a Window, otherwise
    // the renderer doesn't need to have a surface and can simply be used for GPU compute.
    let surface = instance.create_surface(window.unwrap());
    let mut gpu = instance
      .open_primary_gpu(Some(&surface))
      .with_command_pool();

    return Self {
      instance,
      gpu,
      surface: None,
      shader_library: vec![],
      submission_complete_fence: None,
      rendering_complete_semaphore: None,
      graphic_pipelines: None,
      pipeline_layouts: None,
      render_passes: None,
    };
  }

  /// Create a graphical pipeline using a single shader with an associated
  /// render pass. This will currently return all gfx_hal related pipeline assets
  pub fn create_gpu_pipeline(
    &mut self,
    shader: LambdaShader,
    render_pass: &B::RenderPass,
  ) -> (B::ShaderModule, B::PipelineLayout, B::GraphicsPipeline) {
    let module = self.gpu.create_shader_module(shader.get_shader_binary());
    // TODO(vmarcella): Abstract the gfx hal assembler away from the
    // render module directly.
    let entry = EntryPoint::<B> {
      entry: "main",
      module: &module,
      specialization: Specialization::default(),
    };

    // TODO(vmarcella): This process could use a more consistent abstraction
    // for getting a pipeline created.
    let assembler = create_vertex_assembler(entry);
    let pipeline_layout = self.gpu.create_pipeline_layout();
    let mut logical_pipeline = pipeline::create_graphics_pipeline(
      assembler,
      &pipeline_layout,
      render_pass,
    );

    let physical_pipeline =
      self.gpu.create_graphics_pipeline(&mut logical_pipeline);

    return (module, pipeline_layout, physical_pipeline);
  }
}

impl Renderer for RenderAPI {
  fn resize(&mut self, width: u32, height: u32) {
    todo!("Need to implement resizing for the LambdaRenderer!")
  }
}
