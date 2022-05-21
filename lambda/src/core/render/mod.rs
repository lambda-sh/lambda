pub mod assembler;
pub mod shader;

use std::time::Duration;

use lambda_platform::{
  gfx,
  gfx::{
    gfx_hal_exports,
    gfx_hal_exports::{
      CommandBuffer,
      PresentationSurface,
    },
  },
  shaderc::ShaderKind,
};
use shader::Shader;

use super::events::Event;
use crate::core::{
  component::Component,
  render::assembler::create_vertex_assembler,
};

pub struct LambdaRenderer<B: gfx_hal_exports::Backend> {
  instance: gfx::GfxInstance<B>,
  gpu: gfx::gpu::Gpu<B>,
  format: gfx_hal_exports::Format,
  shader_library: Vec<Shader>,

  surface: Option<B::Surface>,
  extent: Option<gfx_hal_exports::Extent2D>,
  frame_buffer_attachment: Option<gfx_hal_exports::FramebufferAttachment>,
  submission_complete_fence: Option<B::Fence>,
  rendering_complete_semaphore: Option<B::Semaphore>,
  command_buffer: Option<B::CommandBuffer>,

  // TODO(vmarcella): Isolate pipeline & render pass management away from
  // the Renderer.
  graphic_pipelines: Option<Vec<B::GraphicsPipeline>>,
  pipeline_layouts: Option<Vec<B::PipelineLayout>>,
  render_passes: Option<Vec<B::RenderPass>>,
}

impl<B: gfx_hal_exports::Backend> LambdaRenderer<B> {
  /// Create a graphical pipeline using a single shader with an associated
  /// render pass. This will currently return all gfx_hal related pipeline assets
  pub fn create_gpu_pipeline(
    &mut self,
    vertex_shader: Shader,
    fragment_shader: Shader,
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
    let vertex_entry = gfx_hal_exports::EntryPoint::<B> {
      entry: "main",
      module: &vertex_module,
      specialization: gfx_hal_exports::Specialization::default(),
    };

    let fragment_entry = gfx_hal_exports::EntryPoint::<B> {
      entry: "main",
      module: &fragment_module,
      specialization: gfx_hal_exports::Specialization::default(),
    };

    // TODO(vmarcella): This process could use a more consistent abstraction
    // for getting a pipeline created.
    let assembler = create_vertex_assembler(vertex_entry);
    let pipeline_layout = self.gpu.create_pipeline_layout();
    let mut logical_pipeline = gfx::pipeline::create_graphics_pipeline(
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

/// Platform RenderAPI for layers to use for issuing calls to
/// a LambdaRenderer using the default rendering nackend provided
/// by the platform.
pub type RenderAPI = LambdaRenderer<gfx::api::RenderingAPI::Backend>;

/// A render API that is provided by the lambda runnable.
pub struct RenderClient<'a> {
  api: &'a RenderAPI,
}

impl<'a> RenderClient<'a> {
  fn new(renderer: &'a RenderAPI) -> Self {
    return Self { api: renderer };
  }

  pub fn upload_shader() {}
}

impl<B: gfx_hal_exports::Backend> Component for LambdaRenderer<B> {
  /// Allocates resources on the GPU to enable rendering for other
  fn on_attach(&mut self) {
    println!("The Rendering API has been attached and is being initialized.");

    let command_buffer = self.gpu.allocate_command_buffer();
    let render_pass = self.gpu.create_render_pass(None, None, None);
    let (submission_fence, rendering_semaphore) =
      self.gpu.create_access_fences();

    let vertex_shader = Shader::from_file(
      "/home/vmarcella/dev/lambda/lambda/assets/shaders/triangle.vert",
      ShaderKind::Vertex,
    );

    let fragment_shader = Shader::from_file(
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

  /// Detaches physical rendering resources that were allocated by this
  /// component.
  fn on_detach(&mut self) {
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

    println!("Destroyed all GPU resources");
  }

  fn on_event(&mut self, event: &Event) {
    match event {
      Event::Resized {
        new_width,
        new_height,
      } => {}
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

    // TODO(vmarcella): This code will fail if there are no render passes
    // attached to the renderer.
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
      gfx_hal_exports::Viewport {
        rect: gfx_hal_exports::Rect {
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
        .begin_primary(gfx_hal_exports::CommandBufferFlags::ONE_TIME_SUBMIT);

      // Configure the vieports & the scissor rectangles for the rasterizer
      let viewports = vec![viewport.clone()].into_iter();
      let rect = vec![viewport.rect].into_iter();
      command_buffer.set_viewports(0, viewports);
      command_buffer.set_scissors(0, rect);

      // Render attachments to specify for the current render pass.
      let render_attachments = vec![gfx_hal_exports::RenderAttachmentInfo {
        image_view: image.borrow(),
        clear_value: gfx_hal_exports::ClearValue {
          color: gfx_hal_exports::ClearColor {
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
        gfx_hal_exports::SubpassContents::Inline,
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
