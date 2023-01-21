use super::{
  vertex::{
    Vertex,
    VertexAttribute,
  },
  RenderContext,
};

// ---------------------------------- Mesh ------------------------------------

/// Collection of vertices and indices that define a 3D object.
#[derive(Debug)]
pub struct Mesh {
  vertices: Vec<Vertex>,
  attributes: Vec<VertexAttribute>,
}

impl Mesh {
  pub fn vertices(&self) -> &[Vertex] {
    &self.vertices
  }

  pub fn attributes(&self) -> &[VertexAttribute] {
    &self.attributes
  }
}

// ------------------------------ MeshBuilder ---------------------------------

/// Construction for a mesh.
#[derive(Clone, Debug)]
pub struct MeshBuilder {
  capacity: usize,
  vertices: Vec<Vertex>,
  attributes: Vec<VertexAttribute>,
}

impl MeshBuilder {
  pub fn new() -> Self {
    return Self {
      capacity: 0,
      vertices: Vec::new(),
      attributes: Vec::new(),
    };
  }

  pub fn with_capacity(&mut self, size: usize) -> &mut Self {
    self.capacity = size;
    return self;
  }

  pub fn with_vertex(&mut self, vertex: Vertex) -> &mut Self {
    self.vertices.push(vertex);
    return self;
  }

  pub fn with_attributes(
    &mut self,
    attributes: Vec<VertexAttribute>,
  ) -> &mut Self {
    self.attributes = attributes;
    return self;
  }

  /// Builds a mesh from the vertices and indices that have been added to the
  /// builder and allocates the memory for the mesh on the GPU.
  pub fn build(&self) -> Mesh {
    return Mesh {
      vertices: self.vertices.clone(),
      attributes: self.attributes.clone(),
    };
  }

  pub fn build_from_obj(&self, file_path: &str) -> Mesh {
    let obj = lambda_platform::obj::load_textured_obj_from_file(file_path);

    let mut vertices = obj
      .vertices
      .iter()
      .map(|v| {
        return Vertex {
          position: v.position,
          color: v.texture,
          normal: v.normal,
        };
      })
      .collect::<Vec<Vertex>>();

    return Mesh {
      vertices,
      attributes: vec![],
    };
  }
}

#[cfg(test)]
mod tests {
  #[test]
  fn mesh_building() {
    let mut mesh = super::MeshBuilder::new();

    assert_eq!(mesh.vertices.len(), 0);
  }
}
