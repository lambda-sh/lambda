//! Vertex attribute formats and a simple `Vertex` type.
//!
//! Pipelines declare per‑buffer `VertexAttribute`s that map engine vertex
//! data into shader inputs by `location`. This module hosts common color
//! formats and a convenience `Vertex`/`VertexBuilder` used in examples.

/// Canonical color/attribute formats used by engine pipelines.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ColorFormat {
  Rgb32Sfloat,
  Rgba8Srgb,
}

impl ColorFormat {
  pub(crate) fn to_platform(
    self,
  ) -> lambda_platform::wgpu::vertex::ColorFormat {
    match self {
      ColorFormat::Rgb32Sfloat => {
        lambda_platform::wgpu::vertex::ColorFormat::Rgb32Sfloat
      }
      ColorFormat::Rgba8Srgb => {
        lambda_platform::wgpu::vertex::ColorFormat::Rgba8Srgb
      }
    }
  }
}

/// Step mode applied to a vertex buffer layout.
///
/// `PerVertex` advances attributes once per vertex; `PerInstance` advances
/// attributes once per instance. This mirrors the platform step mode without
/// exposing backend types.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VertexStepMode {
  PerVertex,
  PerInstance,
}

/// Layout for a single vertex buffer slot.
///
/// `stride` describes the size in bytes of one element in the buffer. The
/// `step_mode` field determines whether attributes sourced from this buffer
/// advance per vertex or per instance.
#[derive(Clone, Copy, Debug)]
pub struct VertexBufferLayout {
  pub stride: u64,
  pub step_mode: VertexStepMode,
}

/// A single vertex element (format + byte offset).
#[derive(Clone, Copy, Debug)]
///
/// Combine one or more elements to form a `VertexAttribute` bound at a shader
/// location. Offsets are in bytes from the start of the vertex and the element.
pub struct VertexElement {
  pub format: ColorFormat,
  pub offset: u32,
}

/// Vertex attribute bound to a shader `location` plus relative offsets.
#[derive(Clone, Copy, Debug)]
///
/// `location` MUST match the shader input. The final attribute byte offset is
/// `offset + element.offset`.
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

unsafe impl crate::pod::PlainOldData for Vertex {}

/// Builder for constructing a `Vertex` instance incrementally.
#[derive(Clone, Copy, Debug)]
pub struct VertexBuilder {
  pub position: [f32; 3],
  pub normal: [f32; 3],
  pub color: [f32; 3],
}

impl Default for VertexBuilder {
  fn default() -> Self {
    return Self::new();
  }
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

  /// Ensures `VertexBuilder` defaults and chained setters produce the expected
  /// `Vertex` output.
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
