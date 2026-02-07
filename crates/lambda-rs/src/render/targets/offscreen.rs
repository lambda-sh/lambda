//! Offscreen render targets and builders.
//!
//! Provides `OffscreenTarget` and `OffscreenTargetBuilder` for render‑to‑texture
//! workflows without exposing platform texture types at call sites.

use super::surface::TextureView;
use crate::render::{
  gpu::Gpu,
  texture,
  validation,
  RenderContext,
};

#[derive(Debug)]
/// Offscreen render target with color and optional depth attachments.
///
/// An `OffscreenTarget` owns a color texture (and optional depth texture) sized
/// independently of the presentation surface. It is intended for render‑to‑
/// texture workflows such as post‑processing, shadow maps, and UI composition.
pub struct OffscreenTarget {
  resolve_color: texture::Texture,
  msaa_color: Option<texture::ColorAttachmentTexture>,
  depth: Option<texture::DepthTexture>,
  size: (u32, u32),
  color_format: texture::TextureFormat,
  depth_format: Option<texture::DepthFormat>,
  sample_count: u32,
  label: Option<String>,
}

impl OffscreenTarget {
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

  /// Access the resolve color texture for sampling.
  pub fn color_texture(&self) -> &texture::Texture {
    return &self.resolve_color;
  }

  /// Access the optional depth attachment texture.
  pub(crate) fn depth_texture(&self) -> Option<&texture::DepthTexture> {
    return self.depth.as_ref();
  }

  /// Access the multi-sampled color attachment used for rendering.
  pub fn msaa_color_texture(&self) -> Option<&texture::ColorAttachmentTexture> {
    return self.msaa_color.as_ref();
  }

  pub(crate) fn resolve_view(&self) -> TextureView<'_> {
    return self.resolve_color.view_ref();
  }

  pub(crate) fn msaa_view(&self) -> Option<TextureView<'_>> {
    return self.msaa_color.as_ref().map(|t| t.view_ref());
  }

  /// Optional debug label assigned at creation time.
  pub fn label(&self) -> Option<&str> {
    return self.label.as_deref();
  }

  /// Explicitly destroy this render target.
  ///
  /// Dropping the value also releases the underlying GPU resources; this
  /// method mirrors other engine resource destruction patterns.
  pub fn destroy(self, _render_context: &mut RenderContext) {}
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Errors returned when building an `OffscreenTarget`.
pub enum OffscreenTargetError {
  /// Color attachment was not configured.
  MissingColorAttachment,
  /// Width or height was zero.
  InvalidSize { width: u32, height: u32 },
  /// Sample count is not supported for the chosen format or device limits.
  UnsupportedSampleCount { requested: u32 },
  /// Color or depth format incompatible with render‑target usage.
  UnsupportedFormat { message: String },
  /// Device‑level failure propagated from the platform layer.
  DeviceError(String),
}

impl std::fmt::Display for OffscreenTargetError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    return match self {
      OffscreenTargetError::MissingColorAttachment => {
        write!(f, "Missing color attachment configuration")
      }
      OffscreenTargetError::InvalidSize { width, height } => write!(
        f,
        "Invalid offscreen target size {}x{} (width and height must be > 0)",
        width, height
      ),
      OffscreenTargetError::UnsupportedSampleCount { requested } => {
        write!(
          f,
          "Unsupported sample count {} (allowed: 1, 2, 4, 8)",
          requested
        )
      }
      OffscreenTargetError::UnsupportedFormat { message } => {
        write!(f, "Unsupported format: {}", message)
      }
      OffscreenTargetError::DeviceError(message) => {
        write!(f, "Device error: {}", message)
      }
    };
  }
}

impl std::error::Error for OffscreenTargetError {}

/// Builder for creating an `OffscreenTarget`.
pub struct OffscreenTargetBuilder {
  label: Option<String>,
  color_format: Option<texture::TextureFormat>,
  width: u32,
  height: u32,
  depth_format: Option<texture::DepthFormat>,
  sample_count: u32,
}

impl Default for OffscreenTargetBuilder {
  fn default() -> Self {
    return Self::new();
  }
}

