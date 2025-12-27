//! Presentation surface render targets.
//!
//! A surface render target acquires a frame from a swapchain-like surface and
//! presents it after encoding completes. This is the on-screen rendering path.

use lambda_platform::wgpu as platform;

use crate::render::{
  gpu::Gpu,
  instance::Instance,
  surface::{
    Frame,
    PresentMode,
    SurfaceConfig,
    SurfaceError,
  },
  texture::{
    TextureFormat,
    TextureUsages,
  },
  window::Window,
};

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
    self
      .inner
      .configure_with_defaults(
        gpu.platform(),
        size,
        present_mode.to_platform(),
        usage.to_platform(),
      )
      .map_err(|e| e)?;

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

  /// Mutably borrow the underlying platform surface for internal use.
  #[inline]
  pub(crate) fn platform_mut(
    &mut self,
  ) -> &mut platform::surface::Surface<'static> {
    return &mut self.inner;
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
