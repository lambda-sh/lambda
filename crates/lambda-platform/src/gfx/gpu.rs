use std::mem::size_of;

use gfx_hal::{
  adapter::Adapter,
  device::Device,
  format::Format,
  image::{
    Access,
    Layout,
  },
  memory::Dependencies,
  pass::{
    Attachment,
    AttachmentLoadOp,
    AttachmentOps,
    AttachmentStoreOp,
    SubpassDependency,
    SubpassDesc,
  },
  prelude::{
    PhysicalDevice,
    QueueFamily,
  },
  pso::{
    PipelineStage,
    ShaderStageFlags,
  },
  queue::{
    Queue,
    QueueGroup,
  },
  window::{
    Extent2D,
    PresentError,
    PresentationSurface,
    Suboptimal,
  },
};

use super::pipeline::GraphicsPipeline;

///
/// Commands oriented around creating resources on & for the GPU.
///
pub struct Gpu<B: gfx_hal::Backend> {
  adapter: gfx_hal::adapter::Adapter<B>,
  gpu: gfx_hal::adapter::Gpu<B>,
  queue_group: QueueGroup<B>,
}

#[derive(Clone, Copy)]
pub enum RenderQueueType {
  Compute,
  Graphical,
  GraphicalCompute,
  Transfer,
}

impl<B: gfx_hal::Backend> Gpu<B> {
  /// Instantiates a new GPU given an adapter that is implemented by the GPUs
  /// current rendering backend B. A new GPU does not come with a command pool unless specified.
  pub fn new(
    adapter: Adapter<B>,
    queue_family: gfx_hal::queue::QueueFamilyId,
  ) -> Self {
    let queue_family = adapter
      .queue_families
      .iter()
      .find(|family| family.id() == queue_family)
      .expect("Failed to find the queue family requested for the GPU.");

    let mut gpu = unsafe {
      adapter
        .physical_device
        .open(&[(queue_family, &[1.0])], gfx_hal::Features::empty())
        .expect("Failed to open the device.")
    };

    let queue_group = gpu.queue_groups.pop().unwrap();

    return Self {
      adapter,
      gpu,
      queue_group,
    };
  }

  pub fn get_queue_family_id(&self) -> gfx_hal::queue::QueueFamilyId {
    return self.queue_group.family;
  }

  // TODO(vmarcella): A command pool allocated GPU should be implemented via a
  // GPU typestate. For example, with_command_pool should return a gpu with an
  // type signature something along the lines of: GPU<CommandReady>
  /// Attaches a command pool to the current gfx gpu.
  pub fn create_command_pool(
    &self,
    flags: gfx_hal::pool::CommandPoolCreateFlags,
  ) -> B::CommandPool {
    return unsafe {
      self.gpu.device.create_command_pool(self.queue_group.family, flags)
				.expect("The GPU could not allocate a command pool because it is out of memory")
    };
  }

  /// Submits a command buffer to the GPU.
  pub fn submit_command_buffer(
    &mut self,
    command_buffer: &B::CommandBuffer,
    semaphore: &B::Semaphore,
    fence: &mut B::Fence,
  ) {
    let commands = vec![command_buffer].into_iter();
    let semaphores = vec![semaphore].into_iter();
    unsafe {
      self.queue_group.queues[0].submit(
        commands,
        vec![].into_iter(),
        semaphores,
        Some(fence),
      );
    }
  }

  /// Get the underlying logical device for the logical GPU.
  pub fn get_logical_device(&self) -> &B::Device {
    return &self.gpu.device;
  }

  /// Get the underlying physical device for the virtual GPU.
  pub fn get_physical_device(&self) -> &B::PhysicalDevice {
    return &self.adapter.physical_device;
  }

  /// Render to the surface and return the result from the GPU.
  pub fn render_to_surface(
    &mut self,
    surface: &mut B::Surface,
    image: <B::Surface as PresentationSurface<B>>::SwapchainImage,
    semaphore: &mut B::Semaphore,
  ) -> Result<Option<Suboptimal>, PresentError> {
    unsafe {
      let result =
        self.queue_group.queues[0].present(surface, image, Some(semaphore));

      return result;
    }
  }

  /// Create a frame buffer on the GPU.
  pub fn create_frame_buffer(
    &mut self,
    render_pass: &B::RenderPass,
    image: gfx_hal::image::FramebufferAttachment,
    dimensions: &Extent2D,
  ) -> B::Framebuffer {
    unsafe {
      use gfx_hal::image::Extent;
      return self
        .gpu
        .device
        .create_framebuffer(
          &render_pass,
          vec![image].into_iter(),
          Extent {
            width: dimensions.width,
            height: dimensions.height,
            depth: 1,
          },
        )
        .unwrap();
    }
  }

