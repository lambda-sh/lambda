//! Low level shader implementations used by the lambda-platform crate to load
//! SPIR-V compiled shaders into the GPU.

use gfx_hal::{
  device::Device,
  pso::Specialization as ShaderSpecializations,
};
#[cfg(test)]
use mockall::automock;

use super::gpu;

/// The type of shader that a shader module represents. Different shader types
/// are used for different operations in the rendering pipeline.
pub enum ShaderModuleType {
  Vertex,
  Fragment,
  Compute,
}

/// Builder class for creating a shader module.
pub struct ShaderModuleBuilder {
  entry_name: String,
  specializations: ShaderSpecializations<'static>,
}

#[cfg_attr(test, automock)]
impl ShaderModuleBuilder {
  pub fn new() -> Self {
    return Self {
      entry_name: "main".to_string(),
      specializations: ShaderSpecializations::default(),
    };
  }

  /// Define the shader entry point (Defaults to main)
  pub fn with_entry_name(mut self, entry_name: &str) -> Self {
    self.entry_name = entry_name.to_string();
    return self;
  }

  /// Attach specializations to the shader.
  pub fn with_specializations(
    mut self,
    specializations: ShaderSpecializations<'static>,
  ) -> Self {
    self.specializations = specializations;
    return self;
  }

  /// Builds the shader binary into a shader module located on the GPU.
  /// ShaderModules are specific to gfx-hal and can be used for building
  /// RenderPipelines
  pub fn build<RenderBackend: gfx_hal::Backend>(
    self,
    gpu: &mut gpu::Gpu<RenderBackend>,
    shader_binary: &Vec<u32>,
    shader_type: ShaderModuleType,
  ) -> ShaderModule<RenderBackend> {
    let shader_module = unsafe {
      gpu::internal::logical_device_for(gpu)
        .create_shader_module(&shader_binary)
        .expect("Failed to create a shader module.")
    };

    return ShaderModule {
      entry_name: self.entry_name,
      shader_module,
      specializations: self.specializations,
      shader_type,
    };
  }
}

/// Shader modules are used for uploading shaders into the render pipeline.
pub struct ShaderModule<RenderBackend: gfx_hal::Backend> {
  entry_name: String,
  shader_module: RenderBackend::ShaderModule,
  specializations: ShaderSpecializations<'static>,
  shader_type: ShaderModuleType,
}

#[cfg_attr(test, automock)]
impl<RenderBackend: gfx_hal::Backend> ShaderModule<RenderBackend> {
  /// Destroy the shader module and free the memory on the GPU.
  pub fn destroy(self, gpu: &mut gpu::Gpu<RenderBackend>) {
    unsafe {
      gpu::internal::logical_device_for(gpu)
        .destroy_shader_module(self.shader_module)
    }
  }

  /// Get the entry point that this shader module is using.
  pub fn entry(&self) -> &str {
    return self.entry_name.as_str();
  }

  /// Get the specializations being applied to the current shader module.
  pub fn specializations(&self) -> &ShaderSpecializations<'static> {
    return &self.specializations;
  }
}

#[cfg(test)]
mod tests {

  /// Test that we can create a shader module builder and it has the correct
  /// defaults.
  #[test]
  fn shader_builder_initial_state() {
    let shader_builder = super::ShaderModuleBuilder::new();
    assert_eq!(shader_builder.entry_name, "main");
    assert_eq!(shader_builder.specializations.data.len(), 0);
  }

  /// Test that we can create a shader module builder with a custom entry point
  /// & default specializations.
  #[test]
  fn shader_builder_with_properties() {
    let shader_builder = super::ShaderModuleBuilder::new()
      .with_entry_name("test")
      .with_specializations(super::ShaderSpecializations::default());
    assert_eq!(shader_builder.entry_name, "test");
    assert_eq!(
      shader_builder.specializations.data,
      super::ShaderSpecializations::default().data
    );
  }

  #[test]
  fn shader_builder_builds_correctly() {
    let shader_builder = super::ShaderModuleBuilder::new()
      .with_entry_name("test")
      .with_specializations(super::ShaderSpecializations::default());
  }
}

/// Internal functions for the shader module. User applications most likely
/// should not use these functions directly nor should they need to.
pub mod internal {
  use super::ShaderModule;

  /// Retrieve the underlying gfx-hal shader module given the lambda-platform
  /// implemented shader module. Useful for creating gfx-hal entry points and
  /// attaching the shader to rendering pipelines.
  #[inline]
  pub fn module_for<RenderBackend: gfx_hal::Backend>(
    shader_module: &ShaderModule<RenderBackend>,
  ) -> &RenderBackend::ShaderModule {
    return &shader_module.shader_module;
  }
}
