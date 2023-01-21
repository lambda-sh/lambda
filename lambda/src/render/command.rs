use std::ops::Range;

use lambda_platform::gfx::viewport::ViewPort as PlatformViewPort;

use super::{
  internal::surface_from_context,
  PlatformRenderCommand,
  RenderContext,
};

/// Commands that are used to render a frame within the RenderContext.
#[derive(Debug, Clone)]
pub enum RenderCommand {
  /// sets the viewports for the render context.
  SetViewports {
    start_at: u32,
    viewports: Vec<super::viewport::Viewport>,
  },
  /// sets the scissor rectangles for the render context.
  SetScissors {
    start_at: u32,
    viewports: Vec<super::viewport::Viewport>,
  },
  SetPipeline {
    pipeline: super::ResourceId,
  },
  /// Begins the render pass.
  BeginRenderPass {
    render_pass: super::ResourceId,
    viewport: super::viewport::Viewport,
  },
  /// Ends the render pass.
  EndRenderPass,
  PushConstants {
    pipeline: super::ResourceId,
    stage: super::pipeline::PipelineStage,
    offset: u32,
    bytes: Vec<u32>,
  },
  BindVertexBuffer {
    pipeline: super::ResourceId,
    buffer: u32,
  },
  /// Draws a graphical primitive.
  Draw {
    vertices: Range<u32>,
  },
}

impl RenderCommand {
  /// Converts the RenderCommand into a platform compatible render command.
  // TODO(vmarcella): implement this using Into<PlatformRenderCommand>
  pub fn into_platform_command(
    &self,
    render_context: &mut RenderContext,
  ) -> PlatformRenderCommand {
    return match self {
      RenderCommand::SetViewports {
        start_at,
        viewports,
      } => PlatformRenderCommand::SetViewports {
        start_at: *start_at,
        viewports: viewports
          .into_iter()
          .map(|viewport| viewport.clone_gfx_viewport())
          .collect::<Vec<PlatformViewPort>>(),
      },
      RenderCommand::SetScissors {
        start_at,
        viewports,
      } => PlatformRenderCommand::SetScissors {
        start_at: *start_at,
        viewports: viewports
          .into_iter()
          .map(|viewport| viewport.clone_gfx_viewport())
          .collect::<Vec<PlatformViewPort>>(),
      },
      RenderCommand::BeginRenderPass {
        render_pass,
        viewport,
      } => {
        let surface = surface_from_context(render_context);
        let frame_buffer = render_context.allocate_and_get_frame_buffer(
          render_context
            .get_render_pass(*render_pass)
            .into_gfx_render_pass()
            .as_ref(),
        );

        PlatformRenderCommand::BeginRenderPass {
          render_pass: render_context
            .get_render_pass(*render_pass)
            .into_gfx_render_pass(),
          surface: surface.clone(),
          frame_buffer: frame_buffer.clone(),
          viewport: viewport.clone_gfx_viewport(),
        }
      }
      RenderCommand::EndRenderPass => PlatformRenderCommand::EndRenderPass,
      RenderCommand::SetPipeline { pipeline } => {
        PlatformRenderCommand::AttachGraphicsPipeline {
          pipeline: render_context
            .render_pipelines
            .get(*pipeline)
            .unwrap()
            .into_platform_render_pipeline(),
        }
      }
      RenderCommand::PushConstants {
        pipeline,
        stage,
        offset,
        bytes,
      } => PlatformRenderCommand::PushConstants {
        pipeline: render_context
          .render_pipelines
          .get(*pipeline)
          .unwrap()
          .into_platform_render_pipeline(),
        stage: *stage,
        offset: *offset,
        bytes: bytes.clone(),
      },
      RenderCommand::BindVertexBuffer { pipeline, buffer } => {
        PlatformRenderCommand::BindVertexBuffer {
          buffer: render_context
            .render_pipelines
            .get(*pipeline)
            .unwrap()
            .buffers()
            .get(*buffer as usize)
            .unwrap()
            .internal_buffer_rc(),
        }
      }
      RenderCommand::Draw { vertices } => PlatformRenderCommand::Draw {
        vertices: vertices.clone(),
      },
    };
  }
}
