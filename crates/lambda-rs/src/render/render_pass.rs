//! Render pass descriptions used to clear and begin drawing.
//!
//! A `RenderPass` captures immutable parameters used when beginning a pass
//! against the swapchain (currently a single color attachment and clear color).
//! The pass is referenced by handle from `RenderCommand::BeginRenderPass`.

use logging;

use super::RenderContext;
use crate::render::validation;

/// Color load operation for the first color attachment.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColorLoadOp {
  /// Load existing contents.
  Load,
  /// Clear to the provided RGBA color.
  Clear([f64; 4]),
}

/// Store operation for the first color attachment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoreOp {
  /// Store results at the end of the pass.
  Store,
  /// Discard results when possible.
  Discard,
}

/// Combined color operations for the first color attachment.
#[derive(Debug, Clone, Copy, PartialEq)]
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

/// Depth load operation for the depth attachment.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DepthLoadOp {
  /// Load existing depth.
  Load,
  /// Clear to the provided depth value in [0,1].
  Clear(f64),
}

/// Depth operations for the first depth attachment.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DepthOperations {
  pub load: DepthLoadOp,
  pub store: StoreOp,
}

impl Default for DepthOperations {
  fn default() -> Self {
    return Self {
      load: DepthLoadOp::Clear(1.0),
      store: StoreOp::Store,
    };
  }
}

/// Immutable parameters used when beginning a render pass.
#[derive(Debug, Clone)]
///
/// The pass defines the initial clear for the color attachment and an optional
/// label. Depth/stencil may be added in a future iteration.
pub struct RenderPass {
  clear_color: [f64; 4],
  label: Option<String>,
  color_operations: ColorOperations,
  depth_operations: Option<DepthOperations>,
  stencil_operations: Option<StencilOperations>,
  sample_count: u32,
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

  pub(crate) fn depth_operations(&self) -> Option<DepthOperations> {
    return self.depth_operations;
  }

  pub(crate) fn sample_count(&self) -> u32 {
    return self.sample_count.max(1);
  }

  pub(crate) fn stencil_operations(&self) -> Option<StencilOperations> {
    return self.stencil_operations;
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
  depth_operations: Option<DepthOperations>,
  stencil_operations: Option<StencilOperations>,
  sample_count: u32,
}

impl RenderPassBuilder {
  /// Creates a new render pass builder.
  pub fn new() -> Self {
    return Self {
      clear_color: [0.0, 0.0, 0.0, 1.0],
      label: None,
      color_operations: ColorOperations::default(),
      depth_operations: None,
      stencil_operations: None,
      sample_count: 1,
    };
  }

  /// Specify the clear color used for the first color attachment.
  pub fn with_clear_color(mut self, color: [f64; 4]) -> Self {
    self.clear_color = color;
    self.color_operations = ColorOperations {
      load: ColorLoadOp::Clear(color),
      store: StoreOp::Store,
    };
    return self;
  }

  /// Attach a label to the render pass for debugging/profiling.
  pub fn with_label(mut self, label: &str) -> Self {
    self.label = Some(label.to_string());
    return self;
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

  /// Enable a depth attachment with default clear to 1.0 and store.
  pub fn with_depth(mut self) -> Self {
    self.depth_operations = Some(DepthOperations::default());
    return self;
  }

  /// Enable a depth attachment with an explicit clear value.
  pub fn with_depth_clear(mut self, clear: f64) -> Self {
    self.depth_operations = Some(DepthOperations {
      load: DepthLoadOp::Clear(clear),
      store: StoreOp::Store,
    });
    return self;
  }

  /// Enable a stencil attachment with default clear to 0 and store.
  pub fn with_stencil(mut self) -> Self {
    self.stencil_operations = Some(StencilOperations::default());
    return self;
  }

  /// Enable a stencil attachment with an explicit clear value.
  pub fn with_stencil_clear(mut self, clear: u32) -> Self {
    self.stencil_operations = Some(StencilOperations {
      load: StencilLoadOp::Clear(clear),
      store: StoreOp::Store,
    });
    return self;
  }

  /// Configure multi-sample anti-aliasing for this pass.
  pub fn with_multi_sample(mut self, samples: u32) -> Self {
    match validation::validate_sample_count(samples) {
      Ok(()) => {
        self.sample_count = samples;
      }
      Err(msg) => {
        logging::error!(
          "{}; falling back to sample_count=1 for render pass",
          msg
        );
        self.sample_count = 1;
      }
    }
    return self;
  }

  /// Build the description used when beginning a render pass.
  pub fn build(self, _render_context: &RenderContext) -> RenderPass {
    RenderPass {
      clear_color: self.clear_color,
      label: self.label,
      color_operations: self.color_operations,
      depth_operations: self.depth_operations,
      stencil_operations: self.stencil_operations,
      sample_count: self.sample_count,
    }
  }
}

/// Stencil load operation for the stencil attachment.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StencilLoadOp {
  /// Load existing stencil value.
  Load,
  /// Clear stencil to the provided value.
  Clear(u32),
}

/// Stencil operations for the first stencil attachment.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StencilOperations {
  pub load: StencilLoadOp,
  pub store: StoreOp,
}

impl Default for StencilOperations {
  fn default() -> Self {
    return Self {
      load: StencilLoadOp::Clear(0),
      store: StoreOp::Store,
    };
  }
}
