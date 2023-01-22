use std::{
  borrow::Borrow,
  collections::HashMap,
  ops::Range,
  rc::Rc,
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
  /// lifetimes (I.E. command buffers that continuously need to render data to
  /// the screen)
  ShortLivedBuffers,
  /// Allows for buffers to be reset individually & manually by the owner of the
  /// command pool.
  ResetBuffersIndividually,
  /// Enable no features on the CommandPool.
  None,
  /// Enable all features for a given CommandPool.
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

/// This enum is used for specifying the type of command buffer to allocate on
/// the command pool.
pub enum CommandBufferLevel {
  /// Use for allocating a top level primary command buffer on the
  /// command pool. A command buffer at this level can then be used to create
  /// other primaries.
  Primary,
  /// Used
  Secondary,
}

/// Enumeration for issuing commands to a CommandBuffer allocated on the GPU.
/// The enumerations are evaluated upon being issued to an active command buffer
/// and correspond to lower level function calls.
pub enum Command<RenderBackend: gfx_hal::Backend> {
  /// Begins recording commands to the GPU. A primary command buffer can only
  /// issue this command once.
  BeginRecording,
  SetViewports {
    start_at: u32,
    viewports: Vec<ViewPort>,
  },
  SetScissors {
    start_at: u32,
    viewports: Vec<ViewPort>,
  },
  BeginRenderPass {
    render_pass: Rc<super::render_pass::RenderPass<RenderBackend>>,
    surface: Rc<super::surface::Surface<RenderBackend>>,
    frame_buffer: Rc<super::framebuffer::Framebuffer<RenderBackend>>,
    viewport: ViewPort,
  },
  /// Ends a currently active render pass.
  EndRenderPass,
  AttachGraphicsPipeline {
    pipeline: Rc<RenderPipeline<RenderBackend>>,
  },
  Draw {
    vertices: Range<u32>,
  },
  PushConstants {
    pipeline: Rc<RenderPipeline<RenderBackend>>,
    stage: super::pipeline::PipelineStage,
    offset: u32,
    bytes: Vec<u32>,
  },
  BindVertexBuffer {
    buffer: Rc<super::buffer::Buffer<RenderBackend>>,
  },
  EndRecording,
}

/// Representation of a command buffer allocated on the GPU. The lifetime of
/// the command is constrained to the lifetime of the command pool that built
/// it to ensure that it cannot be used while
pub struct CommandBuffer<'command_pool, RenderBackend: gfx_hal::Backend> {
  command_buffer: &'command_pool mut RenderBackend::CommandBuffer,
  flags: gfx_hal::command::CommandBufferFlags,
}

impl<'command_pool, RenderBackend: gfx_hal::Backend>
  CommandBuffer<'command_pool, RenderBackend>
{
  /// Validates and issues a command directly to the buffer on the GPU.
  /// If using a newly created Primary CommandBuffer the first and last commands
  /// that should be issued are:
  /// Command<RenderBackend>::BeginRecording
  /// Command<RenderBackend>::EndRecording
  /// Once the command buffer has stopped recording, it can be submitted to the
  /// GPU to start performing work.
  pub fn issue_command(&mut self, command: Command<RenderBackend>) {
    use gfx_hal::command::CommandBuffer as _;
    unsafe {
      match command {
        Command::BeginRecording => {
          self.command_buffer.begin_primary(self.flags)
        }
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
            super::pipeline::internal::pipeline_for(pipeline.as_ref()),
          )
        }
        Command::EndRenderPass => self.command_buffer.end_render_pass(),
        Command::PushConstants {
          pipeline,
          stage,
          offset,
          bytes,
        } => self.command_buffer.push_graphics_constants(
          super::pipeline::internal::pipeline_layout_for(pipeline.as_ref()),
          stage,
          offset,
          bytes.as_slice(),
        ),
        Command::Draw { vertices } => {
          self.command_buffer.draw(vertices.clone(), 0..1)
        }
        Command::BindVertexBuffer { buffer } => {
          self.command_buffer.bind_vertex_buffers(
            0,
            vec![(buffer.internal_buffer(), gfx_hal::buffer::SubRange::WHOLE)]
              .into_iter(),
          )
        }
        Command::EndRecording => self.command_buffer.finish(),
      }
    }
  }

  /// Functions exactly like issue_command except over multiple commands at
  /// once. Command execution is based on the order of commands inside the
  /// vector.
  pub fn issue_commands(&mut self, commands: Vec<Command<RenderBackend>>) {
    for command in commands {
      self.issue_command(command);
    }
  }
}

/// Builder for creating a Command buffer that can issue commands directly to
/// the GPU.
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
    let command_buffer = command_pool.allocate_command_buffer(name, self.level);

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
      gpu
        .internal_logical_device()
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

impl<RenderBackend: gfx_hal::Backend> CommandPool<RenderBackend> {
  /// Allocate a command buffer for lambda.
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

  /// Retrieves a command buffer that has been allocated by this command pool.
  /// This function is most likely not
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

  /// Moves the command pool into itself and destroys any command pool and
  /// buffer resources allocated on the GPU.
  #[inline]
  pub fn destroy(mut self, gpu: &super::gpu::Gpu<RenderBackend>) {
    unsafe {
      self.command_pool.reset(true);
      gpu
        .internal_logical_device()
        .destroy_command_pool(self.command_pool);
    }
  }
}
