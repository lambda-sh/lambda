//! High-level graphics instance abstraction.
//!
//! The `Instance` type wraps the platform instance, providing a stable
//! engine-facing API for instance creation and configuration.
//!
//! # Usage
//!
//! Create an instance using the builder pattern:
//!
//! ```ignore
//! let instance = InstanceBuilder::new()
//!   .with_label("My Application")
//!   .with_backends(Backends::PRIMARY)
//!   .build();
//! ```
//!
//! The instance is then used to create surfaces and GPUs:
//!
//! ```ignore
//! let surface = WindowSurface::new(&instance, &window)?;
//! let gpu = GpuBuilder::new().build(&instance, Some(&surface))?;
//! ```

use lambda_platform::wgpu as platform;

// ---------------------------------------------------------------------------
// Backends
// ---------------------------------------------------------------------------

/// Graphics API backends available for rendering.
///
/// This type mirrors the platform `Backends` bitset, exposing the same
/// options without leaking `wgpu` types to the engine layer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Backends(platform::instance::Backends);

impl Backends {
  /// Primary desktop backends (Vulkan/Metal/DX12).
  ///
  /// This is the recommended default for cross-platform desktop applications.
  pub const PRIMARY: Backends = Backends(platform::instance::Backends::PRIMARY);

  /// Vulkan backend (Linux, Windows, Android).
  pub const VULKAN: Backends = Backends(platform::instance::Backends::VULKAN);

  /// Metal backend (macOS, iOS).
  pub const METAL: Backends = Backends(platform::instance::Backends::METAL);

  /// DirectX 12 backend (Windows).
  pub const DX12: Backends = Backends(platform::instance::Backends::DX12);

  /// OpenGL / WebGL backend.
  pub const GL: Backends = Backends(platform::instance::Backends::GL);

  /// Browser WebGPU backend.
  pub const BROWSER_WEBGPU: Backends =
    Backends(platform::instance::Backends::BROWSER_WEBGPU);

  /// Convert to the platform representation for internal use.
  #[inline]
  pub(crate) fn to_platform(self) -> platform::instance::Backends {
    return self.0;
  }
}

impl Default for Backends {
  fn default() -> Self {
    return Backends::PRIMARY;
  }
}

impl std::ops::BitOr for Backends {
  type Output = Backends;

  fn bitor(self, rhs: Backends) -> Backends {
    return Backends(self.0 | rhs.0);
  }
}

// ---------------------------------------------------------------------------
// InstanceFlags
// ---------------------------------------------------------------------------

/// Configuration flags for instance creation.
///
/// These flags control debugging and validation behavior at the instance
/// level.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InstanceFlags(platform::instance::InstanceFlags);

impl InstanceFlags {
  /// Enable validation layers for debugging.
  ///
  /// This enables GPU validation which can help catch errors but has
  /// performance overhead. Recommended for development builds.
  pub const VALIDATION: InstanceFlags =
    InstanceFlags(platform::instance::InstanceFlags::VALIDATION);

  /// Enable additional debugging features.
  ///
  /// This enables extra debugging information where supported by the
  /// graphics backend.
  pub const DEBUG: InstanceFlags =
    InstanceFlags(platform::instance::InstanceFlags::DEBUG);

  /// Convert to the platform representation for internal use.
  #[inline]
  pub(crate) fn to_platform(self) -> platform::instance::InstanceFlags {
    return self.0;
  }
}

impl Default for InstanceFlags {
  fn default() -> Self {
    return InstanceFlags(platform::instance::InstanceFlags::default());
  }
}

impl std::ops::BitOr for InstanceFlags {
  type Output = InstanceFlags;

  fn bitor(self, rhs: InstanceFlags) -> InstanceFlags {
    return InstanceFlags(self.0 | rhs.0);
  }
}

// ---------------------------------------------------------------------------
// Dx12Compiler
// ---------------------------------------------------------------------------

/// DirectX 12 shader compiler selection (Windows only).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Dx12Compiler {
  /// Use the FXC compiler (legacy, broadly compatible).
  #[default]
  Fxc,
}

impl Dx12Compiler {
  /// Convert to the platform representation for internal use.
  #[inline]
  pub(crate) fn to_platform(self) -> platform::instance::Dx12Compiler {
    return match self {
      Dx12Compiler::Fxc => platform::instance::Dx12Compiler::Fxc,
    };
  }
}

