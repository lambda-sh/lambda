use std::borrow::Borrow;

use gfx_hal::{
  device::Device,
  image::{
    Extent,
    FramebufferAttachment,
  },
};

use super::{
  gpu::Gpu,
  render_pass::RenderPass,
  surface::Surface,
};

pub struct Framebuffer {}

pub struct FramebufferBuilder {}

impl FramebufferBuilder {
  pub fn new() -> Self {
    return Self {};
  }

  pub fn build<RenderBackend: gfx_hal::Backend>(
    self,
    gpu: &mut Gpu<RenderBackend>,
    render_pass: &RenderPass<RenderBackend>,
    surface: &Surface<RenderBackend>,
  ) {
    let [width, height] = surface.size().expect("A surface without a swapchain cannot be used in a framebeen configured with a swapchain");
    let image =
      super::surface::internal::frame_buffer_attachment_from(&surface).unwrap();

    let framebuffer = unsafe {
      super::gpu::internal::logical_device_for(gpu).create_framebuffer(
        super::render_pass::internal::render_pass_for(render_pass),
        vec![image].into_iter(),
        Extent {
          width,
          height,
          depth: 1,
        },
      )
    };
  }
}
