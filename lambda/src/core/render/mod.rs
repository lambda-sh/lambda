use std::time::Duration;

use gfx_hal::{
  command::{
    ClearColor,
    ClearValue,
    CommandBuffer,
  },
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
  component::Component,
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
  shader_library: Vec<LambdaShader>,

  surface: Option<B::Surface>,
  extent: Option<Extent2D>,
  frame_buffer_attachment: Option<gfx_hal::image::FramebufferAttachment>,
  submission_complete_fence: Option<B::Fence>,
  rendering_complete_semaphore: Option<B::Semaphore>,
  command_buffer: Option<B::CommandBuffer>,

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

    let vertex_shader = LambdaShader::from_file(
      "/home/vmarcella/dev/lambda/lambda/assets/shaders/triangle.vert",
      ShaderKind::Vertex,
    );

    let fragment_shader = LambdaShader::from_file(
      "/home/vmarcella/dev/lambda/lambda/assets/shaders/triangle.frag",
      ShaderKind::Fragment,
    );

    let (module, pipeline_layout, pipeline) =
      self.create_gpu_pipeline(vertex_shader, fragment_shader, &render_pass);

    self.gpu.destroy_shader_module(module);

    self.render_passes = Some(vec![render_pass]);
    self.pipeline_layouts = Some(vec![pipeline_layout]);
    self.graphic_pipelines = Some(vec![pipeline]);
    self.submission_complete_fence = Some(submission_fence);
    self.rendering_complete_semaphore = Some(rendering_semaphore);
    self.command_buffer = Some(command_buffer);
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
        let (extent, frame_buffer_attachment) =
          self.gpu.configure_swapchain_and_update_extent(
            self.surface.as_mut().unwrap(),
            self.format,
            [*new_width, *new_height],
          );
        self.extent = Some(extent);
        self.frame_buffer_attachment = Some(frame_buffer_attachment);
      }
      _ => (),
    };
  }

  /// Rendering update loop.
  fn on_update(&mut self, _: &Duration) {
    self.gpu.wait_for_or_reset_fence(
      self.submission_complete_fence.as_mut().unwrap(),
    );

    let surface = self.surface.as_mut().unwrap();

    let acquire_timeout_ns = 1_000_000_000;
    let image = unsafe {
      let i = match surface.acquire_image(acquire_timeout_ns) {
        Ok((image, _)) => Some(image),
        Err(_) => None,
      };
      i.unwrap()
    };

    use std::borrow::Borrow;
    let render_pass = &self.render_passes.as_mut().unwrap()[0];
    let extent = self.extent.as_ref().unwrap();
    let fba = self.frame_buffer_attachment.as_ref().unwrap();

    // Allocate the framebuffer
    let framebuffer = {
      self
        .gpu
        .create_frame_buffer(render_pass, fba.clone(), extent)
    };

    // TODO(vmarcella): Investigate into abstracting the viewport behind a
    // camera.
    let viewport = {
      use gfx_hal::pso::{
        Rect,
        Viewport,
      };

      Viewport {
        rect: Rect {
          x: 0,
          y: 0,
          w: self.extent.as_ref().unwrap().width as i16,
          h: self.extent.as_ref().unwrap().height as i16,
        },
        depth: 0.0..1.0,
      }
    };

    unsafe {
      let command_buffer = self.command_buffer.as_mut().unwrap();
      command_buffer
        .begin_primary(gfx_hal::command::CommandBufferFlags::ONE_TIME_SUBMIT);

      // Configure the vieports & the scissor rectangles for the rasterizer
      let viewports = vec![viewport.clone()].into_iter();
      let rect = vec![viewport.rect].into_iter();
      command_buffer.set_viewports(0, viewports);
      command_buffer.set_scissors(0, rect);

      // Render attachments to specify for the current render pass.
      let render_attachments = vec![gfx_hal::command::RenderAttachmentInfo {
        image_view: image.borrow(),
        clear_value: ClearValue {
          color: ClearColor {
            float32: [0.0, 0.0, 0.0, 1.0],
          },
        },
      }]
      .into_iter();

      // Initialize the render pass on the command buffer & inline the subpass
      // contents.
      command_buffer.begin_render_pass(
        &render_pass,
        &framebuffer,
        viewport.rect,
        render_attachments,
        gfx_hal::command::SubpassContents::Inline,
      );

      // Bind graphical pipeline and submit commands to the GPU.
      let pipeline = &self.graphic_pipelines.as_ref().unwrap()[0];
      command_buffer.bind_graphics_pipeline(pipeline);
      command_buffer.draw(0..3, 0..1);
      command_buffer.end_render_pass();
      command_buffer.finish();

      // Submit the command buffer for rendering on the GPU.
      self.gpu.submit_command_buffer(
        &command_buffer,
        self.rendering_complete_semaphore.as_ref().unwrap(),
        self.submission_complete_fence.as_mut().unwrap(),
      );

      let result = self.gpu.render_to_surface(
        surface,
        image,
        self.rendering_complete_semaphore.as_mut().unwrap(),
      );

      if result.is_err() {
        todo!("Publish an event from the renderer that the swapchain needs to be reconfigured")
      }

      self.gpu.destroy_frame_buffer(framebuffer);
    }
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
      extent: None,
      frame_buffer_attachment: None,
      command_buffer: None,
    };
  }

  /// Create a graphical pipeline using a single shader with an associated
  /// render pass. This will currently return all gfx_hal related pipeline assets
  pub fn create_gpu_pipeline(
    &mut self,
    vertex_shader: LambdaShader,
    fragment_shader: LambdaShader,
    render_pass: &B::RenderPass,
  ) -> (B::ShaderModule, B::PipelineLayout, B::GraphicsPipeline) {
    let vertex_module = self
      .gpu
      .create_shader_module(vertex_shader.get_shader_binary());
    let fragment_module = self
      .gpu
      .create_shader_module(fragment_shader.get_shader_binary());

    // TODO(vmarcella): Abstract the gfx hal assembler away from the
    // render module directly.
    let vertex_entry = EntryPoint::<B> {
      entry: "main",
      module: &vertex_module,
      specialization: Specialization::default(),
    };

    let fragment_entry = EntryPoint::<B> {
      entry: "main",
      module: &&fragment_module,
      specialization: Specialization::default(),
    };

    // TODO(vmarcella): This process could use a more consistent abstraction
    // for getting a pipeline created.
    let assembler = create_vertex_assembler(vertex_entry);
    let pipeline_layout = self.gpu.create_pipeline_layout();
    let mut logical_pipeline = pipeline::create_graphics_pipeline(
      assembler,
      &pipeline_layout,
      render_pass,
      Some(fragment_entry),
    );

    let physical_pipeline =
      self.gpu.create_graphics_pipeline(&mut logical_pipeline);

    return (vertex_module, pipeline_layout, physical_pipeline);
  }
}