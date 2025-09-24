//! Cross‑platform GPU abstraction built on top of `wgpu`.
//!
//! This module exposes a small, opinionated wrapper around core `wgpu` types
//! to make engine code concise while keeping configuration explicit. The
//! builders here (for the instance, surface, and device/queue) provide sane
//! defaults and narrow the surface area used by Lambda, without hiding
//! important handles when you need to drop down to raw `wgpu`.

use pollster::block_on;
use wgpu::rwh::{HasDisplayHandle as _, HasWindowHandle as _};

use crate::winit::WindowHandle;

pub use wgpu as types;

#[derive(Debug, Clone)]
/// Builder for creating a `wgpu::Instance` with consistent defaults.
///
/// - Defaults to primary backends and no special flags.
/// - All options map 1:1 to the underlying `wgpu::InstanceDescriptor`.
pub struct InstanceBuilder {
  label: Option<String>,
  backends: wgpu::Backends,
  flags: wgpu::InstanceFlags,
  dx12_shader_compiler: wgpu::Dx12Compiler,
  gles_minor_version: wgpu::Gles3MinorVersion,
}

impl InstanceBuilder {
  /// Construct a new builder with Lambda defaults.
  pub fn new() -> Self {
    Self {
      label: None,
      backends: wgpu::Backends::PRIMARY,
      flags: wgpu::InstanceFlags::default(),
      dx12_shader_compiler: wgpu::Dx12Compiler::default(),
      gles_minor_version: wgpu::Gles3MinorVersion::default(),
    }
  }

  /// Attach a debug label to the instance.
  pub fn with_label(mut self, label: &str) -> Self {
    self.label = Some(label.to_string());
    self
  }

  /// Select which graphics backends to enable.
  pub fn with_backends(mut self, backends: wgpu::Backends) -> Self {
    self.backends = backends;
    self
  }

  /// Set additional instance flags (e.g., debugging).
  pub fn with_flags(mut self, flags: wgpu::InstanceFlags) -> Self {
    self.flags = flags;
    self
  }

  /// Choose a DX12 shader compiler variant when on Windows.
  pub fn with_dx12_shader_compiler(
    mut self,
    compiler: wgpu::Dx12Compiler,
  ) -> Self {
    self.dx12_shader_compiler = compiler;
    self
  }

  /// Configure the GLES minor version for WebGL/OpenGL ES targets.
  pub fn with_gles_minor_version(
    mut self,
    version: wgpu::Gles3MinorVersion,
  ) -> Self {
    self.gles_minor_version = version;
    self
  }

  /// Build the `Instance` wrapper from the accumulated options.
  pub fn build(self) -> Instance {
    let descriptor = wgpu::InstanceDescriptor {
      backends: self.backends,
      flags: self.flags,
      dx12_shader_compiler: self.dx12_shader_compiler,
      gles_minor_version: self.gles_minor_version,
    };

    Instance {
      label: self.label,
      instance: wgpu::Instance::new(descriptor),
    }
  }
}

#[derive(Debug)]
/// Thin wrapper over `wgpu::Instance` that preserves a user label and exposes
/// a blocking `request_adapter` convenience.
pub struct Instance {
  label: Option<String>,
  instance: wgpu::Instance,
}

impl Instance {
  /// Borrow the underlying `wgpu::Instance`.
  pub fn raw(&self) -> &wgpu::Instance {
    &self.instance
  }

  /// Return the optional label attached at construction time.
  pub fn label(&self) -> Option<&str> {
    self.label.as_deref()
  }

  /// Request a compatible GPU adapter synchronously.
  ///
  /// This simply blocks on `wgpu::Instance::request_adapter` and returns
  /// `None` if no suitable adapter is found.
  pub fn request_adapter<'surface, 'window>(
    &self,
    options: &wgpu::RequestAdapterOptions<'surface, 'window>,
  ) -> Option<wgpu::Adapter> {
    block_on(self.instance.request_adapter(options))
  }
}

#[derive(Debug, Clone)]
/// Builder for creating a `Surface` bound to a `winit` window.
pub struct SurfaceBuilder {
  label: Option<String>,
}

impl SurfaceBuilder {
  /// Create a builder with no label.
  pub fn new() -> Self {
    Self { label: None }
  }

  /// Attach a human‑readable label for debugging/profiling.
  pub fn with_label(mut self, label: &str) -> Self {
    self.label = Some(label.to_string());
    self
  }

  /// Create a `wgpu::Surface` for the provided `WindowHandle`.
  ///
  /// Safety: we use `create_surface_unsafe` by forwarding raw window/display
  /// handles from `winit`. Lambda guarantees the window outlives the surface
  /// for the duration of the runtime.
  pub fn build<'window>(
    self,
    instance: &Instance,
    window: &'window WindowHandle,
  ) -> Result<Surface<'static>, wgpu::CreateSurfaceError> {
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
      instance.raw().create_surface_unsafe(
        wgpu::SurfaceTargetUnsafe::RawHandle {
          raw_display_handle,
          raw_window_handle,
        },
      )?
    };

    Ok(Surface {
      label: self.label.unwrap_or_else(|| "Lambda Surface".to_string()),
      surface,
      configuration: None,
      format: None,
    })
  }
}

