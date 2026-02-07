//! High‑level textures and samplers.
//!
//! Provides `TextureBuilder` and `SamplerBuilder` wrappers that delegate to the
//! platform layer and keep `wgpu` details internal to the platform crate.

use std::rc::Rc;

use lambda_platform::wgpu::texture as platform;

use super::gpu::Gpu;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Engine-level depth texture formats.
///
/// Maps to platform depth formats without exposing `wgpu` in the public API.
pub enum DepthFormat {
  Depth32Float,
  Depth24Plus,
  Depth24PlusStencil8,
}

impl DepthFormat {
  pub(crate) fn to_platform(self) -> platform::DepthFormat {
    return match self {
      DepthFormat::Depth32Float => platform::DepthFormat::Depth32Float,
      DepthFormat::Depth24Plus => platform::DepthFormat::Depth24Plus,
      DepthFormat::Depth24PlusStencil8 => {
        platform::DepthFormat::Depth24PlusStencil8
      }
    };
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Supported color texture formats for sampling and render targets.
pub enum TextureFormat {
  /// 8-bit RGBA, linear (non-sRGB).
  Rgba8Unorm,
  /// 8-bit RGBA, sRGB encoded.
  Rgba8UnormSrgb,
  /// 8-bit BGRA, linear (non-sRGB). Common swapchain format.
  Bgra8Unorm,
  /// 8-bit BGRA, sRGB encoded. Common swapchain format.
  Bgra8UnormSrgb,
}

impl TextureFormat {
  pub(crate) fn to_platform(self) -> platform::TextureFormat {
    return match self {
      TextureFormat::Rgba8Unorm => platform::TextureFormat::RGBA8_UNORM,
      TextureFormat::Rgba8UnormSrgb => {
        platform::TextureFormat::RGBA8_UNORM_SRGB
      }
      TextureFormat::Bgra8Unorm => platform::TextureFormat::BGRA8_UNORM,
      TextureFormat::Bgra8UnormSrgb => {
        platform::TextureFormat::BGRA8_UNORM_SRGB
      }
    };
  }

  pub(crate) fn from_platform(fmt: platform::TextureFormat) -> Option<Self> {
    if fmt == platform::TextureFormat::RGBA8_UNORM {
      return Some(TextureFormat::Rgba8Unorm);
    }
    if fmt == platform::TextureFormat::RGBA8_UNORM_SRGB {
      return Some(TextureFormat::Rgba8UnormSrgb);
    }
    if fmt == platform::TextureFormat::BGRA8_UNORM {
      return Some(TextureFormat::Bgra8Unorm);
    }
    if fmt == platform::TextureFormat::BGRA8_UNORM_SRGB {
      return Some(TextureFormat::Bgra8UnormSrgb);
    }
    return None;
  }

  /// Whether this format is sRGB encoded.
  pub fn is_srgb(self) -> bool {
    return matches!(
      self,
      TextureFormat::Rgba8UnormSrgb | TextureFormat::Bgra8UnormSrgb
    );
  }

  /// Number of bytes per pixel for this format.
  pub fn bytes_per_pixel(self) -> u32 {
    return 4;
  }
}

#[derive(Debug, Clone, Copy)]
/// View dimensionality exposed to shaders when sampling.
pub enum ViewDimension {
  D2,
  D3,
}

impl ViewDimension {
  pub(crate) fn to_platform(self) -> platform::ViewDimension {
    return match self {
      ViewDimension::D2 => platform::ViewDimension::TwoDimensional,
      ViewDimension::D3 => platform::ViewDimension::ThreeDimensional,
    };
  }
}

#[derive(Debug, Clone, Copy)]
/// Sampler filtering mode.
pub enum FilterMode {
  Nearest,
  Linear,
}

impl FilterMode {
  fn to_platform(self) -> platform::FilterMode {
    return match self {
      FilterMode::Nearest => platform::FilterMode::Nearest,
      FilterMode::Linear => platform::FilterMode::Linear,
    };
  }
}

#[derive(Debug, Clone, Copy)]
/// Sampler address mode.
pub enum AddressMode {
  ClampToEdge,
  Repeat,
  MirrorRepeat,
}

impl AddressMode {
  fn to_platform(self) -> platform::AddressMode {
    return match self {
      AddressMode::ClampToEdge => platform::AddressMode::ClampToEdge,
      AddressMode::Repeat => platform::AddressMode::Repeat,
      AddressMode::MirrorRepeat => platform::AddressMode::MirrorRepeat,
    };
  }
}

/// Texture usage flags.
///
/// Use bitwise-OR to combine flags when creating textures with multiple usages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextureUsages(platform::TextureUsages);

impl TextureUsages {
  /// Texture can be used as a render attachment (color or depth/stencil).
  pub const RENDER_ATTACHMENT: TextureUsages =
    TextureUsages(platform::TextureUsages::RENDER_ATTACHMENT);
  /// Texture can be bound for sampling in shaders.
  pub const TEXTURE_BINDING: TextureUsages =
    TextureUsages(platform::TextureUsages::TEXTURE_BINDING);
  /// Texture can be used as the destination of a copy operation.
  pub const COPY_DST: TextureUsages =
    TextureUsages(platform::TextureUsages::COPY_DST);
  /// Texture can be used as the source of a copy operation.
  pub const COPY_SRC: TextureUsages =
    TextureUsages(platform::TextureUsages::COPY_SRC);

  /// Create an empty flags set.
  pub const fn empty() -> Self {
    return TextureUsages(platform::TextureUsages::empty());
  }

  pub(crate) fn to_platform(self) -> platform::TextureUsages {
    return self.0;
  }

  pub(crate) fn from_platform(usage: platform::TextureUsages) -> Self {
    return TextureUsages(usage);
  }

  /// Check whether this flags set contains another set.
  pub fn contains(self, other: TextureUsages) -> bool {
    return self.0.contains(other.0);
  }
}

impl std::ops::BitOr for TextureUsages {
  type Output = TextureUsages;

  fn bitor(self, rhs: TextureUsages) -> TextureUsages {
    return TextureUsages(self.0 | rhs.0);
  }
}

impl std::ops::BitOrAssign for TextureUsages {
  fn bitor_assign(&mut self, rhs: TextureUsages) {
    self.0 |= rhs.0;
  }
}

// ---------------------------------------------------------------------------
// ColorAttachmentTexture
// ---------------------------------------------------------------------------

/// High-level wrapper for a multi-sampled color render target texture.
///
/// This type is used for MSAA color attachments and other intermediate render
/// targets that need to be resolved to a single-sample texture before
/// presentation.
#[derive(Debug)]
pub struct ColorAttachmentTexture {
  inner: platform::ColorAttachmentTexture,
}

impl ColorAttachmentTexture {
  /// Create a high-level color attachment texture from a platform texture.
  pub(crate) fn from_platform(
    texture: platform::ColorAttachmentTexture,
  ) -> Self {
    return ColorAttachmentTexture { inner: texture };
  }

  /// Borrow a texture view reference for use in render pass attachments.
  pub(crate) fn view_ref(
    &self,
  ) -> crate::render::targets::surface::TextureView<'_> {
    return crate::render::targets::surface::TextureView::from_platform(
      self.inner.view_ref(),
    );
  }
}

/// Builder for creating a color attachment texture (commonly used for MSAA).
pub struct ColorAttachmentTextureBuilder {
  label: Option<String>,
  format: TextureFormat,
  width: u32,
  height: u32,
  sample_count: u32,
}

impl ColorAttachmentTextureBuilder {
  /// Create a builder with zero size and sample count 1.
  pub fn new(format: TextureFormat) -> Self {
    return Self {
      label: None,
      format,
      width: 0,
      height: 0,
      sample_count: 1,
    };
  }

