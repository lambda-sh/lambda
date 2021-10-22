use std::time::Duration;

use gfx_hal::{
  pso::{
    EntryPoint,
    Specialization,
  },
  window::{
    Extent2D,
    PresentationSurface,
  },
};

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

pub struct LambdaRenderer<B: gfx_hal::Backend> {
  instance: gfx::GfxInstance<B>,
  gpu: gfx::gpu::GfxGpu<B>,
  format: gfx_hal::format::Format,

  // Both surface and Window are optional
  surface: Option<B::Surface>,
  extent: Option<Extent2D>,
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

impl<B: gfx_hal::Backend> Component for LambdaRenderer<B> {
  /// Allocates resources on the GPU to enable rendering for other
  fn attach(&mut self) {
    println!("The Rendering API has been attached and is being initialized.");

    let command_buffer = self.gpu.allocate_command_buffer();
    let render_pass = self.gpu.create_render_pass(None, None, None);
    let (submission_fence, rendering_semaphore) =
      self.gpu.create_access_fences();

    let shader = LambdaShader::from_file(
      "/home/vmarcella/dev/lambda/lambda/assets/shaders/triangle.vert",
      ShaderKind::Vertex,
    );

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
    println!("Destroying GPU resources allocated during run.");
    self.gpu.destroy_access_fences(
      self.submission_complete_fence.take().unwrap(),
      self.rendering_complete_semaphore.take().unwrap(),
    );

    for pipeline_layout in self.pipeline_layouts.take().unwrap() {
      self.gpu.destroy_pipeline_layout(pipeline_layout);
    }

    for render_pass in self.render_passes.take().unwrap() {
      self.gpu.destroy_render_pass(render_pass);
    }

    for pipeline in self.graphic_pipelines.take().unwrap() {
      self.gpu.destroy_graphics_pipeline(pipeline);
    }

    // Destroy command pool allocated on the GPU.
    self.gpu.destroy_command_pool();
    let mut surface = self.surface.take().unwrap();

    // Unconfigure the swapchain and destroy the surface context.
    self.gpu.unconfigure_swapchain(&mut surface);
    self.instance.destroy_surface(surface);

    println!("Destroyed all GPU resources");
  }

  fn on_event(&mut self, event: &LambdaEvent) {
    match event {
      LambdaEvent::Resized {
        new_width,
        new_height,
      } => {
        self.extent = Some(self.gpu.configure_swapchain_and_update_extent(
          self.surface.as_mut().unwrap(),
          self.format,
          [*new_width, *new_height],
        ));
        ()
      }
      _ => (),
    };
  }

  fn on_update(&mut self, _: &Duration) {
    self.gpu.wait_for_or_reset_fence(
      self.submission_complete_fence.as_mut().unwrap(),
    );
    let surface = self.surface.as_mut().unwrap();

    let acquire_timeout_ns = 1_000_000_000;
    let image = unsafe {
      match surface.acquire_image(acquire_timeout_ns) {
        Ok((image, _)) => image,
        Err(_) => {
          return ();
        }
      }
    };

    let framebuffer = unsafe {
      use std::borrow::Borrow;

      let mut render_pass = self.render_passes.as_mut().unwrap()[0].borrow();
      let extent = self.extent.as_ref().unwrap();
      self.gpu.create_frame_buffer(
        render_pass,
        vec![image].into_iter(),
        extent,
      );
    };
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

    let format = gpu.find_supported_color_format(&surface);

    return Self {
      instance,
      gpu,
      format,
      surface: Some(surface),
      shader_library: vec![],
      submission_complete_fence: None,
      rendering_complete_semaphore: None,
      graphic_pipelines: None,
      pipeline_layouts: None,
      render_passes: None,
      extent: todo!(),
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
