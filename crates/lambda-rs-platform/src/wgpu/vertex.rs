/// Canonical color/attribute formats used by engine pipelines.
#[derive(Clone, Copy, Debug)]
pub enum ColorFormat {
  Rgb32Sfloat,
  Rgba8Srgb,
}

impl ColorFormat {
  pub fn to_texture_format(self) -> wgpu::TextureFormat {
    return match self {
      ColorFormat::Rgb32Sfloat => wgpu::TextureFormat::Rgba32Float,
      ColorFormat::Rgba8Srgb => wgpu::TextureFormat::Rgba8UnormSrgb,
    };
  }

  pub fn to_vertex_format(self) -> wgpu::VertexFormat {
    return match self {
      ColorFormat::Rgb32Sfloat => wgpu::VertexFormat::Float32x3,
      ColorFormat::Rgba8Srgb => wgpu::VertexFormat::Unorm8x4,
    };
  }
}