// ---------------------------------------------------------------------------
// Gles3MinorVersion
// ---------------------------------------------------------------------------

/// OpenGL ES 3.x minor version selection.
///
/// Used for WebGL and OpenGL ES targets to specify the required minor
/// version of the OpenGL ES 3.x API.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Gles3MinorVersion {
  /// Let the platform select an appropriate version.
  #[default]
  Automatic,
  /// OpenGL ES 3.0
  Version0,
  /// OpenGL ES 3.1
  Version1,
  /// OpenGL ES 3.2
  Version2,
}

impl Gles3MinorVersion {
  /// Convert to the platform representation for internal use.
  #[inline]
  pub(crate) fn to_platform(self) -> platform::instance::Gles3MinorVersion {
    return match self {
      Gles3MinorVersion::Automatic => {
        platform::instance::Gles3MinorVersion::Automatic
      }
      Gles3MinorVersion::Version0 => {
        platform::instance::Gles3MinorVersion::Version0
      }
      Gles3MinorVersion::Version1 => {
        platform::instance::Gles3MinorVersion::Version1
      }
      Gles3MinorVersion::Version2 => {
        platform::instance::Gles3MinorVersion::Version2
      }
    };
  }
}

// ---------------------------------------------------------------------------
// Instance
// ---------------------------------------------------------------------------

/// High-level graphics instance.
///
/// The instance is the root object for the graphics subsystem. It manages
/// the connection to the graphics backend and is used to create surfaces
/// and enumerate adapters.
///
/// Create an instance using `InstanceBuilder`:
///
/// ```ignore
/// let instance = InstanceBuilder::new()
///   .with_label("My Application")
///   .build();
/// ```
pub struct Instance {
  inner: platform::instance::Instance,
}

impl Instance {
  /// Return the optional label attached at construction time.
  #[inline]
  pub fn label(&self) -> Option<&str> {
    return self.inner.label();
  }

  /// Borrow the underlying platform instance for internal use.
  ///
  /// This is crate-visible to allow surfaces and GPUs to access the
  /// platform instance without exposing it publicly.
  #[inline]
  pub(crate) fn platform(&self) -> &platform::instance::Instance {
    return &self.inner;
  }
}

impl std::fmt::Debug for Instance {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    return f
      .debug_struct("Instance")
      .field("label", &self.inner.label())
      .finish_non_exhaustive();
  }
}

// ---------------------------------------------------------------------------
// InstanceBuilder
// ---------------------------------------------------------------------------

/// Builder for creating a graphics `Instance` with configurable options.
///
/// The builder provides a fluent interface for configuring instance
/// creation parameters. All options have sensible defaults for desktop
/// applications.
///
/// # Defaults
///
/// - **Backends**: `Backends::PRIMARY` (Vulkan/Metal/DX12)
/// - **Flags**: Platform defaults (no validation)
///
/// # Example
///
/// ```ignore
/// let instance = InstanceBuilder::new()
///   .with_label("My Application Instance")
///   .with_backends(Backends::PRIMARY)
///   .with_flags(InstanceFlags::VALIDATION)
///   .build();
/// ```
pub struct InstanceBuilder {
  inner: platform::instance::InstanceBuilder,
}

impl InstanceBuilder {
  /// Create a new builder with default settings.
  pub fn new() -> Self {
    return InstanceBuilder {
      inner: platform::instance::InstanceBuilder::new(),
    };
  }

  /// Attach a debug label to the instance.
  ///
  /// Labels appear in debug output and profiling tools, making it easier
  /// to identify resources during development.
  pub fn with_label(mut self, label: &str) -> Self {
    self.inner = self.inner.with_label(label);
    return self;
  }

  /// Select which graphics backends to enable.
  ///
  /// Multiple backends can be combined using the `|` operator. The runtime
  /// will select the best available backend from the enabled set.
  ///
  /// # Example
  ///
  /// ```ignore
  /// // Enable Vulkan and Metal
  /// builder.with_backends(Backends::VULKAN | Backends::METAL)
  /// ```
  pub fn with_backends(mut self, backends: Backends) -> Self {
    self.inner = self.inner.with_backends(backends.to_platform());
    return self;
  }

