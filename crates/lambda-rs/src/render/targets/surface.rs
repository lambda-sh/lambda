//! Presentation surface render targets.
//!
//! A surface render target acquires a frame from a swapchain-like surface and
//! presents it after encoding completes. This is the on-screen rendering path.

use lambda_platform::wgpu as platform;

use crate::render::{
  gpu::Gpu,
  instance::Instance,
  texture::{
    TextureFormat,
    TextureUsages,
  },
  window::Window,
};

// ---------------------------------------------------------------------------
// TextureView
// ---------------------------------------------------------------------------

/// High-level reference to a texture view for render pass attachments.
///
/// This type wraps the platform `TextureViewRef` and provides a stable
/// engine-level API for referencing texture views without exposing `wgpu`
/// types at call sites.
#[derive(Clone, Copy)]
pub struct TextureView<'a> {
  inner: platform::surface::TextureViewRef<'a>,
}

impl<'a> TextureView<'a> {
  /// Create a high-level texture view from a platform texture view reference.
  #[inline]
  pub(crate) fn from_platform(
    view: platform::surface::TextureViewRef<'a>,
  ) -> Self {
    return TextureView { inner: view };
  }

  /// Convert to the platform texture view reference for internal use.
  #[inline]
  pub(crate) fn to_platform(self) -> platform::surface::TextureViewRef<'a> {
    return self.inner;
  }
}

impl<'a> std::fmt::Debug for TextureView<'a> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    return f.debug_struct("TextureView").finish_non_exhaustive();
  }
}

// ---------------------------------------------------------------------------
// Frame
// ---------------------------------------------------------------------------

/// A single acquired frame from the presentation surface.
///
/// This type wraps the platform `Frame` and provides access to its texture
/// view for rendering. The frame must be presented after rendering is complete
/// by calling `present()`.
pub struct Frame {
  inner: platform::surface::Frame,
}

impl Frame {
  /// Create a high-level frame from a platform frame.
  #[inline]
  pub(crate) fn from_platform(frame: platform::surface::Frame) -> Self {
    return Frame { inner: frame };
  }

  /// Borrow the default texture view for rendering to this frame.
  #[inline]
  pub fn texture_view(&self) -> TextureView<'_> {
    return TextureView::from_platform(self.inner.texture_view());
  }

  /// Present the frame to the swapchain.
  ///
  /// This consumes the frame and submits it for display. After calling this
  /// method, the frame's texture is no longer valid for rendering.
  #[inline]
  pub fn present(self) {
    self.inner.present();
  }
}

impl std::fmt::Debug for Frame {
  fn fmt(&self, frame: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    return frame.debug_struct("Frame").finish_non_exhaustive();
  }
}

// ---------------------------------------------------------------------------
// PresentMode
// ---------------------------------------------------------------------------

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
  pub(crate) fn to_platform(self) -> platform::surface::PresentMode {
    return match self {
      PresentMode::Fifo => platform::surface::PresentMode::Fifo,
      PresentMode::FifoRelaxed => platform::surface::PresentMode::FifoRelaxed,
      PresentMode::Immediate => platform::surface::PresentMode::Immediate,
      PresentMode::Mailbox => platform::surface::PresentMode::Mailbox,
      PresentMode::AutoVsync => platform::surface::PresentMode::AutoVsync,
      PresentMode::AutoNoVsync => platform::surface::PresentMode::AutoNoVsync,
    };
  }

  #[inline]
  pub(crate) fn from_platform(
    mode: platform::surface::PresentMode,
  ) -> PresentMode {
    return match mode {
      platform::surface::PresentMode::Fifo => PresentMode::Fifo,
      platform::surface::PresentMode::FifoRelaxed => PresentMode::FifoRelaxed,
      platform::surface::PresentMode::Immediate => PresentMode::Immediate,
      platform::surface::PresentMode::Mailbox => PresentMode::Mailbox,
      platform::surface::PresentMode::AutoVsync => PresentMode::AutoVsync,
      platform::surface::PresentMode::AutoNoVsync => PresentMode::AutoNoVsync,
    };
  }
}

impl Default for PresentMode {
  fn default() -> Self {
    return PresentMode::Fifo;
  }
}

// ---------------------------------------------------------------------------
// SurfaceConfig
// ---------------------------------------------------------------------------

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
    config: &platform::surface::SurfaceConfig,
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

// ---------------------------------------------------------------------------
// SurfaceError
// ---------------------------------------------------------------------------

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

impl From<platform::surface::SurfaceError> for SurfaceError {
  fn from(error: platform::surface::SurfaceError) -> Self {
    return match error {
      platform::surface::SurfaceError::Lost => SurfaceError::Lost,
      platform::surface::SurfaceError::Outdated => SurfaceError::Outdated,
      platform::surface::SurfaceError::OutOfMemory => SurfaceError::OutOfMemory,
      platform::surface::SurfaceError::Timeout => SurfaceError::Timeout,
      platform::surface::SurfaceError::Other(msg) => SurfaceError::Other(msg),
    };
  }
}

// ---------------------------------------------------------------------------
// RenderTarget trait
// ---------------------------------------------------------------------------

/// Presentation render target that can acquire and present frames.
pub trait RenderTarget {
  /// Acquire the next frame for rendering.
  fn acquire_frame(&mut self) -> Result<Frame, SurfaceError>;

  /// Resize the render target to the specified dimensions.
  fn resize(&mut self, gpu: &Gpu, size: (u32, u32)) -> Result<(), String>;