  /// Set the 2D attachment size in pixels.
  pub fn with_size(mut self, width: u32, height: u32) -> Self {
    self.width = width;
    self.height = height;
    return self;
  }

  /// Configure multisampling. Count MUST be >= 1.
  pub fn with_sample_count(mut self, count: u32) -> Self {
    self.sample_count = count.max(1);
    return self;
  }

  /// Attach a debug label for the created texture.
  pub fn with_label(mut self, label: &str) -> Self {
    self.label = Some(label.to_string());
    return self;
  }

  /// Create the color attachment texture on the device.
  pub fn build(self, gpu: &Gpu) -> ColorAttachmentTexture {
    let mut builder =
      platform::ColorAttachmentTextureBuilder::new(self.format.to_platform())
        .with_size(self.width, self.height)
        .with_sample_count(self.sample_count);

    if let Some(ref label) = self.label {
      builder = builder.with_label(label);
    }

    let texture = builder.build(gpu.platform());
    return ColorAttachmentTexture::from_platform(texture);
  }
}

// ---------------------------------------------------------------------------
// DepthTexture
// ---------------------------------------------------------------------------

/// High-level wrapper for a depth (and optional stencil) render attachment.
///
/// This type manages a depth texture used for depth testing and stencil
/// operations in render passes.
#[derive(Debug)]
pub struct DepthTexture {
  inner: platform::DepthTexture,
}

impl DepthTexture {
  /// Create a high-level depth texture from a platform texture.
  pub(crate) fn from_platform(texture: platform::DepthTexture) -> Self {
    return DepthTexture { inner: texture };
  }

