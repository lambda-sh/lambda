use std::ops::Range;

use lambda_platform::gfx::viewport::ViewPort as PlatformViewPort;

use super::PlatformRenderCommand;
/// Commands that are used to render a frame within the RenderContext.
pub enum RenderCommand {
  /// sets the viewports for the render context.
  SetViewports {
    start_at: u32,
    viewports: Vec<super::viewport::Viewport>,
  },
  SetScissors {
    start_at: u32,
    viewports: Vec<super::viewport::Viewport>,
  },
  BeginRenderPass {
    render_pass: super::render_pass::RenderPass,
    start_at: u32,
    viewports: Vec<super::viewport::Viewport>,
  },
  EndRenderPass,
  AttachGraphicsPipeline {
    pipeline: super::pipeline::RenderPipeline,
  },
  Draw {
    vertices: Range<u32>,
  },
}

impl RenderCommand {
  pub fn into_platform_command(self) -> PlatformRenderCommand {
    return match self {
      Self::SetViewports {
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
        start_at,
        viewports,
      } => todo!(),
      RenderCommand::EndRenderPass => todo!(),
      RenderCommand::AttachGraphicsPipeline { pipeline } => todo!(),
      RenderCommand::Draw { vertices } => todo!(),
    };
  }
}
