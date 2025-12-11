use wgpu::rwh::{
  HasDisplayHandle as _,
  HasWindowHandle as _,
};

use super::{
  gpu::Gpu,
  instance::Instance,
  texture::TextureUsages,
};
use crate::winit::WindowHandle;

/// Present modes supported by the surface.
///
/// This wrapper hides the underlying `wgpu` type from higher layers while
/// preserving the same semantics.
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
  pub(crate) fn to_wgpu(self) -> wgpu::PresentMode {
    return match self {
      PresentMode::Fifo => wgpu::PresentMode::Fifo,
      PresentMode::FifoRelaxed => wgpu::PresentMode::FifoRelaxed,
      PresentMode::Immediate => wgpu::PresentMode::Immediate,
      PresentMode::Mailbox => wgpu::PresentMode::Mailbox,
      PresentMode::AutoVsync => wgpu::PresentMode::AutoVsync,
      PresentMode::AutoNoVsync => wgpu::PresentMode::AutoNoVsync,
    };
  }

  pub(crate) fn from_wgpu(mode: wgpu::PresentMode) -> Self {
    return match mode {
      wgpu::PresentMode::Fifo => PresentMode::Fifo,
      wgpu::PresentMode::FifoRelaxed => PresentMode::FifoRelaxed,
      wgpu::PresentMode::Immediate => PresentMode::Immediate,
      wgpu::PresentMode::Mailbox => PresentMode::Mailbox,
      wgpu::PresentMode::AutoVsync => PresentMode::AutoVsync,
      wgpu::PresentMode::AutoNoVsync => PresentMode::AutoNoVsync,
      _ => PresentMode::Fifo,
    };
  }
}

/// Wrapper around a surface color format.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SurfaceFormat(wgpu::TextureFormat);

impl SurfaceFormat {
  /// Common sRGB swapchain format used for windowed rendering.
  pub const BGRA8_UNORM_SRGB: SurfaceFormat =
    SurfaceFormat(wgpu::TextureFormat::Bgra8UnormSrgb);

  pub(crate) fn to_wgpu(self) -> wgpu::TextureFormat {
    return self.0;
  }

  pub(crate) fn from_wgpu(fmt: wgpu::TextureFormat) -> Self {
    return SurfaceFormat(fmt);
  }

  /// Whether this format is sRGB.
  pub fn is_srgb(self) -> bool {
    return self.0.is_srgb();
  }

  /// Return the sRGB variant of the format when applicable.
  pub fn add_srgb_suffix(self) -> Self {
    return SurfaceFormat(self.0.add_srgb_suffix());
  }
}

/// Public, engine-facing surface configuration that avoids exposing `wgpu`.
#[derive(Clone, Debug)]
pub struct SurfaceConfig {
  pub width: u32,
  pub height: u32,
  pub format: SurfaceFormat,
  pub present_mode: PresentMode,
  pub usage: TextureUsages,
  pub view_formats: Vec<SurfaceFormat>,
}

impl SurfaceConfig {
  pub(crate) fn from_wgpu(config: &wgpu::SurfaceConfiguration) -> Self {
    return SurfaceConfig {
      width: config.width,
      height: config.height,
      format: SurfaceFormat::from_wgpu(config.format),
      present_mode: PresentMode::from_wgpu(config.present_mode),
      usage: TextureUsages::from_wgpu(config.usage),
      view_formats: config
        .view_formats
        .iter()
        .copied()
        .map(SurfaceFormat::from_wgpu)
        .collect(),
    };
  }

