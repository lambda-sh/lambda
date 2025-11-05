//! Instance abstractions and type wrappers to avoid leaking raw `wgpu` types
//! at higher layers. This keeps the platform crate free to evolve with `wgpu`
//! while presenting a stable surface to the engine.

use pollster::block_on;

/// Wrapper over `wgpu::Backends` as a bitset.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Backends(wgpu::Backends);

impl Backends {
  /// Primary desktop backends (Vulkan/Metal/DX12) per `wgpu` defaults.
  pub const PRIMARY: Backends = Backends(wgpu::Backends::PRIMARY);
  /// Vulkan backend.
  pub const VULKAN: Backends = Backends(wgpu::Backends::VULKAN);
  /// Metal backend (macOS/iOS).
  pub const METAL: Backends = Backends(wgpu::Backends::METAL);
  /// DirectX 12 backend (Windows).
  pub const DX12: Backends = Backends(wgpu::Backends::DX12);
  /// OpenGL / WebGL backend.
  pub const GL: Backends = Backends(wgpu::Backends::GL);
  /// Browser WebGPU backend.
  pub const BROWSER_WEBGPU: Backends = Backends(wgpu::Backends::BROWSER_WEBGPU);

  pub(crate) fn to_wgpu(self) -> wgpu::Backends {
    self.0
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

/// Wrapper over `wgpu::InstanceFlags` as a bitset.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InstanceFlags(wgpu::InstanceFlags);

impl InstanceFlags {
  /// Validation flags (debugging and validation).
  pub const VALIDATION: InstanceFlags =
    InstanceFlags(wgpu::InstanceFlags::VALIDATION);
  /// Enable additional debugging features where available.
  pub const DEBUG: InstanceFlags = InstanceFlags(wgpu::InstanceFlags::DEBUG);

  pub(crate) fn to_wgpu(self) -> wgpu::InstanceFlags {
    self.0
  }
}

impl Default for InstanceFlags {
  fn default() -> Self {
    return InstanceFlags(wgpu::InstanceFlags::default());
  }
}

impl std::ops::BitOr for InstanceFlags {
  type Output = InstanceFlags;
  fn bitor(self, rhs: InstanceFlags) -> InstanceFlags {
    return InstanceFlags(self.0 | rhs.0);
  }
}

/// Which DX12 shader compiler to use on Windows platforms.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dx12Compiler {
  Fxc,
}

impl Dx12Compiler {
  pub(crate) fn to_wgpu(self) -> wgpu::Dx12Compiler {
    return match self {
      Dx12Compiler::Fxc => wgpu::Dx12Compiler::Fxc,
    };
  }
}

/// OpenGL ES 3 minor version (used by GL/Web targets).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gles3MinorVersion {
  Automatic,
  Version0,
  Version1,
  Version2,
}

impl Gles3MinorVersion {
  pub(crate) fn to_wgpu(self) -> wgpu::Gles3MinorVersion {
    return match self {
      Gles3MinorVersion::Automatic => wgpu::Gles3MinorVersion::Automatic,
      Gles3MinorVersion::Version0 => wgpu::Gles3MinorVersion::Version0,
      Gles3MinorVersion::Version1 => wgpu::Gles3MinorVersion::Version1,
      Gles3MinorVersion::Version2 => wgpu::Gles3MinorVersion::Version2,
    };
  }
}

/// Builder for creating a `wgpu::Instance` with consistent defaults.
#[derive(Debug, Clone)]
///
/// Defaults to primary backends and no special flags. Options map to
/// `wgpu::InstanceDescriptor` internally without leaking raw types.
pub struct InstanceBuilder {
  label: Option<String>,
  backends: Backends,
  flags: InstanceFlags,
  // Keep backend options/memory thresholds internal; expose focused knobs via
  // typed methods to avoid leaking raw structs.
  backend_options: wgpu::BackendOptions,
  memory_budget_thresholds: wgpu::MemoryBudgetThresholds,
}

impl InstanceBuilder {
  /// Construct a new builder with Lambda defaults.
  pub fn new() -> Self {
    Self {
      label: None,
      backends: Backends::PRIMARY,
      flags: InstanceFlags::default(),
      backend_options: wgpu::BackendOptions::default(),
      memory_budget_thresholds: wgpu::MemoryBudgetThresholds::default(),
    }
  }

  /// Attach a debug label to the instance.
  pub fn with_label(mut self, label: &str) -> Self {
    self.label = Some(label.to_string());
    self
  }

  /// Select which graphics backends to enable.
  pub fn with_backends(mut self, backends: Backends) -> Self {
    self.backends = backends;
    self
  }

  /// Set additional instance flags (e.g., debugging/validation).
  pub fn with_flags(mut self, flags: InstanceFlags) -> Self {
    self.flags = flags;
    self
  }

  /// Choose a DX12 shader compiler variant when on Windows.
  pub fn with_dx12_shader_compiler(mut self, compiler: Dx12Compiler) -> Self {
    self.backend_options.dx12.shader_compiler = compiler.to_wgpu();
    self
  }

  /// Configure the GLES minor version for WebGL/OpenGL ES targets.
  pub fn with_gles_minor_version(mut self, version: Gles3MinorVersion) -> Self {
    self.backend_options.gl.gles_minor_version = version.to_wgpu();
    self
  }

  /// Build the `Instance` wrapper from the accumulated options.
  pub fn build(self) -> Instance {
    let descriptor = wgpu::InstanceDescriptor {
      backends: self.backends.to_wgpu(),
      flags: self.flags.to_wgpu(),
      memory_budget_thresholds: self.memory_budget_thresholds,
      backend_options: self.backend_options,
    };

    Instance {
      label: self.label,
      instance: wgpu::Instance::new(&descriptor),
    }
  }
}

#[derive(Debug)]
/// Thin wrapper over `wgpu::Instance` that preserves a user label and exposes
/// a blocking `request_adapter` convenience.
pub struct Instance {
  pub(crate) label: Option<String>,
  pub(crate) instance: wgpu::Instance,
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
  pub(crate) fn request_adapter<'surface, 'window>(
    &self,
    options: &wgpu::RequestAdapterOptions<'surface, 'window>,
  ) -> Result<wgpu::Adapter, wgpu::RequestAdapterError> {
    block_on(self.instance.request_adapter(options))
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
}
