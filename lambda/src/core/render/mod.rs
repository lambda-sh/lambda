use std::convert::TryInto;

use gfx_hal::{
  adapter::{
    self,
    Adapter,
  },
  command::Level,
  device::Device as HalDevice,
  format::{
    ChannelType,
    Format,
  },
  pool::{
    CommandPool,
    CommandPoolCreateFlags,
  },
  prelude::{
    PhysicalDevice,
    QueueFamily,
  },
  window::{
    Extent2D as HalExtent2D,
    InitError as HalWindowInitError,
    Surface,
  },
  Instance as HalInstance,
};

use super::{
  event_loop::LambdaEvent,
  window::{
    LambdaWindow,
    Window,
  },
};

pub trait Renderer {
  fn resize(&mut self, width: u32, height: u32);
  fn on_update(&self);
  fn on_event(&self, event: LambdaEvent);
}

pub struct LambdaRenderer<Backend: gfx_hal::Backend> {
  instance: Backend::Instance,

  // Both surface and Window are optional
  surface: Option<Backend::Surface>,
}

impl<B: gfx_hal::Backend> Default for LambdaRenderer<B> {
  /// The default constructor returns a named renderer that has no
  /// window attached.
  fn default() -> Self {
    return Self::new("LambdaRenderer", None);
  }
}

impl<B: gfx_hal::Backend> LambdaRenderer<B> {
  /// Create a Surface for a backend using a WinitP.
  fn create_surface(
    instance: &B::Instance,
    window: &LambdaWindow,
  ) -> B::Surface {
    return unsafe {
      instance
        .create_surface(window.winit_window_ref().unwrap())
        .unwrap()
    };
  }

  fn find_supported_render_queue<'a>(
    surface: &'a B::Surface,
    adapter: &'a Adapter<B>,
  ) -> &'a B::QueueFamily {
    return adapter
      .queue_families
      .iter()
      .find(|family| {
        let supports_queue = surface.supports_queue_family(family);
        let supports_graphics = family.queue_type().supports_graphics();

        supports_queue && supports_graphics
      })
      .unwrap();
  }

  pub fn new(name: &str, window: Option<&LambdaWindow>) -> Self {
    let instance = B::Instance::create(name, 1)
      .expect("gfx backend not supported by LambdaRenderer.");

    // Surfaces are only required if the renderer is constructed with a Window, otherwise
    // the renderer doesn't need to have a surface and can simply be used for GPU compute.
    let surface: B::Surface = Self::create_surface(&instance, window.unwrap());

    // a device adapter using the first adapter attached to the
    // current backend instance and then finds the supported queue family
    let primary_adapter = instance.enumerate_adapters().remove(0);
    let queue_family =
      Self::find_supported_render_queue(&surface, &primary_adapter);

    // Open up the GPU through our primary adapter.
    let mut gpu = unsafe {
      primary_adapter
        .physical_device
        .open(&[(queue_family, &[1.0])], gfx_hal::Features::empty())
        .expect("Failed to open GPU.")
    };

    let queue_group = gpu.queue_groups.pop().unwrap();

    // create a command pool on the primary graphics card device and allocate one command buffer for
    // sending instructions to the GPU.
    let (command_pool, mut command_buffer) = unsafe {
      let mut command_pool = gpu
        .device
        .create_command_pool(
          queue_group.family,
          CommandPoolCreateFlags::empty(),
        )
        .unwrap();

      let command_buffer = command_pool.allocate_one(Level::Primary);

      (command_pool, command_buffer)
    };

    let surface_color_format = {
      let supported_formats = surface
        .supported_formats(&primary_adapter.physical_device)
        .unwrap_or(vec![]);

      let default_format =
        *supported_formats.get(0).unwrap_or(&Format::Rgba8Srgb);

      supported_formats
        .into_iter()
        .find(|format| -> bool { format.base_format().1 == ChannelType::Srgb })
        .unwrap_or(default_format)
    };

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
