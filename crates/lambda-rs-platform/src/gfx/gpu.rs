use gfx_hal::{
  adapter::Adapter,
  prelude::{
    PhysicalDevice,
    QueueFamily,
  },
  queue::{
    Queue,
    QueueGroup,
  },
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
        let adapter = instance.first_adapter();

        let queue_family = adapter
          .queue_families
          .iter()
          .find(|family| {
            return surface.can_support_queue_family(family)
              && family.queue_type().supports_graphics();
          })
          .expect("No compatible queue family found.")
          .id();

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
  pub(super) fn new(
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
      self
        .queue_group
        .queues
        .first_mut()
        .expect("Couldn't find the primary queue to submit commands to. ")
        .submit(
          commands,
          vec![].into_iter(),
          // TODO(vmarcella): This was needed to allow the push constants to
          // properly render to the screen. Look into a better way to do this.
          signal_semaphores.into_iter().map(|semaphore| {
            return semaphore.internal_semaphore();
          }),
          Some(fence.internal_fence_mut()),
        );
    }
  }

  /// Render to the surface and return the result from the GPU.
  pub fn render_to_surface(
    &mut self,
    surface: &mut surface::Surface<RenderBackend>,
    semaphore: &mut RenderSemaphore<RenderBackend>,
  ) -> Result<(), &str> {
    let (render_surface, render_image) = surface.internal_surface_and_image();

    let result = unsafe {
      self.queue_group.queues[0].present(
        render_surface,
        render_image,
        Some(semaphore.internal_semaphore_mut()),
      )
    };

    if result.is_err() {
      logging::error!(
        "Failed to present to the surface: {:?}",
        result.err().unwrap()
      );
      surface.remove_swapchain(self);
      return Err(
        "Rendering failed. Swapchain for the surface needs to be reconfigured.",
      );
    }

    return Ok(());
  }
}

impl<RenderBackend: gfx_hal::Backend> Gpu<RenderBackend> {
  pub(super) fn internal_logical_device(&self) -> &RenderBackend::Device {
    return &self.gpu.device;
  }

  pub(super) fn internal_physical_device(
    &self,
  ) -> &RenderBackend::PhysicalDevice {
    return &self.adapter.physical_device;
  }

  pub(super) fn internal_queue_family(&self) -> gfx_hal::queue::QueueFamilyId {
    return self.queue_group.family;
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