  pub(crate) fn to_wgpu(&self) -> wgpu::SurfaceConfiguration {
    let mut view_formats: Vec<wgpu::TextureFormat> = Vec::new();
    for vf in &self.view_formats {
      view_formats.push(vf.to_wgpu());
    }
    return wgpu::SurfaceConfiguration {
      usage: self.usage.to_wgpu(),
      format: self.format.to_wgpu(),
      width: self.width,
      height: self.height,
      present_mode: self.present_mode.to_wgpu(),
      desired_maximum_frame_latency: 2,
      alpha_mode: wgpu::CompositeAlphaMode::Opaque,
      view_formats,
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
  /// Other/unclassified error (opaque).
  Other(String),
}

impl From<wgpu::SurfaceError> for SurfaceError {
  fn from(error: wgpu::SurfaceError) -> Self {
    use wgpu::SurfaceError as We;
    match error {
      We::Lost => return SurfaceError::Lost,
      We::Outdated => return SurfaceError::Outdated,
      We::OutOfMemory => return SurfaceError::OutOfMemory,
      We::Timeout => return SurfaceError::Timeout,
      _ => return SurfaceError::Other(format!("{:?}", error)),
    }
  }
}

/// Builder for creating a `Surface` bound to a `winit` window.
#[derive(Debug, Clone)]
pub struct SurfaceBuilder {
  label: Option<String>,
}

impl SurfaceBuilder {
  /// Create a builder with no label.
  pub fn new() -> Self {
    Self { label: None }
  }

  /// Attach a human-readable label for debugging/profiling.
  pub fn with_label(mut self, label: &str) -> Self {
    self.label = Some(label.to_string());
    self
  }

  /// Create a presentation surface for the provided `WindowHandle`.
  ///
  /// Safety: we use `create_surface_unsafe` by forwarding raw window/display
  /// handles from `winit`. Lambda guarantees the window outlives the surface
  /// for the duration of the runtime.
  pub fn build<'window>(
    self,
    instance: &Instance,
    window: &'window WindowHandle,
  ) -> Result<Surface<'static>, CreateSurfaceError> {
    // SAFETY: We ensure the raw window/display handles outlive the surface by
    // keeping the window alive for the duration of the application runtime.
    // Obtain raw handles via raw-window-handle 0.6 traits.
    let raw_display_handle = window
      .window_handle
      .display_handle()
      .expect("Failed to get display handle from window")
      .as_raw();
    let raw_window_handle = window
      .window_handle
      .window_handle()
      .expect("Failed to get window handle from window")
      .as_raw();

    let surface = unsafe {
      instance
        .raw()
        .create_surface_unsafe(wgpu::SurfaceTargetUnsafe::RawHandle {
          raw_display_handle,
          raw_window_handle,
        })
        .map_err(CreateSurfaceError::from)?
    };

    Ok(Surface {
      label: self.label.unwrap_or_else(|| "Lambda Surface".to_string()),
      surface,
      configuration: None,
      format: None,
    })
  }
}

/// Opaque error returned when surface creation fails.
#[derive(Debug)]
pub struct CreateSurfaceError;

impl From<wgpu::CreateSurfaceError> for CreateSurfaceError {
  fn from(_: wgpu::CreateSurfaceError) -> Self {
    return CreateSurfaceError;
  }
}

/// Presentation surface wrapper with cached configuration and format.
#[derive(Debug)]
pub struct Surface<'window> {
  label: String,
  surface: wgpu::Surface<'window>,
  configuration: Option<SurfaceConfig>,
  format: Option<SurfaceFormat>,
}

impl<'window> Surface<'window> {
  /// Immutable label used for debugging.
  pub fn label(&self) -> &str {
    &self.label
  }