  /// Set instance flags for debugging and validation.
  ///
  /// Validation is recommended during development but has performance
  /// overhead and should be disabled in release builds.
  pub fn with_flags(mut self, flags: InstanceFlags) -> Self {
    self.inner = self.inner.with_flags(flags.to_platform());
    return self;
  }

  /// Choose a DX12 shader compiler variant (Windows only).
  ///
  /// This option only affects DirectX 12 backends on Windows.
  pub fn with_dx12_shader_compiler(mut self, compiler: Dx12Compiler) -> Self {
    self.inner = self.inner.with_dx12_shader_compiler(compiler.to_platform());
    return self;
  }

  /// Configure the GLES minor version for WebGL/OpenGL ES targets.
  ///
  /// This option only affects OpenGL and WebGL backends.
  pub fn with_gles_minor_version(mut self, version: Gles3MinorVersion) -> Self {
    self.inner = self.inner.with_gles_minor_version(version.to_platform());
    return self;
  }

  /// Build the `Instance` from the accumulated options.
  pub fn build(self) -> Instance {
    return Instance {
      inner: self.inner.build(),
    };
  }
}

impl Default for InstanceBuilder {
  fn default() -> Self {
    return Self::new();
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn instance_builder_sets_label() {
    let instance = InstanceBuilder::new().with_label("Test Instance").build();
    assert_eq!(instance.label(), Some("Test Instance"));
  }

  #[test]
  fn instance_builder_default_backends() {
    // Just ensure we can build with defaults without panicking
    let _instance = InstanceBuilder::new().build();
  }

  #[test]
  fn backends_bitor() {
    let combined = Backends::VULKAN | Backends::METAL;
    // Verify the operation doesn't panic and produces a valid result
    assert_ne!(combined, Backends::VULKAN);
    assert_ne!(combined, Backends::METAL);
  }

  #[test]
  fn instance_flags_bitor_maps_to_platform() {
    let flags = InstanceFlags::VALIDATION | InstanceFlags::DEBUG;
    let platform = flags.to_platform();
    assert_eq!(
      platform,
      platform::instance::InstanceFlags::VALIDATION
        | platform::instance::InstanceFlags::DEBUG
    );
  }

  #[test]
  fn dx12_compiler_maps_to_platform() {
    assert!(matches!(
      Dx12Compiler::Fxc.to_platform(),
      platform::instance::Dx12Compiler::Fxc
    ));
  }

  #[test]
  fn gles_minor_version_maps_to_platform() {
    assert!(matches!(
      Gles3MinorVersion::Automatic.to_platform(),
      platform::instance::Gles3MinorVersion::Automatic
    ));
    assert!(matches!(
      Gles3MinorVersion::Version0.to_platform(),
      platform::instance::Gles3MinorVersion::Version0
    ));
    assert!(matches!(
      Gles3MinorVersion::Version1.to_platform(),
      platform::instance::Gles3MinorVersion::Version1
    ));
    assert!(matches!(
      Gles3MinorVersion::Version2.to_platform(),
      platform::instance::Gles3MinorVersion::Version2
    ));
  }

  #[test]
  fn instance_debug_includes_label_field() {
    let instance = InstanceBuilder::new().with_label("debug instance").build();
    let formatted = format!("{:?}", instance);
    assert!(formatted.contains("Instance"));
    assert!(formatted.contains("label"));
  }

  #[test]
  fn backends_map_to_platform_constants() {
    assert_eq!(
      Backends::PRIMARY.to_platform(),
      platform::instance::Backends::PRIMARY
    );
    assert_eq!(
      Backends::VULKAN.to_platform(),
      platform::instance::Backends::VULKAN
    );
    assert_eq!(
      Backends::METAL.to_platform(),
      platform::instance::Backends::METAL
    );
    assert_eq!(
      Backends::DX12.to_platform(),
      platform::instance::Backends::DX12
    );
    assert_eq!(Backends::GL.to_platform(), platform::instance::Backends::GL);
  }

  #[test]
  fn instance_builder_accepts_all_options() {
    let _instance = InstanceBuilder::new()
      .with_label("options")
      .with_backends(Backends::VULKAN | Backends::METAL)
      .with_flags(InstanceFlags::VALIDATION | InstanceFlags::DEBUG)
      .with_dx12_shader_compiler(Dx12Compiler::Fxc)
      .with_gles_minor_version(Gles3MinorVersion::Version2)
      .build();
  }
}
