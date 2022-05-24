use std::collections::HashMap;

use gfx_hal::pool::CommandPool as _;

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

  pub fn build<B: gfx_hal::Backend>(self, gpu: &Gpu<B>) -> CommandPool<B> {
    let command_pool = gpu.create_command_pool(self.command_pool_flags);

    return CommandPool {
      command_pool,
      command_buffers: HashMap::new(),
    };
  }
}

pub struct CommandPool<B: gfx_hal::Backend> {
  command_pool: B::CommandPool,
  command_buffers: HashMap<String, B::CommandBuffer>,
}

pub struct BufferID;

impl<B: gfx_hal::Backend> CommandPool<B> {
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
  pub fn deallocate_command_buffer(&mut self, name: &str) {
    let buffer = self.command_buffers.remove(&name.to_string()).unwrap();

    unsafe { self.command_pool.free(vec![buffer].into_iter()) }
  }
}
