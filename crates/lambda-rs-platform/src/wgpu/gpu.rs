use pollster::block_on;

use super::{
  command::CommandBuffer,
  instance::Instance,
  surface::Surface,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// Power preference for selecting a GPU adapter.
pub enum PowerPreference {
  HighPerformance,
  LowPower,
}

impl PowerPreference {
  pub(crate) fn to_wgpu(self) -> wgpu::PowerPreference {
    return match self {
      PowerPreference::HighPerformance => {
        wgpu::PowerPreference::HighPerformance
      }
      PowerPreference::LowPower => wgpu::PowerPreference::LowPower,
    };
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// Memory allocation hints for device and resource creation.
pub enum MemoryHints {
  Performance,
  MemoryUsage,
}

impl MemoryHints {
  pub(crate) fn to_wgpu(self) -> wgpu::MemoryHints {
    return match self {
      MemoryHints::Performance => wgpu::MemoryHints::Performance,
      MemoryHints::MemoryUsage => wgpu::MemoryHints::MemoryUsage,
    };
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// Feature bitset required/enabled on the device.
pub struct Features(wgpu::Features);

impl Features {
  /// Enable push constants support.
  pub const PUSH_CONSTANTS: Features = Features(wgpu::Features::PUSH_CONSTANTS);

  pub(crate) fn to_wgpu(self) -> wgpu::Features {
    self.0
  }
}

impl std::ops::BitOr for Features {
  type Output = Features;
  fn bitor(self, rhs: Features) -> Features {
    return Features(self.0 | rhs.0);
  }
}

#[derive(Clone, Copy, Debug)]
/// Public, engine-facing subset of device limits.
pub struct GpuLimits {
  pub max_uniform_buffer_binding_size: u64,
  pub max_bind_groups: u32,
  pub min_uniform_buffer_offset_alignment: u32,
}

#[derive(Debug, Clone)]
/// Builder for a `Gpu` (adapter, device, queue) with feature validation.
pub struct GpuBuilder {
  label: Option<String>,
  power_preference: PowerPreference,
  force_fallback_adapter: bool,
  required_features: Features,
  memory_hints: MemoryHints,
}

impl GpuBuilder {
  /// Create a builder with defaults favoring performance and push constants.
  pub fn new() -> Self {
    Self {
      label: Some("Lambda GPU".to_string()),
      power_preference: PowerPreference::HighPerformance,
      force_fallback_adapter: false,
      required_features: Features::PUSH_CONSTANTS,
      memory_hints: MemoryHints::Performance,
    }
  }

  /// Attach a label used for the device.
  pub fn with_label(mut self, label: &str) -> Self {
    self.label = Some(label.to_string());
    self
  }

  /// Select the adapter power preference (e.g., LowPower for laptops).
  pub fn with_power_preference(mut self, preference: PowerPreference) -> Self {
    self.power_preference = preference;
    self
  }

  /// Force using a fallback adapter when a primary device is unavailable.
  pub fn force_fallback(mut self, force: bool) -> Self {
    self.force_fallback_adapter = force;
    self
  }

  /// Require `wgpu::Features` to be present on the adapter.
  pub fn with_required_features(mut self, features: Features) -> Self {
    self.required_features = features;
    self
  }

  /// Provide memory allocation hints for the device.
  pub fn with_memory_hints(mut self, hints: MemoryHints) -> Self {
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
        power_preference: self.power_preference.to_wgpu(),
        force_fallback_adapter: self.force_fallback_adapter,
        compatible_surface: surface.map(|surface| surface.surface()),
      })
      .map_err(|_| GpuBuildError::AdapterUnavailable)?;

    let adapter_features = adapter.features();
    if !adapter_features.contains(self.required_features.to_wgpu()) {
      return Err(GpuBuildError::MissingFeatures {
        requested: self.required_features,
        available: Features(adapter_features),
      });
    }

    let descriptor = wgpu::DeviceDescriptor {
      label: self.label.as_deref(),
      required_features: self.required_features.to_wgpu(),
      required_limits: adapter.limits(),
      memory_hints: self.memory_hints.to_wgpu(),
      trace: wgpu::Trace::Off,
    };

    let (device, queue) = block_on(adapter.request_device(&descriptor))?;

    return Ok(Gpu {
      adapter,
      device,
      queue,
      features: descriptor.required_features,
      limits: descriptor.required_limits,
    });
  }
}

/// Errors emitted while building a `Gpu`.
#[derive(Debug)]
pub enum GpuBuildError {
  /// No compatible adapter could be found.
  AdapterUnavailable,
  /// The requested features are not supported by the selected adapter.
  MissingFeatures {
    requested: Features,
    available: Features,
  },
  /// Wrapper for `wgpu::RequestDeviceError`.
  RequestDevice(String),
}

impl From<wgpu::RequestDeviceError> for GpuBuildError {
  fn from(error: wgpu::RequestDeviceError) -> Self {
    return GpuBuildError::RequestDevice(format!("{:?}", error));
  }
}

/// Holds the chosen adapter along with its logical device and submission queue
/// plus immutable copies of features and limits used to create the device.
#[derive(Debug)]
pub struct Gpu {
  adapter: wgpu::Adapter,
  device: wgpu::Device,
  queue: wgpu::Queue,
  features: wgpu::Features,
  limits: wgpu::Limits,
}

impl Gpu {
  /// Borrow the adapter used to create the device.
  ///
  /// Crate-visible to avoid exposing raw `wgpu` to higher layers.
  pub(crate) fn adapter(&self) -> &wgpu::Adapter {
    &self.adapter
  }

  /// Borrow the logical device for resource creation.
  ///
  /// Crate-visible to avoid exposing raw `wgpu` to higher layers.
  pub(crate) fn device(&self) -> &wgpu::Device {
    &self.device
  }

  /// Borrow the submission queue for command submission.
  ///
  /// Crate-visible to avoid exposing raw `wgpu` to higher layers.
  pub(crate) fn queue(&self) -> &wgpu::Queue {
    &self.queue
  }

  /// Features that were required and enabled during device creation.
  pub(crate) fn features(&self) -> wgpu::Features {
    self.features
  }

  /// Limits captured at device creation time.
  pub fn limits(&self) -> GpuLimits {
    return GpuLimits {
      max_uniform_buffer_binding_size: self
        .limits
        .max_uniform_buffer_binding_size
        .into(),
      max_bind_groups: self.limits.max_bind_groups,
      min_uniform_buffer_offset_alignment: self
        .limits
        .min_uniform_buffer_offset_alignment,
    };
  }

  /// Submit one or more command buffers to the device queue.
  pub fn submit<I>(&self, list: I)
  where
    I: IntoIterator<Item = CommandBuffer>,
  {
    let iter = list.into_iter().map(|cb| cb.into_raw());
    self.queue.submit(iter);
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn gpu_build_error_wraps_request_device_error() {
    // RequestDeviceError is opaque in wgpu 26 (no public constructors or variants).
    // This test previously validated pattern matching on a specific variant; now we
    // simply assert the From<wgpu::RequestDeviceError> implementation exists by
    // checking the trait bound at compile time.
    fn assert_from_impl<T: From<wgpu::RequestDeviceError>>() {}
    assert_from_impl::<GpuBuildError>();
  }
}