  /// The depth format used by this attachment.
  pub fn format(&self) -> DepthFormat {
    return match self.inner.format() {
      platform::DepthFormat::Depth32Float => DepthFormat::Depth32Float,
      platform::DepthFormat::Depth24Plus => DepthFormat::Depth24Plus,
      platform::DepthFormat::Depth24PlusStencil8 => {
        DepthFormat::Depth24PlusStencil8
      }
    };
  }

  /// Access the underlying platform texture view reference directly.
  ///
  /// This is needed for the render pass builder which expects the platform
  /// type.
  pub(crate) fn platform_view_ref(
    &self,
  ) -> lambda_platform::wgpu::surface::TextureViewRef<'_> {
    return self.inner.view_ref();
  }
}

// ---------------------------------------------------------------------------
// Texture (sampled)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
/// High‑level texture wrapper that owns a platform texture.
pub struct Texture {
  inner: Rc<platform::Texture>,
}

impl Texture {
  pub(crate) fn platform_texture(&self) -> Rc<platform::Texture> {
    return self.inner.clone();
  }

  /// Borrow a texture view reference for use in render pass attachments.
  pub(crate) fn view_ref(
    &self,
  ) -> crate::render::targets::surface::TextureView<'_> {
    return crate::render::targets::surface::TextureView::from_platform(
      self.inner.view_ref(),
    );
  }
}

#[derive(Debug, Clone)]
/// High‑level sampler wrapper that owns a platform sampler.
pub struct Sampler {
  inner: Rc<platform::Sampler>,
}

impl Sampler {
  pub(crate) fn platform_sampler(&self) -> Rc<platform::Sampler> {
    return self.inner.clone();
  }
}

/// Builder for creating a 2D sampled texture with optional initial data.
pub struct TextureBuilder {
  label: Option<String>,
  format: TextureFormat,
  width: u32,
  height: u32,
  depth: u32,
  data: Option<Vec<u8>>, // tightly packed rows
  is_render_target: bool,
}

impl TextureBuilder {
  /// Begin building a 2D texture.
  pub fn new_2d(format: TextureFormat) -> Self {
    return Self {
      label: None,
      format,
      width: 0,
      height: 0,
      depth: 1,
      data: None,
      is_render_target: false,
    };
  }

  /// Begin building a 3D texture.
  ///
  /// Call `with_size_3d(width, height, depth)` to specify the voxel size
  /// before building. The builder will select the 3D upload path based on the
  /// configured depth.
  pub fn new_3d(format: TextureFormat) -> Self {
    return Self {
      label: None,
      format,
      width: 0,
      height: 0,
      // Depth > 1 ensures the 3D path is chosen once size is provided.
      depth: 2,
      data: None,
      is_render_target: false,
    };
  }

