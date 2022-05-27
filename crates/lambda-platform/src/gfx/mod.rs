use gfx_hal::queue::QueueFamily;

use self::gpu::RenderQueueType;
use super::winit::WindowHandle;

pub mod api;
pub mod command;
pub mod fence;
pub mod gpu;
pub mod pipeline;
pub mod render_pass;
pub mod resource;
pub mod surface;

use api::RenderingAPI;
use gfx_hal::Instance as _;

/// Exports directly from the gfx_hal crate to be used while lambda-platform
/// stabilizes it's API.
pub mod gfx_hal_exports {
  pub use gfx_hal::{
    command::{
      ClearColor,
      ClearValue,
      CommandBuffer,
      CommandBufferFlags,
      RenderAttachmentInfo,
      SubpassContents,
    },
    format::Format,
    image::FramebufferAttachment,
    pso::{
      EntryPoint,
      InputAssemblerDesc,
      Primitive,
      PrimitiveAssemblerDesc,
      Rect,
      Specialization,
      Viewport,
    },
    window::{
      Extent2D,
      PresentationSurface,
    },
    Backend,
  };
}

pub struct Instance<RenderBackend: gfx_hal::Backend> {
  gfx_hal_instance: RenderBackend::Instance,
}

impl<RenderBackend: gfx_hal::Backend> Instance<RenderBackend> {
  /// Create a new GfxInstance connected to the platforms primary backend.
  pub fn new(name: &str) -> Self {
    let instance = RenderBackend::Instance::create(name, 1)
      .expect("gfx backend not supported by the current platform");

    return Self {
      gfx_hal_instance: instance,
    };
  }

  /// Create a surface for a given lambda window using it's underlying
  /// winit window handle.
  pub fn create_surface(
    &self,
    window_handle: &WindowHandle,
  ) -> surface::Surface<RenderBackend> {
    unsafe {
      let surface = self
        .gfx_hal_instance
        .create_surface(&window_handle.window_handle)
        .unwrap();

      return surface::Surface::new(surface);
    };
  }

  pub fn destroy_surface(&self, surface: RenderBackend::Surface) {
    unsafe {
      self.gfx_hal_instance.destroy_surface(surface);
    }
  }
}

/// GpuBuilder for constructing a GPU
pub struct GpuBuilder {
  render_queue_type: RenderQueueType,
}

impl GpuBuilder {
  pub fn new() -> Self {
    return Self {
      render_queue_type: RenderQueueType::Graphical,
    };
  }

  pub fn with_render_queue_type(mut self, queue_type: RenderQueueType) -> Self {
    self.render_queue_type = queue_type;
    return self;
  }

  /// Builds a GPU
  pub fn build<RenderBackend: gfx_hal::Backend>(
    self,
    instance: &mut Instance<RenderBackend>,
    surface: Option<&surface::Surface<RenderBackend>>,
  ) -> Result<gpu::Gpu<RenderBackend>, String> {
    match (surface, self.render_queue_type) {
      (Some(surface), RenderQueueType::Graphical) => {
        let adapter = instance.gfx_hal_instance.enumerate_adapters().remove(0);

        let queue_family = adapter
          .queue_families
          .iter()
          .find(|family| {
            return surface.can_support_queue_family(family)
              && family.queue_type().supports_graphics();
          })
          .expect("No compatible queue family found.")
          .id();

        let formats =
          surface.get_first_supported_format(&adapter.physical_device);

        return Ok(gpu::Gpu::new(adapter, queue_family));
      }
      (_, _) => return Err("Failed to build GPU.".to_string()),
    }
  }
}

// Create a graphical backend instance using the platforms default installed
// graphical backend
pub fn create_default_gfx_instance() -> Instance<RenderingAPI::Backend> {
  return Instance::<RenderingAPI::Backend>::new("Lambda Application");
}
