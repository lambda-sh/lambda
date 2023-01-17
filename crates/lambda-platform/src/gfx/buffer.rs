pub struct Buffer;

pub struct BufferBuilder {
  buffer_length: usize,
  usage: gfx_hal::buffer::Usage,
}

impl BufferBuilder {
  pub fn new() -> Self {
    return Self {
      buffer_length: 0,
      usage: gfx_hal::buffer::Usage::empty(),
    };
  }

  pub fn with_length(&mut self, length: usize) -> &mut Self {
    self.buffer_length = length;
    return self;
  }

  pub fn with_usage(&mut self, usage: gfx_hal::buffer::Usage) -> &mut Self {
    self.usage = usage;
    return self;
  }

  pub fn build<RenderBackend: super::internal::Backend>(
    self,
    device: &mut RenderBackend::Device,
  ) -> Buffer {
    todo!();
  }
}
