use wgpu::rwh::{
  HasDisplayHandle as _,
  HasWindowHandle as _,
};

use super::{
  gpu::Gpu,
  instance::Instance,
  texture::{
    TextureFormat,
    TextureUsages,
  },
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
    };
  }
}

/// Public, engine-facing surface configuration that avoids exposing `wgpu`.
#[derive(Clone, Debug)]
pub struct SurfaceConfig {
  pub width: u32,
  pub height: u32,
  pub format: TextureFormat,
  pub present_mode: PresentMode,
  pub usage: TextureUsages,
  pub view_formats: Vec<TextureFormat>,
}

impl SurfaceConfig {
  pub(crate) fn from_wgpu(config: &wgpu::SurfaceConfiguration) -> Self {
    return SurfaceConfig {
      width: config.width,
      height: config.height,
      format: TextureFormat::from_wgpu(config.format),
      present_mode: PresentMode::from_wgpu(config.present_mode),
      usage: TextureUsages::from_wgpu(config.usage),
      view_formats: config
        .view_formats
        .iter()
        .copied()
        .map(TextureFormat::from_wgpu)
        .collect(),
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
  pub fn build(
    self,
    instance: &Instance,
    window: &WindowHandle,
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

impl Default for SurfaceBuilder {
  fn default() -> Self {
    return Self::new();
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
  format: Option<TextureFormat>,
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
  pub fn format(&self) -> Option<TextureFormat> {
    return self.format;
  }

  /// Configure the surface and cache the result for queries such as `format()`.
  fn configure_raw(
    &mut self,
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
  ) {
    self.surface.configure(device, config);
    self.configuration = Some(SurfaceConfig::from_wgpu(config));
    self.format = Some(TextureFormat::from_wgpu(config.format));
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
    config.present_mode = select_present_mode(
      requested_present_mode,
      capabilities.present_modes.as_slice(),
    );

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

fn select_present_mode(
  requested: wgpu::PresentMode,
  available: &[wgpu::PresentMode],
) -> wgpu::PresentMode {
  if available.contains(&requested) {
    return requested;
  }

  let candidates: &[wgpu::PresentMode] = match requested {
    wgpu::PresentMode::Immediate | wgpu::PresentMode::AutoNoVsync => &[
      wgpu::PresentMode::Immediate,
      wgpu::PresentMode::Mailbox,
      wgpu::PresentMode::AutoNoVsync,
      wgpu::PresentMode::Fifo,
      wgpu::PresentMode::AutoVsync,
    ],
    wgpu::PresentMode::Mailbox => &[
      wgpu::PresentMode::Mailbox,
      wgpu::PresentMode::Fifo,
      wgpu::PresentMode::AutoVsync,
    ],
    wgpu::PresentMode::FifoRelaxed => &[
      wgpu::PresentMode::FifoRelaxed,
      wgpu::PresentMode::Fifo,
      wgpu::PresentMode::AutoVsync,
    ],
    wgpu::PresentMode::Fifo | wgpu::PresentMode::AutoVsync => &[
      wgpu::PresentMode::Fifo,
      wgpu::PresentMode::AutoVsync,
      wgpu::PresentMode::FifoRelaxed,
      wgpu::PresentMode::Mailbox,
      wgpu::PresentMode::Immediate,
      wgpu::PresentMode::AutoNoVsync,
    ],
  };

  for candidate in candidates {
    if available.contains(candidate) {
      return *candidate;
    }
  }

  return available
    .first()
    .copied()
    .unwrap_or(wgpu::PresentMode::Fifo);
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

  /// Present the frame to the swapchain.
  pub fn present(self) {
    self.texture.present();
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn select_present_mode_prefers_requested() {
    let available = &[wgpu::PresentMode::Fifo, wgpu::PresentMode::Immediate];
    let selected = select_present_mode(wgpu::PresentMode::Immediate, available);
    assert_eq!(selected, wgpu::PresentMode::Immediate);
  }

  #[test]
  fn select_present_mode_falls_back_from_immediate_to_mailbox() {
    let available = &[wgpu::PresentMode::Fifo, wgpu::PresentMode::Mailbox];
    let selected = select_present_mode(wgpu::PresentMode::Immediate, available);
    assert_eq!(selected, wgpu::PresentMode::Mailbox);
  }

  #[test]
  fn select_present_mode_falls_back_from_mailbox_to_fifo() {
    let available = &[wgpu::PresentMode::Fifo, wgpu::PresentMode::Immediate];
    let selected = select_present_mode(wgpu::PresentMode::Mailbox, available);
    assert_eq!(selected, wgpu::PresentMode::Fifo);
  }

  #[test]
  fn select_present_mode_uses_auto_no_vsync_when_available() {
    let available = &[wgpu::PresentMode::AutoNoVsync, wgpu::PresentMode::Fifo];
    let selected = select_present_mode(wgpu::PresentMode::Immediate, available);
    assert_eq!(selected, wgpu::PresentMode::AutoNoVsync);
  }
}
