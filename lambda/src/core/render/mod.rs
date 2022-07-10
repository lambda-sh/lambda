pub mod command;
pub mod pipeline;
pub mod render_pass;
pub mod shader;
pub mod viewport;
pub mod window;

pub mod internal {
  use lambda_platform::gfx::api::RenderingAPI as RenderContext;
  pub type RenderBackend = RenderContext::Backend;

  pub use lambda_platform::{
    gfx::{
      command::{
        CommandBuffer,
        CommandBufferBuilder,
        CommandPool,
        CommandPoolBuilder,
      },
      fence::{
        RenderSemaphore,
        RenderSemaphoreBuilder,
        RenderSubmissionFence,
        RenderSubmissionFenceBuilder,
      },
      gpu::{
        Gpu,
        GpuBuilder,
        RenderQueueType,
      },
      pipeline::RenderPipelineBuilder,
      render_pass::{
        RenderPass,
        RenderPassBuilder,
      },
      surface::{
        Surface,
        SurfaceBuilder,
      },
      Instance,
      InstanceBuilder,
    },
    shaderc::ShaderKind,
  };

  pub fn gpu_from_context(
    context: &super::RenderContext,
  ) -> &Gpu<RenderBackend> {
    return &context.gpu;
  }
}

use lambda_platform::gfx::{
  command::{
    Command,
    CommandBufferBuilder,
    CommandBufferFeatures,
    CommandBufferLevel,
  },
  surface::SwapchainBuilder,
  viewport::ViewPort,
};

pub struct RenderContextBuilder {
  name: String,
  render_timeout: u64,
}

impl RenderContextBuilder {
  /// Create a new localized RenderContext
  pub fn new(name: &str) -> Self {
    return Self {
      name: name.to_string(),
      render_timeout: 1_000_000_000,
    };
  }

  /// The time rendering has to complete before timing out.
  pub fn with_render_timeout(mut self, render_timeout: u64) -> Self {
    self.render_timeout = render_timeout;
    return self;
  }

  /// Builds a RenderContext and injects it into the application window.
  /// Currently only supports building a Rendering Context utilizing the
  /// systems primary GPU.
  pub fn build(self, window: &window::Window) -> RenderContext {
    let RenderContextBuilder {
      name,
      render_timeout,
    } = self;

    let mut instance = internal::InstanceBuilder::new()
      .build::<internal::RenderBackend>(name.as_str());
    let mut surface =
      internal::SurfaceBuilder::new().build(&instance, window.window_handle());

    // Build a GPU with a 3D Render queue that can render to our surface.
    let mut gpu = internal::GpuBuilder::new()
      .with_render_queue_type(internal::RenderQueueType::Graphical)
      .build(&mut instance, Some(&surface))
      .expect("Failed to build a GPU.");

    // Build command pool and allocate a single buffer named Primary
    let mut command_pool = internal::CommandPoolBuilder::new().build(&gpu);
    let command_buffer =
      internal::CommandBufferBuilder::new(CommandBufferLevel::Primary)
        .build(&mut command_pool, "LambdaPrimary");

    // Build our rendering submission fence and semaphore.
    let submission_fence = internal::RenderSubmissionFenceBuilder::new()
      .with_render_timeout(render_timeout)
      .build(&mut gpu);

    let render_semaphore =
      internal::RenderSemaphoreBuilder::new().build(&mut gpu);

    let mut render_pass = internal::RenderPassBuilder::new().build(&gpu);

    // Create the image extent and initial frame buffer attachment description
    // for rendering.
    let dimensions = window.dimensions();
    let swapchain = SwapchainBuilder::new()
      .with_size(dimensions[0], dimensions[1])
      .build(&gpu, &surface);

    surface
      .apply_swapchain(&gpu, swapchain, 1_000_000_000)
      .expect("Failed to apply the swapchain to the surface.");

    return RenderContext {
      name,
      instance,
      gpu,
      surface,
      submission_fence,
      render_semaphore,
      command_pool,
      viewports: vec![],
      render_passes: vec![render_pass],
    };
  }
}

/// Generic Rendering API setup to use the current platforms primary
/// Rendering Backend
pub struct RenderContext {
  name: String,
  instance: internal::Instance<internal::RenderBackend>,
  gpu: internal::Gpu<internal::RenderBackend>,
  surface: internal::Surface<internal::RenderBackend>,
  submission_fence: internal::RenderSubmissionFence<internal::RenderBackend>,
  render_semaphore: internal::RenderSemaphore<internal::RenderBackend>,
  command_pool: internal::CommandPool<internal::RenderBackend>,
  render_passes: Vec<internal::RenderPass<internal::RenderBackend>>,
  viewports: Vec<ViewPort>,
}

impl RenderContext {
  /// destroys the RenderContext and all associated resources.
  pub fn destroy(self) {
    let RenderContext {
      name,
      submission_fence,
      instance,
      mut gpu,
      mut surface,
      render_semaphore,
      command_pool,
      render_passes,
      viewports,
    } = self;

    println!("{} will now start destroying resources.", name);

    // Destroy the submission fence and rendering semaphore.
    submission_fence.destroy(&gpu);
    render_semaphore.destroy(&gpu);

    // Destroy the command pool and all associated command buffers.
    command_pool.destroy(&mut gpu);
    for render_pass in render_passes {
      render_pass.destroy(&gpu);
    }

    surface.remove_swapchain(&gpu);
    surface.destroy(&instance);
  }

