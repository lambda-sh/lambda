use gfx_hal::{
  device::Device,
  pso::Specialization as ShaderSpecializations,
};

use super::gpu;

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

pub enum ShaderModuleType {
  Vertex,
  Fragment,
  Compute,
}

/// Builder class for
pub struct ShaderModuleBuilder {
  entry_name: String,
  specializations: ShaderSpecializations<'static>,
}

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

impl<RenderBackend: gfx_hal::Backend> ShaderModule<RenderBackend> {
  pub fn destroy(self, gpu: &mut gpu::Gpu<RenderBackend>) {
    // TODO(vmarcella): Add documentation for the shader module.
    println!("Destroying shader module.");
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
  pub fn specializations(&self) -> &ShaderSpecializations {
    return &self.specializations;
  }
}