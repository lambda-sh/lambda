use gfx_hal::{
  device::Device,
  image::Extent,
};

use super::{
  gpu::Gpu,
  render_pass::RenderPass,
  surface::Surface,
};

/// Framebuffer for the given render backend.
pub struct Framebuffer<RenderBackend: gfx_hal::Backend> {
  frame_buffer: RenderBackend::Framebuffer,
}

impl<RenderBackend: gfx_hal::Backend> Framebuffer<RenderBackend> {
  /// Destroys the framebuffer from the given GPU.
  pub fn destroy(self, gpu: &super::gpu::Gpu<RenderBackend>) {
    unsafe {
      super::gpu::internal::logical_device_for(gpu)
        .destroy_framebuffer(self.frame_buffer);
    }
  }
}
pub struct FramebufferBuilder {}

impl FramebufferBuilder {
  pub fn new() -> Self {
    return Self {};
  }

  /// Build a frame buffer on a given GPU for the given surface.
  pub fn build<RenderBackend: gfx_hal::Backend>(
    self,
    gpu: &mut Gpu<RenderBackend>,
    render_pass: &RenderPass<RenderBackend>,
    surface: &Surface<RenderBackend>,
  ) -> Framebuffer<RenderBackend> {
    use super::surface::internal::frame_buffer_attachment_from;

    let (width, height) = surface.size().expect("A surface without a swapchain cannot be used in a framebeen configured with a swapchain");
    let image = frame_buffer_attachment_from(surface).unwrap();

    let frame_buffer = unsafe {
      super::gpu::internal::logical_device_for(gpu)
        .create_framebuffer(
          super::render_pass::internal::render_pass_for(render_pass),
          vec![image].into_iter(),
          Extent {
            width,
            height,
            depth: 1,
          },
        )
        .expect("Failed to create a framebuffer")
    };
    return Framebuffer { frame_buffer };
  }
}

/// Internal functions to work with gfx-hal framebuffers directly. Applications
/// should not need to use these functions directly.
pub mod internal {
  pub fn frame_buffer_for<RenderBackend: gfx_hal::Backend>(
    frame_buffer: &super::Framebuffer<RenderBackend>,
  ) -> &RenderBackend::Framebuffer {
    return &frame_buffer.frame_buffer;
  }
}