  /// Borrow the raw `wgpu::Surface` (crate visibility only).
  pub(crate) fn surface(&self) -> &wgpu::Surface<'window> {
    &self.surface
  }

  /// Current configuration, if the surface has been configured.
  pub fn configuration(&self) -> Option<&SurfaceConfig> {
    return self.configuration.as_ref();
  }

  /// Preferred surface format if known (set during configuration).
  pub fn format(&self) -> Option<SurfaceFormat> {
    return self.format;
  }

  /// Configure the surface and cache the result for queries such as `format()`.
  pub(crate) fn configure_raw(
    &mut self,
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
  ) {
    self.surface.configure(device, config);
    self.configuration = Some(SurfaceConfig::from_wgpu(config));
    self.format = Some(SurfaceFormat::from_wgpu(config.format));
  }

  /// Configure the surface using common engine defaults:
  /// - sRGB color format if available
  /// - fallback present mode compatible with the platform
  /// - `RENDER_ATTACHMENT` usage if requested usage is unsupported
  pub fn configure_with_defaults(
    &mut self,
    gpu: &Gpu,
    size: (u32, u32),
    present_mode: PresentMode,
    usage: TextureUsages,
  ) -> Result<(), String> {
    let width = size.0.max(1);
    let height = size.1.max(1);

    let mut config = self
      .surface
      .get_default_config(gpu.adapter(), width, height)
      .ok_or_else(|| "Surface not supported by adapter".to_string())?;

    let capabilities = self.surface.get_capabilities(gpu.adapter());

    config.format = capabilities
      .formats
      .iter()
      .copied()
      .find(|format| format.is_srgb())
      .unwrap_or_else(|| *capabilities.formats.first().unwrap());

    let requested_present_mode = present_mode.to_wgpu();
    config.present_mode = if capabilities
      .present_modes
      .contains(&requested_present_mode)
    {
      requested_present_mode
    } else {
      capabilities
        .present_modes
        .iter()
        .copied()
        .find(|mode| {
          matches!(mode, wgpu::PresentMode::Fifo | wgpu::PresentMode::AutoVsync)
        })
        .unwrap_or(wgpu::PresentMode::Fifo)
    };

    if capabilities.usages.contains(usage.to_wgpu()) {
      config.usage = usage.to_wgpu();
    } else {
      config.usage = wgpu::TextureUsages::RENDER_ATTACHMENT;
    }

    if config.view_formats.is_empty() && !config.format.is_srgb() {
      config.view_formats.push(config.format.add_srgb_suffix());
    }

    self.configure_raw(gpu.device(), &config);
    Ok(())
  }

  /// Resize the surface while preserving present mode and usage when possible.
  pub fn resize(&mut self, gpu: &Gpu, size: (u32, u32)) -> Result<(), String> {
    let present_mode = self
      .configuration
      .as_ref()
      .map(|config| config.present_mode)
      .unwrap_or(PresentMode::Fifo);
    let usage = self
      .configuration
      .as_ref()
      .map(|config| config.usage)
      .unwrap_or(TextureUsages::RENDER_ATTACHMENT);

    return self.configure_with_defaults(gpu, size, present_mode, usage);
  }

  /// Acquire the next swapchain texture and a default view.
  pub fn acquire_next_frame(&self) -> Result<Frame, SurfaceError> {
    let texture = match self.surface.get_current_texture() {
      Ok(t) => t,
      Err(e) => return Err(SurfaceError::from(e)),
    };
    let view = texture
      .texture
      .create_view(&wgpu::TextureViewDescriptor::default());

    return Ok(Frame { texture, view });
  }
}

/// A single acquired frame and its default `TextureView`.
#[derive(Debug)]
pub struct Frame {
  texture: wgpu::SurfaceTexture,
  view: wgpu::TextureView,
}

/// Borrowed reference to a texture view used for render passes.
#[derive(Clone, Copy)]
pub struct TextureViewRef<'a> {
  pub(crate) raw: &'a wgpu::TextureView,
}

impl Frame {
  /// Borrow the default view for rendering.
  pub fn texture_view(&self) -> TextureViewRef<'_> {
    return TextureViewRef { raw: &self.view };
  }

  /// Consume and return the underlying parts.
  pub(crate) fn into_parts(self) -> (wgpu::SurfaceTexture, wgpu::TextureView) {
    return (self.texture, self.view);
  }

  /// Present the frame to the swapchain.
  pub fn present(self) {
    self.texture.present();
  }
}
