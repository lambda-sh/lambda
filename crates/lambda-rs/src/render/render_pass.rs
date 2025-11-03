//! Render pass descriptions used to clear and begin drawing.
//!
//! A `RenderPass` captures immutable parameters used when beginning a pass
//! against the swapchain (currently a single color attachment and clear color).
//! The pass is referenced by handle from `RenderCommand::BeginRenderPass`.

use super::RenderContext;

#[derive(Debug, Clone)]
/// Immutable parameters used when beginning a render pass.
///
/// The pass defines the initial clear for the color attachment and an optional
/// label. Depth/stencil may be added in a future iteration.
pub struct RenderPass {
  clear_color: [f64; 4],
  label: Option<String>,
}

impl RenderPass {
  /// Destroy the pass. Kept for symmetry with other resources.
  pub fn destroy(self, _render_context: &RenderContext) {}

  pub(crate) fn clear_color(&self) -> [f64; 4] {
    return self.clear_color;
  }

  pub(crate) fn label(&self) -> Option<&str> {
    self.label.as_deref()
  }
}

/// Builder for a `RenderPass` description.
///
/// The default pass clears to opaque black. Attach a label and a clear color
/// as needed, then register the pass on a `RenderContext` and reference it by
/// handle in a command stream.
pub struct RenderPassBuilder {
  clear_color: [f64; 4],
  label: Option<String>,
}

impl RenderPassBuilder {
  /// Creates a new render pass builder.
  pub fn new() -> Self {
    Self {
      clear_color: [0.0, 0.0, 0.0, 1.0],
      label: None,
    }
  }

  /// Specify the clear color used for the first color attachment.
  pub fn with_clear_color(mut self, color: [f64; 4]) -> Self {
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
