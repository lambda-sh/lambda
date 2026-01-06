//! High-level GPU abstraction for resource creation and command submission.
//!
//! The `Gpu` type wraps the platform GPU device and queue, providing a stable
//! engine-facing API for creating resources and submitting work. This
//! abstraction enables future support for multiple render targets and
//! backend flexibility.
//!
//! # Usage
//!
//! The `Gpu` is typically created during render context initialization and
//! shared across resource builders:
//!
//! ```ignore
//! let gpu = GpuBuilder::new()
//!   .with_label("My GPU")
//!   .build(&instance, Some(&surface))?;
//!
//! // Use gpu for resource creation
//! let buffer = BufferBuilder::new()
//!   .with_size(1024)
//!   .build(&gpu);
//! ```

use lambda_platform::wgpu as platform;

use super::{
  instance::Instance,
  targets::surface::WindowSurface,
  texture::{
    DepthFormat,
    TextureFormat,
  },
};

// ---------------------------------------------------------------------------
// GpuLimits
// ---------------------------------------------------------------------------

/// Device limits exposed to the engine layer.
///
/// These limits are queried from the physical device and constrain resource
/// creation and binding. The engine uses these to validate configurations
/// before creating GPU resources.
#[derive(Clone, Copy, Debug)]
pub struct GpuLimits {
  /// Maximum bytes that can be bound for a single uniform buffer binding.
  pub max_uniform_buffer_binding_size: u64,
  /// Maximum number of bind groups that can be used by a pipeline layout.
  pub max_bind_groups: u32,
  /// Maximum number of vertex buffers that can be bound.
  pub max_vertex_buffers: u32,
  /// Maximum number of vertex attributes that can be declared.
  pub max_vertex_attributes: u32,
  /// Required alignment in bytes for dynamic uniform buffer offsets.
  pub min_uniform_buffer_offset_alignment: u32,
}

impl GpuLimits {
  /// Create limits from the platform GPU limits.
  pub(crate) fn from_platform(limits: platform::gpu::GpuLimits) -> Self {
    return GpuLimits {
      max_uniform_buffer_binding_size: limits.max_uniform_buffer_binding_size,
      max_bind_groups: limits.max_bind_groups,
      max_vertex_buffers: limits.max_vertex_buffers,
      max_vertex_attributes: limits.max_vertex_attributes,
      min_uniform_buffer_offset_alignment: limits
        .min_uniform_buffer_offset_alignment,
    };
  }
}

// ---------------------------------------------------------------------------
// Gpu
// ---------------------------------------------------------------------------

/// High-level GPU device and queue wrapper.
///
/// The `Gpu` provides a stable interface for:
/// - Submitting command buffers to the GPU queue
/// - Querying device limits for resource validation
/// - Checking format and sample count support
///
/// This type does not expose platform internals directly, allowing the
/// engine to evolve independently of the underlying graphics API.
pub struct Gpu {
  inner: platform::gpu::Gpu,
  limits: GpuLimits,
}

impl Gpu {
  /// Create a new high-level GPU from a platform GPU.
  fn from_platform(gpu: platform::gpu::Gpu) -> Self {
    let limits = GpuLimits::from_platform(gpu.limits());
    return Gpu { inner: gpu, limits };
  }

  /// Borrow the underlying platform GPU for internal use.
  ///
  /// This is crate-visible to allow resource builders and other internal
  /// code to access the platform device without exposing it publicly.
  #[inline]
  pub(crate) fn platform(&self) -> &platform::gpu::Gpu {
    return &self.inner;
  }

  /// Query the device limits.
  #[inline]
  pub fn limits(&self) -> &GpuLimits {
    return &self.limits;
  }

  /// Submit command buffers to the GPU queue.
  ///
  /// The submitted buffers are executed in order. This method does not block;
  /// use fences or map callbacks for synchronization.
  #[inline]
  pub fn submit<I>(&self, buffers: I)
  where
    I: IntoIterator<Item = platform::command::CommandBuffer>,
  {
    self.inner.submit(buffers);
  }

  /// Check if the GPU supports the given sample count for a texture format.
  ///
  /// Returns `true` if the format can be used as a render attachment with
  /// the specified sample count for MSAA.
  pub fn supports_sample_count_for_format(
    &self,
    format: TextureFormat,
    sample_count: u32,
  ) -> bool {
    return self
      .inner
      .supports_sample_count_for_format(format.to_platform(), sample_count);
  }

