//! High-level bind group and bind group layout wrappers and builders.
//!
//! This module exposes ergonomic builders for declaring uniform buffer
//! bindings and constructing bind groups, following the same style used by the
//! buffer, pipeline, and render pass builders.

use std::rc::Rc;

#[derive(Clone, Copy, Debug)]
/// Visibility of a binding across shader stages (engine facing).
pub enum BindingVisibility {
  Vertex,
  Fragment,
  Compute,
  VertexAndFragment,
  All,
}

impl BindingVisibility {
  fn to_platform(self) -> lambda_platform::wgpu::bind::Visibility {
    match self {
      BindingVisibility::Vertex => {
        lambda_platform::wgpu::bind::Visibility::Vertex
      }
      BindingVisibility::Fragment => {
        lambda_platform::wgpu::bind::Visibility::Fragment
      }
      BindingVisibility::Compute => {
        lambda_platform::wgpu::bind::Visibility::Compute
      }
      BindingVisibility::VertexAndFragment => {
        lambda_platform::wgpu::bind::Visibility::VertexAndFragment
      }
      BindingVisibility::All => lambda_platform::wgpu::bind::Visibility::All,
    }
  }
}

use super::{
  buffer::Buffer,
  RenderContext,
};

#[cfg(test)]
mod tests {
  use super::*;
}

#[derive(Debug, Clone)]
/// Bind group layout used when creating pipelines and bind groups.
pub struct BindGroupLayout {
  layout: Rc<lambda_platform::wgpu::bind::BindGroupLayout>,
  /// Total number of dynamic bindings declared in this layout.
  dynamic_binding_count: u32,
}

impl BindGroupLayout {
  /// Borrow the underlying platform bind group layout wrapper.
  pub(crate) fn platform_layout(
    &self,
  ) -> &lambda_platform::wgpu::bind::BindGroupLayout {
    return &self.layout;
  }

  /// Number of dynamic bindings declared in this layout.
  pub fn dynamic_binding_count(&self) -> u32 {
    return self.dynamic_binding_count;
  }
}

#[derive(Debug, Clone)]
/// Bind group that binds one or more resources to a pipeline set index.
pub struct BindGroup {
  group: Rc<lambda_platform::wgpu::bind::BindGroup>,
  /// Cached number of dynamic bindings expected when binding this group.
  dynamic_binding_count: u32,
}

impl BindGroup {
  pub(crate) fn platform_group(
    &self,
  ) -> &lambda_platform::wgpu::bind::BindGroup {
    return &self.group;
  }

  /// Number of dynamic bindings expected when calling set_bind_group.
  pub fn dynamic_binding_count(&self) -> u32 {
    return self.dynamic_binding_count;
  }
}

/// Builder for creating a bind group layout with uniform buffer bindings.
pub struct BindGroupLayoutBuilder {
  label: Option<String>,
  entries: Vec<(u32, BindingVisibility, bool)>,
}

impl BindGroupLayoutBuilder {
  /// Create a new builder with no bindings.
  pub fn new() -> Self {
    Self {
      label: None,
      entries: Vec::new(),
    }
  }

  /// Attach a label for debugging and profiling.
  pub fn with_label(mut self, label: &str) -> Self {
    self.label = Some(label.to_string());
    return self;
  }

  /// Add a uniform buffer binding visible to the specified stages.
  pub fn with_uniform(
    mut self,
    binding: u32,
    visibility: BindingVisibility,
  ) -> Self {
    self.entries.push((binding, visibility, false));
    return self;
  }

  /// Add a uniform buffer binding with dynamic offset support.
  pub fn with_uniform_dynamic(
    mut self,
    binding: u32,
    visibility: BindingVisibility,
  ) -> Self {
    self.entries.push((binding, visibility, true));
    return self;
  }

  /// Build the layout using the `RenderContext` device.
  pub fn build(self, render_context: &RenderContext) -> BindGroupLayout {
    let mut builder =
      lambda_platform::wgpu::bind::BindGroupLayoutBuilder::new();

    #[cfg(debug_assertions)]
    {
      // In debug builds, check for duplicate binding indices.
      use std::collections::HashSet;
      let mut seen = HashSet::new();

      for (binding, _, _) in &self.entries {
        assert!(
          seen.insert(binding),
          "BindGroupLayoutBuilder: duplicate binding index {}",
          binding
        );
      }
    }

    let dynamic_binding_count =
      self.entries.iter().filter(|(_, _, d)| *d).count() as u32;

    if let Some(label) = &self.label {
      builder = builder.with_label(label);
    }

    for (binding, visibility, dynamic) in self.entries.into_iter() {
      builder = if dynamic {
        builder.with_uniform_dynamic(binding, visibility.to_platform())
      } else {
        builder.with_uniform(binding, visibility.to_platform())
      };
    }

    let layout = builder.build(render_context.gpu());

    return BindGroupLayout {
      layout: Rc::new(layout),
      dynamic_binding_count,
    };
  }
}

/// Builder for creating a bind group for a previously built layout.
pub struct BindGroupBuilder<'a> {
  label: Option<String>,
  layout: Option<&'a BindGroupLayout>,
  entries: Vec<(u32, &'a Buffer, u64, Option<std::num::NonZeroU64>)>,
}

impl<'a> BindGroupBuilder<'a> {
  /// Create a new builder with no layout.
  pub fn new() -> Self {
    return Self {
      label: None,
      layout: None,
      entries: Vec::new(),
    };
  }

  /// Attach a label for debugging and profiling.
  pub fn with_label(mut self, label: &str) -> Self {
    self.label = Some(label.to_string());
    return self;
  }

  /// Use a previously created layout for this bind group.
  pub fn with_layout(mut self, layout: &'a BindGroupLayout) -> Self {
    self.layout = Some(layout);
    return self;
  }

  /// Bind a uniform buffer to the specified binding index.
  pub fn with_uniform(
    mut self,
    binding: u32,
    buffer: &'a Buffer,
    offset: u64,
    size: Option<std::num::NonZeroU64>,
  ) -> Self {
    self.entries.push((binding, buffer, offset, size));
    return self;
  }

  /// Build the bind group on the current device.
  pub fn build(self, render_context: &RenderContext) -> BindGroup {
    let layout = self
      .layout
      .expect("BindGroupBuilder requires a layout before build");

    let mut platform = lambda_platform::wgpu::bind::BindGroupBuilder::new()
      .with_layout(&layout.layout);

    if let Some(label) = &self.label {
      platform = platform.with_label(label);
    }

    let max_binding = render_context.limit_max_uniform_buffer_binding_size();

    for (binding, buffer, offset, size) in self.entries.into_iter() {
      if let Some(sz) = size {
        assert!(
          sz.get() <= max_binding,
          "Uniform binding at binding={} requests size={} > device limit {}",
          binding,
          sz.get(),
          max_binding
        );
      }
      platform = platform.with_uniform(binding, buffer.raw(), offset, size);
    }

    let group = platform.build(render_context.gpu());
    return BindGroup {
      group: Rc::new(group),
      dynamic_binding_count: layout.dynamic_binding_count(),
    };
  }
}
