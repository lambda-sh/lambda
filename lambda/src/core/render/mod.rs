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
  image::{
    Access,
    Layout,
  },
  memory::Dependencies,
  pass::{
    Attachment,
    AttachmentLoadOp,
    AttachmentOps,
    AttachmentStoreOp,
    SubpassDependency,
    SubpassDesc,
  },
  pool::{
    CommandPool,
    CommandPoolCreateFlags,
  },
  prelude::{
    PhysicalDevice,
    QueueFamily,
  },
  pso::PipelineStage,
  window::Surface,
  Instance as HalInstance,
};

use super::{
  event_loop::LambdaEvent,
  window::LambdaWindow,
};
use crate::platform;

pub trait Renderer {
  fn resize(&mut self, width: u32, height: u32);
  fn on_update(&self);
  fn on_event(&self, event: LambdaEvent);
}

pub struct LambdaRenderer<B: gfx_hal::Backend> {
  instance: B::Instance,

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
    let instance = B::Instance::create(name, 1)
      .expect("gfx backend not supported by LambdaRenderer.");

    // Surfaces are only required if the renderer is constructed with a Window, otherwise
    // the renderer doesn't need to have a surface and can simply be used for GPU compute.
    let surface =
      platform::gfx::create_surface::<B>(&instance, window.unwrap());

    // a device adapter using the first adapter attached to the
    // current backend instance and then finds the supported queue family
    let primary_adapter = instance.enumerate_adapters().remove(0);
    let queue_family =
      platform::gfx::find_supported_render_queue(&surface, &primary_adapter);

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

    let render_pass =
      platform::gfx::create_render_pass(&mut gpu, None, None, None);

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