  /// Destroy a frame buffer that was created by the GPU.
  pub fn destroy_frame_buffer(&mut self, frame_buffer: B::Framebuffer) {
    unsafe {
      self.gpu.device.destroy_framebuffer(frame_buffer);
    }
  }

  /// Create a pipeline layout on the GPU.
  pub fn create_pipeline_layout(&mut self) -> B::PipelineLayout {
    unsafe {
      // wait, I think I have a hack for this
      let max: u32 = size_of::<u32>() as u32;
      return self
        .gpu
        .device
        .create_pipeline_layout(
          vec![].iter(),
          vec![(ShaderStageFlags::VERTEX, 0..max)].into_iter(),
        )
        .expect("Out of memory.");
    }
  }

  /// Destroy a pipeline layout that was allocated by this GPU.
  pub fn destroy_pipeline_layout(
    &mut self,
    pipeline_layout: B::PipelineLayout,
  ) {
    unsafe { self.gpu.device.destroy_pipeline_layout(pipeline_layout) }
  }

  /// Create a render pass with the current using the current GPU resources.
  pub fn create_render_pass(
    &mut self,
    resource_attachments: Option<Vec<Attachment>>,
    render_subpasses: Option<Vec<SubpassDesc>>,
    dependencies: Option<Vec<SubpassDependency>>,
  ) -> B::RenderPass {
    // Use attached resources or create a stub for pipeline compatibility
    let attachments = match resource_attachments {
      Some(attachments) => attachments,
      None => {
        vec![Attachment {
          format: Some(Format::Rgba8Srgb),
          samples: 1,
          ops: AttachmentOps::new(
            AttachmentLoadOp::Clear,
            AttachmentStoreOp::Store,
          ),
          stencil_ops: AttachmentOps::DONT_CARE,
          layouts: Layout::Undefined..Layout::Present,
        }]
      }
    };

    // Use attached render subpasses or create a stub for pipeline compatibility.
    let subpasses = match render_subpasses {
      Some(subpasses) => subpasses,
      None => vec![SubpassDesc {
        colors: &[(0, Layout::ColorAttachmentOptimal)],
        depth_stencil: None,
        inputs: &[],
        resolves: &[],
        preserves: &[],
      }],
    };

    // Use attached dependencies or create a stub for pipeline compatibility.
    let deps = match dependencies {
      Some(deps) => deps,
      None => vec![SubpassDependency {
        accesses: Access::COLOR_ATTACHMENT_READ..Access::COLOR_ATTACHMENT_WRITE,
        flags: Dependencies::empty(),
        passes: None..None,
        stages: PipelineStage::BOTTOM_OF_PIPE
          ..PipelineStage::COLOR_ATTACHMENT_OUTPUT,
      }],
    };

    // TODO(vmarcella): Error handling here should propagate an Error upwards.
    unsafe {
      return self
        .gpu
        .device
        .create_render_pass(attachments.into_iter(), subpasses.into_iter(), deps.into_iter())
        .expect("Your primary graphics card does not have enough memory for this render pass.");
    };
  }

  pub fn destroy_render_pass(&mut self, render_pass: B::RenderPass) {
    unsafe {
      self.gpu.device.destroy_render_pass(render_pass);
    }
  }

  pub fn create_shader_module(&mut self, binary: &Vec<u32>) -> B::ShaderModule {
    unsafe {
      let module = self
        .gpu
        .device
        .create_shader_module(&binary)
        .expect("Failed to create a shader module.");
      return module;
    }
  }

  /// Destroy a Shader Module created by this GPU.
  pub fn destroy_shader_module(&mut self, shader_module: B::ShaderModule) {
    unsafe {
      self.gpu.device.destroy_shader_module(shader_module);
    }
  }

  /// Create a graphics pipeline from the GPU.
  pub fn create_graphics_pipeline(
    &self,
    pipeline: &mut GraphicsPipeline<B>,
  ) -> B::GraphicsPipeline {
    unsafe {
      let pipeline = self
        .gpu
        .device
        .create_graphics_pipeline(&mut pipeline.get_pipeline(), None)
        .expect("Failed to create a Graphics pipeline on the GPU.");
      return pipeline;
    }
  }

  /// Destroy a graphics pipeline allocated by this GPU.
  pub fn destroy_graphics_pipeline(&self, pipeline: B::GraphicsPipeline) {
    unsafe {
      self.gpu.device.destroy_graphics_pipeline(pipeline);
    }
  }

  /// Create access fences for synchronizing with the current GPU.
  pub fn create_access_fences(&mut self) -> (B::Fence, B::Semaphore) {
    let submission_complete_fence = self
      .gpu
      .device
      .create_fence(true)
      .expect("Ran out of memory when trying to create.");

    let semaphore_fence =
      self.gpu.device.create_semaphore().expect("Out of memory");

    return (submission_complete_fence, semaphore_fence);
  }
}