  /// Set the texture size in pixels.
  pub fn with_size(mut self, width: u32, height: u32) -> Self {
    self.width = width;
    self.height = height;
    self.depth = 1;
    return self;
  }

  /// Set the 3D texture size in voxels.
  pub fn with_size_3d(mut self, width: u32, height: u32, depth: u32) -> Self {
    self.width = width;
    self.height = height;
    self.depth = depth;
    return self;
  }

  /// Provide tightly packed pixel data for full‑image upload.
  pub fn with_data(mut self, pixels: &[u8]) -> Self {
    self.data = Some(pixels.to_vec());
    return self;
  }

  /// Attach a debug label.
  pub fn with_label(mut self, label: &str) -> Self {
    self.label = Some(label.to_string());
    return self;
  }

  /// Configure this texture for use as a render target.
  ///
  /// Render target textures are created with usage flags suitable for both
  /// sampling and attachment, and allow copying from the texture for
  /// readback.
  pub fn for_render_target(mut self) -> Self {
    self.is_render_target = true;
    return self;
  }

  /// Create the texture and upload initial data if provided.
  pub fn build(self, gpu: &Gpu) -> Result<Texture, &'static str> {
    let mut builder =
      if self.depth <= 1 {
        platform::TextureBuilder::new_2d(self.format.to_platform())
          .with_size(self.width, self.height)
      } else {
        platform::TextureBuilder::new_3d(self.format.to_platform())
          .with_size_3d(self.width, self.height, self.depth)
      };

    if self.is_render_target {
      builder = builder.with_usage(
        platform::TextureUsages::TEXTURE_BINDING
          | platform::TextureUsages::RENDER_ATTACHMENT
          | platform::TextureUsages::COPY_SRC
          | platform::TextureUsages::COPY_DST,
      );
    }

    if let Some(ref label) = self.label {
      builder = builder.with_label(label);
    }

    if let Some(ref pixels) = self.data {
      builder = builder.with_data(pixels);
    }

    return match builder.build(gpu.platform()) {
      Ok(texture) => Ok(Texture {
        inner: Rc::new(texture),
      }),
      Err(platform::TextureBuildError::InvalidDimensions { .. }) => {
        Err("Invalid texture dimensions")
      }
      Err(platform::TextureBuildError::DataLengthMismatch { .. }) => {
        Err("Texture data length does not match width * height * bpp")
      }
      Err(platform::TextureBuildError::Overflow) => {
        Err("Overflow while computing texture layout")
      }
      Err(platform::TextureBuildError::UnsupportedFormat) => {
        Err("Texture format does not support bytes_per_pixel calculation")
      }
    };
  }
}

/// Builder for creating a depth texture attachment.
pub struct DepthTextureBuilder {
  label: Option<String>,
  width: u32,
  height: u32,
  format: DepthFormat,
  sample_count: u32,
}

impl Default for DepthTextureBuilder {
  fn default() -> Self {
    return Self::new();
  }
}

impl DepthTextureBuilder {
  /// Create a new depth texture builder with no size and `Depth32Float` format.
  pub fn new() -> Self {
    return Self {
      label: None,
      width: 0,
      height: 0,
      format: DepthFormat::Depth32Float,
      sample_count: 1,
    };
  }

  /// Set the 2D depth texture size in pixels.
  pub fn with_size(mut self, width: u32, height: u32) -> Self {
    self.width = width;
    self.height = height;
    return self;
  }

  /// Choose a depth format.
  pub fn with_format(mut self, format: DepthFormat) -> Self {
    self.format = format;
    return self;
  }

  /// Configure multisampling. Count values less than one are clamped to `1`.
  pub fn with_sample_count(mut self, count: u32) -> Self {
    self.sample_count = count.max(1);
    return self;
  }

