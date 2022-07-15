use std::ops::Range;

use lambda_platform::gfx::viewport::ViewPort as PlatformViewPort;

use super::{
  internal::surface_for_context,
  PlatformRenderCommand,
  RenderContext,
};
/// Commands that are used to render a frame within the RenderContext.
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
  /// Begins the render pass.
  BeginRenderPass {
    render_pass: super::render_pass::RenderPass,
    viewport: super::viewport::Viewport,
  },
  /// Ends the render pass.
  EndRenderPass,
  /// Attaches a graphical pipeline to the render context.
  AttachGraphicsPipeline {
    pipeline: super::pipeline::RenderPipeline,
  },
  /// Draws a graphical primitive.
  Draw { vertices: Range<u32> },
}

impl RenderCommand {
  /// Converts the RenderCommand into a platform compatible render command.
  // TODO(vmarcella): implement this using Into<PlatformRenderCommand>
  pub fn into_platform_command(
    self,
    render_context: &RenderContext,
  ) -> PlatformRenderCommand {
    return match self {
      RenderCommand::SetViewports {
        start_at,
        viewports,
      } => PlatformRenderCommand::SetViewports {
        start_at,
        viewports: viewports
          .into_iter()
          .map(|viewport| viewport.into_gfx_viewport())
          .collect::<Vec<PlatformViewPort>>(),
      },
      RenderCommand::SetScissors {
        start_at,
        viewports,
      } => PlatformRenderCommand::SetScissors {
        start_at,
        viewports: viewports
          .into_iter()
          .map(|viewport| viewport.into_gfx_viewport())
          .collect::<Vec<PlatformViewPort>>(),
      },
      RenderCommand::BeginRenderPass {
        render_pass,
        viewport,
      } => PlatformRenderCommand::BeginRenderPass {
        render_pass: render_pass.into_gfx_render_pass(),
        surface: todo!(),
        frame_buffer: todo!("FrameBuffer"),
        viewport: viewport.into_gfx_viewport(),
      },
      RenderCommand::EndRenderPass => todo!(),
      RenderCommand::AttachGraphicsPipeline { pipeline } => todo!(),
      RenderCommand::Draw { vertices } => todo!(),
    };
  }
}
