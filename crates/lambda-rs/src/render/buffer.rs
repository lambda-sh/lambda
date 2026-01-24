//! GPU buffers for vertex/index data and uniforms.
//!
//! Purpose
//! - Allocate memory on the GPU for vertex and index streams, per‑draw or
//!   per‑frame uniform data, and general storage when needed.
//! - Provide a stable engine‑facing `Buffer` with logical type and stride so
//!   pipelines and commands can bind and validate buffers correctly.
//!
//! Usage
//! - Use `BufferBuilder` to create typed buffers with explicit usage and
//!   residency properties.
//! - Use `UniformBuffer<T>` for a concise pattern when a single `T` value is
//!   updated on the CPU and bound as a uniform.
//!
//! Examples
//! - Creating a vertex buffer from a mesh: `BufferBuilder::build_from_mesh`.
//! - Creating a uniform buffer and updating it each frame:
//!   see `UniformBuffer<T>` below and the runnable example
//!   `crates/lambda-rs/examples/uniform_buffer_triangle.rs`.

use std::rc::Rc;

use lambda_platform::wgpu::buffer as platform_buffer;

use super::{
  gpu::Gpu,
  mesh::Mesh,
  RenderContext,
};
pub use crate::pod::PlainOldData;

/// High‑level classification for buffers created by the engine.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
///
/// The type guides default usage flags and how a buffer is bound during
/// encoding:
/// - `Vertex`: per‑vertex attribute streams consumed by the vertex stage.
/// - `Index`: index streams used for indexed drawing.
/// - `Uniform`: small, read‑only parameters used by shaders.
/// - `Storage`: general read/write data (not yet surfaced by high‑level APIs).
pub enum BufferType {
  Vertex,
  Index,
  Uniform,
  Storage,
}

/// Buffer usage flags (engine‑facing), mapped to platform usage internally.
#[derive(Clone, Copy, Debug)]
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

/// Buffer allocation properties that control residency and CPU visibility.
#[derive(Clone, Copy, Debug)]
///
/// Use `CPU_VISIBLE` for frequently updated data (e.g., uniform uploads).
/// Prefer `DEVICE_LOCAL` for static geometry uploaded once.
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
/// Wraps a platform GPU buffer and tracks the element stride and logical type
/// used when binding to pipeline inputs.
///
/// Notes
/// - Writing is performed via the device queue using `write_value` or by
///   creating CPU‑visible buffers and re‑building with new contents when
///   appropriate.
#[derive(Debug)]
pub struct Buffer {
  buffer: Rc<platform_buffer::Buffer>,
  stride: u64,
  buffer_type: BufferType,
}

impl Buffer {
  /// Destroy the buffer and all its resources with the render context that
  /// created it. Dropping the buffer will release GPU resources.
  pub fn destroy(self, _render_context: &RenderContext) {}

  pub(super) fn raw(&self) -> &platform_buffer::Buffer {
    return self.buffer.as_ref();
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
  /// the CPU. The `data` type must implement `PlainOldData`.
  pub fn write_value<T: PlainOldData>(&self, gpu: &Gpu, offset: u64, data: &T) {
    let bytes = value_as_bytes(data);
    self.write_bytes(gpu, offset, bytes);
  }

  /// Write raw bytes into this buffer at the specified byte offset.
  ///
  /// This is useful when data is already available as a byte slice (for
  /// example, asset blobs or staging buffers).
  ///
  /// Example
  /// ```rust,ignore
  /// let raw_data: &[u8] = load_binary_data();
  /// buffer.write_bytes(render_context.gpu(), 0, raw_data);
  /// ```
  pub fn write_bytes(&self, gpu: &Gpu, offset: u64, data: &[u8]) {
    self.buffer.write_bytes(gpu.platform(), offset, data);
  }

  /// Write a slice of plain-old-data values into this buffer at the
  /// specified byte offset.
  ///
  /// This is intended for uploading arrays of vertices, indices, instance
  /// data, or uniform blocks. The `T` type MUST be plain-old-data (POD) and
  /// safely representable as bytes. This is enforced by requiring `T` to
  /// implement `PlainOldData`.
  ///
  /// Example
  /// ```rust,ignore
  /// let transforms: Vec<InstanceTransform> = compute_transforms();
  /// instance_buffer
  ///   .write_slice(render_context.gpu(), 0, &transforms)
  ///   .unwrap();
  /// ```
  pub fn write_slice<T: PlainOldData>(
    &self,
    gpu: &Gpu,
    offset: u64,
    data: &[T],
  ) -> Result<(), &'static str> {
    let bytes = slice_as_bytes(data)?;
    self.write_bytes(gpu, offset, bytes);
    return Ok(());
  }
}

