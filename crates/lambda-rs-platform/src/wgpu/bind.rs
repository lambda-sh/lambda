//! Bind group and bind group layout builders for the platform layer.
//!
//! These types provide a thin, explicit wrapper around `wgpu` bind resources
//! so higher layers can compose layouts and groups without pulling in raw
//! `wgpu` descriptors throughout the codebase.

use std::num::NonZeroU64;

use wgpu;

use crate::wgpu::{
  buffer,
  gpu::Gpu,
};

#[derive(Debug)]
/// Wrapper around `wgpu::BindGroupLayout` that preserves a label.
pub struct BindGroupLayout {
  pub(crate) raw: wgpu::BindGroupLayout,
  pub(crate) label: Option<String>,
}

impl BindGroupLayout {
  /// Borrow the underlying `wgpu::BindGroupLayout`.
  pub fn raw(&self) -> &wgpu::BindGroupLayout {
    return &self.raw;
  }

  /// Optional debug label used during creation.
  pub fn label(&self) -> Option<&str> {
    return self.label.as_deref();
  }
}

#[derive(Debug)]
/// Wrapper around `wgpu::BindGroup` that preserves a label.
pub struct BindGroup {
  pub(crate) raw: wgpu::BindGroup,
  pub(crate) label: Option<String>,
}

impl BindGroup {
  /// Borrow the underlying `wgpu::BindGroup`.
  pub fn raw(&self) -> &wgpu::BindGroup {
    return &self.raw;
  }

  /// Optional debug label used during creation.
  pub fn label(&self) -> Option<&str> {
    return self.label.as_deref();
  }
}

#[derive(Clone, Copy, Debug)]
/// Visibility of a binding across shader stages.
pub enum Visibility {
  Vertex,
  Fragment,
  Compute,
  VertexAndFragment,
  All,
}

impl Visibility {
  fn to_wgpu(self) -> wgpu::ShaderStages {
    return match self {
      Visibility::Vertex => wgpu::ShaderStages::VERTEX,
      Visibility::Fragment => wgpu::ShaderStages::FRAGMENT,
      Visibility::Compute => wgpu::ShaderStages::COMPUTE,
      Visibility::VertexAndFragment => {
        wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT
      }
      Visibility::All => wgpu::ShaderStages::all(),
    };
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  /// This test verifies that each public binding visibility option is
  /// converted into the correct set of shader stage flags expected by the
  /// underlying graphics layer. It checks single stage selections, a
  /// combination of vertex and fragment stages, and the catch‑all option that
  /// enables all stages. The goal is to demonstrate that the mapping logic is
  /// precise and predictable so higher level code can rely on it when building
  /// layouts and groups.
  #[test]
  fn visibility_maps_to_expected_shader_stages() {
    assert_eq!(Visibility::Vertex.to_wgpu(), wgpu::ShaderStages::VERTEX);
    assert_eq!(Visibility::Fragment.to_wgpu(), wgpu::ShaderStages::FRAGMENT);
    assert_eq!(Visibility::Compute.to_wgpu(), wgpu::ShaderStages::COMPUTE);
    assert_eq!(
      Visibility::VertexAndFragment.to_wgpu(),
      wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT
    );
    assert_eq!(Visibility::All.to_wgpu(), wgpu::ShaderStages::all());
  }
}

#[derive(Default)]
/// Builder for creating a `wgpu::BindGroupLayout`.
pub struct BindGroupLayoutBuilder {
  label: Option<String>,
  entries: Vec<wgpu::BindGroupLayoutEntry>,
}

impl BindGroupLayoutBuilder {
  /// Create a builder with no entries.
  pub fn new() -> Self {
    return Self {
      label: None,
      entries: Vec::new(),
    };
  }

  /// Attach a human‑readable label.
  pub fn with_label(mut self, label: &str) -> Self {
    self.label = Some(label.to_string());
    return self;
  }

  /// Declare a uniform buffer binding at the provided index.
  pub fn with_uniform(mut self, binding: u32, visibility: Visibility) -> Self {
    self.entries.push(wgpu::BindGroupLayoutEntry {
      binding,
      visibility: visibility.to_wgpu(),
      ty: wgpu::BindingType::Buffer {
        ty: wgpu::BufferBindingType::Uniform,
        has_dynamic_offset: false,
        min_binding_size: None,
      },
      count: None,
    });
    return self;
  }

  /// Declare a uniform buffer binding with dynamic offsets at the provided index.
  pub fn with_uniform_dynamic(
    mut self,
    binding: u32,
    visibility: Visibility,
  ) -> Self {
    self.entries.push(wgpu::BindGroupLayoutEntry {
      binding,
      visibility: visibility.to_wgpu(),
      ty: wgpu::BindingType::Buffer {
        ty: wgpu::BufferBindingType::Uniform,
        has_dynamic_offset: true,
        min_binding_size: None,
      },
      count: None,
    });
    return self;
  }

  /// Build the layout using the provided device.
  pub fn build(self, gpu: &Gpu) -> BindGroupLayout {
    let raw =
      gpu
        .device()
        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
          label: self.label.as_deref(),
          entries: &self.entries,
        });
    return BindGroupLayout {
      raw,
      label: self.label,
    };
  }
}

#[derive(Default)]
/// Builder for creating a `wgpu::BindGroup`.
pub struct BindGroupBuilder<'a> {
  label: Option<String>,
  layout: Option<&'a wgpu::BindGroupLayout>,
  entries: Vec<wgpu::BindGroupEntry<'a>>,
}

impl<'a> BindGroupBuilder<'a> {
  /// Create a new builder with no layout or entries.
  pub fn new() -> Self {
    return Self {
      label: None,
      layout: None,
      entries: Vec::new(),
    };
  }

  /// Attach a human‑readable label.
  pub fn with_label(mut self, label: &str) -> Self {
    self.label = Some(label.to_string());
    return self;
  }

  /// Specify the layout to use for this bind group.
  pub fn with_layout(mut self, layout: &'a BindGroupLayout) -> Self {
    self.layout = Some(layout.raw());
    return self;
  }

  /// Bind a uniform buffer at a binding index with optional size slice.
  pub fn with_uniform(
    mut self,
    binding: u32,
    buffer: &'a buffer::Buffer,
    offset: u64,
    size: Option<NonZeroU64>,
  ) -> Self {
    self.entries.push(wgpu::BindGroupEntry {
      binding,
      resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
        buffer: buffer.raw(),
        offset,
        size,
      }),
    });
    return self;
  }

  /// Build the bind group with the accumulated entries.
  pub fn build(self, gpu: &Gpu) -> BindGroup {
    let layout = self
      .layout
      .expect("BindGroupBuilder requires a layout before build");
    let raw = gpu.device().create_bind_group(&wgpu::BindGroupDescriptor {
      label: self.label.as_deref(),
      layout,
      entries: &self.entries,
    });
    return BindGroup {
      raw,
      label: self.label,
    };
  }
}
