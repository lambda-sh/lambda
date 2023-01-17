use gfx_hal::{
  memory::SparseFlags,
  Backend,
};

use super::gpu::Gpu;

// Reuse gfx-hal buffer usage & properties for now.
pub type Usage = gfx_hal::buffer::Usage;
pub type Properties = gfx_hal::memory::Properties;

/// A buffer is a block of memory that can be used to store data that can be
/// accessed by the GPU.
pub struct Buffer<RenderBackend: super::internal::Backend> {
  buffer: RenderBackend::Buffer,
  memory: RenderBackend::Memory,
}

pub struct BufferBuilder {
  buffer_length: usize,
  usage: Usage,
  properties: Properties,
}

impl BufferBuilder {
  pub fn new() -> Self {
    return Self {
      buffer_length: 0,
      usage: gfx_hal::buffer::Usage::empty(),
      properties: gfx_hal::memory::Properties::empty(),
    };
  }

  pub fn with_length(&mut self, length: usize) -> &mut Self {
    self.buffer_length = length;
    return self;
  }

  pub fn with_usage(&mut self, usage: Usage) -> &mut Self {
    self.usage = usage;
    return self;
  }

  pub fn with_properties(&mut self, properties: Properties) -> &mut Self {
    return self;
  }

  /// Builds & binds a buffer of memory to the GPU. If the buffer cannot be
  /// bound to the GPU, the buffer memory is freed before the error is returned.
  pub fn build<RenderBackend: super::internal::Backend>(
    self,
    gpu: &mut Gpu<RenderBackend>,
  ) -> Result<Buffer<RenderBackend>, &'static str> {
    use gfx_hal::{
      adapter::PhysicalDevice,
      device::Device,
      MemoryTypeId,
    };
    let logical_device = super::internal::logical_device_for(gpu);
    let physical_device = super::internal::physical_device_for(gpu);

    // TODO(vmarcella): Add the ability for the user to specify the memory
    // properties (I.E. SparseFlags::SPARSE_MEMORY).
    let buffer_result = unsafe {
      logical_device.create_buffer(
        self.buffer_length as u64,
        self.usage,
        SparseFlags::empty(),
      )
    };

    if buffer_result.is_err() {
      return Err("Failed to create buffer for allocating memory.");
    }

    let mut buffer = buffer_result.unwrap();

    let requirements =
      unsafe { logical_device.get_buffer_requirements(&buffer) };
    let memory_types = physical_device.memory_properties().memory_types;

    // Find a memory type that supports the requirements of the buffer.
    let memory_type = memory_types
      .iter()
      .enumerate()
      .find(|(id, memory_type)| {
        let type_supported = requirements.type_mask & (1 << id) != 0;
        type_supported && memory_type.properties.contains(self.properties)
      })
      .map(|(id, _)| MemoryTypeId(id))
      .unwrap();

    // Allocates the memory on the GPU for the buffer.
    let buffer_memory_allocation =
      unsafe { logical_device.allocate_memory(memory_type, requirements.size) };

    if buffer_memory_allocation.is_err() {
      return Err("Failed to allocate memory for buffer.");
    }

    let buffer_memory = buffer_memory_allocation.unwrap();

    // Bind the buffer to the GPU memory
    let buffer_binding = unsafe {
      logical_device.bind_buffer_memory(&buffer_memory, 0, &mut buffer)
    };

    // Destroy the buffer if we failed to bind it to memory.
    if buffer_binding.is_err() {
      unsafe { logical_device.destroy_buffer(buffer) };
      return Err("Failed to bind buffer memory.");
    }

    return Ok(Buffer {
      buffer,
      memory: buffer_memory,
    });
  }
}
