#![allow(clippy::needless_return)]

// Integration tests for `lambda-rs-platform::wgpu::texture`

fn create_test_device() -> lambda_platform::wgpu::gpu::Gpu {
  let instance = lambda_platform::wgpu::instance::InstanceBuilder::new()
    .with_label("platform-itest")
    .build();
  return lambda_platform::wgpu::gpu::GpuBuilder::new()
    .with_label("platform-itest-device")
    .build(&instance, None)
    .expect("create offscreen device");
}

#[test]
fn wgpu_texture_build_and_upload_succeeds() {
  let gpu = create_test_device();

  let (w, h) = (8u32, 8u32);
  let mut pixels = vec![0u8; (w * h * 4) as usize];
  for y in 0..h {
    for x in 0..w {
      let idx = ((y * w + x) * 4) as usize;
      let c = if ((x + y) % 2) == 0 { 255 } else { 0 };
      pixels[idx + 0] = c;
      pixels[idx + 1] = c;
      pixels[idx + 2] = c;
      pixels[idx + 3] = 255;
    }
  }

  let _texture = lambda_platform::wgpu::texture::TextureBuilder::new_2d(
    lambda_platform::wgpu::texture::TextureFormat::RGBA8_UNORM_SRGB,
  )
  .with_size(w, h)
  .with_data(&pixels)
  .with_label("p-itest-texture")
  .build(&gpu)
  .expect("texture created");
}

#[test]
fn wgpu_texture_upload_with_padding_bytes_per_row() {
  let gpu = create_test_device();

  let (w, h) = (13u32, 7u32);
  let pixels = vec![128u8; (w * h * 4) as usize];
  let _ = lambda_platform::wgpu::texture::TextureBuilder::new_2d(
    lambda_platform::wgpu::texture::TextureFormat::RGBA8_UNORM,
  )
  .with_size(w, h)
  .with_data(&pixels)
  .with_label("p-itest-pad")
  .build(&gpu)
  .expect("padded write_texture works");
}