#[derive(Debug)]
/// Presentation surface wrapper with cached configuration and format.
pub struct Surface<'window> {
  label: String,
  surface: wgpu::Surface<'window>,
  configuration: Option<wgpu::SurfaceConfiguration>,
  format: Option<wgpu::TextureFormat>,
}

impl<'window> Surface<'window> {
  /// Immutable label used for debugging.
  pub fn label(&self) -> &str {
    &self.label
  }

  /// Borrow the raw `wgpu::Surface`.
  pub fn surface(&self) -> &wgpu::Surface<'window> {
    &self.surface
  }

  /// Current configuration, if the surface has been configured.
  pub fn configuration(&self) -> Option<&wgpu::SurfaceConfiguration> {
    self.configuration.as_ref()
  }

  /// Preferred surface format if known (set during configuration).
  pub fn format(&self) -> Option<wgpu::TextureFormat> {
    self.format
  }

  /// Configure the surface with the provided `wgpu::SurfaceConfiguration` and
  /// cache the result for queries such as `format()`.
  pub fn configure(
    &mut self,
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
  ) {
    self.surface.configure(device, config);
    self.configuration = Some(config.clone());
    self.format = Some(config.format);
  }

  /// Configure the surface using common engine defaults:
  /// - sRGB color format if available
  /// - fallback present mode compatible with the platform
  /// - `RENDER_ATTACHMENT` usage if requested usage is unsupported
  pub fn configure_with_defaults(
    &mut self,
    adapter: &wgpu::Adapter,
    device: &wgpu::Device,
    size: (u32, u32),
    present_mode: wgpu::PresentMode,
    usage: wgpu::TextureUsages,
  ) -> Result<wgpu::SurfaceConfiguration, String> {
    let width = size.0.max(1);
    let height = size.1.max(1);

    let mut config = self
      .surface
      .get_default_config(adapter, width, height)
      .ok_or_else(|| "Surface not supported by adapter".to_string())?;

    let capabilities = self.surface.get_capabilities(adapter);

    config.format = capabilities
      .formats
      .iter()
      .copied()
      .find(|format| format.is_srgb())
      .unwrap_or_else(|| *capabilities.formats.first().unwrap());

    config.present_mode = if capabilities.present_modes.contains(&present_mode)
    {
      present_mode
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

    if capabilities.usages.contains(usage) {
      config.usage = usage;
    } else {
      config.usage = wgpu::TextureUsages::RENDER_ATTACHMENT;
    }

    if config.view_formats.is_empty() && !config.format.is_srgb() {
      config.view_formats.push(config.format.add_srgb_suffix());
    }

    self.configure(device, &config);
    Ok(config)
  }

  /// Resize the surface while preserving present mode and usage when possible.
  pub fn resize(
    &mut self,
    adapter: &wgpu::Adapter,
    device: &wgpu::Device,
    size: (u32, u32),
  ) -> Result<(), String> {
    let present_mode = self
      .configuration
      .as_ref()
      .map(|config| config.present_mode)
      .unwrap_or(wgpu::PresentMode::Fifo);
    let usage = self
      .configuration
      .as_ref()
      .map(|config| config.usage)
      .unwrap_or(wgpu::TextureUsages::RENDER_ATTACHMENT);

    self
      .configure_with_defaults(adapter, device, size, present_mode, usage)
      .map(|_| ())
  }

  /// Acquire the next swapchain texture and a default view.
  pub fn acquire_next_frame(&self) -> Result<Frame, wgpu::SurfaceError> {
    let texture = self.surface.get_current_texture()?;
    let view = texture
      .texture
      .create_view(&wgpu::TextureViewDescriptor::default());

    Ok(Frame { texture, view })
  }
}

#[derive(Debug)]
/// A single acquired frame and its default `TextureView`.
pub struct Frame {
  texture: wgpu::SurfaceTexture,
  view: wgpu::TextureView,
}

impl Frame {
  /// Borrow the default view for rendering.
  pub fn texture_view(&self) -> &wgpu::TextureView {
    &self.view
  }

  /// Consume and return the underlying parts.
  pub fn into_parts(self) -> (wgpu::SurfaceTexture, wgpu::TextureView) {
    (self.texture, self.view)
  }

  /// Present the frame to the swapchain.
  pub fn present(self) {
    self.texture.present();
  }
}

#[derive(Debug, Clone)]
/// Builder for a `Gpu` (adapter, device, queue) with feature validation.
pub struct GpuBuilder {
  label: Option<String>,
  power_preference: wgpu::PowerPreference,
  force_fallback_adapter: bool,
  required_features: wgpu::Features,
  memory_hints: wgpu::MemoryHints,
}