fn value_as_bytes<T: PlainOldData>(data: &T) -> &[u8] {
  let bytes = unsafe {
    std::slice::from_raw_parts(
      (data as *const T) as *const u8,
      std::mem::size_of::<T>(),
    )
  };
  return bytes;
}

fn checked_byte_len(
  element_size: usize,
  element_count: usize,
) -> Result<usize, &'static str> {
  let Some(byte_len) = element_size.checked_mul(element_count) else {
    return Err("Buffer byte length overflow.");
  };
  return Ok(byte_len);
}

fn slice_as_bytes<T: PlainOldData>(data: &[T]) -> Result<&[u8], &'static str> {
  let element_size = std::mem::size_of::<T>();
  let byte_len = checked_byte_len(element_size, data.len())?;

  let bytes =
    unsafe { std::slice::from_raw_parts(data.as_ptr() as *const u8, byte_len) };
  return Ok(bytes);
}

/// Strongly‑typed uniform buffer wrapper for ergonomics and safety.
///
/// Stores a single value of type `T` and provides a convenience method to
/// upload updates to the GPU. The underlying buffer has `UNIFORM` usage and
/// is CPU‑visible by default for direct queue writes.
///
/// Example
/// ```rust,ignore
/// // Model‑view‑projection updated every frame
/// #[repr(C)]
/// #[derive(Clone, Copy)]
/// struct Mvp { m: [[f32;4];4] }
/// let mut mvp = Mvp { m: [[0.0;4];4] };
/// let mvp_ubo = UniformBuffer::new(render_context, &mvp, Some("mvp")).unwrap();
/// // ... later per‑frame
/// mvp = compute_next_mvp();
/// mvp_ubo.write(&render_context, &mvp);
/// ```
pub struct UniformBuffer<T> {
  inner: Buffer,
  _phantom: core::marker::PhantomData<T>,
}

impl<T: PlainOldData> UniformBuffer<T> {
  /// Create a new uniform buffer initialized with `initial`.
  pub fn new(
    gpu: &Gpu,
    initial: &T,
    label: Option<&str>,
  ) -> Result<Self, &'static str> {
    let mut builder = BufferBuilder::new()
      .with_length(core::mem::size_of::<T>())
      .with_usage(Usage::UNIFORM)
      .with_properties(Properties::CPU_VISIBLE);

    if let Some(l) = label {
      builder = builder.with_label(l);
    }

    let inner = builder.build(gpu, vec![*initial])?;
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
  pub fn write(&self, gpu: &Gpu, value: &T) {
    self.inner.write_value(gpu, 0, value);
  }
}

/// Builder for creating `Buffer` objects with explicit usage and properties.
///
/// A buffer is a block of memory the GPU can access. Supply a total byte
/// length, usage flags, and residency properties; the builder initializes the
/// buffer with provided contents and adds the necessary copy usage when CPU
/// visibility is requested.
///
/// Example (vertex buffer)
/// ```rust,ignore
/// use lambda::render::buffer::{BufferBuilder, Usage, Properties, BufferType};
/// let vertices: Vec<Vertex> = build_vertices();
/// let vb = BufferBuilder::new()
///   .with_usage(Usage::VERTEX)
///   .with_properties(Properties::DEVICE_LOCAL)
///   .with_buffer_type(BufferType::Vertex)
///   .build(render_context, vertices)
///   .unwrap();
/// ```
pub struct BufferBuilder {
  buffer_length: usize,
  usage: Usage,
  properties: Properties,
  buffer_type: BufferType,
  label: Option<String>,
}

impl Default for BufferBuilder {
  fn default() -> Self {
    return Self::new();
  }
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
  pub fn with_length(mut self, size: usize) -> Self {
    self.buffer_length = size;
    return self;
  }

