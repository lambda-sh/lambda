use lambda_platform::gfx::viewport::ViewPort as PlatformViewPort;

use super::PlatformRenderCommand;
pub enum RenderCommand {
  SetViewports {
    start_at: u32,
    viewports: Vec<super::viewport::Viewport>,
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
    };
  }
}
