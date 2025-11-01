#![allow(clippy::needless_return)]

// Integration tests for 3D textures in the platform layer

#[test]
fn wgpu_texture3d_build_and_upload() {
  let instance = lambda_platform::wgpu::InstanceBuilder::new()
    .with_label("p-itest-3d")
    .build();
  let gpu = lambda_platform::wgpu::GpuBuilder::new()
    .with_label("p-itest-3d-device")
    .build(&instance, None)
    .expect("create device");
  let device = gpu.device();
  let queue = gpu.queue();

  let (w, h, d) = (4u32, 4u32, 3u32);
  let pixels = vec![180u8; (w * h * d * 4) as usize];

  let _tex3d = lambda_platform::wgpu::texture::TextureBuilder::new_3d(
    lambda_platform::wgpu::texture::TextureFormat::Rgba8Unorm,
  )
  .with_size_3d(w, h, d)
  .with_data(&pixels)
  .with_label("p-itest-3d-texture")
  .build(device, queue)
  .expect("3D texture build");
}
