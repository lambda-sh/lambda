use std::collections::HashMap;

use gfx_hal::{
  device::Device,
  pool::CommandPool as _,
};

use super::gpu::Gpu;

pub struct CommandPoolBuilder {
  command_pool_flags: gfx_hal::pool::CommandPoolCreateFlags,
}

impl CommandPoolBuilder {
  pub fn new() -> Self {
    return Self {
      command_pool_flags: gfx_hal::pool::CommandPoolCreateFlags::empty(),
    };
  }

  /// Attach command pool create flags to the command pool builder.
  pub fn with_flags(
    mut self,
    flags: gfx_hal::pool::CommandPoolCreateFlags,
  ) -> Self {
    self.command_pool_flags = flags;
    return self;
  }

  /// Builds a command pool.
  pub fn build<B: gfx_hal::Backend>(self, gpu: &Gpu<B>) -> CommandPool<B> {
    let command_pool = gpu.create_command_pool(self.command_pool_flags);

    return CommandPool {
      command_pool,
      command_buffers: HashMap::new(),
    };
  }
}

pub struct CommandPool<RenderBackend: gfx_hal::Backend> {
  command_pool: RenderBackend::CommandPool,
  command_buffers: HashMap<String, RenderBackend::CommandBuffer>,
}

pub struct BufferID;

impl<RenderBackend: gfx_hal::Backend> CommandPool<RenderBackend> {
  /// Allocate a command buffer for lambda.
  // TODO(vmarcella): This should expose the level that will be allocated.
  pub fn allocate_command_buffer(&mut self, name: &str) {
    let buffer = unsafe {
      self
        .command_pool
        .allocate_one(gfx_hal::command::Level::Primary)
    };

    self.command_buffers.insert(name.to_string(), buffer);
  }

  /// Deallocate a command buffer
  // TODO(vmarcella): This function should return a result based on the status
  // of the deallocation.
  pub fn deallocate_command_buffer(&mut self, name: &str) {
    let buffer = self.command_buffers.remove(&name.to_string()).unwrap();

    unsafe { self.command_pool.free(vec![buffer].into_iter()) }
  }

  /// Buffers can be looked up with the same name that they're given when
  /// calling `allocate_command_buffer`
  pub fn get_mutable_command_buffer(
    &mut self,
    name: &str,
  ) -> Option<&mut RenderBackend::CommandBuffer> {
    return self.command_buffers.get_mut(name);
  }

  pub fn get_command_buffer(
    &self,
    name: &str,
  ) -> Option<&RenderBackend::CommandBuffer> {
    return self.command_buffers.get(name);
  }

  /// Resets the command pool and all of the command buffers.
  pub fn reset_pool(&mut self, release_resources: bool) {
    unsafe {
      self.command_pool.reset(release_resources);
    }
  }

  pub fn destroy(self, gpu: &mut Gpu<RenderBackend>) {
    unsafe {
      gpu
        .get_logical_device()
        .destroy_command_pool(self.command_pool);
    }
  }
}
