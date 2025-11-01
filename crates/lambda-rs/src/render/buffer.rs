//! Buffers for allocating memory on the GPU.

use std::rc::Rc;

use lambda_platform::wgpu::{
  buffer as platform_buffer,
  types as wgpu,
};

use super::{
  mesh::Mesh,
  vertex::Vertex,
  RenderContext,
};

#[derive(Clone, Copy, Debug)]
/// High‑level classification for buffers created by the engine.
///
/// The type guides default usage flags and how a buffer is bound during
/// encoding (e.g., as a vertex or index buffer).
pub enum BufferType {
  Vertex,
  Index,
  Uniform,
  Storage,
}

#[derive(Clone, Copy, Debug)]
/// Buffer usage flags (engine-facing), mapped to platform usage internally.
pub struct Usage(platform_buffer::Usage);

impl Usage {
  /// Mark buffer usable as a vertex buffer.
  pub const VERTEX: Usage = Usage(platform_buffer::Usage::VERTEX);
  /// Mark buffer usable as an index buffer.
  pub const INDEX: Usage = Usage(platform_buffer::Usage::INDEX);
  /// Mark buffer usable as a uniform buffer.
  pub const UNIFORM: Usage = Usage(platform_buffer::Usage::UNIFORM);
  /// Mark buffer usable as a storage buffer.
  pub const STORAGE: Usage = Usage(platform_buffer::Usage::STORAGE);

  fn to_platform(self) -> platform_buffer::Usage {
    self.0
  }
}

impl std::ops::BitOr for Usage {
  type Output = Usage;

  fn bitor(self, rhs: Usage) -> Usage {
    return Usage(self.0 | rhs.0);
  }
}

impl Default for Usage {
  fn default() -> Self {
    Usage::VERTEX
  }
}

#[derive(Clone, Copy, Debug)]
/// Buffer allocation properties that control residency and CPU visibility.
pub struct Properties {
  cpu_visible: bool,
}

impl Properties {
  /// Allocate in CPU‑visible memory (upload/streaming friendly).
  pub const CPU_VISIBLE: Properties = Properties { cpu_visible: true };
  /// Allocate in device‑local memory (prefer GPU residency/perf).
  pub const DEVICE_LOCAL: Properties = Properties { cpu_visible: false };

  /// Whether the buffer should be writable from the CPU.
  pub fn cpu_visible(self) -> bool {
    self.cpu_visible
  }
}

impl Default for Properties {
  fn default() -> Self {
    Properties::CPU_VISIBLE
  }
}

/// Buffer for storing data on the GPU.
///
/// Wraps a `wgpu::Buffer` and tracks the element stride and logical type used
/// when binding to pipeline inputs.
#[derive(Debug)]
pub struct Buffer {
  buffer: Rc<platform_buffer::Buffer>,
  stride: u64,
  buffer_type: BufferType,
}

impl Buffer {
  /// Destroy the buffer and all it's resources with the render context that
  /// created it. Dropping the buffer will release GPU resources.
  pub fn destroy(self, _render_context: &RenderContext) {}

  pub(super) fn raw(&self) -> &wgpu::Buffer {
    return self.buffer.raw();
  }

  pub(super) fn stride(&self) -> u64 {
    return self.stride;
  }

  /// The logical buffer type used by the engine (e.g., Vertex).
  pub fn buffer_type(&self) -> BufferType {
    return self.buffer_type;
  }

  /// Write a single plain-old-data value into this buffer at the specified
  /// byte offset. This is intended for updating uniform buffer contents from
  /// the CPU. The `data` type must be trivially copyable.
  pub fn write_value<T: Copy>(
    &self,
    render_context: &RenderContext,
    offset: u64,
    data: &T,
  ) {
    let bytes = unsafe {
      std::slice::from_raw_parts(
        (data as *const T) as *const u8,
        std::mem::size_of::<T>(),
      )
    };
    render_context
      .queue()
      .write_buffer(self.raw(), offset, bytes);
  }
}

/// Strongly‑typed uniform buffer wrapper for ergonomics and safety.
///
/// Stores a single value of type `T` and provides a convenience method to
/// upload updates to the GPU. The underlying buffer has `UNIFORM` usage and
/// is CPU‑visible by default for easy updates via `Queue::write_buffer`.
pub struct UniformBuffer<T> {
  inner: Buffer,
  _phantom: core::marker::PhantomData<T>,
}

impl<T: Copy> UniformBuffer<T> {
  /// Create a new uniform buffer initialized with `initial`.
  pub fn new(
    render_context: &mut RenderContext,
    initial: &T,
    label: Option<&str>,
  ) -> Result<Self, &'static str> {
    let mut builder = BufferBuilder::new();
    builder.with_length(core::mem::size_of::<T>());
    builder.with_usage(Usage::UNIFORM);
    builder.with_properties(Properties::CPU_VISIBLE);
    if let Some(l) = label {
      builder.with_label(l);
    }
    let inner = builder.build(render_context, vec![*initial])?;
    return Ok(Self {
      inner,
      _phantom: core::marker::PhantomData,
    });
  }

  /// Borrow the underlying generic `Buffer` for binding.
  pub fn raw(&self) -> &Buffer {
    return &self.inner;
  }

  /// Write a new value to the GPU buffer at offset 0.
  pub fn write(&self, render_context: &RenderContext, value: &T) {
    self.inner.write_value(render_context, 0, value);
  }
}

