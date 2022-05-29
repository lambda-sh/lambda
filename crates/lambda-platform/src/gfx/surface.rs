/// ColorFormat for the surface.
pub use gfx_hal::format::Format as ColorFormat;
use gfx_hal::window::{
  Extent2D, PresentationSurface, Surface as _, SwapchainConfig,
};

use super::{gpu::Gpu, internal, Instance};

pub struct SurfaceBuilder {
  name: Option<String>,
}

impl SurfaceBuilder {
  pub fn new() -> Self {
    return Self { name: None };
  }

  pub fn with_name(mut self, name: &str) -> Self {
    self.name = Some(name.to_string());
    return self;
  }

  /// Build a surface using a graphical instance & active window
  pub fn build<RenderBackend: gfx_hal::Backend>(
    self,
    instance: &Instance<RenderBackend>,
    window: &crate::winit::WindowHandle,
  ) -> Surface<RenderBackend> {
    let gfx_hal_surface = internal::create_surface(instance, window);
    let name = match self.name {
      Some(name) => name,
      None => "LambdaSurface".to_string(),
    };

    return Surface {
      name,
      gfx_hal_surface,
    };
  }
}

/// Defines a surface bound
pub struct Surface<RenderBackend: gfx_hal::Backend> {
  name: String,
  gfx_hal_surface: RenderBackend::Surface,
}

impl<RenderBackend: gfx_hal::Backend> Surface<RenderBackend> {
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
  ) -> ColorFormat {
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
    let physical_device = internal::physical_device_for(gpu);

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
    let device = internal::logical_device_for(gpu);
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
    println!("Removing the swapchain configuration from: {}", self.name);

    unsafe {
      self
        .gfx_hal_surface
        .unconfigure_swapchain(internal::logical_device_for(gpu));
    }
  }

  /// Destroy the current surface and it's underlying resources.
  #[inline]
  pub fn destroy(self, instance: &Instance<RenderBackend>) {
    println!("Destroying the surface: {}", self.name);
    internal::destroy_surface(&instance, self.gfx_hal_surface);
  }
}
