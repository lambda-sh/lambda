use std::borrow::Borrow;

/// ColorFormat for the surface.
pub use gfx_hal::format::Format as ColorFormat;
use gfx_hal::{
  queue::Queue,
  window::{
    PresentationSurface,
    Surface as _,
  },
};

use super::{
  gpu::{
    internal::primary_queue_for,
    Gpu,
  },
  Instance,
};

/// Internal Surface functions.
pub mod internal {
  use std::{
    borrow::Borrow,
    fmt::Debug,
  };

  use gfx_hal::window::{
    PresentationSurface,
    Surface as _,
  };

  /// Checks the queue family if the current Surface can support the GPU.
  pub fn can_support_queue_family<RenderBackend: gfx_hal::Backend>(
    surface: &super::Surface<RenderBackend>,
    queue_family: &RenderBackend::QueueFamily,
  ) -> bool {
    return surface.gfx_hal_surface.supports_queue_family(queue_family);
  }

  /// Get the supported gfx_hal color formats for a given format.
  pub fn get_supported_formats<RenderBackend: gfx_hal::Backend>(
    surface: &super::Surface<RenderBackend>,
    physical_device: &RenderBackend::PhysicalDevice,
  ) -> Vec<gfx_hal::format::Format> {
    return surface
      .gfx_hal_surface
      .supported_formats(physical_device)
      .unwrap_or(vec![]);
  }

  /// Helper function to retrieve the first supported format given a physical
  /// GPU device.
  pub fn get_first_supported_format<RenderBackend: gfx_hal::Backend>(
    surface: &super::Surface<RenderBackend>,
    physical_device: &RenderBackend::PhysicalDevice,
  ) -> gfx_hal::format::Format {
    let supported_formats = get_supported_formats(&surface, physical_device);

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

  /// Acquires a surface image for attaching to a framebuffer.
  pub fn take_surface_image_for<RenderBackend: gfx_hal::Backend>(
    surface: &mut super::Surface<RenderBackend>,
  ) -> Option<<RenderBackend::Surface as PresentationSurface<RenderBackend>>::SwapchainImage>{
    return surface.image.take();
  }

  /// Acquires a surface image for attaching to a framebuffer.
  pub fn borrow_surface_image_for<RenderBackend: gfx_hal::Backend>(
    surface: &super::Surface<RenderBackend>,
  ) -> Option<&<RenderBackend::Surface as PresentationSurface<RenderBackend>>::SwapchainImage>{
    return surface.image.as_ref();
  }

  /// FrameBuffer Attachment
  pub fn frame_buffer_attachment_from<RenderBackend: gfx_hal::Backend>(
    surface: &super::Surface<RenderBackend>,
  ) -> Option<gfx_hal::image::FramebufferAttachment> {
    return surface.frame_buffer_attachment.clone();
  }

  pub fn surface_for<RenderBackend: gfx_hal::Backend>(
    surface: &mut super::Surface<RenderBackend>,
  ) -> &mut RenderBackend::Surface {
    return &mut surface.gfx_hal_surface;
  }

  /// Borrow the surface and take the image. This internal function is used for
  /// rendering and composes surface_for + take image.
  pub fn borrow_surface_and_take_image<RenderBackend: gfx_hal::Backend>(
    surface: &mut super::Surface<RenderBackend>,
  ) -> (&mut RenderBackend::Surface, <RenderBackend::Surface as PresentationSurface<RenderBackend>>::SwapchainImage){
    return (
      &mut surface.gfx_hal_surface,
      surface.image.take().expect(""),
    );
  }
}

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
    instance: &super::Instance<RenderBackend>,
    window: &crate::winit::WindowHandle,
  ) -> Surface<RenderBackend> {
    let gfx_hal_surface = super::internal::create_surface(instance, window);
    let name = match self.name {
      Some(name) => name,
      None => "LambdaSurface".to_string(),
    };

    return Surface {
      name,
      extent: None,
      gfx_hal_surface,
      swapchain_is_valid: true,
      image: None,
      frame_buffer_attachment: None,
    };
  }
}

