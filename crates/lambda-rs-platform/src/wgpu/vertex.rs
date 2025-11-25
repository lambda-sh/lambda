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

/// Step mode applied to a vertex buffer layout.
///
/// `Vertex` advances attributes per vertex; `Instance` advances attributes per
/// instance. This mirrors `wgpu::VertexStepMode` without exposing the raw
/// dependency to higher layers.
#[derive(Clone, Copy, Debug)]
pub enum VertexStepMode {
  Vertex,
  Instance,
}

impl VertexStepMode {
  /// Map the engine step mode to the underlying graphics API.
  pub(crate) fn to_wgpu(self) -> wgpu::VertexStepMode {
    return match self {
      VertexStepMode::Vertex => wgpu::VertexStepMode::Vertex,
      VertexStepMode::Instance => wgpu::VertexStepMode::Instance,
    };
  }
}