  /// Set the logical type of buffer to be created (vertex/index/...).
  pub fn with_buffer_type(mut self, buffer_type: BufferType) -> Self {
    self.buffer_type = buffer_type;
    return self;
  }

  /// Set `wgpu` usage flags (bit‑or `Usage` values).
  pub fn with_usage(mut self, usage: Usage) -> Self {
    self.usage = usage;
    return self;
  }

  /// Control CPU visibility and residency preferences.
  pub fn with_properties(mut self, properties: Properties) -> Self {
    self.properties = properties;
    return self;
  }

  /// Attach a human‑readable label for debugging/profiling.
  pub fn with_label(mut self, label: &str) -> Self {
    self.label = Some(label.to_string());
    return self;
  }

  /// Create a buffer initialized with the provided `data`.
  ///
  /// Returns an error if the resolved length would be zero.
  ///
  /// The element type MUST implement `PlainOldData` because the engine uploads
  /// the in-memory representation to the GPU.
  pub fn build<Data: PlainOldData>(
    &self,
    gpu: &Gpu,
    data: Vec<Data>,
  ) -> Result<Buffer, &'static str> {
    let element_size = std::mem::size_of::<Data>();
    let buffer_length = self.resolve_length(element_size, data.len())?;
    let byte_len = checked_byte_len(element_size, data.len())?;

    // SAFETY: Converting data to bytes is safe because its underlying
    // type, Data, is constrained to PlainOldData and the lifetime of the slice
    // does not outlive data.
    let bytes = unsafe {
      std::slice::from_raw_parts(data.as_ptr() as *const u8, byte_len)
    };

    let mut builder = platform_buffer::BufferBuilder::new()
      .with_size(buffer_length)
      .with_usage(self.usage.to_platform())
      .with_cpu_visible(self.properties.cpu_visible());
    if let Some(label) = &self.label {
      builder = builder.with_label(label);
    }

    let buffer = builder.build_init(gpu.platform(), bytes);

    return Ok(Buffer {
      buffer: Rc::new(buffer),
      stride: element_size as u64,
      buffer_type: self.buffer_type,
    });
  }

  /// Convenience: create a vertex buffer from a `Mesh`'s vertices.
  pub fn build_from_mesh(
    mesh: &Mesh,
    gpu: &Gpu,
  ) -> Result<Buffer, &'static str> {
    let builder = Self::new();
    return builder
      .with_length(std::mem::size_of_val(mesh.vertices()))
      .with_usage(Usage::VERTEX)
      .with_properties(Properties::CPU_VISIBLE)
      .with_buffer_type(BufferType::Vertex)
      .build(gpu, mesh.vertices().to_vec());
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
      checked_byte_len(element_size, data_len)?
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
    let builder = BufferBuilder::new().with_label("buffer-test");
    // Indirect check: validate the internal label is stored on the builder.
    // Test module is a child of this module and can access private fields.
    assert_eq!(builder.label.as_deref(), Some("buffer-test"));
  }

  #[test]
  fn resolve_length_rejects_overflow() {
    let builder = BufferBuilder::new();
    let result = builder.resolve_length(usize::MAX, 2);
    assert!(result.is_err());
  }

  #[test]
  fn value_as_bytes_matches_native_bytes() {
    let value: u32 = 0x1122_3344;
    let expected = value.to_ne_bytes();
    assert_eq!(value_as_bytes(&value), expected.as_slice());
  }

  #[test]
  fn slice_as_bytes_matches_native_bytes() {
    let values: [u16; 3] = [0x1122, 0x3344, 0x5566];
    let mut expected: Vec<u8> = Vec::new();
    for value in values {
      expected.extend_from_slice(&value.to_ne_bytes());
    }
    assert_eq!(slice_as_bytes(&values).unwrap(), expected.as_slice());
  }

  #[test]
  fn slice_as_bytes_empty_is_empty() {
    let values: [u32; 0] = [];
    assert_eq!(slice_as_bytes(&values).unwrap(), &[]);
  }

  #[test]
  fn checked_byte_len_rejects_overflow() {
    let result = checked_byte_len(usize::MAX, 2);
    assert!(result.is_err());
  }
}
