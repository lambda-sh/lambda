//! High level Rendering API designed for cross platform rendering and
//! windowing.

// Module Exports
pub mod buffer;
pub mod command;
pub mod mesh;
pub mod pipeline;
pub mod render_pass;
pub mod shader;
pub mod vertex;
pub mod viewport;
pub mod window;

use std::{
  mem::swap,
  rc::Rc,
};

/// ColorFormat is a type alias for the color format used by the surface and
/// vertex buffers. They denote the size of the color channels and the number of
/// channels being used.
pub use lambda_platform::gfx::surface::ColorFormat;
use lambda_platform::gfx::{
  command::{
    Command,
    CommandBufferBuilder,
    CommandBufferFeatures,
    CommandBufferLevel,
  },
  framebuffer::FramebufferBuilder,
  surface::SwapchainBuilder,
};

use self::{
  command::RenderCommand,
  pipeline::RenderPipeline,
  render_pass::RenderPass,
};

/// A RenderContext is a localized rendering context that can be used to render
/// to a window. It is localized to a single window at the moment.
pub struct RenderContextBuilder {
  name: String,
  render_timeout: u64,
}

impl RenderContextBuilder {
  /// Create a new localized RenderContext with the given name.
  pub fn new(name: &str) -> Self {
    return Self {
      name: name.to_string(),
      render_timeout: 1_000_000_000,
    };
  }

  /// The time rendering has to complete before a timeout occurs.
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
    let surface = Rc::new(
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

    return RenderContext {
      name,
      instance,
      gpu,
      surface: surface.clone(),
      frame_buffer: None,
      submission_fence: Some(submission_fence),
      render_semaphore: Some(render_semaphore),
      command_pool: Some(command_pool),
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
  render_passes: Vec<RenderPass>,
  render_pipelines: Vec<RenderPipeline>,
}

pub type ResourceId = usize;

impl RenderContext {
  /// Permanently transfer a render pipeline to the render context in exchange
  /// for a resource ID that you can use in render commands.
  pub fn attach_pipeline(&mut self, pipeline: RenderPipeline) -> ResourceId {
    let index = self.render_pipelines.len();
    self.render_pipelines.push(pipeline);
    return index;
  }

  /// Permanently transfer a render pipeline to the render context in exchange
  /// for a resource ID that you can use in render commands.
  pub fn attach_render_pass(&mut self, render_pass: RenderPass) -> ResourceId {
    let index = self.render_passes.len();
    self.render_passes.push(render_pass);
    return index;
  }

  /// destroys the RenderContext and all associated resources.
  pub fn destroy(mut self) {
    logging::debug!("{} will now start destroying resources.", self.name);

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

    self
      .command_pool
      .as_mut()
      .unwrap()
      .deallocate_command_buffer("primary");

    self
      .command_pool
      .take()
      .expect("Couldn't take the command pool from the context and destroy it.")
      .destroy(&self.gpu);

    // Destroy render passes.
    let mut render_passes = vec![];
    swap(&mut self.render_passes, &mut render_passes);

    for render_pass in render_passes {
      render_pass.destroy(&self);
    }

    // Destroy render pipelines.
    let mut render_pipelines = vec![];
    swap(&mut self.render_pipelines, &mut render_pipelines);

    for render_pipeline in render_pipelines {
      render_pipeline.destroy(&self);
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

  /// Allocates a command buffer and records commands to the GPU. This is the
  /// primary entry point for submitting commands to the GPU and where rendering
  /// will occur.
  pub fn render(&mut self, commands: Vec<RenderCommand>) {
    let (width, height) = self
      .surface
      .size()
      .expect("Surface has no size configured.");

    let swapchain = SwapchainBuilder::new()
      .with_size(width, height)
      .build(&self.gpu, &self.surface);

    if self.surface.needs_swapchain() {
      Rc::get_mut(&mut self.surface)
        .expect("Failed to get mutable reference to surface.")
        .apply_swapchain(&self.gpu, swapchain, 1_000_000_000)
        .expect("Failed to apply the swapchain to the surface.");
    }

    self
      .submission_fence
      .as_mut()
      .expect("Failed to get the submission fence.")
      .block_until_ready(&mut self.gpu, None);

    let platform_command_list = commands
      .into_iter()
      .map(|command| command.into_platform_command(self))
      .collect();

    let mut command_buffer =
      CommandBufferBuilder::new(CommandBufferLevel::Primary)
        .with_feature(CommandBufferFeatures::ResetEverySubmission)
        .build(
          self
            .command_pool
            .as_mut()
            .expect("No command pool to create a buffer from"),
          "primary",
        );

    // Start recording commands, issue the high level render commands
    // that came from an application, and then submit the commands to the GPU
    // for rendering.
    command_buffer.issue_command(PlatformRenderCommand::BeginRecording);
    command_buffer.issue_commands(platform_command_list);
    command_buffer.issue_command(PlatformRenderCommand::EndRecording);

    self.gpu.submit_command_buffer(
      &mut command_buffer,
      vec![self.render_semaphore.as_ref().unwrap()],
      self.submission_fence.as_mut().unwrap(),
    );

    self
      .gpu
      .render_to_surface(
        Rc::get_mut(&mut self.surface)
          .expect("Failed to obtain a surface to render on."),
        self.render_semaphore.as_mut().unwrap(),
      )
      .expect("Failed to render to the surface");

    // Destroys the frame buffer after the commands have been submitted and the
    // frame buffer is no longer needed.
    match self.frame_buffer {
      Some(_) => {
        Rc::try_unwrap(self.frame_buffer.take().unwrap())
          .expect("Failed to unwrap the frame buffer.")
          .destroy(&self.gpu);
      }
      None => {}
    }
  }

  pub fn resize(&mut self, width: u32, height: u32) {
    let swapchain = SwapchainBuilder::new()
      .with_size(width, height)
      .build(&self.gpu, &self.surface);

    if self.surface.needs_swapchain() {
      Rc::get_mut(&mut self.surface)
        .expect("Failed to get mutable reference to surface.")
        .apply_swapchain(&self.gpu, swapchain, 1_000_000_000)
        .expect("Failed to apply the swapchain to the surface.");
    }
  }

  /// Get the render pass with the resource ID that was provided upon
  /// attachment.
  pub fn get_render_pass(&self, id: ResourceId) -> &RenderPass {
    return &self.render_passes[id];
  }

  /// Get the render pipeline with the resource ID that was provided upon
  /// attachment.
  pub fn get_render_pipeline(&mut self, id: ResourceId) -> &RenderPipeline {
    return &self.render_pipelines[id];
  }
}

impl RenderContext {
  /// Internal access to the RenderContext's GPU.
  pub(super) fn internal_gpu(&self) -> &internal::Gpu<internal::RenderBackend> {
    return &self.gpu;
  }

  /// Internal mutable access to the RenderContext's GPU.
  pub(super) fn internal_mutable_gpu(
    &mut self,
  ) -> &mut internal::Gpu<internal::RenderBackend> {
    return &mut self.gpu;
  }

  pub(super) fn internal_surface(
    &self,
  ) -> Rc<lambda_platform::gfx::surface::Surface<internal::RenderBackend>> {
    return self.surface.clone();
  }
}

type PlatformRenderCommand = Command<internal::RenderBackend>;

pub(crate) mod internal {

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
}
