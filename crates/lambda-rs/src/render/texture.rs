//! High‑level textures and samplers.
//!
//! Provides `TextureBuilder` and `SamplerBuilder` wrappers that delegate to the
//! platform layer and keep `wgpu` details internal to the platform crate.

use std::rc::Rc;

use lambda_platform::wgpu::texture as platform;

use super::RenderContext;

#[derive(Debug, Clone, Copy)]
/// Supported color texture formats for sampling.
pub enum TextureFormat {
  Rgba8Unorm,
  Rgba8UnormSrgb,
}

impl TextureFormat {
  fn to_platform(self) -> platform::TextureFormat {
    return match self {
      TextureFormat::Rgba8Unorm => platform::TextureFormat::Rgba8Unorm,
      TextureFormat::Rgba8UnormSrgb => platform::TextureFormat::Rgba8UnormSrgb,
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
      data: None,
    };
  }

  /// Set the texture size in pixels.
  pub fn with_size(mut self, width: u32, height: u32) -> Self {
    self.width = width;
    self.height = height;
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
      platform::TextureBuilder::new_2d(self.format.to_platform())
        .with_size(self.width, self.height);
    if let Some(ref label) = self.label {
      builder = builder.with_label(label);
    }
    if let Some(ref pixels) = self.data {
      builder = builder.with_data(pixels);
    }
    match builder.build(render_context.device(), render_context.queue()) {
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
    }
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

  /// Attach a debug label.
  pub fn with_label(mut self, label: &str) -> Self {
    self.inner = self.inner.with_label(label);
    return self;
  }

  /// Create the sampler on the current device.
  pub fn build(self, render_context: &mut RenderContext) -> Sampler {
    let sampler = self.inner.build(render_context.device());
    return Sampler {
      inner: Rc::new(sampler),
    };
  }
}
