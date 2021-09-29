use gfx_hal::{
  adapter::Adapter,
  format::{
    ChannelType,
    Format,
  },
  window::Surface,
  Instance,
};

use crate::core::window::LambdaWindow;

pub mod gpu;

pub struct GfxInstance<B: gfx_hal::Backend> {
  instance: B::Instance,
}

impl<B: gfx_hal::Backend> GfxInstance<B> {
  pub fn new(name: &str) -> Self {
    let instance = B::Instance::create(name, 1)
      .expect("gfx backend not supported by the current platform");

    return Self { instance };
  }

  /// Create a surface for a given lambda window using it's underlying
  /// winit window handle.
  /// TODO(vmarcella): Wrap up the B::Surface type to a custom type of our own.
  pub fn create_surface(&self, window: &LambdaWindow) -> B::Surface {
    return unsafe {
      self
        .instance
        .create_surface(window.winit_window_ref().unwrap())
        .unwrap()
    };
  }

  // Open a connection to the primary GPU with an optional surface attached.
  //
  pub fn open_primary_gpu(
    &self,
    surface: Option<&B::Surface>,
  ) -> gpu::GfxGpu<B> {
    let adapter = self.instance.enumerate_adapters().remove(0);
    return gpu::GfxGpu::<B>::new(
      adapter,
      gpu::RenderQueueType::Graphical,
      surface,
    );
  }
}

// Create a graphical backend instance using the platforms default installed
// graphical backend
pub fn create_default_gfx_instance() -> GfxInstance<backend::Backend> {
  return GfxInstance::<backend::Backend>::new("Lambda Application");
}

/// Finds the first supported color format or default to Rgba8Srgb.
pub fn find_supported_color_format<B: gfx_hal::Backend>(
  surface: &B::Surface,
  adapter: &Adapter<B>,
) -> Format {
  // Define a surface color format compatible with the graphics
  // device & surface
  let supported_formats = surface
    .supported_formats(&adapter.physical_device)
    .unwrap_or(vec![]);

  let default_format = *supported_formats.get(0).unwrap_or(&Format::Rgba8Srgb);

  return supported_formats
    .into_iter()
    .find(|format| -> bool { format.base_format().1 == ChannelType::Srgb })
    .unwrap_or(default_format);
}
