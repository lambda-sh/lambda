// -------------------------- GFX PLATFORM EXPORTS -----------------------------

pub mod api;
pub mod assembler;
pub mod buffer;
pub mod command;
pub mod fence;
pub mod framebuffer;
pub mod gpu;
pub mod pipeline;
pub mod render_pass;
pub mod resource;
pub mod shader;
pub mod surface;
pub mod viewport;

use gfx_hal::Instance as _;

// ----------------------- INSTANCE BUILDER AND INSTANCE -------------------------------

pub struct InstanceBuilder {}

#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
impl InstanceBuilder {
  pub fn new() -> Self {
    return Self {};
  }

  /// Builds a graphical instance for the current platform.
  pub fn build<RenderBackend: internal::Backend>(
    self,
    name: &str,
  ) -> Instance<RenderBackend> {
    return Instance::new(name);
  }
}

pub struct Instance<RenderBackend: internal::Backend> {
  gfx_hal_instance: RenderBackend::Instance,
}

impl<RenderBackend: internal::Backend> Instance<RenderBackend> {
  /// Create a new GfxInstance connected to the current platforms primary backend.
  fn new(name: &str) -> Self {
    let instance = RenderBackend::Instance::create(name, 1)
      .expect("gfx backend not supported by the current platform");

    return Self {
      gfx_hal_instance: instance,
    };
  }
}

// ----------------------- INTERNAL INSTANCE OPERATIONS ------------------------

pub mod internal {
  use gfx_hal::{
    adapter::Adapter,
    Instance as _,
  };

  pub use super::{
    assembler::internal::*,
    gpu::internal::*,
    pipeline::internal::*,
    render_pass::internal::*,
    shader::internal::*,
    Instance,
  };

  /// Helper function to create a low level gfx_hal surface. Not meant to be
  /// used outside of lambda-platform.
  #[inline]
  pub fn create_surface<RenderBackend: gfx_hal::Backend>(
    instance: &Instance<RenderBackend>,
    window_handle: &crate::winit::WindowHandle,
  ) -> RenderBackend::Surface {
    unsafe {
      let surface = instance
        .gfx_hal_instance
        .create_surface(&window_handle.window_handle)
        .expect("Failed to create a surface using the current instance and window handle.");

      return surface;
    };
  }

  /// Destroy a low level gfx_hal surface using the instance abstraction.
  pub fn destroy_surface<RenderBackend: gfx_hal::Backend>(
    instance: &Instance<RenderBackend>,
    surface: RenderBackend::Surface,
  ) {
    unsafe {
      instance.gfx_hal_instance.destroy_surface(surface);
    }
  }

  /// Returns a graphical adapter from an instance.
  pub fn get_adapter<RenderBackend: gfx_hal::Backend>(
    instance: &mut Instance<RenderBackend>,
    adapter_num: usize,
  ) -> Adapter<RenderBackend> {
    return instance
      .gfx_hal_instance
      .enumerate_adapters()
      .remove(adapter_num);
  }
}
