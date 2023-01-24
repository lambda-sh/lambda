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

impl<RenderBackend: internal::Backend> Instance<RenderBackend> {
  /// Returns a list of all available adapters.
  pub(super) fn enumerate_adapters(
    &self,
  ) -> Vec<gfx_hal::adapter::Adapter<RenderBackend>> {
    return self.gfx_hal_instance.enumerate_adapters();
  }

  pub(super) fn first_adapter(
    &self,
  ) -> gfx_hal::adapter::Adapter<RenderBackend> {
    return self.gfx_hal_instance.enumerate_adapters().remove(0);
  }

  pub(super) fn create_surface(
    &self,
    window_handle: &crate::winit::WindowHandle,
  ) -> RenderBackend::Surface {
    return unsafe {
      self
        .gfx_hal_instance
        .create_surface(&window_handle.window_handle)
        .expect("Failed to create a surface using the current instance and window handle.")
    };
  }

  pub(super) fn destroy_surface(&self, surface: RenderBackend::Surface) {
    unsafe {
      self.gfx_hal_instance.destroy_surface(surface);
    }
  }
}

// ----------------------- INTERNAL INSTANCE OPERATIONS ------------------------

pub mod internal {

  pub use super::{
    pipeline::internal::*,
    shader::internal::*,
  };
}
