//! High‑level textures and samplers.
//!
//! Provides `TextureBuilder` and `SamplerBuilder` wrappers that delegate to the
//! platform layer and keep `wgpu` details internal to the platform crate.

use std::rc::Rc;

use lambda_platform::wgpu::texture as platform;

use super::RenderContext;

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

#[derive(Debug, Clone, Copy)]
/// Supported color texture formats for sampling.
pub enum TextureFormat {
  Rgba8Unorm,
  Rgba8UnormSrgb,
}

impl TextureFormat {
  fn to_platform(self) -> platform::TextureFormat {
    return match self {
      TextureFormat::Rgba8Unorm => platform::TextureFormat::RGBA8_UNORM,
      TextureFormat::Rgba8UnormSrgb => {
        platform::TextureFormat::RGBA8_UNORM_SRGB
      }
    };
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
#[derive(Debug, Clone)]
/// High‑level texture wrapper that owns a platform texture.
pub struct Texture {
  inner: Rc<platform::Texture>,
}

impl Texture {
  pub(crate) fn platform_texture(&self) -> Rc<platform::Texture> {
    return self.inner.clone();
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

  /// Create the texture and upload initial data if provided.
  pub fn build(
    self,
    render_context: &mut RenderContext,
  ) -> Result<Texture, &'static str> {
    let mut builder =
      if self.depth <= 1 {
        platform::TextureBuilder::new_2d(self.format.to_platform())
          .with_size(self.width, self.height)
      } else {
        platform::TextureBuilder::new_3d(self.format.to_platform())
          .with_size_3d(self.width, self.height, self.depth)
      };

    if let Some(ref label) = self.label {
      builder = builder.with_label(label);
    }

    if let Some(ref pixels) = self.data {
      builder = builder.with_data(pixels);
    }

    return match builder.build(render_context.gpu()) {
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

/// Builder for creating a sampler.
pub struct SamplerBuilder {
  inner: platform::SamplerBuilder,
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

  /// Create the sampler on the current device.
  pub fn build(self, render_context: &mut RenderContext) -> Sampler {
    let sampler = self.inner.build(render_context.gpu());
    return Sampler {
      inner: Rc::new(sampler),
    };
  }
}
