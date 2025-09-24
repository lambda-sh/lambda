//! Mesh Implementation

use lambda_platform::obj::load_textured_obj_from_file;

use super::{
  vertex::{Vertex, VertexAttribute, VertexElement},
  ColorFormat,
};

// ---------------------------------- Mesh ------------------------------------

/// Collection of vertices and indices that define a 3D object.
#[derive(Debug)]
pub struct Mesh {
  vertices: Vec<Vertex>,
  attributes: Vec<VertexAttribute>,
}

impl Mesh {
  /// Gets the vertices of the mesh.
  pub fn vertices(&self) -> &[Vertex] {
    &self.vertices
  }

  /// Gets the attributes of the mesh.
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
  /// Creates a new mesh builder.
  pub fn new() -> Self {
    return Self {
      capacity: 0,
      vertices: Vec::new(),
      attributes: Vec::new(),
    };
  }

  /// Allocates memory for the given number of vertices and fills
  /// the mesh with empty vertices.
  pub fn with_capacity(&mut self, size: usize) -> &mut Self {
    self.capacity = size;
    self.vertices.resize(
      size,
      Vertex {
        position: [0.0, 0.0, 0.0],
        normal: [0.0, 0.0, 0.0],
        color: [0.0, 0.0, 0.0],
      },
    );
    return self;
  }

  /// Adds a vertex to the mesh.
  pub fn with_vertex(&mut self, vertex: Vertex) -> &mut Self {
    self.vertices.push(vertex);
    return self;
  }

  /// Specify the attributes of the mesh. This is used to map the vertex data to
  /// the input of the vertex shader.
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

  /// Builds a mesh from the vertices of an OBJ file. The mesh will have the same
  /// attributes as the OBJ file and can be allocated on to the GPU with
  /// `BufferBuilder::build_from_mesh`.
  pub fn build_from_obj(&self, file_path: &str) -> Mesh {
    let obj = load_textured_obj_from_file(file_path);

    let vertices = obj
      .vertices
      .iter()
      .map(|v| {
        return Vertex {
          position: v.position,
          normal: v.normal,
          color: [1.0, 1.0, 1.0],
        };
      })
      .collect::<Vec<Vertex>>();

    // Returns a mesh with the given vertices with attributes for position,
    // normal, and color.
    return Mesh {
      vertices,
      attributes: vec![
        VertexAttribute {
          location: 0,
          offset: 0,
          element: VertexElement {
            format: ColorFormat::Rgb32Sfloat,
            offset: 0,
          },
        },
        VertexAttribute {
          location: 1,
          offset: 0,
          element: VertexElement {
            format: ColorFormat::Rgb32Sfloat,
            offset: 12,
          },
        },
        VertexAttribute {
          location: 2,
          offset: 0,
          element: VertexElement {
            format: ColorFormat::Rgb32Sfloat,
            offset: 24,
          },
        },
      ],
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
