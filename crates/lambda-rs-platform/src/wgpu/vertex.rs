//! Vertex color/attribute formats used by the platform layer.
//!
//! These map directly to `wgpu` texture/vertex formats and are re‑exported via
//! the high‑level rendering module. This is an internal surface that may
//! evolve with engine needs.

/// Canonical color/attribute formats used by engine pipelines.
#[derive(Clone, Copy, Debug)]
pub enum ColorFormat {
  Rgb32Sfloat,
  Rgba8Srgb,
}

impl ColorFormat {
  pub(crate) fn to_texture_format(self) -> wgpu::TextureFormat {
    return match self {
      ColorFormat::Rgb32Sfloat => wgpu::TextureFormat::Rgba32Float,
      ColorFormat::Rgba8Srgb => wgpu::TextureFormat::Rgba8UnormSrgb,
    };
  }

  pub(crate) fn to_vertex_format(self) -> wgpu::VertexFormat {
    return match self {
      ColorFormat::Rgb32Sfloat => wgpu::VertexFormat::Float32x3,
      ColorFormat::Rgba8Srgb => wgpu::VertexFormat::Unorm8x4,
    };
  }
}
