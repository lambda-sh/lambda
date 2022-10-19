use gfx_hal::{
  adapter::Adapter,
  device::Device,
  prelude::{
    PhysicalDevice,
    QueueFamily,
  },
  queue::{
    Queue,
    QueueGroup,
  },
  window::Extent2D,
};
#[cfg(test)]
use mockall::automock;

use super::{
  command::CommandBuffer,
  fence::{
    RenderSemaphore,
    RenderSubmissionFence,
  },
  surface,
};

/// GpuBuilder for constructing a GPU
pub struct GpuBuilder {
  render_queue_type: RenderQueueType,
}

impl GpuBuilder {
  /// Create a new GpuBuilder to configure and build a GPU to use for rendering.
  pub fn new() -> Self {
    return Self {
      render_queue_type: RenderQueueType::Graphical,
    };
  }

  /// Set the type of queue to use for rendering. The GPU defaults to graphical.
  pub fn with_render_queue_type(mut self, queue_type: RenderQueueType) -> Self {
    self.render_queue_type = queue_type;
    return self;
  }

  /// If passing in a surface, the gpu will be built using the queue that best
  /// supports both the render queue & surface.
  pub fn build<RenderBackend: gfx_hal::Backend>(
    self,
    instance: &mut super::Instance<RenderBackend>,
    surface: Option<&surface::Surface<RenderBackend>>,
  ) -> Result<Gpu<RenderBackend>, String> {
    match (surface, self.render_queue_type) {
      (Some(surface), RenderQueueType::Graphical) => {
        let adapter = super::internal::get_adapter(instance, 0);

        let queue_family = adapter
          .queue_families
          .iter()
          .find(|family| {
            return surface::internal::can_support_queue_family(
              surface, family,
            ) && family.queue_type().supports_graphics();
          })
          .expect("No compatible queue family found.")
          .id();

        let _formats = surface::internal::get_first_supported_format(
          surface,
          &adapter.physical_device,
        );

        return Ok(Gpu::new(adapter, queue_family));
      }
      (Some(_surface), RenderQueueType::Compute) => {
        todo!("Support a Compute based GPU.")
      }
      (_, _) => return Err("Failed to build GPU.".to_string()),
    }
  }
}

/// Commands oriented around creating resources on & for the GPU.
pub struct Gpu<B: gfx_hal::Backend> {
  adapter: gfx_hal::adapter::Adapter<B>,
  gpu: gfx_hal::adapter::Gpu<B>,
  queue_group: QueueGroup<B>,
}

/// The render queue types that the GPU can use for
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RenderQueueType {
  Compute,
  Graphical,
  GraphicalCompute,
  Transfer,
}

impl<RenderBackend: gfx_hal::Backend> Gpu<RenderBackend> {
  /// Instantiates a new GPU given an adapter that is implemented by the GPUs
  /// current rendering backend B. A new GPU does not come with a command pool
  /// unless specified.
  pub fn new(
    adapter: Adapter<RenderBackend>,
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
  pub fn submit_command_buffer<'render_context>(
    &mut self,
    command_buffer: &mut CommandBuffer<RenderBackend>,
    signal_semaphores: Vec<&RenderSemaphore<RenderBackend>>,
    fence: &mut RenderSubmissionFence<RenderBackend>,
  ) {
    let commands =
      vec![super::command::internal::command_buffer_for(command_buffer)]
        .into_iter();
    unsafe {
      self.queue_group.queues[0].submit(
        commands,
        vec![].into_iter(),
        // TODO(vmarcella): This was needed to allow the push constants to
        // properly render to the screen. Look into a better way to do this.
        signal_semaphores.into_iter().map(|semaphore| {
          return super::fence::internal::semaphore_for(semaphore);
        }),
        Some(super::fence::internal::mutable_fence_for(fence)),
      );
    }
  }

  /// Render to the surface and return the result from the GPU.
  pub fn render_to_surface(
    &mut self,
    surface: &mut surface::Surface<RenderBackend>,
    semaphore: &mut RenderSemaphore<RenderBackend>,
  ) -> Result<(), &str> {
    let (render_surface, render_image) =
      super::surface::internal::borrow_surface_and_take_image(surface);

    let result = unsafe {
      self.queue_group.queues[0].present(
        render_surface,
        render_image,
        Some(super::fence::internal::mutable_semaphore_for(semaphore)),
      )
    };

    if result.is_err() {
      surface.remove_swapchain(self);
      return Err(
        "Rendering failed. Swapchain for the surface needs to be reconfigured.",
      );
    }

    return Ok(());
  }
}

#[cfg(test)]
mod tests {
  #[test]
  fn test_gpu_builder_default_state() {
    use super::{
      GpuBuilder,
      RenderQueueType,
    };

    let builder = GpuBuilder::new();

    assert_eq!(builder.render_queue_type, RenderQueueType::Graphical);
  }

  #[test]
  fn test_gpu_builder_with_render_queue_type() {
    use super::{
      GpuBuilder,
      RenderQueueType,
    };

    let builder =
      GpuBuilder::new().with_render_queue_type(RenderQueueType::Compute);

    assert_eq!(builder.render_queue_type, RenderQueueType::Compute);
  }

  #[test]
  fn test_gpu_builder_build() {}
}

// --------------------------------- GPU INTERNALS -----------------------------

pub mod internal {
  use super::Gpu;

  /// Retrieves the gfx_hal logical device for a given GPU.
  #[inline]
  pub fn logical_device_for<RenderBackend: gfx_hal::Backend>(
    gpu: &Gpu<RenderBackend>,
  ) -> &RenderBackend::Device {
    return &gpu.gpu.device;
  }

  /// Retrieves the gfx_hal physical device for a given GPU.
  #[inline]
  pub fn physical_device_for<RenderBackend: gfx_hal::Backend>(
    gpu: &Gpu<RenderBackend>,
  ) -> &RenderBackend::PhysicalDevice {
    return &gpu.adapter.physical_device;
  }

  /// Retrieves the gfx_hal queue group for a given GPU.
  #[inline]
  pub fn queue_family_for<RenderBackend: gfx_hal::Backend>(
    gpu: &Gpu<RenderBackend>,
  ) -> gfx_hal::queue::QueueFamilyId {
    return gpu.queue_group.family;
  }

  /// Retrieve the primary queue from the GPU.
  #[inline]
  pub fn primary_queue_for<RenderBackend: gfx_hal::Backend>(
    gpu: &Gpu<RenderBackend>,
  ) -> &RenderBackend::Queue {
    return &gpu.queue_group.queues[0];
  }
}