  /// Allocates a command buffer and records commands to the GPU.
  pub fn render(&mut self) {
    let mut command_buffer =
      CommandBufferBuilder::new(CommandBufferLevel::Primary)
        .with_feature(CommandBufferFeatures::ResetEverySubmission)
        .build(&mut self.command_pool, "primary");

    let command_list = vec![
      PlatformRenderCommand::BeginRecording,
      PlatformRenderCommand::SetViewports {
        start_at: 0,
        viewports: self.viewports.clone(),
      },
      PlatformRenderCommand::SetScissors {
        start_at: 0,
        viewports: self.viewports.clone(),
      },
      PlatformRenderCommand::EndRenderPass,
      PlatformRenderCommand::EndRecording,
    ];

    command_buffer.issue_commands(command_list);

    self.gpu.submit_command_buffer(
      &mut command_buffer,
      vec![],
      &mut self.submission_fence,
    );

    self
      .gpu
      .render_to_surface(&mut self.surface, &mut self.render_semaphore)
      .expect("Failed to render to the surface");
  }
}

type PlatformRenderCommand = Command<internal::RenderBackend>;

// TODO(vmarcella): Abstract the gfx hal assembler away from the
// render module directly.
// let vertex_entry = gfx_hal_exports::EntryPoint::<B> {
// entry: "main",
// module: &vertex_module,
// specialization: gfx_hal_exports::Specialization::default(),
// };

// let fragment_entry = gfx_hal_exports::EntryPoint::<B> {
// entry: "main",
// module: &fragment_module,
// specialization: gfx_hal_exports::Specialization::default(),
// };

// TODO(vmarcella): This process could use a more consistent abstraction
// for getting a pipeline created.
// let assembler = create_vertex_assembler(vertex_entry);
// let pipeline_layout = self.gpu.create_pipeline_layout();
// let mut logical_pipeline = gfx::pipeline::create_graphics_pipeline(
// assembler,
// &pipeline_layout,
// render_pass,
// Some(fragment_entry),
// );

// let physical_pipeline =
// self.gpu.create_graphics_pipeline(&mut logical_pipeline);

// return (vertex_module, pipeline_layout, physical_pipeline);

// let render_pass = self.gpu.create_render_pass(None, None, None);

// let (module, pipeline_layout, pipeline) =
// self.create_gpu_pipeline(vertex_shader, fragment_shader, &render_pass);
// self.gpu.destroy_shader_module(module);
// self.pipeline_layouts = Some(vec![pipeline_layout]);
// self.graphic_pipelines = Some(vec![pipeline]);

// let surface = self.surface.as_mut().unwrap();

// let acquire_timeout_ns = 1_000_000_000;
//  let image = unsafe {
//    let i = match surface.acquire_image(acquire_timeout_ns) {
//      Ok((image, _)) => Some(image),
//      Err(_) => None,
//    };
//    i.unwrap()
//  };

// TODO(vmarcella): This code will fail if there are no render passes
// attached to the renderer.

// TODO(vmarcella): Investigate into abstracting the viewport behind a
// camera.
// let viewport = {
//  gfx_hal_exports::Viewport {
//    rect: gfx_hal_exports::Rect {
//      x: 0,
//      y: 0,
//      w: self.extent.as_ref().unwrap().width as i16,
//      h: self.extent.as_ref().unwrap().height as i16,
//    },
//    depth: 0.0..1.0,
//  }
// };

// unsafe {
//  let command_buffer = self.command_buffer.as_mut().unwrap();
//  command_buffer
//    .begin_primary(gfx_hal_exports::CommandBufferFlags::ONE_TIME_SUBMIT);
//
// Configure the vieports & the scissor rectangles for the rasterizer
//  let viewports = vec![viewport.clone()].into_iter();
//   let rect = vec![viewport.rect].into_iter();
// command_buffer.set_viewports(0, viewports);
// command_buffer.set_scissors(0, rect);

// Render attachments to specify for the current render pass.
// let render_attachments = vec![gfx_hal_exports::RenderAttachmentInfo {
// image_view: image.borrow(),
// clear_value: gfx_hal_exports::ClearValue {
// color: gfx_hal_exports::ClearColor {
//   float32: [0.0, 0.0, 0.0, 1.0],
// },
//},
//}]
//.into_iter();

// Initialize the render pass on the command buffer & inline the subpass
// contents.
//   command_buffer.begin_render_pass(
//    &render_pass,
//    &framebuffer,
//    viewport.rect,
//    render_attachments,
//    gfx_hal_exports::SubpassContents::Inline,
//  );

// Bind graphical pipeline and submit commands to the GPU.
//   let pipeline = &self.graphic_pipelines.as_ref().unwrap()[0];
//  command_buffer.bind_graphics_pipeline(pipeline);
//  command_buffer.draw(0..3, 0..1);
//  command_buffer.end_render_pass();
//  command_buffer.finish();

// Submit the command buffer for rendering on the GPU.
// self.gpu.submit_command_buffer(
//   &command_buffer,
//   self.rendering_complete_semaphore.as_ref().unwrap(),
//   self.submission_complete_fence.as_mut().unwrap(),
//  );

// let result = self.gpu.render_to_surface(
//   surface,
//   image,
//   self.rendering_complete_semaphore.as_mut().unwrap(),
// );
// if result.is_err() {
//    todo!("Publish an event from the renderer that the swapchain needs to be reconfigured")
// }
