use gfx_hal::{Instance as HalInstance, device::Device, prelude::PhysicalDevice, pso::{EntryPoint, Face, GraphicsPipelineDesc, InputAssemblerDesc, Primitive, PrimitiveAssemblerDesc, Rasterizer, Specialization}};

use super::{
  event_loop::LambdaEvent,
  window::LambdaWindow,
};

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
	shader_library: Vec<LambdaShader>
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

    return Self {
      instance,
			gpu,
      surface: None,
			shader_library: vec!()
    };
  }

	pub fn init(&mut self) {
		println!("Initializing Renderer");
	}

	pub fn create_gpu_pipeline(&mut self, shader: LambdaShader) {
		let (module, entry) = self.gpu.create_shader_module(shader.get_shader_binary());
			// TODO(vmarcella): Abstract the gfx hal assembler away from the
			// render module directly.
		let primitive_assembler = PrimitiveAssemblerDesc::Vertex {
			buffers: &[],
			attributes: &[],
			input_assembler: InputAssemblerDesc::new(Primitive::TriangleList),
			vertex: entry,
			tessellation: None,
			geometry: None
		};
	}
}

impl<B: gfx_hal::Backend> Renderer for LambdaRenderer<B> {
  fn on_update(&self, ) {}
  fn on_event(&self, event: LambdaEvent) {}

  fn resize(&mut self, width: u32, height: u32) {
    todo!("Need to implement resizing for the LambdaRenderer!")
  }
}
