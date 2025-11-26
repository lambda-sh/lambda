//! Offscreen render targets and builders.
//!
//! Provides `RenderTarget` and `RenderTargetBuilder` for render‑to‑texture
//! workflows without exposing platform texture types at call sites.

use logging;

use super::{
  texture,
  RenderContext,
};
use crate::render::validation;

#[derive(Debug, Clone)]
/// Offscreen render target with color and optional depth attachments.
///
/// A `RenderTarget` owns a color texture (and optional depth texture) sized
/// independently of the presentation surface. It is intended for render‑to‑
/// texture workflows such as post‑processing, shadow maps, and UI composition.
pub struct RenderTarget {
  color: texture::Texture,
  depth: Option<texture::DepthTexture>,
  size: (u32, u32),
  color_format: texture::TextureFormat,
  depth_format: Option<texture::DepthFormat>,
  sample_count: u32,
  label: Option<String>,
}

impl RenderTarget {
  /// Texture size in pixels.
  pub fn size(&self) -> (u32, u32) {
    return self.size;
  }

  /// Color format of the render target.
  pub fn color_format(&self) -> texture::TextureFormat {
    return self.color_format;
  }

  /// Optional depth format configured for this target.
  pub fn depth_format(&self) -> Option<texture::DepthFormat> {
    return self.depth_format;
  }

  /// Multi‑sample count configured for this target. Always at least `1`.
  pub fn sample_count(&self) -> u32 {
    return self.sample_count.max(1);
  }

  /// Access the color attachment texture for sampling.
  pub fn color_texture(&self) -> &texture::Texture {
    return &self.color;
  }

  /// Access the optional depth attachment texture.
  pub(crate) fn depth_texture(&self) -> Option<&texture::DepthTexture> {
    return self.depth.as_ref();
  }

  /// Optional debug label assigned at creation time.
  pub(crate) fn label(&self) -> Option<&str> {
    return self.label.as_deref();
  }

  /// Explicitly destroy this render target.
  ///
  /// Dropping the value also releases the underlying GPU resources; this
  /// method mirrors other engine resource destruction patterns.
  pub fn destroy(self, _render_context: &mut RenderContext) {}
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Errors returned when building a `RenderTarget`.
pub enum RenderTargetError {
  /// Color attachment was not configured.
  MissingColorAttachment,
  /// Width or height was zero after resolving defaults.
  InvalidSize { width: u32, height: u32 },
  /// Sample count is not supported for the chosen format or device limits.
  UnsupportedSampleCount { requested: u32 },
  /// Color or depth format incompatible with render‑target usage.
  UnsupportedFormat { message: String },
  /// Device‑level failure propagated from the platform layer.
  DeviceError(String),
}

/// Builder for creating a `RenderTarget`.
pub struct RenderTargetBuilder {
  label: Option<String>,
  color_format: Option<texture::TextureFormat>,
  width: u32,
  height: u32,
  depth_format: Option<texture::DepthFormat>,
  sample_count: u32,
}

impl RenderTargetBuilder {
  /// Create a new builder with no attachments configured.
  pub fn new() -> Self {
    return Self {
      label: None,
      color_format: None,
      width: 0,
      height: 0,
      depth_format: None,
      sample_count: 1,
    };
  }

  /// Configure the color attachment format and size.
  ///
  /// When `width` or `height` is zero, the builder falls back to the current
  /// `RenderContext` surface size during `build`. A resolved size of zero in
  /// either dimension is treated as an error.
  pub fn with_color(
    mut self,
    format: texture::TextureFormat,
    width: u32,
    height: u32,
  ) -> Self {
    self.color_format = Some(format);
    self.width = width;
    self.height = height;
    return self;
  }

  /// Configure an optional depth attachment for this target.
  pub fn with_depth(mut self, format: texture::DepthFormat) -> Self {
    self.depth_format = Some(format);
    return self;
  }

