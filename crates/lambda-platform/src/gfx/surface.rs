/// ColorFormat for the surface.
pub use gfx_hal::format::Format as ColorFormat;
use gfx_hal::{
  window::{
    PresentationSurface,
    Surface as _,
  },
  Backend,
};
#[cfg(test)]
use mockall::automock;

use super::{
  gpu::Gpu,
  Instance,
};

/// The API to use for building surfaces from a graphical instance.
#[derive(Debug, Clone)]
pub struct SurfaceBuilder {
  name: Option<String>,
}

#[cfg_attr(test, automock)]
impl SurfaceBuilder {
  pub fn new() -> Self {
    return Self { name: None };
  }

  /// Set the name of the surface.
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
    let gfx_hal_surface = instance.create_surface(window);
    let name = match self.name {
      Some(name) => name,
      None => "RenderSurface".to_string(),
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
#[derive(Debug)]
pub struct Surface<RenderBackend: gfx_hal::Backend> {
  name: String,
  gfx_hal_surface: RenderBackend::Surface,
  extent: Option<gfx_hal::window::Extent2D>,
  swapchain_is_valid: bool,
  // TODO(vmarcella): the Image type is very large
  image: Option<
    <RenderBackend::Surface as gfx_hal::window::PresentationSurface<
      RenderBackend,
    >>::SwapchainImage,
  >,
  frame_buffer_attachment: Option<gfx_hal::image::FramebufferAttachment>,
}

#[derive(Debug, Clone)]
pub struct Swapchain {
  config: gfx_hal::window::SwapchainConfig,
  format: gfx_hal::format::Format,
}

#[cfg_attr(test, automock)]
impl<RenderBackend: gfx_hal::Backend> Surface<RenderBackend> {
  /// Apply a swapchain to the current surface. This is required whenever a
  /// swapchain has been invalidated (I.E. by window resizing)
  pub fn apply_swapchain<'surface>(
    &mut self,
    gpu: &Gpu<RenderBackend>,
    swapchain: Swapchain,
    timeout_in_nanoseconds: u64,
  ) -> Result<(), &'surface str> {
    let device = gpu.internal_logical_device();
    self.extent = Some(swapchain.config.extent);

    unsafe {
      self
        .gfx_hal_surface
        .configure_swapchain(device, swapchain.config.clone())
        .expect("Failed to configure the swapchain");

      self.frame_buffer_attachment =
        Some(swapchain.config.framebuffer_attachment());

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

  pub fn needs_swapchain(&self) -> bool {
    return self.swapchain_is_valid;
  }

  /// Remove the swapchain configuration that this surface used on this given
  /// GPU.
  pub fn remove_swapchain(&mut self, gpu: &Gpu<RenderBackend>) {
    println!("Removing the swapchain configuration from: {}", self.name);
    unsafe {
      self
        .gfx_hal_surface
        .unconfigure_swapchain(gpu.internal_logical_device());
    }
  }

  /// Destroy the current surface and it's underlying resources.
  pub fn destroy(self, instance: &Instance<RenderBackend>) {
    println!("Destroying the surface: {}", self.name);

    instance.destroy_surface(self.gfx_hal_surface);
  }

  /// Get the size of the surface's extent. Will only return a size if a
  /// swapchain has been applied to the surface to render with.
  pub fn size(&self) -> Option<(u32, u32)> {
    return match self.extent {
      Some(extent) => Some((extent.width, extent.height)),
      None => None,
    };
  }
}

// ------------------------------ SWAPCHAIN BUILDER ----------------------------

pub struct SwapchainBuilder {
  size: (u32, u32),
}

impl SwapchainBuilder {
  pub fn new() -> Self {
    return Self { size: (480, 360) };
  }

  /// Set the size of the swapchain for the surface image.
  pub fn with_size(mut self, width: u32, height: u32) -> Self {
    self.size = (width, height);
    return self;
  }

  pub fn build<RenderBackend: Backend>(
    self,
    gpu: &Gpu<RenderBackend>,
    surface: &Surface<RenderBackend>,
  ) -> Swapchain {
    let physical_device = gpu.internal_physical_device();
    let caps = surface.gfx_hal_surface.capabilities(physical_device);
    let format = surface.get_first_supported_format(physical_device);
    let (width, height) = self.size;

    let mut swapchain_config = gfx_hal::window::SwapchainConfig::from_caps(
      &caps,
      format,
      gfx_hal::window::Extent2D { width, height },
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

#[cfg(test)]
mod tests {
  use super::*;
  use crate::gfx::MockInstanceBuilder;

  #[test]
  fn test_surface_builder() {
    let surface_builder = SurfaceBuilder::new();
    assert_eq!(surface_builder.name, None);

    let surface_builder = SurfaceBuilder::new().with_name("TestSurface");
    assert_eq!(surface_builder.name, Some("TestSurface".to_string()));
  }

  #[test]
  fn test_swapchain_builder() {
    let swapchain_builder = SwapchainBuilder::new();
    assert_eq!(swapchain_builder.size, (480, 360));

    let swapchain_builder = SwapchainBuilder::new().with_size(1920, 1080);
    assert_eq!(swapchain_builder.size, (1920, 1080));
  }

  #[test]
  fn test_surface_builder_e2e() {
    //let instance = MockInstanceBuilder::new().build("TestInstance");
    let surface_builder = SurfaceBuilder::new().with_name("TestSurface");
    //let surface = surface_builder.build(&instance);

    //assert_eq!(surface.name, "TestSurface".to_string());
    //assert_eq!(surface.swapchain_is_valid, false);
    //assert_eq!(surface.image, None);
    //assert_eq!(surface.frame_buffer_attachment, None);
  }
}

impl<RenderBackend: Backend> Surface<RenderBackend> {
  /// Checks the queue family if the current Surface can support the GPU.
  pub(super) fn can_support_queue_family(
    &self,
    queue_family: &RenderBackend::QueueFamily,
  ) -> bool {
    return self.gfx_hal_surface.supports_queue_family(queue_family);
  }

  pub(super) fn get_supported_formats(
    &self,
    physical_device: &RenderBackend::PhysicalDevice,
  ) -> Vec<gfx_hal::format::Format> {
    return self
      .gfx_hal_surface
      .supported_formats(physical_device)
      .unwrap_or(vec![]);
  }

  pub(super) fn get_first_supported_format(
    &self,
    physical_device: &RenderBackend::PhysicalDevice,
  ) -> gfx_hal::format::Format {
    return self
      .get_supported_formats(physical_device)
      .get(0)
      .unwrap_or(&gfx_hal::format::Format::Rgba8Srgb)
      .clone();
  }

  pub(super) fn internal_surface_image(
    &self,
  ) -> Option<&<RenderBackend::Surface as PresentationSurface<RenderBackend>>::SwapchainImage>{
    return self.image.as_ref();
  }

  pub(super) fn internal_frame_buffer_attachment(
    &self,
  ) -> Option<gfx_hal::image::FramebufferAttachment> {
    return self.frame_buffer_attachment.clone();
  }

  pub(super) fn internal_surface_and_image(
    &mut self,
  ) -> (
    &mut RenderBackend::Surface,
    <RenderBackend::Surface as PresentationSurface<RenderBackend>>::SwapchainImage,
  ){
    return (
      &mut self.gfx_hal_surface,
      self.image.take().expect("Surface image is not present"),
    );
  }
}
