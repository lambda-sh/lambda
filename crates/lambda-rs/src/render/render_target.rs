//! Render target abstraction for different presentation backends.
//!
//! The `RenderTarget` trait defines the interface for acquiring frames and
//! presenting rendered content. Implementations include:
//!
//! - `WindowSurface`: Renders to a window's swapchain (the common case)
//! - Future: `OffscreenTarget` for headless rendering to textures
//!
//! # Usage
//!
//! Render targets are used by the render context to acquire frames:
//!
//! ```ignore
//! let frame = render_target.acquire_frame()?;
//! // ... encode and submit commands ...
//! frame.present();
//! ```

use lambda_platform::wgpu as platform;

use super::{
  gpu::Gpu,
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

/// Trait for render targets that can acquire and present frames.
///
/// This abstraction enables different rendering backends:
/// - Window surfaces for on-screen rendering
/// - Offscreen textures for headless/screenshot rendering
/// - Custom targets for specialized use cases
pub trait RenderTarget {
  /// Acquire the next frame for rendering.
  ///
  /// Returns a `Frame` that can be rendered to and then presented. The frame
  /// owns the texture view for the duration of rendering.
  fn acquire_frame(&mut self) -> Result<Frame, SurfaceError>;

  /// Resize the render target to the specified dimensions.
  ///
  /// This reconfigures the underlying resources (swapchain, textures) to
  /// match the new size. Pass the `Gpu` for resource recreation.
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
/// and presentation through the GPU's swapchain.
pub struct WindowSurface {
  inner: platform::surface::Surface<'static>,
  config: Option<SurfaceConfig>,
  size: (u32, u32),
}

impl WindowSurface {
  /// Create a new window surface bound to the given window.
  ///
  /// The surface must be configured before use by calling
  /// `configure_with_defaults` or `resize`.
  pub fn new(
    instance: &platform::instance::Instance,
    window: &Window,
  ) -> Result<Self, WindowSurfaceError> {
    let surface = platform::surface::SurfaceBuilder::new()
      .with_label("Lambda Window Surface")
      .build(instance, window.window_handle())
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
  ///
  /// This selects an sRGB format if available, uses the specified present
  /// mode (falling back to Fifo if unsupported), and enables render
  /// attachment usage.
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

    // Cache the configuration
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

    // Update cached configuration
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
      WindowSurfaceError::CreationFailed(msg) => write!(f, "{}", msg),
    };
  }
}

impl std::error::Error for WindowSurfaceError {}
