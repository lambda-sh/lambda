#![allow(clippy::needless_return)]

// Integration tests for 3D textures in the platform layer

fn create_test_device() -> Option<lambda_platform::wgpu::gpu::Gpu> {
  let instance = lambda_platform::wgpu::instance::InstanceBuilder::new()
    .with_label("p-itest-3d")
    .build();
  let result = lambda_platform::wgpu::gpu::GpuBuilder::new()
    .with_label("p-itest-3d-device")
    .build(&instance, None);

  match result {
    Ok(gpu) => return Some(gpu),
    Err(lambda_platform::wgpu::gpu::GpuBuildError::AdapterUnavailable) => {
      return None;
    }
    Err(err) => panic!("create device: {:?}", err),
  }
}

#[test]
fn wgpu_texture3d_build_and_upload() {
  let Some(gpu) = create_test_device() else {
    return;
  };

  let (w, h, d) = (4u32, 4u32, 3u32);
  let pixels = vec![180u8; (w * h * d * 4) as usize];

  let _tex3d = lambda_platform::wgpu::texture::TextureBuilder::new_3d(
    lambda_platform::wgpu::texture::TextureFormat::RGBA8_UNORM,
  )
  .with_size_3d(w, h, d)
  .with_data(&pixels)
  .with_label("p-itest-3d-texture")
  .build(&gpu)
  .expect("3D texture build");
}
