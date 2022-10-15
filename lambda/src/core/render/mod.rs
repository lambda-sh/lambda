//! High level Rendering API designed for cross platform rendering and
//! windowing.

pub mod command;
pub mod pipeline;
pub mod render_pass;
pub mod shader;
pub mod viewport;
pub mod window;

use std::{
  mem::swap,
  rc::Rc,
};

use lambda_platform::gfx::{
  command::{
    Command,
    CommandBufferBuilder,
    CommandBufferFeatures,
    CommandBufferLevel,
  },
  framebuffer::FramebufferBuilder,
  surface::SwapchainBuilder,
  viewport::ViewPort,
};

use self::{
  command::RenderCommand,
  pipeline::RenderPipeline,
  render_pass::RenderPass,
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
    let mut surface = Rc::new(
      internal::SurfaceBuilder::new().build(&instance, window.window_handle()),
    );

    // Build a GPU with a Graphical Render queue that can render to our surface.
    let mut gpu = internal::GpuBuilder::new()
      .with_render_queue_type(internal::RenderQueueType::Graphical)
      .build(&mut instance, Some(&surface))
      .expect("Failed to build a GPU with a graphical render queue.");

    // Build command pool and allocate a single buffer named Primary
    let command_pool = internal::CommandPoolBuilder::new().build(&gpu);

    // Build our rendering submission fence and semaphore.
    let submission_fence = internal::RenderSubmissionFenceBuilder::new()
      .with_render_timeout(render_timeout)
      .build(&mut gpu);

    let render_semaphore =
      internal::RenderSemaphoreBuilder::new().build(&mut gpu);

    // Create the image extent and initial frame buffer attachment description
    // for rendering.
    let (width, height) = window.dimensions();
    let swapchain = SwapchainBuilder::new()
      .with_size(width, height)
      .build(&gpu, &surface);

    Rc::get_mut(&mut surface)
      .expect("Failed to get mutable reference to surface.")
      .apply_swapchain(&gpu, swapchain, self.render_timeout)
      .expect("Failed to apply the swapchain to the surface.");

    return RenderContext {
      name,
      instance,
      gpu,
      surface: surface.clone(),
      frame_buffer: None,
      submission_fence: Some(submission_fence),
      render_semaphore: Some(render_semaphore),
      command_pool: Some(command_pool),
      viewports: vec![],
      render_passes: vec![],
      render_pipelines: vec![],
    };
  }
}

/// Generic Rendering API setup to use the current platforms primary
/// Rendering Backend
pub struct RenderContext {
  name: String,
  instance: internal::Instance<internal::RenderBackend>,
  gpu: internal::Gpu<internal::RenderBackend>,
  surface: Rc<internal::Surface<internal::RenderBackend>>,
  frame_buffer: Option<Rc<internal::Framebuffer<internal::RenderBackend>>>,
  submission_fence:
    Option<internal::RenderSubmissionFence<internal::RenderBackend>>,
  render_semaphore: Option<internal::RenderSemaphore<internal::RenderBackend>>,
  command_pool: Option<internal::CommandPool<internal::RenderBackend>>,
  render_passes: Vec<Option<RenderPass>>,
  render_pipelines: Vec<Option<RenderPipeline>>,
  viewports: Vec<ViewPort>,
}

impl RenderContext {
  /// destroys the RenderContext and all associated resources.
  pub fn destroy(mut self) {
    println!("{} will now start destroying resources.", self.name);

    // Destroy the submission fence and rendering semaphore.
    self
      .submission_fence
      .take()
      .expect(
        "Couldn't take the submission fence from the context and destroy it.",
      )
      .destroy(&self.gpu);
    self
      .render_semaphore
      .take()
      .expect("Couldn't take the rendering semaphore from the context and destroy it.")
      .destroy(&self.gpu);

    // Destroy render passes.
    let mut render_passes = vec![];
    swap(&mut self.render_passes, &mut render_passes);

    for render_pass in &mut render_passes {
      render_pass
        .take()
        .expect(
          "Couldn't take the render pass from the context and destroy it.",
        )
        .destroy(&self);
    }

    // Destroy render pipelines.
    let mut render_pipelines = vec![];
    swap(&mut self.render_pipelines, &mut render_pipelines);

    for render_pipeline in &mut render_pipelines {
      render_pipeline
        .take()
        .expect(
          "Couldn't take the render pipeline from the context and destroy it.",
        )
        .destroy(&self);
    }

    // Takes the inner surface and destroys it.
    let mut surface = Rc::try_unwrap(self.surface)
      .expect("Couldn't obtain the surface from the context.");

    surface.remove_swapchain(&self.gpu);
    surface.destroy(&self.instance);
  }

