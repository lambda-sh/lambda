use gfx_hal::device::Device;

use super::gpu;

pub mod internal {
  use super::ShaderModule;
  #[inline]
  pub fn module_for<RenderBackend: gfx_hal::Backend>(
    shader_module: ShaderModule<RenderBackend>,
  ) -> RenderBackend::ShaderModule {
    return shader_module.shader_module;
  }
}

pub struct ShaderModuleBuilder {}

impl ShaderModuleBuilder {
  pub fn new() -> Self {
    return Self {};
  }

  pub fn build<RenderBackend: gfx_hal::Backend>(
    self,
    gpu: &mut gpu::Gpu<RenderBackend>,
    shader_binary: &Vec<u32>,
  ) -> ShaderModule<RenderBackend> {
    let shader_module = unsafe {
      gpu::internal::logical_device_for(gpu)
        .create_shader_module(&shader_binary)
        .expect("Failed to create a shader module.")
    };

    return ShaderModule { shader_module };
  }
}

pub struct ShaderModule<RenderBackend: gfx_hal::Backend> {
  shader_module: RenderBackend::ShaderModule,
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
}
