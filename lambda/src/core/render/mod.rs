pub(crate) use gfx_hal::pso::{
  EntryPoint,
  InputAssemblerDesc,
  Primitive,
  PrimitiveAssemblerDesc,
  Specialization,
};

use self::pipeline::GraphicsPipeline;
use super::{
  event_loop::LambdaEvent,
  window::LambdaWindow,
};
use crate::core::render::assembler::create_vertex_assembler;

pub mod assembler;
pub mod pipeline;
pub mod shader;

use shader::LambdaShader;

use crate::platform::gfx;

pub trait Renderer {
  fn resize(&mut self, width: u32, height: u32);
  fn on_update(&self);
  fn on_event(&self, event: LambdaEvent);
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

impl<B: gfx_hal::Backend> LambdaRenderer<B> {
  pub fn new(name: &str, window: Option<&LambdaWindow>) -> Self {
    let instance = gfx::GfxInstance::<B>::new(name);

    // Surfaces are only required if the renderer is constructed with a Window, otherwise
    // the renderer doesn't need to have a surface and can simply be used for GPU compute.
    let surface = instance.create_surface(window.unwrap());
    let mut gpu = instance
      .open_primary_gpu(Some(&surface))
      .with_command_pool();

    let command_buffer = gpu.allocate_command_buffer();
    let render_pass = gpu.create_render_pass(None, None, None);
    let (submission_fence, rendering_semaphore) = gpu.create_access_fences();

    return Self {
      instance,
      gpu,
      surface: None,
      shader_library: vec![],
    };
  }

  pub fn init(&mut self) {
    println!("Initializing Renderer");
  }

  pub fn shutdown(&mut self) {
    self.instance.destroy_surface(&surface);
  }

  // TODO(vmarcella):
  pub fn attach_pipeline(
    &mut self,
    mut graphics_pipeline: GraphicsPipeline<'static, B>,
  ) {
    let pipeline = self.gpu.create_graphics_pipeline(&mut graphics_pipeline);
  }

  pub fn create_gpu_pipeline(&mut self, shader: LambdaShader) {
    let module = self.gpu.create_shader_module(shader.get_shader_binary());
    // TODO(vmarcella): Abstract the gfx hal assembler away from the
    // render module directly.
    let entry = EntryPoint::<B> {
      entry: "main",
      module: &module,
      specialization: Specialization::default(),
    };

    let assembler = create_vertex_assembler(entry);
  }
}

impl<B: gfx_hal::Backend> Renderer for LambdaRenderer<B> {
  fn on_update(&self) {}
  fn on_event(&self, event: LambdaEvent) {}

  fn resize(&mut self, width: u32, height: u32) {
    todo!("Need to implement resizing for the LambdaRenderer!")
  }
}
