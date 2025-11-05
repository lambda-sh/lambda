//! Render pass descriptions used to clear and begin drawing.
//!
//! A `RenderPass` captures immutable parameters used when beginning a pass
//! against the swapchain (currently a single color attachment and clear color).
//! The pass is referenced by handle from `RenderCommand::BeginRenderPass`.

use super::RenderContext;

#[derive(Debug, Clone, Copy, PartialEq)]
/// Color load operation for the first color attachment.
pub enum ColorLoadOp {
  /// Load existing contents.
  Load,
  /// Clear to the provided RGBA color.
  Clear([f64; 4]),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Store operation for the first color attachment.
pub enum StoreOp {
  /// Store results at the end of the pass.
  Store,
  /// Discard results when possible.
  Discard,
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// Combined color operations for the first color attachment.
pub struct ColorOperations {
  pub load: ColorLoadOp,
  pub store: StoreOp,
}

impl Default for ColorOperations {
  fn default() -> Self {
    return Self {
      load: ColorLoadOp::Clear([0.0, 0.0, 0.0, 1.0]),
      store: StoreOp::Store,
    };
  }
}

#[derive(Debug, Clone)]
/// Immutable parameters used when beginning a render pass.
///
/// The pass defines the initial clear for the color attachment and an optional
/// label. Depth/stencil may be added in a future iteration.
pub struct RenderPass {
  clear_color: [f64; 4],
  label: Option<String>,
  color_operations: ColorOperations,
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

  pub(crate) fn color_operations(&self) -> ColorOperations {
    return self.color_operations;
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
  color_operations: ColorOperations,
}

impl RenderPassBuilder {
  /// Creates a new render pass builder.
  pub fn new() -> Self {
    Self {
      clear_color: [0.0, 0.0, 0.0, 1.0],
      label: None,
      color_operations: ColorOperations::default(),
    }
  }

  /// Specify the clear color used for the first color attachment.
  pub fn with_clear_color(mut self, color: [f64; 4]) -> Self {
    self.clear_color = color;
    self.color_operations = ColorOperations {
      load: ColorLoadOp::Clear(color),
      store: StoreOp::Store,
    };
    self
  }

  /// Attach a label to the render pass for debugging/profiling.
  pub fn with_label(mut self, label: &str) -> Self {
    self.label = Some(label.to_string());
    self
  }

  /// Specify the color load operation for the first color attachment.
  pub fn with_color_load_op(mut self, load: ColorLoadOp) -> Self {
    self.color_operations.load = load;
    if let ColorLoadOp::Clear(color) = load {
      self.clear_color = color;
    }
    return self;
  }

  /// Specify the color store operation for the first color attachment.
  pub fn with_store_op(mut self, store: StoreOp) -> Self {
    self.color_operations.store = store;
    return self;
  }

  /// Provide combined color operations for the first color attachment.
  pub fn with_color_operations(mut self, operations: ColorOperations) -> Self {
    self.color_operations = operations;
    if let ColorLoadOp::Clear(color) = operations.load {
      self.clear_color = color;
    }
    return self;
  }

  /// Build the description used when beginning a render pass.
  pub fn build(self, _render_context: &RenderContext) -> RenderPass {
    RenderPass {
      clear_color: self.clear_color,
      label: self.label,
      color_operations: self.color_operations,
    }
  }
}
