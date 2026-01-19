#![allow(clippy::needless_return)]

// Bind group layout and group test for 3D texture dimension

fn create_test_device() -> Option<lambda_platform::wgpu::gpu::Gpu> {
  let instance = lambda_platform::wgpu::instance::InstanceBuilder::new()
    .with_label("p-itest-3d-bind")
    .build();
  let result = lambda_platform::wgpu::gpu::GpuBuilder::new()
    .with_label("p-itest-3d-bind-device")
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
fn wgpu_bind_layout_dim3_and_group() {
  let Some(gpu) = create_test_device() else {
    return;
  };

  let (w, h, d) = (2u32, 2u32, 2u32);
  let pixels = vec![255u8; (w * h * d * 4) as usize];
  let tex3d = lambda_platform::wgpu::texture::TextureBuilder::new_3d(
    lambda_platform::wgpu::texture::TextureFormat::RGBA8_UNORM,
  )
  .with_size_3d(w, h, d)
  .with_data(&pixels)
  .with_label("p-itest-3d-view")
  .build(&gpu)
  .expect("3D texture build");

  let sampler = lambda_platform::wgpu::texture::SamplerBuilder::new()
    .nearest_clamp()
    .build(&gpu);

  let layout = lambda_platform::wgpu::bind::BindGroupLayoutBuilder::new()
    .with_sampled_texture_dim(
      1,
      lambda_platform::wgpu::bind::Visibility::Fragment,
      lambda_platform::wgpu::texture::ViewDimension::ThreeDimensional,
    )
    .with_sampler(2, lambda_platform::wgpu::bind::Visibility::Fragment)
    .build(&gpu);

  let _group = lambda_platform::wgpu::bind::BindGroupBuilder::new()
    .with_layout(&layout)
    .with_texture(1, &tex3d)
    .with_sampler(2, &sampler)
    .build(&gpu);
}