  /// Configure multi‑sampling for this target.
  ///
  /// Values outside the supported set `{1, 2, 4, 8}` fall back to `1` and
  /// emit validation logs under `render-validation-msaa` or debug assertions.
  pub fn with_multi_sample(mut self, samples: u32) -> Self {
    let allowed = matches!(samples, 1 | 2 | 4 | 8);
    if allowed {
      self.sample_count = samples;
    } else {
      #[cfg(any(debug_assertions, feature = "render-validation-msaa",))]
      {
        if let Err(message) = validation::validate_sample_count(samples) {
          logging::error!(
            "{}; falling back to sample_count=1 for render target",
            message
          );
        }
      }
      self.sample_count = 1;
    }
    return self;
  }

  /// Attach a debug label to the render target.
  pub fn with_label(mut self, label: &str) -> Self {
    self.label = Some(label.to_string());
    return self;
  }

  /// Create the render target color (and optional depth) attachments.
  pub fn build(
    self,
    render_context: &mut RenderContext,
  ) -> Result<RenderTarget, RenderTargetError> {
    let format = match self.color_format {
      Some(format) => format,
      None => return Err(RenderTargetError::MissingColorAttachment),
    };

    let surface_size = render_context.surface_size();
    let (width, height) = self.resolve_size(surface_size)?;

    // Clamp to at least one sample; device‑limit checks are added in a
    // validation milestone.
    let sample_count = self.sample_count.max(1);

    let mut color_builder = texture::TextureBuilder::new_2d(format)
      .with_size(width, height)
      .for_render_target();

    if let Some(ref label) = self.label {
      color_builder = color_builder.with_label(label);
    }

    let color_texture = match color_builder.build(render_context) {
      Ok(texture) => texture,
      Err(message) => {
        return Err(RenderTargetError::DeviceError(message.to_string()));
      }
    };

    let depth_texture = if let Some(depth_format) = self.depth_format {
      let mut depth_builder = texture::DepthTextureBuilder::new()
        .with_size(width, height)
        .with_format(depth_format);

      if let Some(ref label) = self.label {
        depth_builder = depth_builder.with_label(label);
      }

      Some(depth_builder.build(render_context))
    } else {
      None
    };

    return Ok(RenderTarget {
      color: color_texture,
      depth: depth_texture,
      size: (width, height),
      color_format: format,
      depth_format: self.depth_format,
      sample_count,
      label: self.label,
    });
  }

  /// Resolve the final size using an optional explicit size and surface default.
  ///
  /// When no explicit size was provided, the builder falls back to
  /// `surface_size`. A resolved size with zero width or height is treated as
  /// an error.
  pub(crate) fn resolve_size(
    &self,
    surface_size: (u32, u32),
  ) -> Result<(u32, u32), RenderTargetError> {
    let mut width = self.width;
    let mut height = self.height;
    if width == 0 || height == 0 {
      width = surface_size.0;
      height = surface_size.1;
    }

    if width == 0 || height == 0 {
      return Err(RenderTargetError::InvalidSize { width, height });
    }

    return Ok((width, height));
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  /// Defaults size to the surface when no explicit dimensions are provided.
  #[test]
  fn resolve_size_defaults_to_surface_size() {
    let builder = RenderTargetBuilder::new().with_color(
      texture::TextureFormat::Rgba8Unorm,
      0,
      0,
    );
    let surface_size = (800, 600);

    let resolved = builder.resolve_size(surface_size).unwrap();
    assert_eq!(resolved, surface_size);
  }

  /// Fails when the resolved size has a zero dimension.
  #[test]
  fn resolve_size_rejects_zero_dimensions() {
    let builder = RenderTargetBuilder::new().with_color(
      texture::TextureFormat::Rgba8Unorm,
      0,
      0,
    );
    let surface_size = (0, 0);

    let resolved = builder.resolve_size(surface_size);
    assert_eq!(
      resolved,
      Err(RenderTargetError::InvalidSize {
        width: 0,
        height: 0
      })
    );
  }

  /// Clamps sample counts less than one to one.
  #[test]
  fn sample_count_is_clamped_to_one() {
    let builder = RenderTargetBuilder::new().with_multi_sample(0);
    assert_eq!(builder.sample_count, 1);
  }
}
