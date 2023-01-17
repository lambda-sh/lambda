use super::vertex::Vertex;

#[derive(Clone, Debug)]
pub struct Mesh {
  vertices: Vec<Vertex>,
  indices: Vec<usize>,
}

#[derive(Clone, Debug)]
pub struct MeshBuilder {
  size: usize,
  vertices: Vec<Vertex>,
  indices: Vec<usize>,
}

impl MeshBuilder {
  pub fn new() -> Self {
    return Self {
      size: 0,
      vertices: vec![],
      indices: vec![],
    };
  }

  pub fn with_capacity(&mut self, size: usize) -> &mut Self {
    self.size = size;
    return self;
  }

  pub fn with_vertex(&mut self, vertex: Vertex) -> &mut Self {
    self.vertices.push(vertex);
    return self;
  }

  pub fn build(&self) -> Mesh {
    return Mesh {
      vertices: self.vertices.clone(),
      indices: self.indices.clone(),
    };
  }
}

#[cfg(test)]
mod tests {
  #[test]
  fn mesh_building() {
    let mut mesh = super::MeshBuilder::new();

    assert_eq!(mesh.vertices.len(), 0);
    assert_eq!(mesh.indices.len(), 0);

    let mut mesh = mesh
      .with_capacity(10)
      .with_vertex(crate::core::render::vertex::VertexBuilder::new().build())
      .build();

    assert_eq!(mesh.vertices.len(), 1);
    assert_eq!(mesh.indices.len(), 0);
    assert_eq!(mesh.vertices[0].position, [0.0, 0.0, 0.0]);
    assert_eq!(mesh.vertices[0].normal, [0.0, 0.0, 0.0]);
    assert_eq!(mesh.vertices[0].color, [0.0, 0.0, 0.0]);
  }
}
