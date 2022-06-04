use std::mem::size_of;

use gfx_hal::{
  adapter::Adapter,
  device::Device,
  prelude::{
    PhysicalDevice,
    QueueFamily,
  },
  pso::ShaderStageFlags,
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

pub mod internal {
  use super::Gpu;

  /// Retrieves the gfx_hal logical device for a given GPU.
  #[inline]
  pub fn logical_device_for<RenderBackend: gfx_hal::Backend>(
    gpu: &Gpu<RenderBackend>,
  ) -> &RenderBackend::Device {
    return &gpu.gpu.device;
  }

  #[inline]
  pub fn physical_device_for<RenderBackend: gfx_hal::Backend>(
    gpu: &Gpu<RenderBackend>,
  ) -> &RenderBackend::PhysicalDevice {
    return &gpu.adapter.physical_device;
  }

  #[inline]
  pub fn queue_family_for<RenderBackend: gfx_hal::Backend>(
    gpu: &Gpu<RenderBackend>,
  ) -> gfx_hal::queue::QueueFamilyId {
    return gpu.queue_group.family;
  }
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
}
