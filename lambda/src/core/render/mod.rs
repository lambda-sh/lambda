use gfx_hal::{
  prelude::PhysicalDevice,
  Instance as HalInstance,
};

use super::{
  event_loop::LambdaEvent,
  window::LambdaWindow,
};
use crate::platform::gfx;

pub trait Renderer {
  fn resize(&mut self, width: u32, height: u32);
  fn on_update(&self);
  fn on_event(&self, event: LambdaEvent);
}

pub struct LambdaRenderer<B: gfx_hal::Backend> {
  instance: gfx::GfxInstance<B>,

  // Both surface and Window are optional
  surface: Option<B::Surface>,
}

impl<B: gfx_hal::Backend> Default for LambdaRenderer<B> {
  /// The default constructor returns a named renderer that has no
  /// window attached.
  fn default() -> Self {
    return Self::new("LambdaRenderer", None);
  }
}

impl<B: gfx_hal::Backend> LambdaRenderer<B> {
  pub fn new(name: &str, window: Option<&LambdaWindow>) -> Self {
    let instance = gfx::GfxInstance::<B>::new(name);

    // Surfaces are only required if the renderer is constructed with a Window, otherwise
    // the renderer doesn't need to have a surface and can simply be used for GPU compute.
    let surface = instance.create_surface(window.unwrap());
    let mut gpu = instance
      .open_primary_gpu(Some(&surface))
      .with_command_pool();

    let command_buffer = gpu.allocate_command_buffer();
    let render_pass = gpu.create_render_pass(None, None, None);

    return Self {
      instance,
      surface: None,
    };
  }
}

impl<B: gfx_hal::Backend> Renderer for LambdaRenderer<B> {
  fn on_update(&self) {}
  fn on_event(&self, event: LambdaEvent) {}

  fn resize(&mut self, width: u32, height: u32) {
    todo!("Need to implement resizing for the LambdaRenderer!")
  }
}
