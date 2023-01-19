use super::{
  buffer::{
    Buffer,
    BufferBuilder,
    Properties,
    Usage,
  },
  vertex::Vertex,
  RenderContext,
};

// ---------------------------------- Mesh ------------------------------------

/// Collection of vertices and indices that define a 3D object.
#[derive(Debug)]
pub struct Mesh {
  vertices: Vec<Vertex>,
  buffer: Buffer,
}

// ------------------------------ MeshBuilder ---------------------------------

/// Construction for a mesh.
#[derive(Clone, Debug)]
pub struct MeshBuilder {
  capacity: usize,
  vertices: Vec<Vertex>,
}

impl MeshBuilder {
  pub fn new() -> Self {
    return Self {
      capacity: 0,
      vertices: vec![],
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

  /// Builds a mesh from the vertices and indices that have been added to the
  /// builder and allocates the memory for the mesh on the GPU.
  pub fn build(
    &self,
    render_context: &mut RenderContext,
  ) -> Result<Mesh, &'static str> {
    let gpu_memory_required =
      self.vertices.len() * std::mem::size_of::<Vertex>();
    println!(
      "Allocating {} bytes of GPU memory for mesh.",
      gpu_memory_required
    );

    // Allocate memory for the mesh on the GPU.
    let buffer_allocation = BufferBuilder::new()
      .with_length(gpu_memory_required)
      .with_usage(Usage::VERTEX)
      .with_properties(Properties::CPU_VISIBLE | Properties::COHERENT)
      .build(render_context, self.vertices.clone());

    match buffer_allocation {
      Ok(buffer) => {
        return Ok(Mesh {
          vertices: self.vertices.clone(),
          buffer,
        });
      }
      Err(error) => {
        return Err(error);
      }
    }
  }
}

#[cfg(test)]
mod tests {
  #[test]
  fn mesh_building() {
    let mut mesh = super::MeshBuilder::new();

    assert_eq!(mesh.vertices.len(), 0);
  }

  // TODO(vmarcella): Add more tests for mesh building once the render context
  // is mockable. As of right now, testing would require the creation of a real
  // render context to perform the GPU memory allocation & binding for the mesh.
}
