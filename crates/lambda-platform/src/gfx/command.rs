use std::{
  collections::HashMap,
  ops::Range,
};

use gfx_hal::{
  device::Device,
  pool::CommandPool as _,
};

use super::{
  framebuffer::Framebuffer,
  gpu::Gpu,
  render_pass::RenderPass,
  viewport::ViewPort,
};

/// Command Pool Flag used to define optimizations/properties of the command
/// pool prior to being built.
pub enum CommandPoolFeatures {
  ShortLivedBuffers,
  ResetBuffersIndividually,
  None,
  All,
}

/// Features that can be used to optimize command buffers for specific
/// usages/scenarios
pub enum CommandBufferFeatures {
  /// This feature specifies that the
  ResetEverySubmission,
  TiedToRenderPass,
  SimultaneousRecording,
  None,
  All,
}

pub enum CommandBufferLevel {
  Primary,
  Secondary,
}

pub enum Command<'render_context, RenderBackend: gfx_hal::Backend> {
  Begin,
  SetViewports {
    first: u32,
    viewports: Vec<ViewPort>,
  },
  SetScissors {
    first: u32,
    viewports: Vec<ViewPort>,
  },
  BeginRenderPass {
    render_pass: RenderPass<RenderBackend>,
    frame_buffer: &'render_context Framebuffer,
    viewport: ViewPort,
  },
  EndRenderPass,
  Draw {
    vertices: Range<u32>,
  },
}

pub struct CommandBuffer<'command_pool, RenderBackend: gfx_hal::Backend> {
  command_buffer: &'command_pool RenderBackend::CommandBuffer,
}

impl<'command_pool, RenderBackend: gfx_hal::Backend>
  CommandBuffer<'command_pool, RenderBackend>
{
  pub fn _recording() {}
}

pub struct CommandBufferBuilder {
  flags: gfx_hal::command::CommandBufferFlags,
  level: CommandBufferLevel,
}

impl CommandBufferBuilder {
  pub fn new(level: CommandBufferLevel) -> Self {
    let flags = gfx_hal::command::CommandBufferFlags::empty();
    return CommandBufferBuilder { flags, level };
  }

  /// Build the command buffer and tie it to the lifetime of the command pool
  /// that gets created.
  pub fn build<'command_pool, RenderBackend: gfx_hal::Backend>(
    self,
    command_pool: &'command_pool mut CommandPool<RenderBackend>,
    name: &str,
  ) -> CommandBuffer<'command_pool, RenderBackend> {
    let command_buffer = command_pool.allocate_command_buffer(name, self.level);
    return CommandBuffer { command_buffer };
  }
}

pub struct CommandPoolBuilder {
  command_pool_flags: gfx_hal::pool::CommandPoolCreateFlags,
}

pub mod internal {}

impl CommandPoolBuilder {
  pub fn new() -> Self {
    return Self {
      command_pool_flags: gfx_hal::pool::CommandPoolCreateFlags::empty(),
    };
  }

  /// Attach command pool create flags to the command pool builder.
  pub fn with_features(mut self, flag: CommandPoolFeatures) -> Self {
    let flags = match flag {
      CommandPoolFeatures::ShortLivedBuffers => {
        gfx_hal::pool::CommandPoolCreateFlags::TRANSIENT
      }
      CommandPoolFeatures::ResetBuffersIndividually => {
        gfx_hal::pool::CommandPoolCreateFlags::RESET_INDIVIDUAL
      }
      CommandPoolFeatures::None => {
        gfx_hal::pool::CommandPoolCreateFlags::empty()
      }
      CommandPoolFeatures::All => gfx_hal::pool::CommandPoolCreateFlags::all(),
    };

    self.command_pool_flags.insert(flags);
    return self;
  }

  /// Builds a command pool.
  pub fn build<B: gfx_hal::Backend>(self, gpu: &Gpu<B>) -> CommandPool<B> {
    let command_pool = unsafe {
      super::internal::logical_device_for(gpu)
        .create_command_pool(
          super::internal::queue_family_for(gpu),
          self.command_pool_flags,
        )
        .expect("")
    };

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
  fn allocate_command_buffer(
    &mut self,
    name: &str,
    level: CommandBufferLevel,
  ) -> &RenderBackend::CommandBuffer {
    let buffer = unsafe {
      self
        .command_pool
        .allocate_one(gfx_hal::command::Level::Primary)
    };

    self.command_buffers.insert(name.to_string(), buffer);
    return self.command_buffers.get_mut(name).unwrap();
  }

  /// Deallocate a command buffer
  // TODO(vmarcella): This function should return a result based on the status
  // of the deallocation.
  pub fn deallocate_command_buffer(&mut self, name: &str) {
    let buffer = self
      .command_buffers
      .remove(&name.to_string())
      .expect(format!("Command Buffer {} doesn't exist", name).as_str());

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

  #[inline]
  pub fn get_command_buffer(
    &self,
    name: &str,
  ) -> Option<&RenderBackend::CommandBuffer> {
    return self.command_buffers.get(name);
  }

  /// Resets the command pool and all of the command buffers.
  #[inline]
  pub fn reset_pool(&mut self, release_resources: bool) {
    unsafe {
      self.command_pool.reset(release_resources);
    }
  }

  #[inline]
  pub fn destroy(self, gpu: &Gpu<RenderBackend>) {
    unsafe {
      super::gpu::internal::logical_device_for(gpu)
        .destroy_command_pool(self.command_pool);
    }
  }
}
