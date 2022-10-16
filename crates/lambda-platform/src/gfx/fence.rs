//! GPU synchronization Implementations that are built on top of gfx-hal and
//! are used by the lambda-platform rendering implementations to synchronize
//! GPU operations.

use gfx_hal::device::Device;

pub struct RenderSemaphoreBuilder {}

impl RenderSemaphoreBuilder {
  pub fn new() -> Self {
    return Self {};
  }

  /// Builds a new render semaphore using the provided GPU. This semaphore can
  /// only be used with the GPU that it was created with.
  pub fn build<RenderBackend: gfx_hal::Backend>(
    self,
    gpu: &mut super::gpu::Gpu<RenderBackend>,
  ) -> RenderSemaphore<RenderBackend> {
    let semaphore = super::internal::logical_device_for(gpu)
      .create_semaphore()
      .expect("The GPU has no memory to allocate the semaphore");

    return RenderSemaphore { semaphore };
  }
}

/// Render semaphores are used to synchronize operations happening within the
/// GPU. This allows for us to tell the GPU to wait for a frame to finish
/// rendering before presenting it to the screen.
pub struct RenderSemaphore<RenderBackend: gfx_hal::Backend> {
  semaphore: RenderBackend::Semaphore,
}

impl<RenderBackend: gfx_hal::Backend> RenderSemaphore<RenderBackend> {
  /// Destroys the semaphore using the GPU that created it.
  pub fn destroy(self, gpu: &super::gpu::Gpu<RenderBackend>) {
    unsafe {
      super::internal::logical_device_for(gpu).destroy_semaphore(self.semaphore)
    }
  }
}

pub struct RenderSubmissionFenceBuilder {
  default_render_timeout: u64,
}

impl RenderSubmissionFenceBuilder {
  /// Creates a new Render Submission Fence Builder that defaults to a 1 second
  /// timeout for waiting on the fence.
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

  /// Builds a new submission fence using the provided GPU. This fence can only
  /// be used to block operation on the GPU that created it.
  pub fn build<RenderBackend: gfx_hal::Backend>(
    self,
    gpu: &mut super::gpu::Gpu<RenderBackend>,
  ) -> RenderSubmissionFence<RenderBackend> {
    let fence = super::gpu::internal::logical_device_for(gpu)
      .create_fence(true)
      .expect("There is not enough memory to create a fence on this device.");

    return RenderSubmissionFence {
      fence,
      default_render_timeout: self.default_render_timeout,
    };
  }
}

/// A GPU fence is used to synchronize GPU operations. It is used to ensure that
/// a GPU operation has completed before the CPU attempts to submit commands to
/// it.
pub struct RenderSubmissionFence<RenderBackend: gfx_hal::Backend> {
  fence: RenderBackend::Fence,
  default_render_timeout: u64,
}

impl<RenderBackend: gfx_hal::Backend> RenderSubmissionFence<RenderBackend> {
  /// Block a GPU until the fence is ready and then reset the fence status.
  pub fn block_until_ready(
    &mut self,
    gpu: &mut super::gpu::Gpu<RenderBackend>,
    render_timeout_override: Option<u64>,
  ) {
    let timeout = match render_timeout_override {
      Some(render_timeout_override) => render_timeout_override,
      None => self.default_render_timeout,
    };

    unsafe {
      super::gpu::internal::logical_device_for(gpu)
        .wait_for_fence(&self.fence, timeout)
    }
    .expect("The GPU ran out of memory or has become detached from the current context.");

    unsafe {
      super::gpu::internal::logical_device_for(gpu).reset_fence(&mut self.fence)
    }
    .expect("The fence failed to reset.");
  }

  /// Destroy this fence given the GPU that created it.
  #[inline]
  pub fn destroy(self, gpu: &super::gpu::Gpu<RenderBackend>) {
    unsafe {
      super::gpu::internal::logical_device_for(gpu).destroy_fence(self.fence)
    }
  }
}

pub mod internal {
  /// Retrieve the underlying submission fence.
  pub fn mutable_fence_for<RenderBackend: gfx_hal::Backend>(
    fence: &mut super::RenderSubmissionFence<RenderBackend>,
  ) -> &mut RenderBackend::Fence {
    return &mut fence.fence;
  }

  /// Retrieve the underlying semaphore.
  pub fn mutable_semaphore_for<RenderBackend: gfx_hal::Backend>(
    semaphore: &mut super::RenderSemaphore<RenderBackend>,
  ) -> &mut RenderBackend::Semaphore {
    return &mut semaphore.semaphore;
  }
}
