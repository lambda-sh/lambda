//! Vertex data structures.

pub use lambda_platform::gfx::assembler::{
  VertexAttribute,
  VertexElement,
};

/// Vertex data structure with position, normal, and color.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Vertex {
  pub position: [f32; 3],
  pub normal: [f32; 3],
  pub color: [f32; 3],
}

/// Construction for
#[derive(Clone, Copy, Debug)]
pub struct VertexBuilder {
  pub position: [f32; 3],
  pub normal: [f32; 3],
  pub color: [f32; 3],
}

impl VertexBuilder {
  /// Creates a new vertex builder.
  pub fn new() -> Self {
    return Self {
      position: [0.0, 0.0, 0.0],
      normal: [0.0, 0.0, 0.0],
      color: [0.0, 0.0, 0.0],
    };
  }

  /// Set the position of the vertex.
  pub fn with_position(&mut self, position: [f32; 3]) -> &mut Self {
    self.position = position;
    return self;
  }

  /// Set the normal of the vertex.
  pub fn with_normal(&mut self, normal: [f32; 3]) -> &mut Self {
    self.normal = normal;
    return self;
  }

  /// Set the color of the vertex.
  pub fn with_color(&mut self, color: [f32; 3]) -> &mut Self {
    self.color = color;
    return self;
  }

  /// Build the vertex.
  pub fn build(&self) -> Vertex {
    return Vertex {
      position: self.position,
      normal: self.normal,
      color: self.color,
    };
  }
}

#[cfg(test)]
mod test {
  #[test]
  fn vertex_building() {
    let mut vertex = super::VertexBuilder::new();

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
