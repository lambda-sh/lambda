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

    return CommandPool { command_pool };
  }
}

pub struct CommandPool<B: gfx_hal::Backend> {
  command_pool: B::CommandPool,
}