impl OffscreenTargetBuilder {
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
  pub fn with_multi_sample(mut self, samples: u32) -> Self {
    self.sample_count = samples.max(1);
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
    gpu: &Gpu,
  ) -> Result<OffscreenTarget, OffscreenTargetError> {
    let format = match self.color_format {
      Some(format) => format,
      None => return Err(OffscreenTargetError::MissingColorAttachment),
    };

    let (width, height) = self.resolve_size()?;

    let sample_count = self.sample_count.max(1);
    if validation::validate_sample_count(sample_count).is_err() {
      return Err(OffscreenTargetError::UnsupportedSampleCount {
        requested: sample_count,
      });
    }

    if sample_count > 1
      && !gpu.supports_sample_count_for_format(format, sample_count)
    {
      return Err(OffscreenTargetError::UnsupportedSampleCount {
        requested: sample_count,
      });
    }

    if let Some(depth_format) = self.depth_format {
      if sample_count > 1
        && !gpu.supports_sample_count_for_depth(depth_format, sample_count)
      {
        return Err(OffscreenTargetError::UnsupportedSampleCount {
          requested: sample_count,
        });
      }
    }

    let mut color_builder = texture::TextureBuilder::new_2d(format)
      .with_size(width, height)
      .for_render_target();

    if let Some(ref label) = self.label {
      if sample_count > 1 {
        color_builder = color_builder.with_label(&format!("{}-resolve", label));
      } else {
        color_builder = color_builder.with_label(label);
      }
    }

    let resolve_texture = match color_builder.build(gpu) {
      Ok(texture) => texture,
      Err(message) => {
        const UNSUPPORTED_FORMAT_MESSAGE: &str =
          "Texture format does not support bytes_per_pixel calculation";
        let error_message = message.to_string();
        if message == UNSUPPORTED_FORMAT_MESSAGE {
          return Err(OffscreenTargetError::UnsupportedFormat {
            message: error_message,
          });
        }

        let label = self.label.as_deref().unwrap_or("unnamed offscreen target");
        return Err(OffscreenTargetError::DeviceError(format!(
          "Failed to build resolve color texture for '{}': {}",
          label, error_message
        )));
      }
    };

    let msaa_texture = if sample_count > 1 {
      let mut msaa_builder =
        texture::ColorAttachmentTextureBuilder::new(format)
          .with_size(width, height)
          .with_sample_count(sample_count);
      if let Some(ref label) = self.label {
        msaa_builder = msaa_builder.with_label(&format!("{}-msaa", label));
      }
      Some(msaa_builder.build(gpu))
    } else {
      None
    };

    let depth_texture = if let Some(depth_format) = self.depth_format {
      let mut depth_builder = texture::DepthTextureBuilder::new()
        .with_size(width, height)
        .with_format(depth_format)
        .with_sample_count(sample_count);

      if let Some(ref label) = self.label {
        depth_builder = depth_builder.with_label(&format!("{}-depth", label));
      }

      Some(depth_builder.build(gpu))
    } else {
      None
    };

    return Ok(OffscreenTarget {
      resolve_color: resolve_texture,
      msaa_color: msaa_texture,
      depth: depth_texture,
      size: (width, height),
      color_format: format,
      depth_format: self.depth_format,
      sample_count,
      label: self.label,
    });
  }

  pub(crate) fn resolve_size(
    &self,
  ) -> Result<(u32, u32), OffscreenTargetError> {
    let width = self.width;
    let height = self.height;
    if width == 0 || height == 0 {
      return Err(OffscreenTargetError::InvalidSize { width, height });
    }

    return Ok((width, height));
  }
}

#[cfg(test)]
mod tests {
  use lambda_platform::wgpu as platform;

  use super::*;
  use crate::render::{
    gpu::GpuBuilder,
    instance::InstanceBuilder,
  };

  /// Fails when the builder has a zero dimension.
  #[test]
  fn resolve_size_rejects_zero_dimensions() {
    let builder = OffscreenTargetBuilder::new().with_color(
      texture::TextureFormat::Rgba8Unorm,
      0,
      0,
    );

    let resolved = builder.resolve_size();
    assert_eq!(
      resolved,
      Err(OffscreenTargetError::InvalidSize {
        width: 0,
        height: 0
      })
    );
  }

  /// Clamps sample counts less than one to one.
  #[test]
  fn sample_count_is_clamped_to_one() {
    let builder = OffscreenTargetBuilder::new().with_multi_sample(0);
    assert_eq!(builder.sample_count, 1);
  }

  fn create_test_gpu() -> Option<Gpu> {
    let instance = InstanceBuilder::new()
      .with_label("lambda-offscreen-target-test-instance")
      .build();
    let built = GpuBuilder::new()
      .with_label("lambda-offscreen-target-test-gpu")
      .build(&instance, None)
      .ok();
    if built.is_some() {
      return built;
    }

    let fallback = GpuBuilder::new()
      .with_label("lambda-offscreen-target-test-gpu-fallback")
      .force_fallback(true)
      .build(&instance, None)
      .ok();

    if fallback.is_none() && crate::render::gpu::require_gpu_adapter_for_tests()
    {
      panic!("No GPU adapter available for tests (set LAMBDA_REQUIRE_GPU_ADAPTER=0 to allow skipping)");
    }

    return fallback;
  }

