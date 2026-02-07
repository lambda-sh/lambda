//! Render pass descriptions used to clear and begin drawing.
//!
//! A `RenderPass` captures immutable parameters used when beginning a pass
//! against the swapchain. A pass MAY omit color attachments entirely to
//! perform depth/stencil-only operations (e.g., stencil mask pre-pass).
//! The pass is referenced by handle from `RenderCommand::BeginRenderPass`.

use lambda_platform::wgpu as platform;
use logging;

use super::{
  gpu::Gpu,
  texture,
};
use crate::render::validation;

/// Color load operation for the first color attachment.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColorLoadOp {
  /// Load existing contents.
  Load,
  /// Clear to the provided RGBA color.
  Clear([f64; 4]),
}

impl ColorLoadOp {
  /// Convert to the platform color load operation.
  pub(crate) fn to_platform(self) -> platform::render_pass::ColorLoadOp {
    return match self {
      ColorLoadOp::Load => platform::render_pass::ColorLoadOp::Load,
      ColorLoadOp::Clear(color) => {
        platform::render_pass::ColorLoadOp::Clear(color)
      }
    };
  }
}

/// Store operation for the first color attachment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoreOp {
  /// Store results at the end of the pass.
  Store,
  /// Discard results when possible.
  Discard,
}

