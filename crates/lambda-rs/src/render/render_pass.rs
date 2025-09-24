//! Render pass builders and definitions for lambda runtimes and applications.

use lambda_platform::wgpu::types as wgpu;

use super::RenderContext;

#[derive(Debug, Clone)]
/// Immutable parameters used when beginning a render pass.
pub struct RenderPass {
  clear_color: wgpu::Color,
  label: Option<String>,
}

impl RenderPass {
  /// Destroy the pass. Kept for symmetry with other resources.
  pub fn destroy(self, _render_context: &RenderContext) {}

  pub(crate) fn color_ops(&self) -> wgpu::Operations<wgpu::Color> {
    wgpu::Operations {
      load: wgpu::LoadOp::Clear(self.clear_color),
      store: wgpu::StoreOp::Store,
    }
  }

  pub(crate) fn label(&self) -> Option<&str> {
    self.label.as_deref()
  }
}

/// Builder for a `RenderPass` description.
pub struct RenderPassBuilder {
  clear_color: wgpu::Color,
  label: Option<String>,
}

impl RenderPassBuilder {
  /// Creates a new render pass builder.
  pub fn new() -> Self {
    Self {
      clear_color: wgpu::Color::BLACK,
      label: None,
    }
  }

  /// Specify the clear color used for the first color attachment.
  pub fn with_clear_color(mut self, color: wgpu::Color) -> Self {
    self.clear_color = color;
    self
  }

  /// Attach a label to the render pass for debugging/profiling.
  pub fn with_label(mut self, label: &str) -> Self {
    self.label = Some(label.to_string());
    self
  }

  /// Build the description used when beginning a render pass.
  pub fn build(self, _render_context: &RenderContext) -> RenderPass {
    RenderPass {
      clear_color: self.clear_color,
      label: self.label,
    }
  }
}