  /// Attach a debug label.
  pub fn with_label(mut self, label: &str) -> Self {
    self.label = Some(label.to_string());
    return self;
  }

  /// Create the depth texture on the device.
  pub fn build(self, gpu: &Gpu) -> DepthTexture {
    let mut builder = platform::DepthTextureBuilder::new()
      .with_size(self.width.max(1), self.height.max(1))
      .with_format(self.format.to_platform())
      .with_sample_count(self.sample_count);

    if let Some(ref label) = self.label {
      builder = builder.with_label(label);
    }

    let texture = builder.build(gpu.platform());
    return DepthTexture::from_platform(texture);
  }
}

/// Builder for creating a sampler.
pub struct SamplerBuilder {
  inner: platform::SamplerBuilder,
}

impl Default for SamplerBuilder {
  fn default() -> Self {
    return Self::new();
  }
}

impl SamplerBuilder {
  /// Create a new sampler builder with nearest/clamp defaults.
  pub fn new() -> Self {
    return Self {
      inner: platform::SamplerBuilder::new(),
    };
  }

  /// Linear min/mag filter.
  pub fn linear(mut self) -> Self {
    self.inner = self.inner.linear();
    return self;
  }

  /// Nearest min/mag filter.
  pub fn nearest(mut self) -> Self {
    self.inner = self.inner.nearest();
    return self;
  }

  /// Convenience: linear filter + clamp addressing.
  pub fn linear_clamp(mut self) -> Self {
    self.inner = self.inner.linear_clamp();
    return self;
  }

  /// Convenience: nearest filter + clamp addressing.
  pub fn nearest_clamp(mut self) -> Self {
    self.inner = self.inner.nearest_clamp();
    return self;
  }

  /// Set address mode for U (x) coordinate.
  pub fn with_address_mode_u(mut self, mode: AddressMode) -> Self {
    self.inner = self.inner.with_address_mode_u(mode.to_platform());
    return self;
  }

  /// Set address mode for V (y) coordinate.
  pub fn with_address_mode_v(mut self, mode: AddressMode) -> Self {
    self.inner = self.inner.with_address_mode_v(mode.to_platform());
    return self;
  }

  /// Set address mode for W (z) coordinate.
  pub fn with_address_mode_w(mut self, mode: AddressMode) -> Self {
    self.inner = self.inner.with_address_mode_w(mode.to_platform());
    return self;
  }

  /// Set mipmap filtering.
  pub fn with_mip_filter(mut self, mode: FilterMode) -> Self {
    self.inner = self.inner.with_mip_filter(mode.to_platform());
    return self;
  }

  /// Set LOD clamp range.
  pub fn with_lod(mut self, min: f32, max: f32) -> Self {
    self.inner = self.inner.with_lod(min, max);
    return self;
  }

  /// Attach a debug label.
  pub fn with_label(mut self, label: &str) -> Self {
    self.inner = self.inner.with_label(label);
    return self;
  }

