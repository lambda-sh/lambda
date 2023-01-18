mod internal {
  pub use lambda_platform::gfx::buffer::{
    Buffer,
    BufferBuilder,
  };
}

// publicly use Properties and Usage from buffer.rs
pub use lambda_platform::gfx::buffer::{
  BufferType,
  Properties,
  Usage,
};

use super::{
  internal::mut_gpu_from_context,
  RenderContext,
};

#[derive(Debug)]
pub struct Buffer {
  buffer: internal::Buffer<super::internal::RenderBackend>,
  buffer_type: BufferType,
}

pub struct BufferBuilder {
  buffer_builder: internal::BufferBuilder,
  buffer_type: BufferType,
}

impl BufferBuilder {
  pub fn new() -> Self {
    return Self {
      buffer_builder: internal::BufferBuilder::new(),
      buffer_type: BufferType::Vertex,
    };
  }

  pub fn with_length(&mut self, size: usize) -> &mut Self {
    self.buffer_builder.with_length(size);
    return self;
  }

  pub fn with_buffer_type(&mut self, buffer_type: BufferType) -> &mut Self {
    self.buffer_type = buffer_type;
    self.buffer_builder.with_buffer_type(buffer_type);
    return self;
  }

  pub fn with_usage(&mut self, usage: Usage) -> &mut Self {
    self.buffer_builder.with_usage(usage);
    return self;
  }

  pub fn with_properties(&mut self, properties: Properties) -> &mut Self {
    self.buffer_builder.with_properties(properties);
    return self;
  }

  /// Build a buffer from the render context.
  pub fn build<Data: Sized>(
    &self,
    render_context: &mut RenderContext,
    data: Vec<Data>,
  ) -> Result<Buffer, &'static str> {
    let buffer_allocation = self
      .buffer_builder
      .build(&mut mut_gpu_from_context(render_context), data);

    match buffer_allocation {
      Ok(buffer) => {
        return Ok(Buffer {
          buffer,
          buffer_type: self.buffer_type,
        });
      }
      Err(error) => {
        return Err(error);
      }
    }
  }
}