/// Defines a surface that can be rendered on to.
pub struct Surface<RenderBackend: gfx_hal::Backend> {
  name: String,
  gfx_hal_surface: RenderBackend::Surface,
  extent: Option<gfx_hal::window::Extent2D>,
  swapchain_is_valid: bool,
  image: Option<
    <RenderBackend::Surface as gfx_hal::window::PresentationSurface<
      RenderBackend,
    >>::SwapchainImage,
  >,
  frame_buffer_attachment: Option<gfx_hal::image::FramebufferAttachment>,
}

pub struct Swapchain {
  config: gfx_hal::window::SwapchainConfig,
  format: gfx_hal::format::Format,
}

impl<RenderBackend: gfx_hal::Backend> Surface<RenderBackend> {
  /// Apply a swapchain to the current surface. This is required whenever a
  /// swapchain has been invalidated (I.E. by window resizing)
  pub fn apply_swapchain(
    &mut self,
    gpu: &Gpu<RenderBackend>,
    swapchain: Swapchain,
    timeout_in_nanoseconds: u64,
  ) -> Result<(), &str> {
    let device = super::gpu::internal::logical_device_for(gpu);
    self.extent = Some(swapchain.config.extent);

    unsafe {
      self
        .gfx_hal_surface
        .configure_swapchain(device, swapchain.config)
        .expect("Failed to configure the swapchain");

      let image =
        match self.gfx_hal_surface.acquire_image(timeout_in_nanoseconds) {
          Ok((image, _)) => Some(image),
          Err(_) => {
            self.swapchain_is_valid = false;
            None
          }
        };

      match image {
        Some(image) => {
          self.image = Some(image);
          return Ok(());
        }
        None => {
          return Err("Failed to apply the swapchain.");
        }
      }
    }
  }

  /// Remove the swapchain configuration that this surface used on this given
  /// GPU.
  pub fn remove_swapchain(&mut self, gpu: &Gpu<RenderBackend>) {
    println!("Removing the swapchain configuration from: {}", self.name);
    unsafe {
      self
        .gfx_hal_surface
        .unconfigure_swapchain(super::gpu::internal::logical_device_for(gpu));
    }
  }

  /// private function to invalidate the surface swapchain.
  #[inline]
  fn invalidate_swapchain(&mut self) {
    self.swapchain_is_valid = false;
  }

  /// Destroy the current surface and it's underlying resources.
  #[inline]
  pub fn destroy(self, instance: &Instance<RenderBackend>) {
    println!("Destroying the surface: {}", self.name);

    super::internal::destroy_surface(&instance, self.gfx_hal_surface);
  }

  /// Get the size of the surface's extent. Will only return a size if a
  /// swapchain has been applied to the surface to render with.
  pub fn size(&self) -> Option<[u32; 2]> {
    return match self.extent {
      Some(extent) => Some([extent.width, extent.height]),
      None => None,
    };
  }
}

// ------------------------------ SWAPCHAIN BUILDER ----------------------------

pub struct SwapchainBuilder {
  size: [u32; 2],
}

impl SwapchainBuilder {
  pub fn new() -> Self {
    return Self { size: [480, 360] };
  }

  pub fn with_size(mut self, width: u32, height: u32) -> Self {
    self.size = [width, height];
    return self;
  }

  pub fn build<RenderBackend: super::internal::Backend>(
    self,
    gpu: &Gpu<RenderBackend>,
    surface: &Surface<RenderBackend>,
  ) -> Swapchain {
    let physical_device = super::gpu::internal::physical_device_for(gpu);
    let caps = surface.gfx_hal_surface.capabilities(physical_device);
    let format = internal::get_first_supported_format(surface, physical_device);

    let mut swapchain_config = gfx_hal::window::SwapchainConfig::from_caps(
      &caps,
      format,
      gfx_hal::window::Extent2D {
        width: self.size[0],
        height: self.size[1],
      },
    );

    // TODO(vmarcella) Profile the performance on MacOS to see if this slows
    // down frame times.
    if caps.image_count.contains(&3) {
      swapchain_config.image_count = 3;
    }

    return Swapchain {
      config: swapchain_config,
      format,
    };
  }
}