/// Builder for creating `Buffer` objects with explicit usage and properties.
///
/// A buffer is a block of memory the GPU can access. You supply a total byte
/// length, usage flags, and residency properties; the builder will initialize
/// the buffer with provided contents and add `COPY_DST` when CPU visibility is
/// requested.
pub struct BufferBuilder {
  buffer_length: usize,
  usage: Usage,
  properties: Properties,
  buffer_type: BufferType,
  label: Option<String>,
}

impl BufferBuilder {
  /// Creates a new buffer builder of type vertex.
  pub fn new() -> Self {
    Self {
      buffer_length: 0,
      usage: Usage::VERTEX,
      properties: Properties::CPU_VISIBLE,
      buffer_type: BufferType::Vertex,
      label: None,
    }
  }

  /// Set the length of the buffer in bytes. Defaults to the size of `data`.
  pub fn with_length(&mut self, size: usize) -> &mut Self {
    self.buffer_length = size;
    return self;
  }

  /// Set the logical type of buffer to be created (vertex/index/...).
  pub fn with_buffer_type(&mut self, buffer_type: BufferType) -> &mut Self {
    self.buffer_type = buffer_type;
    return self;
  }

  /// Set `wgpu` usage flags (bit‑or `Usage` values).
  pub fn with_usage(&mut self, usage: Usage) -> &mut Self {
    self.usage = usage;
    return self;
  }

  /// Control CPU visibility and residency preferences.
  pub fn with_properties(&mut self, properties: Properties) -> &mut Self {
    self.properties = properties;
    return self;
  }

  /// Attach a human‑readable label for debugging/profiling.
  pub fn with_label(&mut self, label: &str) -> &mut Self {
    self.label = Some(label.to_string());
    return self;
  }

  /// Create a buffer initialized with the provided `data`.
  ///
  /// Returns an error if the resolved length would be zero.
  pub fn build<Data: Copy>(
    &self,
    render_context: &mut RenderContext,
    data: Vec<Data>,
  ) -> Result<Buffer, &'static str> {
    let element_size = std::mem::size_of::<Data>();
    let buffer_length = self.resolve_length(element_size, data.len())?;

    // SAFETY: Converting data to bytes is safe because it's underlying
    // type, Data, is constrianed to Copy and the lifetime of the slice does
    // not outlive data.
    let bytes = unsafe {
      std::slice::from_raw_parts(
        data.as_ptr() as *const u8,
        element_size * data.len(),
      )
    };

    let mut builder = platform_buffer::BufferBuilder::new()
      .with_size(buffer_length)
      .with_usage(self.usage.to_platform())
      .with_cpu_visible(self.properties.cpu_visible());
    if let Some(label) = &self.label {
      builder = builder.with_label(label);
    }

    let buffer = builder.build_init(render_context.device(), bytes);

    return Ok(Buffer {
      buffer: Rc::new(buffer),
      stride: element_size as u64,
      buffer_type: self.buffer_type,
    });
  }

  /// Convenience: create a vertex buffer from a `Mesh`'s vertices.
  pub fn build_from_mesh(
    mesh: &Mesh,
    render_context: &mut RenderContext,
  ) -> Result<Buffer, &'static str> {
    let mut builder = Self::new();
    return builder
      .with_length(mesh.vertices().len() * std::mem::size_of::<Vertex>())
      .with_usage(Usage::VERTEX)
      .with_properties(Properties::CPU_VISIBLE)
      .with_buffer_type(BufferType::Vertex)
      .build(render_context, mesh.vertices().to_vec());
  }
}

impl BufferBuilder {
  /// Resolve the effective buffer length from explicit size or data length.
  /// Returns an error if the resulting length would be zero.
  pub(crate) fn resolve_length(
    &self,
    element_size: usize,
    data_len: usize,
  ) -> Result<usize, &'static str> {
    let buffer_length = if self.buffer_length == 0 {
      element_size * data_len
    } else {
      self.buffer_length
    };
    if buffer_length == 0 {
      return Err("Attempted to create a buffer with zero length.");
    }
    return Ok(buffer_length);
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn resolve_length_rejects_zero() {
    let builder = BufferBuilder::new();
    let result = builder.resolve_length(std::mem::size_of::<u32>(), 0);
    assert!(result.is_err());
  }

  #[test]
  fn label_is_recorded_on_builder() {
    let mut builder = BufferBuilder::new();
    builder.with_label("buffer-test");
    // Indirect check via building a small buffer would require a device; ensure
    // the label setter stores the value locally instead.
    // Access through an internal helper to avoid exposing label publicly.
    #[allow(clippy::redundant_closure_call)]
    {
      // Create a small closure to read the private label field.
      // The test module shares the parent scope, so it can access fields.
      let read = |b: &BufferBuilder| b.label.as_deref();
      assert_eq!(read(&builder), Some("buffer-test"));
    }
  }
}
