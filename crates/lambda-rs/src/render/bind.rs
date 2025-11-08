//! Bind group layouts and bind groups for resource binding.
//!
//! Purpose
//! - Describe how shader stages access resources via `BindGroupLayout`.
//! - Create `BindGroup` instances that bind buffers to specific indices for a
//!   pipeline layout set.
//!
//! Scope and usage
//! - The engine exposes uniform buffer bindings first, with optional dynamic
//!   offsets. Storage textures and samplers may be added in the future.
//! - A layout declares binding indices and stage visibility. A bind group then
//!   provides concrete buffers for those indices. At draw time, a group is
//!   bound to a set index on the pipeline.
//! - For dynamic uniform bindings, pass one offset per dynamic binding at
//!   `SetBindGroup` time. Offsets MUST follow the device’s
//!   `min_uniform_buffer_offset_alignment`.
//!
//! See `crates/lambda-rs/examples/uniform_buffer_triangle.rs` for a complete
//! example.

use std::rc::Rc;

use super::{
  buffer::Buffer,
  texture::{
    Sampler,
    Texture,
    ViewDimension,
  },
  RenderContext,
};

/// Visibility of a binding across shader stages (engine‑facing).
///
/// Select one or more shader stages that read a bound resource. Use
/// `VertexAndFragment` for shared layouts in typical graphics pipelines.
#[derive(Clone, Copy, Debug)]
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

#[cfg(test)]
mod tests {
  use super::*;
}

/// Bind group layout used when creating pipelines and bind groups.
#[derive(Debug, Clone)]
///
/// Holds a platform layout and the number of dynamic bindings so callers can
/// validate dynamic offset counts at bind time.
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

/// Bind group that binds one or more resources to a pipeline set index.
#[derive(Debug, Clone)]
///
/// The group mirrors the structure of its `BindGroupLayout`. When using
/// dynamic uniforms, record a corresponding list of byte offsets in the
/// `RenderCommand::SetBindGroup` command.
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
///
/// Example
/// ```rust,ignore
/// // One static camera UBO at binding 0 and one dynamic model UBO at 1.
/// // Visible in both vertex and fragment stages.
/// use lambda::render::bind::{BindGroupLayoutBuilder, BindingVisibility};
/// let bgl = BindGroupLayoutBuilder::new()
///   .with_uniform(0, BindingVisibility::VertexAndFragment)
///   .with_uniform_dynamic(1, BindingVisibility::VertexAndFragment);
/// ```
pub struct BindGroupLayoutBuilder {
  label: Option<String>,
  entries: Vec<(u32, BindingVisibility, bool)>,
  textures_2d: Vec<(u32, BindingVisibility)>,
  textures_dim: Vec<(u32, BindingVisibility, ViewDimension)>,
  samplers: Vec<(u32, BindingVisibility)>,
}

impl BindGroupLayoutBuilder {
  /// Create a new builder with no bindings.
  pub fn new() -> Self {
    Self {
      label: None,
      entries: Vec::new(),
      textures_2d: Vec::new(),
      textures_dim: Vec::new(),
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

  /// Add a sampled texture binding with an explicit view dimension and visibility.
  pub fn with_sampled_texture_dim(
    mut self,
    binding: u32,
    dim: ViewDimension,
    visibility: BindingVisibility,
  ) -> Self {
    self.textures_dim.push((binding, visibility, dim));
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
      for (binding, _, _) in &self.textures_dim {
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

    for (binding, visibility, dim) in self.textures_dim.into_iter() {
      builder = builder.with_sampled_texture_dim(
        binding,
        visibility.to_platform(),
        dim.to_platform(),
      );
    }

    for (binding, visibility) in self.samplers.into_iter() {
      builder = builder.with_sampler(binding, visibility.to_platform());
    }

    let layout = builder.build(render_context.gpu());

    return BindGroupLayout {
      layout: Rc::new(layout),
      dynamic_binding_count,
    };
  }
}

/// Builder for creating a bind group for a previously built layout.
///
/// Example
/// ```rust,ignore
/// // Assume `camera_ubo` and `model_ubo` are created `UniformBuffer<T>`.
/// // Bind them to match the layout: camera at 0, model at 1 with dynamic offset.
/// use lambda::render::bind::BindGroupBuilder;
/// let group = BindGroupBuilder::new()
///   .with_layout(&layout)
///   .with_uniform(0, camera_ubo.raw(), 0, None)
///   .with_uniform(1, model_ubo.raw(), 0, None);
/// // During rendering, provide a dynamic offset for binding 1 in bytes.
/// ```
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
        if sz.get() > max_binding {
          logging::error!(
            "Uniform binding at binding={} requests size={} > device limit {}",
            binding,
            sz.get(),
            max_binding
          );
        }
        debug_assert!(
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

    let group = platform.build(render_context.gpu());
    return BindGroup {
      group: Rc::new(group),
      dynamic_binding_count: layout.dynamic_binding_count(),
    };
  }
}