  /// Ensures the builder rejects attempts to build without configuring a color
  /// attachment.
  #[test]
  fn build_rejects_missing_color_attachment() {
    let Some(gpu) = create_test_gpu() else {
      return;
    };

    let built = OffscreenTargetBuilder::new().build(&gpu);
    assert_eq!(
      built.unwrap_err(),
      OffscreenTargetError::MissingColorAttachment
    );
  }

  /// Ensures unsupported MSAA sample counts are rejected with an explicit
  /// error rather than silently falling back.
  #[test]
  fn build_rejects_unsupported_sample_count() {
    let Some(gpu) = create_test_gpu() else {
      return;
    };

    let built = OffscreenTargetBuilder::new()
      .with_color(texture::TextureFormat::Rgba8Unorm, 1, 1)
      .with_multi_sample(3)
      .build(&gpu);

    assert_eq!(
      built.unwrap_err(),
      OffscreenTargetError::UnsupportedSampleCount { requested: 3 }
    );
  }

  /// Ensures the resolve texture can be bound for sampling and also used as a
  /// render attachment (required for render-to-texture workflows).
  #[test]
  fn resolve_texture_supports_sampling_and_render_attachment() {
    let Some(gpu) = create_test_gpu() else {
      return;
    };

    let target = OffscreenTargetBuilder::new()
      .with_color(texture::TextureFormat::Rgba8Unorm, 4, 4)
      .with_label("offscreen-usage-test")
      .build(&gpu)
      .expect("build offscreen target");

    let resolve_platform_texture = target.color_texture().platform_texture();

    let sampler = platform::texture::SamplerBuilder::new()
      .nearest_clamp()
      .with_label("offscreen-usage-sampler")
      .build(gpu.platform());

    let layout = platform::bind::BindGroupLayoutBuilder::new()
      .with_sampled_texture_2d(1, platform::bind::Visibility::Fragment)
      .with_sampler(2, platform::bind::Visibility::Fragment)
      .build(gpu.platform());

    let _group = platform::bind::BindGroupBuilder::new()
      .with_layout(&layout)
      .with_texture(1, resolve_platform_texture.as_ref())
      .with_sampler(2, &sampler)
      .build(gpu.platform());

    let mut encoder = platform::command::CommandEncoder::new(
      gpu.platform(),
      Some("offscreen-usage-encoder"),
    );
    {
      let mut attachments =
        platform::render_pass::RenderColorAttachments::new();
      attachments.push_color(target.resolve_view().to_platform());
      let _pass = platform::render_pass::RenderPassBuilder::new()
        .with_clear_color([0.0, 0.0, 0.0, 1.0])
        .build(&mut encoder, &mut attachments, None, None, None, None);
    }

    let buffer = encoder.finish();
    gpu.platform().submit(std::iter::once(buffer));
  }

  /// Ensures MSAA offscreen targets use compatible sample counts across color
  /// and depth attachments so they can be encoded into a single render pass.
  #[test]
  fn msaa_target_depth_attachment_matches_sample_count() {
    let Some(gpu) = create_test_gpu() else {
      return;
    };

    let target = OffscreenTargetBuilder::new()
      .with_color(texture::TextureFormat::Rgba8Unorm, 4, 4)
      .with_depth(texture::DepthFormat::Depth32Float)
      .with_multi_sample(4)
      .with_label("offscreen-msaa-depth-test")
      .build(&gpu)
      .expect("build offscreen target");

    let msaa_view = target.msaa_view().expect("MSAA view");
    let resolve_view = target.resolve_view();
    let depth_view = target
      .depth_texture()
      .expect("depth texture")
      .platform_view_ref();

    let mut encoder = platform::command::CommandEncoder::new(
      gpu.platform(),
      Some("offscreen-msaa-depth-encoder"),
    );
    {
      let mut attachments =
        platform::render_pass::RenderColorAttachments::new();
      attachments
        .push_msaa_color(msaa_view.to_platform(), resolve_view.to_platform());
      let depth_ops = Some(platform::render_pass::DepthOperations {
        load: platform::render_pass::DepthLoadOp::Clear(1.0),
        store: platform::render_pass::StoreOp::Store,
      });
      let _pass = platform::render_pass::RenderPassBuilder::new()
        .with_clear_color([0.0, 0.0, 0.0, 1.0])
        .build(
          &mut encoder,
          &mut attachments,
          Some(depth_view),
          depth_ops,
          None,
          None,
        );
    }

    let buffer = encoder.finish();
    gpu.platform().submit(std::iter::once(buffer));
  }
}
