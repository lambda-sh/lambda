//! Buffers for allocating memory on the GPU.

use std::rc::Rc;

use lambda_platform::wgpu::types::{self as wgpu, util::DeviceExt};

use super::{mesh::Mesh, vertex::Vertex, RenderContext};

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
/// A thin newtype for `wgpu::BufferUsages` that supports bitwise ops while
/// keeping explicit construction points in the API surface.
pub struct Usage(wgpu::BufferUsages);

impl Usage {
  /// Mark buffer usable as a vertex buffer.
  pub const VERTEX: Usage = Usage(wgpu::BufferUsages::VERTEX);
  /// Mark buffer usable as an index buffer.
  pub const INDEX: Usage = Usage(wgpu::BufferUsages::INDEX);
  /// Mark buffer usable as a uniform buffer.
  pub const UNIFORM: Usage = Usage(wgpu::BufferUsages::UNIFORM);
  /// Mark buffer usable as a storage buffer.
  pub const STORAGE: Usage = Usage(wgpu::BufferUsages::STORAGE);

  /// Extract the inner `wgpu` flags.
  pub fn to_wgpu(self) -> wgpu::BufferUsages {
    self.0
  }
}

impl std::ops::BitOr for Usage {
  type Output = Usage;

  fn bitor(self, rhs: Usage) -> Usage {
    Usage(self.0 | rhs.0)
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
  buffer: Rc<wgpu::Buffer>,
  stride: wgpu::BufferAddress,
  buffer_type: BufferType,
}

impl Buffer {
  /// Destroy the buffer and all it's resources with the render context that
  /// created it. Dropping the buffer will release GPU resources.
  pub fn destroy(self, _render_context: &RenderContext) {}

  pub(super) fn raw(&self) -> &wgpu::Buffer {
    self.buffer.as_ref()
  }

  pub(super) fn raw_rc(&self) -> Rc<wgpu::Buffer> {
    self.buffer.clone()
  }

  pub(super) fn stride(&self) -> wgpu::BufferAddress {
    self.stride
  }

  /// The logical buffer type used by the engine (e.g., Vertex).
  pub fn buffer_type(&self) -> BufferType {
    self.buffer_type
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
    self
  }

  /// Set the logical type of buffer to be created (vertex/index/...).
  pub fn with_buffer_type(&mut self, buffer_type: BufferType) -> &mut Self {
    self.buffer_type = buffer_type;
    self
  }

  /// Set `wgpu` usage flags (bit‑or `Usage` values).
  pub fn with_usage(&mut self, usage: Usage) -> &mut Self {
    self.usage = usage;
    self
  }

  /// Control CPU visibility and residency preferences.
  pub fn with_properties(&mut self, properties: Properties) -> &mut Self {
    self.properties = properties;
    self
  }

  /// Attach a human‑readable label for debugging/profiling.
  pub fn with_label(&mut self, label: &str) -> &mut Self {
    self.label = Some(label.to_string());
    self
  }

  /// Create a buffer initialized with the provided `data`.
  ///
  /// Returns an error if the resolved length would be zero.
  pub fn build<Data: Copy>(
    &self,
    render_context: &mut RenderContext,
    data: Vec<Data>,
  ) -> Result<Buffer, &'static str> {
    let device = render_context.device();
    let element_size = std::mem::size_of::<Data>();
    let buffer_length = if self.buffer_length == 0 {
      element_size * data.len()
    } else {
      self.buffer_length
    };

    if buffer_length == 0 {
      return Err("Attempted to create a buffer with zero length.");
    }

    let bytes = unsafe {
      std::slice::from_raw_parts(
        data.as_ptr() as *const u8,
        element_size * data.len(),
      )
    };

    let mut usage = self.usage.to_wgpu();
    if self.properties.cpu_visible() {
      usage |= wgpu::BufferUsages::COPY_DST;
    }

    let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: self.label.as_deref(),
      contents: bytes,
      usage,
    });

    Ok(Buffer {
      buffer: Rc::new(buffer),
      stride: element_size as wgpu::BufferAddress,
      buffer_type: self.buffer_type,
    })
  }

  /// Convenience: create a vertex buffer from a `Mesh`'s vertices.
  pub fn build_from_mesh(
    mesh: &Mesh,
    render_context: &mut RenderContext,
  ) -> Result<Buffer, &'static str> {
    let mut builder = Self::new();
    builder.with_length(mesh.vertices().len() * std::mem::size_of::<Vertex>());
    builder.with_usage(Usage::VERTEX);
    builder.with_properties(Properties::CPU_VISIBLE);
    builder.with_buffer_type(BufferType::Vertex);

    builder.build(render_context, mesh.vertices().to_vec())
  }
}
