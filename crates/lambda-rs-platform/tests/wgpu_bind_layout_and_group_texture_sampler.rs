#![allow(clippy::needless_return)]

// Integration tests for `lambda-rs-platform::wgpu::bind` with textures/samplers

fn create_test_device() -> Option<lambda_platform::wgpu::gpu::Gpu> {
  let instance = lambda_platform::wgpu::instance::InstanceBuilder::new()
    .with_label("platform-bind-itest")
    .build();
  let result = lambda_platform::wgpu::gpu::GpuBuilder::new()
    .with_label("platform-bind-itest-device")
    .build(&instance, None);

  match result {
    Ok(gpu) => return Some(gpu),
    Err(lambda_platform::wgpu::gpu::GpuBuildError::AdapterUnavailable) => {
      return None;
    }
    Err(err) => panic!("create offscreen device: {:?}", err),
  }
}

#[test]
fn wgpu_bind_layout_and_group_texture_sampler() {
  let Some(gpu) = create_test_device() else {
    return;
  };

  let (w, h) = (4u32, 4u32);
  let pixels = vec![255u8; (w * h * 4) as usize];
  let texture = lambda_platform::wgpu::texture::TextureBuilder::new_2d(
    lambda_platform::wgpu::texture::TextureFormat::RGBA8_UNORM,
  )
  .with_size(w, h)
  .with_data(&pixels)
  .with_label("p-itest-bind-texture")
  .build(&gpu)
  .expect("texture created");

  let sampler = lambda_platform::wgpu::texture::SamplerBuilder::new()
    .nearest_clamp()
    .with_label("p-itest-bind-sampler")
    .build(&gpu);

  let layout = lambda_platform::wgpu::bind::BindGroupLayoutBuilder::new()
    .with_sampled_texture_2d(
      1,
      lambda_platform::wgpu::bind::Visibility::Fragment,
    )
    .with_sampler(2, lambda_platform::wgpu::bind::Visibility::Fragment)
    .build(&gpu);

  let _group = lambda_platform::wgpu::bind::BindGroupBuilder::new()
    .with_layout(&layout)
    .with_texture(1, &texture)
    .with_sampler(2, &sampler)
    .build(&gpu);
}
