//! Vertex data structures.

use lambda_platform::wgpu::types as wgpu;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// Canonical color/attribute formats used by engine pipelines.
pub enum ColorFormat {
  Rgb32Sfloat,
  Rgba8Srgb,
}

impl ColorFormat {
  pub(crate) fn to_texture_format(self) -> wgpu::TextureFormat {
    match self {
      ColorFormat::Rgb32Sfloat => wgpu::TextureFormat::Rgba32Float,
      ColorFormat::Rgba8Srgb => wgpu::TextureFormat::Rgba8UnormSrgb,
    }
  }

  pub(crate) fn to_vertex_format(self) -> wgpu::VertexFormat {
    match self {
      ColorFormat::Rgb32Sfloat => wgpu::VertexFormat::Float32x3,
      ColorFormat::Rgba8Srgb => wgpu::VertexFormat::Unorm8x4,
    }
  }
}

#[derive(Clone, Copy, Debug)]
/// A single vertex element (format + byte offset).
pub struct VertexElement {
  pub format: ColorFormat,
  pub offset: u32,
}

#[derive(Clone, Copy, Debug)]
/// Vertex attribute bound to a shader `location` plus relative offsets.
pub struct VertexAttribute {
  pub location: u32,
  pub offset: u32,
  pub element: VertexElement,
}

/// Vertex data structure with position, normal, and color.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Vertex {
  pub position: [f32; 3],
  pub normal: [f32; 3],
  pub color: [f32; 3],
}

/// Builder for constructing a `Vertex` instance incrementally.
#[derive(Clone, Copy, Debug)]
pub struct VertexBuilder {
  pub position: [f32; 3],
  pub normal: [f32; 3],
  pub color: [f32; 3],
}

impl VertexBuilder {
  /// Creates a new vertex builder.
  pub fn new() -> Self {
    Self {
      position: [0.0, 0.0, 0.0],
      normal: [0.0, 0.0, 0.0],
      color: [0.0, 0.0, 0.0],
    }
  }

  /// Set the position of the vertex.
  pub fn with_position(&mut self, position: [f32; 3]) -> &mut Self {
    self.position = position;
    self
  }

  /// Set the normal of the vertex.
  pub fn with_normal(&mut self, normal: [f32; 3]) -> &mut Self {
    self.normal = normal;
    self
  }

  /// Set the color of the vertex.
  pub fn with_color(&mut self, color: [f32; 3]) -> &mut Self {
    self.color = color;
    self
  }

  /// Build the vertex.
  pub fn build(&self) -> Vertex {
    Vertex {
      position: self.position,
      normal: self.normal,
      color: self.color,
    }
  }
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn vertex_building() {
    let mut vertex = VertexBuilder::new();

    assert_eq!(vertex.position, [0.0, 0.0, 0.0]);
    assert_eq!(vertex.normal, [0.0, 0.0, 0.0]);
    assert_eq!(vertex.color, [0.0, 0.0, 0.0]);

    let vertex = vertex
      .with_position([1.0, 2.0, 3.0])
      .with_normal([4.0, 5.0, 6.0])
      .with_color([7.0, 8.0, 9.0])
      .build();

    assert_eq!(vertex.position, [1.0, 2.0, 3.0]);
    assert_eq!(vertex.normal, [4.0, 5.0, 6.0]);
    assert_eq!(vertex.color, [7.0, 8.0, 9.0]);
  }
}