  /// Get the texture format used by this render target.
  fn format(&self) -> TextureFormat;

  /// Get the current dimensions of the render target.
  fn size(&self) -> (u32, u32);

  /// Get the current configuration, if available.
  fn configuration(&self) -> Option<&SurfaceConfig>;
}

// ---------------------------------------------------------------------------
// WindowSurface
// ---------------------------------------------------------------------------

/// Render target for window-based presentation.
///
/// Wraps a platform surface bound to a window, providing frame acquisition
/// and presentation through the GPU's surface configuration.
pub struct WindowSurface {
  inner: platform::surface::Surface<'static>,
  config: Option<SurfaceConfig>,
  size: (u32, u32),
}

impl WindowSurface {
  /// Create a new window surface bound to the given window.
  ///
  /// The surface MUST be configured before use by calling
  /// `configure_with_defaults` or `resize`.
  pub fn new(
    instance: &Instance,
    window: &Window,
  ) -> Result<Self, WindowSurfaceError> {
    let surface = platform::surface::SurfaceBuilder::new()
      .with_label("Lambda Window Surface")
      .build(instance.platform(), window.window_handle())
      .map_err(|_| {
        WindowSurfaceError::CreationFailed(
          "Failed to create window surface".to_string(),
        )
      })?;

    return Ok(WindowSurface {
      inner: surface,
      config: None,
      size: window.dimensions(),
    });
  }

  /// Configure the surface with sensible defaults for the given GPU.
  pub fn configure_with_defaults(
    &mut self,
    gpu: &Gpu,
    size: (u32, u32),
    present_mode: PresentMode,
    usage: TextureUsages,
  ) -> Result<(), String> {
    self.inner.configure_with_defaults(
      gpu.platform(),
      size,
      present_mode.to_platform(),
      usage.to_platform(),
    )?;

    if let Some(platform_config) = self.inner.configuration() {
      self.config = Some(SurfaceConfig::from_platform(platform_config));
    }
    self.size = size;

    return Ok(());
  }

  /// Borrow the underlying platform surface for internal use.
  #[inline]
  pub(crate) fn platform(&self) -> &platform::surface::Surface<'static> {
    return &self.inner;
  }
}

impl RenderTarget for WindowSurface {
  fn acquire_frame(&mut self) -> Result<Frame, SurfaceError> {
    let platform_frame = self
      .inner
      .acquire_next_frame()
      .map_err(SurfaceError::from)?;
    return Ok(Frame::from_platform(platform_frame));
  }

  fn resize(&mut self, gpu: &Gpu, size: (u32, u32)) -> Result<(), String> {
    self.inner.resize(gpu.platform(), size)?;

    if let Some(platform_config) = self.inner.configuration() {
      self.config = Some(SurfaceConfig::from_platform(platform_config));
    }
    self.size = size;

    return Ok(());
  }

  fn format(&self) -> TextureFormat {
    return self
      .config
      .as_ref()
      .map(|c| c.format)
      .unwrap_or(TextureFormat::Bgra8UnormSrgb);
  }

  fn size(&self) -> (u32, u32) {
    return self.size;
  }

  fn configuration(&self) -> Option<&SurfaceConfig> {
    return self.config.as_ref();
  }
}

impl std::fmt::Debug for WindowSurface {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    return f
      .debug_struct("WindowSurface")
      .field("size", &self.size)
      .field("config", &self.config)
      .finish_non_exhaustive();
  }
}

// ---------------------------------------------------------------------------
// WindowSurfaceError
// ---------------------------------------------------------------------------

/// Errors that can occur when creating a window surface.
#[derive(Debug)]
pub enum WindowSurfaceError {
  /// Surface creation failed.
  CreationFailed(String),
}

impl std::fmt::Display for WindowSurfaceError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    return match self {
      WindowSurfaceError::CreationFailed(message) => write!(f, "{}", message),
    };
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  /// Ensures present mode conversions to/from the platform are lossless.
  #[test]
  fn present_mode_round_trips_through_platform() {
    let modes = [
      PresentMode::Fifo,
      PresentMode::FifoRelaxed,
      PresentMode::Immediate,
      PresentMode::Mailbox,
      PresentMode::AutoVsync,
      PresentMode::AutoNoVsync,
    ];

    for mode in modes {
      let platform = mode.to_platform();
      let back = PresentMode::from_platform(platform);
      assert_eq!(back, mode);
    }
  }

  /// Ensures each platform surface error maps to the corresponding engine
  /// surface error variant.
  #[test]
  fn surface_error_maps_platform_variants() {
    assert!(matches!(
      SurfaceError::from(platform::surface::SurfaceError::Lost),
      SurfaceError::Lost
    ));
    assert!(matches!(
      SurfaceError::from(platform::surface::SurfaceError::Outdated),
      SurfaceError::Outdated
    ));
    assert!(matches!(
      SurfaceError::from(platform::surface::SurfaceError::OutOfMemory),
      SurfaceError::OutOfMemory
    ));
    assert!(matches!(
      SurfaceError::from(platform::surface::SurfaceError::Timeout),
      SurfaceError::Timeout
    ));
    let other = SurfaceError::from(platform::surface::SurfaceError::Other(
      "opaque".to_string(),
    ));
    assert!(matches!(other, SurfaceError::Other(_)));
  }

  /// Ensures window surface errors are displayed using the underlying message.
  #[test]
  fn window_surface_error_is_displayed() {
    let error =
      WindowSurfaceError::CreationFailed("creation failed".to_string());
    assert_eq!(error.to_string(), "creation failed");
  }
}