  pub fn allocate_and_get_frame_buffer(
    &mut self,
    render_pass: &internal::RenderPass<internal::RenderBackend>,
  ) -> Rc<lambda_platform::gfx::framebuffer::Framebuffer<internal::RenderBackend>>
  {
    let frame_buffer = FramebufferBuilder::new().build(
      &mut self.gpu,
      &render_pass,
      self.surface.as_ref(),
    );

    // TODO(vmarcella): Update the framebuffer allocation to not be so hacky.
    // FBAs can only be allocated once a render pass has begun, but must be
    // cleaned up after commands have been submitted forcing us
    self.frame_buffer = Some(Rc::new(frame_buffer));
    return self.frame_buffer.as_ref().unwrap().clone();
  }

  /// Allocates a command buffer and records commands to the GPU.
  pub fn render(&mut self, commands: Vec<RenderCommand>) {
    let (width, height) = self
      .surface
      .size()
      .expect("Surface has no size configured.");

    let swapchain = SwapchainBuilder::new()
      .with_size(width, height)
      .build(&self.gpu, &self.surface);

    Rc::get_mut(&mut self.surface)
      .expect("Failed to get mutable reference to surface.")
      .apply_swapchain(&self.gpu, swapchain, 1_000_000_000)
      .expect("Failed to apply the swapchain to the surface.");

    let platform_command_list = commands
      .into_iter()
      .map(|command| command.into_platform_command(self))
      .collect();

    let mut command_buffer =
      CommandBufferBuilder::new(CommandBufferLevel::Primary)
        .with_feature(CommandBufferFeatures::ResetEverySubmission)
        .build(self.command_pool.as_mut().unwrap(), "primary");

    // Start recording commands, issue the high level render commands
    // that came from an application, and then submit the commands to the GPU
    // for rendering.
    command_buffer.issue_command(PlatformRenderCommand::BeginRecording);
    command_buffer.issue_commands(platform_command_list);
    command_buffer.issue_command(PlatformRenderCommand::EndRecording);

    self.gpu.submit_command_buffer(
      &mut command_buffer,
      vec![],
      self
        .submission_fence
        .as_mut()
        .expect("Failed to get mutable reference to submission fence."),
    );

    self
      .gpu
      .render_to_surface(
        Rc::get_mut(&mut self.surface)
          .expect("Failed to obtain a surface to render on."),
        self.render_semaphore.as_mut().unwrap(),
      )
      .expect("Failed to render to the surface");

    match self.frame_buffer {
      Some(_) => {
        Rc::try_unwrap(self.frame_buffer.take().unwrap())
          .ok()
          .unwrap()
          .destroy(&self.gpu);
      }
      None => {}
    }
  }

  pub fn resize(&mut self, width: u32, height: u32) {
    let swapchain = SwapchainBuilder::new()
      .with_size(width, height)
      .build(&self.gpu, &self.surface);
    Rc::get_mut(&mut self.surface)
      .expect("Failed to acquire the surface while attempting to resize.")
      .apply_swapchain(&self.gpu, swapchain, 1_000_000_000)
      .expect("Failed to apply the swapchain to the surface while attempting to resize.");
  }
}

type PlatformRenderCommand = Command<internal::RenderBackend>;

pub mod internal {
  use std::rc::Rc;

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
      framebuffer::Framebuffer,
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

  /// Returns the GPU instance for the given render context.
  pub fn gpu_from_context(
    context: &super::RenderContext,
  ) -> &Gpu<RenderBackend> {
    return &context.gpu;
  }

  /// Returns a mutable GPU instance for the given render context.
  pub fn mut_gpu_from_context(
    context: &mut super::RenderContext,
  ) -> &mut Gpu<RenderBackend> {
    return &mut context.gpu;
  }

  /// Gets the surface for the given render context.
  pub fn surface_from_context(
    context: &super::RenderContext,
  ) -> Rc<Surface<RenderBackend>> {
    return context.surface.clone();
  }
}
