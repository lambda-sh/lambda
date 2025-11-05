//! Buffer wrappers and builders for the platform layer.
//!
//! This module provides a thin wrapper over `wgpu::Buffer` plus a small
//! builder that handles common initialization patterns and keeps label and
//! usage metadata for debugging/inspection.
use wgpu::{
  self,
  util::DeviceExt,
};

use crate::wgpu::gpu::Gpu;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// Index format for indexed drawing.
pub enum IndexFormat {
  Uint16,
  Uint32,
}

impl IndexFormat {
  pub(crate) fn to_wgpu(self) -> wgpu::IndexFormat {
    return match self {
      IndexFormat::Uint16 => wgpu::IndexFormat::Uint16,
      IndexFormat::Uint32 => wgpu::IndexFormat::Uint32,
    };
  }
}

#[derive(Clone, Copy, Debug)]
/// Platform buffer usage flags.
pub struct Usage(pub(crate) wgpu::BufferUsages);

impl Usage {
  /// Vertex buffer usage.
  pub const VERTEX: Usage = Usage(wgpu::BufferUsages::VERTEX);
  /// Index buffer usage.
  pub const INDEX: Usage = Usage(wgpu::BufferUsages::INDEX);
  /// Uniform buffer usage.
  pub const UNIFORM: Usage = Usage(wgpu::BufferUsages::UNIFORM);
  /// Storage buffer usage.
  pub const STORAGE: Usage = Usage(wgpu::BufferUsages::STORAGE);
  /// Copy destination (for CPU-visible uploads).
  pub const COPY_DST: Usage = Usage(wgpu::BufferUsages::COPY_DST);

  pub(crate) fn to_wgpu(self) -> wgpu::BufferUsages {
    return self.0;
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
    return Usage(wgpu::BufferUsages::VERTEX);
  }
}

#[derive(Debug)]
/// Wrapper around `wgpu::Buffer` with metadata.
pub struct Buffer {
  pub(crate) raw: wgpu::Buffer,
  pub(crate) label: Option<String>,
  pub(crate) size: wgpu::BufferAddress,
  pub(crate) usage: wgpu::BufferUsages,
}

impl Buffer {
  /// Borrow the underlying `wgpu::Buffer`.
  pub(crate) fn raw(&self) -> &wgpu::Buffer {
    return &self.raw;
  }

  /// Optional debug label.
  pub fn label(&self) -> Option<&str> {
    return self.label.as_deref();
  }

  /// Size in bytes at creation time.
  pub fn size(&self) -> u64 {
    return self.size;
  }

  /// Usage flags used to create the buffer.
  pub fn usage(&self) -> Usage {
    return Usage(self.usage);
  }

  /// Write raw bytes into the buffer at the given offset.
  pub fn write_bytes(&self, gpu: &Gpu, offset: u64, data: &[u8]) {
    gpu.queue().write_buffer(&self.raw, offset, data);
  }
}

#[derive(Default)]
/// Builder for creating a `Buffer` with optional initial contents.
pub struct BufferBuilder {
  label: Option<String>,
  size: usize,
  usage: Usage,
  cpu_visible: bool,
}

impl BufferBuilder {
  /// Create a new builder with zero size and VERTEX usage.
  pub fn new() -> Self {
    return Self {
      label: None,
      size: 0,
      usage: Usage::VERTEX,
      cpu_visible: false,
    };
  }

  /// Attach a label for debugging/profiling.
  pub fn with_label(mut self, label: &str) -> Self {
    self.label = Some(label.to_string());
    return self;
  }

  /// Set the total size in bytes. If zero, size is inferred from data length.
  pub fn with_size(mut self, size: usize) -> Self {
    self.size = size;
    return self;
  }

  /// Set usage flags.
  pub fn with_usage(mut self, usage: Usage) -> Self {
    self.usage = usage;
    return self;
  }

  /// Hint that buffer will be updated from CPU via queue writes.
  pub fn with_cpu_visible(mut self, cpu_visible: bool) -> Self {
    self.cpu_visible = cpu_visible;
    return self;
  }

  /// Create a buffer initialized with `contents`.
  pub fn build_init(self, gpu: &Gpu, contents: &[u8]) -> Buffer {
    let size = if self.size == 0 {
      contents.len()
    } else {
      self.size
    };

    let mut usage = self.usage.to_wgpu();
    if self.cpu_visible {
      usage |= wgpu::BufferUsages::COPY_DST;
    }

    let raw =
      gpu
        .device()
        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
          label: self.label.as_deref(),
          contents,
          usage,
        });

    return Buffer {
      raw,
      label: self.label,
      size: size as wgpu::BufferAddress,
      usage,
    };
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn usage_bitor_combines_flags() {
    let u = Usage::VERTEX | Usage::INDEX | Usage::UNIFORM;
    let flags = u.to_wgpu();
    assert!(flags.contains(wgpu::BufferUsages::VERTEX));
    assert!(flags.contains(wgpu::BufferUsages::INDEX));
    assert!(flags.contains(wgpu::BufferUsages::UNIFORM));
  }
}
