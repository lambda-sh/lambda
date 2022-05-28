use gfx_hal::device::Device;

use super::gpu::Gpu;

pub struct RenderSemaphoreBuilder {}

impl RenderSemaphoreBuilder {
  pub fn new() -> Self {
    return Self {};
  }

  pub fn build<RenderBackend: gfx_hal::Backend>(
    self,
    gpu: &mut Gpu<RenderBackend>,
  ) -> RenderSemaphore<RenderBackend> {
    let semaphore = gpu
      .get_logical_device()
      .create_semaphore()
      .expect("The GPU has no memory to allocate the semaphore");

    return RenderSemaphore { semaphore };
  }
}
pub struct RenderSemaphore<RenderBackend: gfx_hal::Backend> {
  semaphore: RenderBackend::Semaphore,
}

impl<RenderBackend: gfx_hal::Backend> RenderSemaphore<RenderBackend> {
  /// Destroys the semaphore using the GPU that created it.
  pub fn destroy(self, gpu: &mut Gpu<RenderBackend>) {
    unsafe { gpu.get_logical_device().destroy_semaphore(self.semaphore) }
  }
}

pub struct RenderSubmissionFenceBuilder {
  default_render_timeout: u64,
}

impl RenderSubmissionFenceBuilder {
  pub fn new() -> Self {
    return Self {
      default_render_timeout: 1_000_000_000,
    };
  }

  /// Provides a default render timeout in nanoseconds. This render timeout is
  /// used to reset the submission fence if it's time-to-live expires.
  pub fn with_render_timeout(mut self, render_timeout: u64) -> Self {
    self.default_render_timeout = render_timeout;
    return self;
  }

  pub fn build<RenderBackend: gfx_hal::Backend>(
    self,
    gpu: &mut Gpu<RenderBackend>,
  ) -> RenderSubmissionFence<RenderBackend> {
    let fence = gpu
      .get_logical_device()
      .create_fence(true)
      .expect("There is not enough memory to create a fence on this device.");

    return RenderSubmissionFence {
      fence,
      default_render_timeout: self.default_render_timeout,
    };
  }
}

pub struct RenderSubmissionFence<RenderBackend: gfx_hal::Backend> {
  fence: RenderBackend::Fence,
  default_render_timeout: u64,
}

impl<RenderBackend: gfx_hal::Backend> RenderSubmissionFence<RenderBackend> {
  /// Block a GPU until the fence is ready and then reset the fence status.
  pub fn block_until_ready(
    &mut self,
    gpu: &mut Gpu<RenderBackend>,
    render_timeout_override: Option<u64>,
  ) {
    let timeout = match render_timeout_override {
      Some(render_timeout_override) => render_timeout_override,
      None => self.default_render_timeout,
    };

    unsafe {
      gpu
        .get_logical_device()
        .wait_for_fence(&self.fence, timeout)
        .expect("The GPU ran out of memory or has become detached from the current context.");

      gpu
        .get_logical_device()
        .reset_fence(&mut self.fence)
        .expect("The fence failed to reset.");
    }
  }

  /// Destroy this fence given the GPU that created it.
  #[inline]
  pub fn destroy(self, gpu: &mut Gpu<RenderBackend>) {
    unsafe { gpu.get_logical_device().destroy_fence(self.fence) }
  }
}
