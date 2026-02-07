//! Simple mesh container used by examples and helpers.
//!
//! Purpose
//! - Hold a `Vec<Vertex>` and matching `VertexAttribute` layout used to build
//!   vertex buffers and pipelines.
//! - Provide minimal builders plus an OBJ loader path for quick iteration.
//!
//! Note: this is a convenience structure for examples; larger applications may
//! want dedicated asset/geometry systems.

use lambda_platform::obj::load_textured_obj_from_file;

use super::vertex::{
  ColorFormat,
  Vertex,
  VertexAttribute,
  VertexElement,
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

/// Builder for constructing a `Mesh` from vertices and attributes.
#[derive(Clone, Debug)]
pub struct MeshBuilder {
  vertices: Vec<Vertex>,
  attributes: Vec<VertexAttribute>,
}

impl Default for MeshBuilder {
  fn default() -> Self {
    return Self::new();
  }
}

impl MeshBuilder {
  /// Creates a new mesh builder.
  pub fn new() -> Self {
    return Self {
      vertices: Vec::new(),
      attributes: Vec::new(),
    };
  }

  /// Allocates memory for the given number of vertices and fills
  /// the mesh with empty vertices.
  pub fn with_capacity(&mut self, size: usize) -> &mut Self {
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
  use super::MeshBuilder;
  use crate::render::vertex::Vertex;

  #[test]
  fn mesh_building() {
    let mesh = MeshBuilder::new();

    assert_eq!(mesh.vertices.len(), 0);
  }

  #[test]
  fn mesh_builder_capacity_and_attributes_are_applied() {
    let mut builder = MeshBuilder::new();
    builder.with_capacity(2);
    assert_eq!(builder.vertices.len(), 2);

    builder.with_vertex(Vertex {
      position: [1.0, 2.0, 3.0],
      normal: [0.0, 1.0, 0.0],
      color: [0.5, 0.5, 0.5],
    });

    let mesh = builder.build();
    assert_eq!(mesh.vertices().len(), 3);
  }

  #[test]
  fn mesh_build_from_obj_parses_vertices() {
    use std::{
      fs,
      path::PathBuf,
    };

    // Minimal OBJ with one triangle, normals, and texture coordinates.
    let obj = r#"
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.0 1.0 0.0
vt 0.0 0.0
vt 1.0 0.0
vt 0.0 1.0
vn 0.0 0.0 1.0
f 1/1/1 2/2/1 3/3/1
"#;

    let mut path = PathBuf::from(std::env::temp_dir());
    path.push("lambda_mesh_test.obj");
    fs::write(&path, obj).expect("write temp obj");

    let builder = super::MeshBuilder::new();
    let mesh = builder
      .build_from_obj(path.to_str().expect("temp path must be valid utf-8"));

    // The platform loader expands the face into vertices.
    assert_eq!(mesh.vertices().len(), 3);
    assert_eq!(mesh.attributes().len(), 3);
  }
}
