use gfx_hal::{
  queue::QueueFamily,
  Instance,
};

use self::gpu::RenderQueueType;
use super::winit::WindowHandle;

pub mod api;
pub mod gpu;
pub mod pipeline;
pub mod surface;

use api::RenderingAPI;

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

pub struct GfxInstance<RenderBackend: gfx_hal::Backend> {
  instance: RenderBackend::Instance,
}

impl<RenderBackend: gfx_hal::Backend> GfxInstance<RenderBackend> {
  /// Create a new GfxInstance connected to the platforms primary backend.
  pub fn new(name: &str) -> Self {
    let instance = RenderBackend::Instance::create(name, 1)
      .expect("gfx backend not supported by the current platform");

    return Self { instance };
  }

  /// Create a surface for a given lambda window using it's underlying
  /// winit window handle.
  pub fn create_surface(
    &self,
    window_handle: &WindowHandle,
  ) -> surface::Surface<RenderBackend> {
    unsafe {
      let surface = self
        .instance
        .create_surface(&window_handle.window_handle)
        .unwrap();

      return surface::Surface::new(surface);
    };
  }

  pub fn destroy_surface(&self, surface: RenderBackend::Surface) {
    unsafe {
      self.instance.destroy_surface(surface);
    }
  }
}

/// GpuBuilder for constructing a GPU
pub struct GpuBuilder<RenderBackend: gfx_hal_exports::Backend> {
  adapter: gfx_hal::adapter::Adapter<RenderBackend>,
  render_queue_type: RenderQueueType,
}

impl<RenderBackend: gfx_hal_exports::Backend> GpuBuilder<RenderBackend> {
  pub fn new(instance: &mut GfxInstance<RenderBackend>) -> Self {
    let adapter = instance.instance.enumerate_adapters().remove(0);
    return Self {
      adapter,
      render_queue_type: RenderQueueType::Graphical,
    };
  }

  pub fn with_render_queue_type(mut self, queue_type: RenderQueueType) -> Self {
    self.render_queue_type = queue_type;
    return self;
  }

  /// Builds a GPU
  pub fn build(
    self,
    surface: Option<&surface::Surface<RenderBackend>>,
  ) -> Result<gpu::Gpu<RenderBackend>, String> {
    match (surface, self.render_queue_type) {
      (Some(surface), RenderQueueType::Graphical) => {
        let adapter = self.adapter;
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
pub fn create_default_gfx_instance() -> GfxInstance<RenderingAPI::Backend> {
  return GfxInstance::<RenderingAPI::Backend>::new("Lambda Application");
}