  /// Check if the GPU supports the given sample count for a depth format.
  ///
  /// Returns `true` if the depth format can be used as a depth/stencil
  /// attachment with the specified sample count for MSAA.
  pub fn supports_sample_count_for_depth(
    &self,
    format: DepthFormat,
    sample_count: u32,
  ) -> bool {
    return self
      .inner
      .supports_sample_count_for_depth(format.to_platform(), sample_count);
  }

  /// Maximum bytes that can be bound for a single uniform buffer binding.
  #[inline]
  pub fn limit_max_uniform_buffer_binding_size(&self) -> u64 {
    return self.limits.max_uniform_buffer_binding_size;
  }

  /// Number of bind groups that can be used by a pipeline layout.
  #[inline]
  pub fn limit_max_bind_groups(&self) -> u32 {
    return self.limits.max_bind_groups;
  }

  /// Maximum number of vertex buffers that can be bound.
  #[inline]
  pub fn limit_max_vertex_buffers(&self) -> u32 {
    return self.limits.max_vertex_buffers;
  }

  /// Maximum number of vertex attributes that can be declared.
  #[inline]
  pub fn limit_max_vertex_attributes(&self) -> u32 {
    return self.limits.max_vertex_attributes;
  }

  /// Required alignment in bytes for dynamic uniform buffer offsets.
  #[inline]
  pub fn limit_min_uniform_buffer_offset_alignment(&self) -> u32 {
    return self.limits.min_uniform_buffer_offset_alignment;
  }
}

impl std::fmt::Debug for Gpu {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    return f
      .debug_struct("Gpu")
      .field("limits", &self.limits)
      .finish_non_exhaustive();
  }
}

// ---------------------------------------------------------------------------
// GpuBuilder
// ---------------------------------------------------------------------------

/// Builder for creating a `Gpu` with configurable options.
///
/// The builder configures adapter selection, required features, and memory
/// hints before requesting the logical device.
pub struct GpuBuilder {
  inner: platform::gpu::GpuBuilder,
}

impl GpuBuilder {
  /// Create a new builder with default settings.
  ///
  /// Defaults:
  /// - High performance power preference
  /// - Immediates enabled
  /// - Performance-oriented memory hints
  pub fn new() -> Self {
    return GpuBuilder {
      inner: platform::gpu::GpuBuilder::new(),
    };
  }

  /// Attach a label for debugging and profiling.
  pub fn with_label(mut self, label: &str) -> Self {
    self.inner = self.inner.with_label(label);
    return self;
  }

  /// Build the GPU using the provided instance and optional surface.
  ///
  /// The surface is used to ensure the adapter is compatible with
  /// presentation. Pass `None` for headless/compute-only contexts.
  pub fn build(
    self,
    instance: &Instance,
    surface: Option<&WindowSurface>,
  ) -> Result<Gpu, GpuBuildError> {
    let platform_surface = surface.map(|s| s.platform());
    let platform_gpu = self
      .inner
      .build(instance.platform(), platform_surface)
      .map_err(GpuBuildError::from_platform)?;
    return Ok(Gpu::from_platform(platform_gpu));
  }
}

impl Default for GpuBuilder {
  fn default() -> Self {
    return Self::new();
  }
}

// ---------------------------------------------------------------------------
// GpuBuildError
// ---------------------------------------------------------------------------

/// Errors that can occur when building a `Gpu`.
#[derive(Debug)]
pub enum GpuBuildError {
  /// No compatible GPU adapter was found.
  AdapterUnavailable,
  /// Required features are not supported by the adapter.
  MissingFeatures(String),
  /// Device creation failed.
  DeviceCreationFailed(String),
}

impl GpuBuildError {
  fn from_platform(error: platform::gpu::GpuBuildError) -> Self {
    return match error {
      platform::gpu::GpuBuildError::AdapterUnavailable => {
        GpuBuildError::AdapterUnavailable
      }
      platform::gpu::GpuBuildError::MissingFeatures {
        requested,
        available,
      } => GpuBuildError::MissingFeatures(format!(
        "Requested features {:?} not available; adapter supports {:?}",
        requested, available
      )),
      platform::gpu::GpuBuildError::RequestDevice(msg) => {
        GpuBuildError::DeviceCreationFailed(msg)
      }
    };
  }
}

impl std::fmt::Display for GpuBuildError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    return match self {
      GpuBuildError::AdapterUnavailable => {
        write!(f, "No compatible GPU adapter found")
      }
      GpuBuildError::MissingFeatures(msg) => write!(f, "{}", msg),
      GpuBuildError::DeviceCreationFailed(msg) => {
        write!(f, "Device creation failed: {}", msg)
      }
    };
  }
}

impl std::error::Error for GpuBuildError {}
