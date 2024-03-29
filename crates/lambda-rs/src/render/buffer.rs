//! Buffers for allocating memory on the GPU.

mod internal {
  // Placed these in an internal module to avoid a name collision with the
  // high level Buffer & BufferBuilder types in the parent module.
  pub use lambda_platform::gfx::buffer::{
    Buffer,
    BufferBuilder,
  };
}

use std::rc::Rc;

// publicly use Properties and Usage from buffer.rs
pub use lambda_platform::gfx::buffer::{
  BufferType,
  Properties,
  Usage,
};
use logging;

use super::{
  mesh::Mesh,
  vertex::Vertex,
  RenderContext,
};

/// Buffer for storing vertex data on the GPU.
#[derive(Debug)]
pub struct Buffer {
  buffer: Rc<internal::Buffer<super::internal::RenderBackend>>,
  buffer_type: BufferType,
}

/// Public interface for a buffer.
impl Buffer {
  /// Destroy the buffer and all it's resources with the render context that
  /// created it.
  pub fn destroy(self, render_context: &RenderContext) {
    Rc::try_unwrap(self.buffer)
      .expect("Failed to get inside buffer")
      .destroy(render_context.internal_gpu());
  }
}

/// Internal interface for working with buffers.
impl Buffer {
  /// Retrieve a reference to the internal buffer.
  pub(super) fn internal_buffer_rc(
    &self,
  ) -> Rc<internal::Buffer<super::internal::RenderBackend>> {
    return self.buffer.clone();
  }

  pub(super) fn internal_buffer(
    &self,
  ) -> &internal::Buffer<super::internal::RenderBackend> {
    return &self.buffer;
  }
}

/// A buffer is a block of memory that can be used to store data that can be
/// accessed by the GPU. The buffer is created with a length, usage, and
/// properties that determine how the buffer can be used.
pub struct BufferBuilder {
  buffer_builder: internal::BufferBuilder,
  buffer_type: BufferType,
}

impl BufferBuilder {
  /// Creates a new buffer builder of type vertex.
  pub fn new() -> Self {
    return Self {
      buffer_builder: internal::BufferBuilder::new(),
      buffer_type: BufferType::Vertex,
    };
  }

  /// Builds a buffer from a given mesh and allocates it's memory on to the GPU.
  pub fn build_from_mesh(
    mesh: &Mesh,
    render_context: &mut RenderContext,
  ) -> Result<Buffer, &'static str> {
    let mut buffer_builder = Self::new();

    // Allocate a buffer with the size of the mesh's vertices.
    let internal_buffer = buffer_builder
      .buffer_builder
      .with_length(mesh.vertices().len() * std::mem::size_of::<Vertex>())
      .with_usage(Usage::VERTEX)
      .with_properties(Properties::CPU_VISIBLE)
      .build(
        render_context.internal_mutable_gpu(),
        mesh.vertices().to_vec(),
      );

    match internal_buffer {
      Ok(internal_buffer) => {
        return Ok(Buffer {
          buffer: Rc::new(internal_buffer),
          buffer_type: BufferType::Vertex,
        });
      }
      Err(_) => {
        return Err("Failed to create buffer from mesh.");
      }
    }
  }

  /// Sets the length of the buffer (In bytes).
  pub fn with_length(&mut self, size: usize) -> &mut Self {
    self.buffer_builder.with_length(size);
    return self;
  }

  /// Sets the type of buffer to create.
  pub fn with_buffer_type(&mut self, buffer_type: BufferType) -> &mut Self {
    self.buffer_type = buffer_type;
    self.buffer_builder.with_buffer_type(buffer_type);
    return self;
  }

  /// Sets the usage of the buffer.
  pub fn with_usage(&mut self, usage: Usage) -> &mut Self {
    self.buffer_builder.with_usage(usage);
    return self;
  }

  /// Sets the properties of the buffer.
  pub fn with_properties(&mut self, properties: Properties) -> &mut Self {
    self.buffer_builder.with_properties(properties);
    return self;
  }

  /// Build a buffer utilizing the current render context
  pub fn build<Data: Sized>(
    &self,
    render_context: &mut RenderContext,
    data: Vec<Data>,
  ) -> Result<Buffer, &'static str> {
    let buffer_allocation = self
      .buffer_builder
      .build(render_context.internal_mutable_gpu(), data);

    match buffer_allocation {
      Ok(buffer) => {
        logging::debug!(
          "Buffer allocation for {:?} succeeded.",
          self.buffer_type
        );
        return Ok(Buffer {
          buffer: Rc::new(buffer),
          buffer_type: self.buffer_type,
        });
      }
      Err(error) => {
        logging::error!(
          "Buffer allocation for {:?} failed with error: {:?}",
          self.buffer_type,
          error
        );
        return Err(error);
      }
    }
  }
}
