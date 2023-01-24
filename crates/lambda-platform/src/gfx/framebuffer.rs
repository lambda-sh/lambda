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
#[derive(Debug)]
pub struct Framebuffer<RenderBackend: gfx_hal::Backend> {
  frame_buffer: RenderBackend::Framebuffer,
}

impl<RenderBackend: gfx_hal::Backend> Framebuffer<RenderBackend> {
  /// Destroys the framebuffer from the given GPU.
  pub fn destroy(self, gpu: &super::gpu::Gpu<RenderBackend>) {
    unsafe {
      gpu
        .internal_logical_device()
        .destroy_framebuffer(self.frame_buffer);
    }
  }
}

impl<RenderBackend: gfx_hal::Backend> Framebuffer<RenderBackend> {
  /// Retrieve a reference to the internal frame buffer.
  pub(super) fn internal_frame_buffer(&self) -> &RenderBackend::Framebuffer {
    return &self.frame_buffer;
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
      gpu
        .internal_logical_device()
        .create_framebuffer(
          render_pass.internal_render_pass(),
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
