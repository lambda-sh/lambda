use gfx_hal::Instance;

use super::winit::WindowHandle;

pub mod gpu;

pub struct GfxInstance<B: gfx_hal::Backend> {
  instance: B::Instance,
}

impl<B: gfx_hal::Backend> GfxInstance<B> {
  /// Create a new GfxInstance connected to the platforms primary backend.
  pub fn new(name: &str) -> Self {
    let instance = B::Instance::create(name, 1)
      .expect("gfx backend not supported by the current platform");

    return Self { instance };
  }

  /// Create a surface for a given lambda window using it's underlying
  /// winit window handle.
  /// TODO(vmarcella): Wrap up the B::Surface type to a custom type of our own.
  pub fn create_surface(&self, window_handle: &WindowHandle) -> B::Surface {
    return unsafe {
      self
        .instance
        .create_surface(&window_handle.window_handle)
        .unwrap()
    };
  }

  /// Destroy a surface created by this graphical instance.
  pub fn destroy_surface(&self, mut surface: B::Surface) {
    unsafe {
      self.instance.destroy_surface(surface);
    }
  }

  /// Open a connection to the primary GPU with an optional surface that the GPU
  /// can access for rendering.
  // TODO(vmarcella): This function should allow the RenderQueueType to be
  // optionally passed in with the default type being Graphical.
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