impl StoreOp {
  /// Convert to the platform store operation.
  pub(crate) fn to_platform(self) -> platform::render_pass::StoreOp {
    return match self {
      StoreOp::Store => platform::render_pass::StoreOp::Store,
      StoreOp::Discard => platform::render_pass::StoreOp::Discard,
    };
  }
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

impl ColorOperations {
  /// Convert to the platform color load and store operations.
  pub(crate) fn to_platform(
    self,
  ) -> (
    platform::render_pass::ColorLoadOp,
    platform::render_pass::StoreOp,
  ) {
    return (self.load.to_platform(), self.store.to_platform());
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

impl DepthLoadOp {
  /// Convert to the platform depth load operation.
  pub(crate) fn to_platform(self) -> platform::render_pass::DepthLoadOp {
    return match self {
      DepthLoadOp::Load => platform::render_pass::DepthLoadOp::Load,
      DepthLoadOp::Clear(value) => {
        platform::render_pass::DepthLoadOp::Clear(value as f32)
      }
    };
  }
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

impl DepthOperations {
  /// Convert to the platform depth operations.
  pub(crate) fn to_platform(self) -> platform::render_pass::DepthOperations {
    return platform::render_pass::DepthOperations {
      load: self.load.to_platform(),
      store: self.store.to_platform(),
    };
  }
}

/// Immutable parameters used when beginning a render pass.
#[derive(Debug, Clone)]
///
/// The pass defines the initial clear for the color attachment and an optional
/// label. Depth/stencil may be added in a future iteration.
pub struct RenderPass {
  label: Option<String>,
  color_operations: ColorOperations,
  depth_operations: Option<DepthOperations>,
  stencil_operations: Option<StencilOperations>,
  sample_count: u32,
  use_color: bool,
}

impl RenderPass {
  /// Destroy the pass. Kept for symmetry with other resources.
  pub fn destroy(self, _gpu: &Gpu) {}

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

  /// Whether this pass declares any color attachments.
  pub(crate) fn uses_color(&self) -> bool {
    return self.use_color;
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
  use_color: bool,
}

impl Default for RenderPassBuilder {
  fn default() -> Self {
    return Self::new();
  }
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
      use_color: true,
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

  /// Disable color attachments for this pass. Depth/stencil MAY still be used.
  pub fn without_color(mut self) -> Self {
    self.use_color = false;
    return self;
  }

  /// Enable a depth attachment with default clear to 1.0 and store.
  pub fn with_depth(mut self) -> Self {
    self.depth_operations = Some(DepthOperations::default());
    return self;
  }

  /// Enable a depth attachment with an explicit clear value.
  pub fn with_depth_clear(mut self, clear: f64) -> Self {
    // Clamp to the valid range [0.0, 1.0] unconditionally.
    let clamped = clear.clamp(0.0, 1.0);
    // Optionally log when clamping is applied.
    #[cfg(any(debug_assertions, feature = "render-validation-depth",))]
    {
      if clamped != clear {
        logging::warn!(
          "Depth clear value {} out of range [0,1]; clamped to {}",
          clear,
          clamped
        );
      }
    }
    self.depth_operations = Some(DepthOperations {
      load: DepthLoadOp::Clear(clamped),
      store: StoreOp::Store,
    });
    return self;
  }

  /// Use a depth attachment and load existing contents (do not clear).
  pub fn with_depth_load(mut self) -> Self {
    self.depth_operations = Some(DepthOperations {
      load: DepthLoadOp::Load,
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

  /// Use a stencil attachment and load existing contents (do not clear).
  pub fn with_stencil_load(mut self) -> Self {
    self.stencil_operations = Some(StencilOperations {
      load: StencilLoadOp::Load,
      store: StoreOp::Store,
    });
    return self;
  }

  /// Configure multi-sample anti-aliasing for this pass.
  pub fn with_multi_sample(mut self, samples: u32) -> Self {
    // Always apply a cheap validity check; log under feature/debug gates.
    let allowed = matches!(samples, 1 | 2 | 4 | 8);
    if allowed {
      self.sample_count = samples;
    } else {
      #[cfg(any(debug_assertions, feature = "render-validation-msaa",))]
      {
        if let Err(msg) = validation::validate_sample_count(samples) {
          logging::error!(
            "{}; falling back to sample_count=1 for render pass",
            msg
          );
        }
      }
      self.sample_count = 1;
    }
    return self;
  }

  /// Build the description used when beginning a render pass.
  ///
  /// # Arguments
  /// * `gpu` - The GPU device for sample count validation.
  /// * `surface_format` - The surface texture format for sample count validation.
  /// * `depth_format` - The depth texture format for sample count validation.
  pub fn build(
    self,
    gpu: &Gpu,
    surface_format: texture::TextureFormat,
    depth_format: texture::DepthFormat,
  ) -> RenderPass {
    let sample_count = self.resolve_sample_count(
      self.sample_count,
      surface_format.to_platform(),
      depth_format,
      |count| gpu.supports_sample_count_for_format(surface_format, count),
      |format, count| gpu.supports_sample_count_for_depth(format, count),
    );

    return RenderPass {
      label: self.label,
      color_operations: self.color_operations,
      depth_operations: self.depth_operations,
      stencil_operations: self.stencil_operations,
      sample_count,
      use_color: self.use_color,
    };
  }

  /// Validate the requested sample count against surface and depth/stencil
  /// capabilities, falling back to `1` when unsupported.
  fn resolve_sample_count<FSurface, FDepth>(
    &self,
    sample_count: u32,
    surface_format: platform::texture::TextureFormat,
    depth_format: texture::DepthFormat,
    supports_surface: FSurface,
    supports_depth: FDepth,
  ) -> u32
  where
    FSurface: Fn(u32) -> bool,
    FDepth: Fn(texture::DepthFormat, u32) -> bool,
  {
    let mut resolved_sample_count = sample_count.max(1);

    if self.use_color
      && resolved_sample_count > 1
      && !supports_surface(resolved_sample_count)
    {
      #[cfg(any(debug_assertions, feature = "render-validation-device",))]
      logging::error!(
        "Sample count {} unsupported for surface format {:?}; falling back to 1",
        resolved_sample_count,
        surface_format
      );
      resolved_sample_count = 1;
    }

    let wants_depth_or_stencil =
      self.depth_operations.is_some() || self.stencil_operations.is_some();
    if wants_depth_or_stencil && resolved_sample_count > 1 {
      let validated_depth_format = if self.stencil_operations.is_some() {
        texture::DepthFormat::Depth24PlusStencil8
      } else {
        depth_format
      };
      if !supports_depth(validated_depth_format, resolved_sample_count) {
        #[cfg(any(debug_assertions, feature = "render-validation-device",))]
        logging::error!(
          "Sample count {} unsupported for depth format {:?}; falling back to 1",
          resolved_sample_count,
          validated_depth_format
        );
        resolved_sample_count = 1;
      }
    }

    return resolved_sample_count;
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

impl StencilLoadOp {
  /// Convert to the platform stencil load operation.
  pub(crate) fn to_platform(self) -> platform::render_pass::StencilLoadOp {
    return match self {
      StencilLoadOp::Load => platform::render_pass::StencilLoadOp::Load,
      StencilLoadOp::Clear(value) => {
        platform::render_pass::StencilLoadOp::Clear(value)
      }
    };
  }
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

impl StencilOperations {
  /// Convert to the platform stencil operations.
  pub(crate) fn to_platform(self) -> platform::render_pass::StencilOperations {
    return platform::render_pass::StencilOperations {
      load: self.load.to_platform(),
      store: self.store.to_platform(),
    };
  }
}

#[cfg(test)]
mod tests {
  use std::cell::RefCell;

  use super::*;

  fn surface_format() -> platform::texture::TextureFormat {
    return platform::texture::TextureFormat::BGRA8_UNORM_SRGB;
  }

  /// Falls back when the surface format rejects the requested sample count.
  #[test]
  fn unsupported_surface_sample_count_falls_back_to_one() {
    let builder = RenderPassBuilder::new().with_multi_sample(4);

    let resolved = builder.resolve_sample_count(
      4,
      surface_format(),
      texture::DepthFormat::Depth32Float,
      |_samples| {
        return false;
      },
      |_format, _samples| {
        return true;
      },
    );

    assert_eq!(resolved, 1);
  }

  /// Falls back when the depth format rejects the requested sample count.
  #[test]
  fn unsupported_depth_sample_count_falls_back_to_one() {
    let builder = RenderPassBuilder::new().with_depth().with_multi_sample(8);

    let resolved = builder.resolve_sample_count(
      8,
      surface_format(),
      texture::DepthFormat::Depth32Float,
      |_samples| {
        return true;
      },
      |_format, _samples| {
        return false;
      },
    );

    assert_eq!(resolved, 1);
  }

  /// Uses a stencil-capable depth format when stencil operations are present.
  #[test]
  fn stencil_support_uses_stencil_capable_depth_format() {
    let builder = RenderPassBuilder::new().with_stencil().with_multi_sample(2);
    let requested_formats: RefCell<Vec<texture::DepthFormat>> =
      RefCell::new(Vec::new());

    let resolved = builder.resolve_sample_count(
      2,
      surface_format(),
      texture::DepthFormat::Depth32Float,
      |_samples| {
        return true;
      },
      |format, _samples| {
        requested_formats.borrow_mut().push(format);
        return true;
      },
    );

    assert_eq!(resolved, 2);
    assert_eq!(
      requested_formats.borrow().first().copied(),
      Some(texture::DepthFormat::Depth24PlusStencil8)
    );
  }

  /// Preserves supported sample counts when color and depth permit them.
  #[test]
  fn supported_sample_count_is_preserved() {
    let builder = RenderPassBuilder::new().with_depth().with_multi_sample(4);

    let resolved = builder.resolve_sample_count(
      4,
      surface_format(),
      texture::DepthFormat::Depth32Float,
      |_samples| {
        return true;
      },
      |_format, _samples| {
        return true;
      },
    );

    assert_eq!(resolved, 4);
  }

  /// Clamps a zero sample count to one before validation.
  #[test]
  fn zero_sample_count_is_clamped_to_one() {
    let builder = RenderPassBuilder::new().without_color();

    let resolved = builder.resolve_sample_count(
      0,
      surface_format(),
      texture::DepthFormat::Depth32Float,
      |_samples| {
        return true;
      },
      |_format, _samples| {
        return true;
      },
    );

    assert_eq!(resolved, 1);
  }

  #[test]
  fn with_depth_clear_clamps_to_unit_interval() {
    let builder = RenderPassBuilder::new().with_depth_clear(2.0);
    let depth_ops = builder.depth_operations.expect("depth ops");
    assert!(
      matches!(depth_ops.load, DepthLoadOp::Clear(v) if (v - 1.0).abs() < f64::EPSILON)
    );

    let builder = RenderPassBuilder::new().with_depth_clear(-5.0);
    let depth_ops = builder.depth_operations.expect("depth ops");
    assert!(
      matches!(depth_ops.load, DepthLoadOp::Clear(v) if (v - 0.0).abs() < f64::EPSILON)
    );
  }

  #[test]
  fn with_multi_sample_invalid_values_fall_back_to_one() {
    let builder = RenderPassBuilder::new().with_multi_sample(3);
    assert_eq!(builder.sample_count, 1);
  }

  #[test]
  fn without_color_disables_color_attachments() {
    let builder = RenderPassBuilder::new().without_color();
    assert!(!builder.use_color);
  }

  #[test]
  fn with_color_operations_updates_clear_color() {
    let builder =
      RenderPassBuilder::new().with_color_operations(ColorOperations {
        load: ColorLoadOp::Clear([0.1, 0.2, 0.3, 0.4]),
        store: StoreOp::Store,
      });

    assert_eq!(builder.clear_color, [0.1, 0.2, 0.3, 0.4]);
  }
}