impl GpuBuilder {
  /// Create a builder with defaults favoring performance and push constants.
  pub fn new() -> Self {
    Self {
      label: Some("Lambda GPU".to_string()),
      power_preference: wgpu::PowerPreference::HighPerformance,
      force_fallback_adapter: false,
      required_features: wgpu::Features::PUSH_CONSTANTS,
      memory_hints: wgpu::MemoryHints::Performance,
    }
  }

  /// Attach a label used for the device.
  pub fn with_label(mut self, label: &str) -> Self {
    self.label = Some(label.to_string());
    self
  }

  /// Select the adapter power preference (e.g., LowPower for laptops).
  pub fn with_power_preference(
    mut self,
    preference: wgpu::PowerPreference,
  ) -> Self {
    self.power_preference = preference;
    self
  }

  /// Force using a fallback adapter when a primary device is unavailable.
  pub fn force_fallback(mut self, force: bool) -> Self {
    self.force_fallback_adapter = force;
    self
  }

  /// Require `wgpu::Features` to be present on the adapter.
  pub fn with_required_features(mut self, features: wgpu::Features) -> Self {
    self.required_features = features;
    self
  }

  /// Provide memory allocation hints for the device.
  pub fn with_memory_hints(mut self, hints: wgpu::MemoryHints) -> Self {
    self.memory_hints = hints;
    self
  }

  /// Request an adapter and device/queue pair and return a `Gpu` wrapper.
  ///
  /// Returns an error if no adapter is available, required features are
  /// missing, or device creation fails.
  pub fn build<'surface, 'window>(
    self,
    instance: &Instance,
    surface: Option<&Surface<'surface>>,
  ) -> Result<Gpu, GpuBuildError> {
    let adapter = instance
      .request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: self.power_preference,
        force_fallback_adapter: self.force_fallback_adapter,
        compatible_surface: surface.map(|surface| surface.surface()),
      })
      .ok_or(GpuBuildError::AdapterUnavailable)?;

    let adapter_features = adapter.features();
    if !adapter_features.contains(self.required_features) {
      return Err(GpuBuildError::MissingFeatures {
        requested: self.required_features,
        available: adapter_features,
      });
    }

    let descriptor = wgpu::DeviceDescriptor {
      label: self.label.as_deref(),
      required_features: self.required_features,
      required_limits: adapter.limits(),
      memory_hints: self.memory_hints,
    };

    let (device, queue) = block_on(adapter.request_device(&descriptor, None))?;

    Ok(Gpu {
      adapter,
      device,
      queue,
      features: descriptor.required_features,
      limits: descriptor.required_limits,
    })
  }
}

#[derive(Debug)]
/// Errors emitted while building a `Gpu`.
pub enum GpuBuildError {
  /// No compatible adapter could be found.
  AdapterUnavailable,
  /// The requested features are not supported by the selected adapter.
  MissingFeatures {
    requested: wgpu::Features,
    available: wgpu::Features,
  },
  /// Wrapper for `wgpu::RequestDeviceError`.
  RequestDevice(wgpu::RequestDeviceError),
}

impl From<wgpu::RequestDeviceError> for GpuBuildError {
  fn from(error: wgpu::RequestDeviceError) -> Self {
    GpuBuildError::RequestDevice(error)
  }
}

#[derive(Debug)]
/// Holds the chosen adapter along with its logical device and submission queue
/// plus immutable copies of features and limits used to create the device.
pub struct Gpu {
  adapter: wgpu::Adapter,
  device: wgpu::Device,
  queue: wgpu::Queue,
  features: wgpu::Features,
  limits: wgpu::Limits,
}

impl Gpu {
  /// Borrow the adapter used to create the device.
  pub fn adapter(&self) -> &wgpu::Adapter {
    &self.adapter
  }

  /// Borrow the logical device for resource creation.
  pub fn device(&self) -> &wgpu::Device {
    &self.device
  }

  /// Borrow the submission queue for command submission.
  pub fn queue(&self) -> &wgpu::Queue {
    &self.queue
  }

  /// Features that were required and enabled during device creation.
  pub fn features(&self) -> wgpu::Features {
    self.features
  }

  /// Limits captured at device creation time.
  pub fn limits(&self) -> &wgpu::Limits {
    &self.limits
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn instance_builder_sets_label() {
    let instance = InstanceBuilder::new().with_label("Test").build();
    assert_eq!(instance.label(), Some("Test"));
  }

  #[test]
  fn gpu_build_error_wraps_request_device_error() {
    let error = GpuBuildError::from(wgpu::RequestDeviceError::NotFound);
    assert!(matches!(
      error,
      GpuBuildError::RequestDevice(wgpu::RequestDeviceError::NotFound)
    ));
  }
}
