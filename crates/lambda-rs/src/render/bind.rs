//! High-level bind group and bind group layout wrappers and builders.
//!
//! This module exposes ergonomic builders for declaring uniform buffer
//! bindings and constructing bind groups, following the same style used by the
//! buffer, pipeline, and render pass builders.

use std::rc::Rc;

use lambda_platform::wgpu::types as wgpu;

use super::{
  buffer::Buffer,
  texture::{
    Sampler,
    Texture,
  },
  RenderContext,
};

#[derive(Debug)]
/// Visibility of a binding across shader stages.
pub enum BindingVisibility {
  Vertex,
  Fragment,
  Compute,
  VertexAndFragment,
  All,
}

impl BindingVisibility {
  fn to_platform(self) -> lambda_platform::wgpu::bind::Visibility {
    use lambda_platform::wgpu::bind::Visibility as V;
    return match self {
      BindingVisibility::Vertex => V::Vertex,
      BindingVisibility::Fragment => V::Fragment,
      BindingVisibility::Compute => V::Compute,
      BindingVisibility::VertexAndFragment => V::VertexAndFragment,
      BindingVisibility::All => V::All,
    };
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  /// This test confirms that every high‑level binding visibility option maps
  /// directly to the corresponding visibility option in the platform layer.
  /// Matching these values ensures that builder code in this module forwards
  /// intent without alteration, which is important for readability and for
  /// maintenance when constructing layouts and groups.
  #[test]
  fn binding_visibility_maps_to_platform_enum() {
    use lambda_platform::wgpu::bind::Visibility as P;

    assert!(matches!(BindingVisibility::Vertex.to_platform(), P::Vertex));
    assert!(matches!(
      BindingVisibility::Fragment.to_platform(),
      P::Fragment
    ));
    assert!(matches!(
      BindingVisibility::Compute.to_platform(),
      P::Compute
    ));
    assert!(matches!(
      BindingVisibility::VertexAndFragment.to_platform(),
      P::VertexAndFragment
    ));
    assert!(matches!(BindingVisibility::All.to_platform(), P::All));
  }
}

#[derive(Debug, Clone)]
/// Bind group layout used when creating pipelines and bind groups.
pub struct BindGroupLayout {
  layout: Rc<lambda_platform::wgpu::bind::BindGroupLayout>,
  /// Total number of dynamic bindings declared in this layout.
  dynamic_binding_count: u32,
}

impl BindGroupLayout {
  pub(crate) fn raw(&self) -> &wgpu::BindGroupLayout {
    return self.layout.raw();
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
  pub(crate) fn raw(&self) -> &wgpu::BindGroup {
    return self.group.raw();
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
  textures_2d: Vec<(u32, BindingVisibility)>,
  samplers: Vec<(u32, BindingVisibility)>,
}

impl BindGroupLayoutBuilder {
  /// Create a new builder with no bindings.
  pub fn new() -> Self {
    Self {
      label: None,
      entries: Vec::new(),
      textures_2d: Vec::new(),
      samplers: Vec::new(),
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

  /// Add a sampled 2D texture binding, defaulting to fragment visibility.
  pub fn with_sampled_texture(mut self, binding: u32) -> Self {
    self
      .textures_2d
      .push((binding, BindingVisibility::Fragment));
    return self;
  }

  /// Add a filtering sampler binding, defaulting to fragment visibility.
  pub fn with_sampler(mut self, binding: u32) -> Self {
    self.samplers.push((binding, BindingVisibility::Fragment));
    return self;
  }

  /// Build the layout using the `RenderContext` device.
  pub fn build(self, render_context: &RenderContext) -> BindGroupLayout {
    let mut builder =
      lambda_platform::wgpu::bind::BindGroupLayoutBuilder::new();

    #[cfg(debug_assertions)]
    {
      // In debug builds, check for duplicate binding indices across all kinds.
      use std::collections::HashSet;
      let mut seen = HashSet::new();
      for (binding, _, _) in &self.entries {
        assert!(
          seen.insert(binding),
          "BindGroupLayoutBuilder: duplicate binding index {}",
          binding
        );
      }
      for (binding, _) in &self.textures_2d {
        assert!(
          seen.insert(binding),
          "BindGroupLayoutBuilder: duplicate binding index {}",
          binding
        );
      }
      for (binding, _) in &self.samplers {
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

    for (binding, visibility) in self.textures_2d.into_iter() {
      builder =
        builder.with_sampled_texture_2d(binding, visibility.to_platform());
    }

    for (binding, visibility) in self.samplers.into_iter() {
      builder = builder.with_sampler(binding, visibility.to_platform());
    }

    let layout = builder.build(render_context.device());

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
  textures: Vec<(u32, Rc<lambda_platform::wgpu::texture::Texture>)>,
  samplers: Vec<(u32, Rc<lambda_platform::wgpu::texture::Sampler>)>,
}

impl<'a> BindGroupBuilder<'a> {
  /// Create a new builder with no layout.
  pub fn new() -> Self {
    return Self {
      label: None,
      layout: None,
      entries: Vec::new(),
      textures: Vec::new(),
      samplers: Vec::new(),
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

  /// Bind a 2D texture at the specified binding index.
  pub fn with_texture(mut self, binding: u32, texture: &'a Texture) -> Self {
    self.textures.push((binding, texture.platform_texture()));
    return self;
  }

  /// Bind a sampler at the specified binding index.
  pub fn with_sampler(mut self, binding: u32, sampler: &'a Sampler) -> Self {
    self.samplers.push((binding, sampler.platform_sampler()));
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

    let textures_hold = self.textures;
    let samplers_hold = self.samplers;

    for (binding, texture_handle) in textures_hold.iter() {
      platform = platform.with_texture(*binding, texture_handle.as_ref());
    }

    for (binding, sampler_handle) in samplers_hold.iter() {
      platform = platform.with_sampler(*binding, sampler_handle.as_ref());
    }

    let group = platform.build(render_context.device());
    return BindGroup {
      group: Rc::new(group),
      dynamic_binding_count: layout.dynamic_binding_count(),
    };
  }
}
