use std::{
  borrow::{
    Borrow,
    Cow,
  },
  collections::HashMap,
  ops::{
    Deref,
    Range,
  },
};

use gfx_hal::{
  command::ClearValue,
  device::Device,
  pool::CommandPool as _,
};

use super::{
  pipeline::RenderPipeline,
  viewport::ViewPort,
};

/// Command Pool Flag used to define optimizations/properties of the command
/// pool prior to being built.
pub enum CommandPoolFeatures {
  /// Optimizes the command pool for buffers that are expected to have short
  /// lifetimes (I.E. for constant rendering)
  ShortLivedBuffers,
  /// Rest
  ResetBuffersIndividually,
  None,
  All,
}

/// Features that can be used to optimize command buffers for specific
/// usages/scenarios
pub enum CommandBufferFeatures {
  /// Enable this feature when you would like the command buffer to reset it's
  /// contents every time its submitted for Rendering.
  ResetEverySubmission,
  /// Enable this feature if the command buffer lives within the lifetime of a
  /// render pass.
  TiedToRenderPass,
  /// Enable this feature if the command buffer allows for silumtaneous
  /// recording
  SimultaneousRecording,
  /// Enables no features.
  None,
  /// Enables all features.
  All,
}

pub enum CommandBufferLevel {
  Primary,
  Secondary,
}

pub struct CommandSetBuilder {
  viewports: Vec<ViewPort>,
  scissors: Vec<ViewPort>,
  recording: bool,
}

impl CommandSetBuilder {
  pub fn new() -> Self {
    return Self {
      viewports: vec![],
      scissors: vec![],
      recording: false,
    };
  }
}

pub enum Command<RenderBackend: gfx_hal::Backend> {
  Begin,
  SetViewports {
    start_at: u32,
    viewports: Vec<ViewPort>,
  },
  SetScissors {
    start_at: u32,
    viewports: Vec<ViewPort>,
  },
  BeginRenderPass {
    render_pass: super::render_pass::RenderPass<RenderBackend>,
    surface: Box<super::surface::Surface<RenderBackend>>,
    frame_buffer: Box<super::framebuffer::Framebuffer<RenderBackend>>,
    viewport: ViewPort,
  },
  EndRenderPass,
  AttachGraphicsPipeline {
    pipeline: RenderPipeline<RenderBackend>,
  },
  Draw {
    vertices: Range<u32>,
  },
  End,
}

pub struct CommandBuffer<'command_pool, RenderBackend: gfx_hal::Backend> {
  command_buffer: &'command_pool mut RenderBackend::CommandBuffer,
  flags: gfx_hal::command::CommandBufferFlags,
}

impl<'command_pool, RenderBackend: gfx_hal::Backend>
  CommandBuffer<'command_pool, RenderBackend>
{
  pub fn issue_command(&mut self, command: Command<RenderBackend>) {
    use gfx_hal::command::CommandBuffer as _;
    unsafe {
      match command {
        Command::Begin => self.command_buffer.begin_primary(self.flags),
        Command::SetViewports {
          start_at,
          viewports,
        } => self.command_buffer.set_viewports(
          start_at,
          viewports
            .into_iter()
            .map(|viewport| super::viewport::internal::viewport_for(&viewport)),
        ),
        Command::SetScissors {
          start_at,
          viewports,
        } => self.command_buffer.set_scissors(
          start_at,
          viewports.into_iter().map(|viewport| {
            super::viewport::internal::viewport_for(&viewport).rect
          }),
        ),
        Command::BeginRenderPass {
          render_pass,
          frame_buffer,
          surface,
          viewport,
        } => self.command_buffer.begin_render_pass(
          super::render_pass::internal::render_pass_for(&render_pass),
          super::framebuffer::internal::frame_buffer_for(&frame_buffer),
          super::viewport::internal::viewport_for(&viewport).rect,
          vec![gfx_hal::command::RenderAttachmentInfo::<RenderBackend> {
            image_view: super::surface::internal::borrow_surface_image_for(
              &surface,
            )
            .unwrap()
            .borrow(),
            clear_value: ClearValue {
              color: gfx_hal::command::ClearColor {
                float32: [0.0, 0.0, 0.0, 1.0],
              },
            },
          }]
          .into_iter(),
          gfx_hal::command::SubpassContents::Inline,
        ),
        Command::AttachGraphicsPipeline { pipeline } => {
          self.command_buffer.bind_graphics_pipeline(
            super::pipeline::internal::pipeline_for(&pipeline),
          )
        }
        Command::EndRenderPass => self.command_buffer.end_render_pass(),
        Command::Draw { vertices } => self.command_buffer.draw(vertices, 0..1),
        Command::End => self.command_buffer.finish(),
      }
    }
  }

  pub fn issue_commands<'render_context>(
    &mut self,
    commands: Vec<Command<RenderBackend>>,
  ) {
    for command in commands {
      self.issue_command(command);
    }
  }
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

  pub fn with_feature(mut self, feature: CommandBufferFeatures) -> Self {
    let flags = match feature {
      CommandBufferFeatures::ResetEverySubmission => {
        gfx_hal::command::CommandBufferFlags::ONE_TIME_SUBMIT
      }
      CommandBufferFeatures::TiedToRenderPass => {
        gfx_hal::command::CommandBufferFlags::RENDER_PASS_CONTINUE
      }
      CommandBufferFeatures::SimultaneousRecording => {
        gfx_hal::command::CommandBufferFlags::SIMULTANEOUS_USE
      }
      CommandBufferFeatures::None => {
        gfx_hal::command::CommandBufferFlags::empty()
      }
      CommandBufferFeatures::All => gfx_hal::command::CommandBufferFlags::all(),
    };

    self.flags.insert(flags);
    return self;
  }

  /// Build the command buffer and tie it to the lifetime of the command pool
  /// that gets created.
  pub fn build<'command_pool, RenderBackend: gfx_hal::Backend>(
    self,
    command_pool: &'command_pool mut CommandPool<RenderBackend>,
    name: &str,
  ) -> CommandBuffer<'command_pool, RenderBackend> {
    let mut command_buffer =
      command_pool.allocate_command_buffer(name, self.level);

    let flags = self.flags;

    return CommandBuffer {
      command_buffer,
      flags,
    };
  }
}

pub struct CommandPoolBuilder {
  command_pool_flags: gfx_hal::pool::CommandPoolCreateFlags,
}

pub mod internal {
  pub fn command_buffer_for<
    'render_context,
    RenderBackend: gfx_hal::Backend,
  >(
    command_buffer: &'render_context super::CommandBuffer<
      'render_context,
      RenderBackend,
    >,
  ) -> &'render_context RenderBackend::CommandBuffer {
    return command_buffer.command_buffer;
  }
}

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
  pub fn build<B: gfx_hal::Backend>(
    self,
    gpu: &super::gpu::Gpu<B>,
  ) -> CommandPool<B> {
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
  ) -> &mut RenderBackend::CommandBuffer {
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
  pub fn destroy(self, gpu: &super::gpu::Gpu<RenderBackend>) {
    unsafe {
      super::gpu::internal::logical_device_for(gpu)
        .destroy_command_pool(self.command_pool);
    }
  }
}
