use gfx_hal::{
  format::Format,
  window::{
    Extent2D,
    PresentationSurface,
    Surface as _,
    SwapchainConfig,
  },
};

use super::{
  gpu::Gpu,
  Instance,
};

/// Defines a surface bound
pub struct Surface<RenderBackend: gfx_hal::Backend> {
  gfx_hal_surface: RenderBackend::Surface,
}

impl<RenderBackend: gfx_hal::Backend> Surface<RenderBackend> {
  /// Create a new surface using the rendering APIs surface and the instance
  /// that created it.
  pub fn new(surface: RenderBackend::Surface) -> Self {
    return Self {
      gfx_hal_surface: surface,
    };
  }

  /// Checks if the surface can support
  pub fn can_support_queue_family(
    &self,
    family: &RenderBackend::QueueFamily,
  ) -> bool {
    return self.gfx_hal_surface.supports_queue_family(family);
  }

  pub fn get_supported_formats(
    &self,
    physical_device: &RenderBackend::PhysicalDevice,
  ) -> Vec<gfx_hal::format::Format> {
    return self
      .gfx_hal_surface
      .supported_formats(physical_device)
      .unwrap_or(vec![]);
  }

  pub fn get_first_supported_format(
    &self,
    physical_device: &RenderBackend::PhysicalDevice,
  ) -> gfx_hal::format::Format {
    let supported_formats = self.get_supported_formats(physical_device);
    let default_format = *supported_formats
      .get(0)
      .unwrap_or(&gfx_hal::format::Format::Rgba8Srgb);

    return supported_formats
      .into_iter()
      .find(|format| -> bool {
        format.base_format().1 == gfx_hal::format::ChannelType::Srgb
      })
      .unwrap_or(default_format);
  }

  /// Generates a swapchain configuration for a the given GPU.
  pub fn generate_swapchain_config(
    &self,
    gpu: &Gpu<RenderBackend>,
    size: [u32; 2],
  ) -> SwapchainConfig {
    let physical_device = gpu.get_physical_device();

    let caps = self.gfx_hal_surface.capabilities(physical_device);
    let format = self.get_first_supported_format(physical_device);
    let mut swapchain_config = SwapchainConfig::from_caps(
      &caps,
      format,
      Extent2D {
        width: size[0],
        height: size[1],
      },
    );

    // TODO(vmarcella) Profile the performance on MacOS to see if this slows
    // down frame times.
    if caps.image_count.contains(&3) {
      swapchain_config.image_count = 3;
    }

    return swapchain_config;
  }

  /// Apply a swapchain configuration for any given device.
  pub fn apply_swapchain_config(
    &mut self,
    gpu: &Gpu<RenderBackend>,
    swapchain_config: SwapchainConfig,
  ) -> (Extent2D, gfx_hal::image::FramebufferAttachment) {
    let device = gpu.get_logical_device();
    let surface_extent = swapchain_config.extent;
    let fba = swapchain_config.framebuffer_attachment();

    unsafe {
      self
        .gfx_hal_surface
        .configure_swapchain(device, swapchain_config)
        .expect("Failed to configure the swapchain");
    }

    return (surface_extent, fba);
  }

  /// Remove the swapchain configuration that this surface used on this given
  /// GPU.
  pub fn remove_swapchain_config(&mut self, gpu: &Gpu<RenderBackend>) {
    unsafe {
      self
        .gfx_hal_surface
        .unconfigure_swapchain(gpu.get_logical_device());
    }
  }
}

/// Destroy a surface given the instant that created it.
pub fn destroy_surface<RenderBackend: gfx_hal::Backend>(
  instance: &Instance<RenderBackend>,
  surface: Surface<RenderBackend>,
) {
  instance.destroy_surface(surface.gfx_hal_surface);
}
