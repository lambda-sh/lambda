use gfx_hal::device::Device;

use super::gpu::Gpu;

pub struct RenderingSemaphoreBuilder {}

impl RenderingSemaphoreBuilder {
  pub fn build<RenderBackend: gfx_hal::Backend>() {}
}
pub struct RenderingSemaphore;

impl RenderingSemaphore {}

pub struct RenderingFenceBuilder {
  default_render_timeout: u64,
}

impl RenderingFenceBuilder {
  pub fn new() -> Self {
    return Self {
      default_render_timeout: 1_000_000_000,
    };
  }

  pub fn with_render_timeout(mut self, render_timeout: u64) -> Self {
    self.default_render_timeout = render_timeout;
    return self;
  }

  pub fn build<RenderBackend: gfx_hal::Backend>(
    self,
    gpu: &mut Gpu<RenderBackend>,
  ) -> RenderingFence<RenderBackend> {
    let fence = gpu
      .get_logical_device()
      .create_fence(true)
      .expect("There is not enough memory to create a fence on this device.");

    return RenderingFence {
      fence,
      default_render_timeout: self.default_render_timeout,
    };
  }
}

pub struct RenderingFence<RenderBackend: gfx_hal::Backend> {
  fence: RenderBackend::Fence,
  default_render_timeout: u64,
}

impl<RenderBackend: gfx_hal::Backend> RenderingFence<RenderBackend> {
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
  pub fn destroy(self, gpu: &mut Gpu<RenderBackend>) {
    unsafe { gpu.get_logical_device().destroy_fence(self.fence) }
  }
}
