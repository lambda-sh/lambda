use lambda_platform::wgpu::surface as platform_surface;

use super::texture::{
  TextureFormat,
  TextureUsages,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PresentMode {
  /// Vsync enabled; frames wait for vertical blanking interval.
  Fifo,
  /// Vsync with relaxed timing; may tear if frames miss the interval.
  FifoRelaxed,
  /// No Vsync; immediate presentation (may tear).
  Immediate,
  /// Triple-buffered presentation when supported.
  Mailbox,
  /// Automatic Vsync selection by the platform.
  AutoVsync,
  /// Automatic non-Vsync selection by the platform.
  AutoNoVsync,
}

impl PresentMode {
  #[inline]
  pub(crate) fn to_platform(&self) -> platform_surface::PresentMode {
    return match self {
      PresentMode::Fifo => platform_surface::PresentMode::Fifo,
      PresentMode::FifoRelaxed => platform_surface::PresentMode::FifoRelaxed,
      PresentMode::Immediate => platform_surface::PresentMode::Immediate,
      PresentMode::Mailbox => platform_surface::PresentMode::Mailbox,
      PresentMode::AutoVsync => platform_surface::PresentMode::AutoVsync,
      PresentMode::AutoNoVsync => platform_surface::PresentMode::AutoNoVsync,
    };
  }

  #[inline]
  pub(crate) fn from_platform(
    mode: platform_surface::PresentMode,
  ) -> PresentMode {
    return match mode {
      platform_surface::PresentMode::Fifo => PresentMode::Fifo,
      platform_surface::PresentMode::FifoRelaxed => PresentMode::FifoRelaxed,
      platform_surface::PresentMode::Immediate => PresentMode::Immediate,
      platform_surface::PresentMode::Mailbox => PresentMode::Mailbox,
      platform_surface::PresentMode::AutoVsync => PresentMode::AutoVsync,
      platform_surface::PresentMode::AutoNoVsync => PresentMode::AutoNoVsync,
    };
  }
}

impl Default for PresentMode {
  fn default() -> Self {
    return PresentMode::Fifo;
  }
}

/// High-level surface configuration.
///
/// Contains the current surface dimensions, format, present mode, and usage
/// flags without exposing platform types.
#[derive(Clone, Debug)]
pub struct SurfaceConfig {
  /// Width in pixels.
  pub width: u32,
  /// Height in pixels.
  pub height: u32,
  /// The texture format used by the surface.
  pub format: TextureFormat,
  /// The presentation mode (vsync behavior).
  pub present_mode: PresentMode,
  /// Texture usage flags for the surface.
  pub usage: TextureUsages,
}

impl SurfaceConfig {
  pub(crate) fn from_platform(
    config: &platform_surface::SurfaceConfig,
  ) -> Self {
    return SurfaceConfig {
      width: config.width,
      height: config.height,
      format: TextureFormat::from_platform(config.format)
        .unwrap_or(TextureFormat::Bgra8UnormSrgb),
      present_mode: PresentMode::from_platform(config.present_mode),
      usage: TextureUsages::from_platform(config.usage),
    };
  }
}

/// Error wrapper for surface acquisition and presentation errors.
#[derive(Clone, Debug)]
pub enum SurfaceError {
  /// The surface has been lost and must be recreated.
  Lost,
  /// The surface configuration is outdated and must be reconfigured.
  Outdated,
  /// Out of memory.
  OutOfMemory,
  /// Timed out waiting for a frame.
  Timeout,
  /// Other/unclassified error.
  Other(String),
}

impl From<platform_surface::SurfaceError> for SurfaceError {
  fn from(error: platform_surface::SurfaceError) -> Self {
    return match error {
      platform_surface::SurfaceError::Lost => SurfaceError::Lost,
      platform_surface::SurfaceError::Outdated => SurfaceError::Outdated,
      platform_surface::SurfaceError::OutOfMemory => SurfaceError::OutOfMemory,
      platform_surface::SurfaceError::Timeout => SurfaceError::Timeout,
      platform_surface::SurfaceError::Other(msg) => SurfaceError::Other(msg),
    };
  }
}