  /// Create the sampler on the provided GPU device.
  pub fn build(self, gpu: &Gpu) -> Sampler {
    let sampler = self.inner.build(gpu.platform());
    return Sampler {
      inner: Rc::new(sampler),
    };
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  /// Ensures texture formats round-trip to/from the platform and report the
  /// expected bytes-per-pixel and sRGB classification.
  #[test]
  fn texture_format_round_trips_through_platform() {
    let formats = [
      TextureFormat::Rgba8Unorm,
      TextureFormat::Rgba8UnormSrgb,
      TextureFormat::Bgra8Unorm,
      TextureFormat::Bgra8UnormSrgb,
    ];

    for fmt in formats {
      let platform = fmt.to_platform();
      let back = TextureFormat::from_platform(platform).expect("round trip");
      assert_eq!(back, fmt);
      assert_eq!(fmt.bytes_per_pixel(), 4);
    }

    assert!(TextureFormat::Rgba8UnormSrgb.is_srgb());
    assert!(!TextureFormat::Rgba8Unorm.is_srgb());
  }

  /// Ensures depth formats map to the platform depth formats.
  #[test]
  fn depth_format_maps_to_platform() {
    assert!(matches!(
      DepthFormat::Depth32Float.to_platform(),
      platform::DepthFormat::Depth32Float
    ));
    assert!(matches!(
      DepthFormat::Depth24Plus.to_platform(),
      platform::DepthFormat::Depth24Plus
    ));
    assert!(matches!(
      DepthFormat::Depth24PlusStencil8.to_platform(),
      platform::DepthFormat::Depth24PlusStencil8
    ));
  }

  /// Ensures view dimensions map to the platform view dimension types.
  #[test]
  fn view_dimension_maps_to_platform() {
    assert!(matches!(
      ViewDimension::D2.to_platform(),
      platform::ViewDimension::TwoDimensional
    ));
    assert!(matches!(
      ViewDimension::D3.to_platform(),
      platform::ViewDimension::ThreeDimensional
    ));
  }

  /// Ensures sampler-related enums map correctly to the platform enums.
  #[test]
  fn sampler_modes_map_to_platform() {
    assert!(matches!(
      FilterMode::Nearest.to_platform(),
      platform::FilterMode::Nearest
    ));
    assert!(matches!(
      FilterMode::Linear.to_platform(),
      platform::FilterMode::Linear
    ));

    assert!(matches!(
      AddressMode::ClampToEdge.to_platform(),
      platform::AddressMode::ClampToEdge
    ));
    assert!(matches!(
      AddressMode::Repeat.to_platform(),
      platform::AddressMode::Repeat
    ));
    assert!(matches!(
      AddressMode::MirrorRepeat.to_platform(),
      platform::AddressMode::MirrorRepeat
    ));
  }

  /// Ensures texture usage flags support bit ops and `contains` checks.
  #[test]
  fn texture_usages_support_bit_ops_and_contains() {
    let mut usages = TextureUsages::empty();
    assert!(!usages.contains(TextureUsages::RENDER_ATTACHMENT));

    usages |= TextureUsages::RENDER_ATTACHMENT;
    assert!(usages.contains(TextureUsages::RENDER_ATTACHMENT));

    let combined = TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST;
    assert!(combined.contains(TextureUsages::TEXTURE_BINDING));
    assert!(combined.contains(TextureUsages::COPY_DST));
  }

  /// Ensures `for_render_target` toggles the internal render-target usage flag.
  #[test]
  fn texture_builder_marks_render_target_usage() {
    let builder =
      TextureBuilder::new_2d(TextureFormat::Rgba8Unorm).for_render_target();

    assert!(builder.is_render_target);
  }

  /// Ensures depth texture builders clamp invalid sample counts to `1`.
  #[test]
  fn depth_texture_builder_clamps_sample_count() {
    let builder = DepthTextureBuilder::new().with_sample_count(0);
    assert_eq!(builder.sample_count, 1);
  }

  /// Ensures textures with invalid dimensions fail during build with an
  /// actionable error message.
  #[test]
  fn texture_builder_rejects_invalid_dimensions() {
    let Some(gpu) = crate::render::gpu::create_test_gpu("lambda-texture-test")
    else {
      return;
    };

    let err = TextureBuilder::new_2d(TextureFormat::Rgba8Unorm)
      .with_size(0, 0)
      .build(&gpu)
      .expect_err("invalid dimensions should error");
    assert_eq!(err, "Invalid texture dimensions");
  }

  /// Ensures the 3D texture builder selects the platform 3D creation path when
  /// the configured depth is greater than `1`.
  #[test]
  fn texture_builder_builds_3d_texture_path() {
    let Some(gpu) =
      crate::render::gpu::create_test_gpu("lambda-texture-3d-test")
    else {
      return;
    };

    // 3D texture builder selects the 3D upload path when depth > 1.
    let tex = TextureBuilder::new_3d(TextureFormat::Rgba8Unorm)
      .with_size_3d(2, 2, 2)
      .build(&gpu)
      .expect("build 3d texture");
    let _ = tex.platform_texture();
  }
}
