//! Texture and sampler enums and mappings for the platform layer.
//!
//! This module defines stable enums for texture formats, dimensions, view
//! dimensions, filtering, and addressing. It provides explicit mappings to the
//! underlying `wgpu` types and basic helpers such as bytes‑per‑pixel.

use super::types as wgpu;

#[derive(Debug)]
/// Errors returned when building textures fails validation.
pub enum TextureBuildError {
  /// Width or height is zero or exceeds device limits.
  InvalidDimensions { width: u32, height: u32 },
  /// Provided data length does not match expected tightly packed size.
  DataLengthMismatch { expected: usize, actual: usize },
  /// Internal arithmetic overflow while computing sizes or paddings.
  Overflow,
}

fn align_up(value: u32, alignment: u32) -> u32 {
  let mask = alignment - 1;
  return (value + mask) & !mask;
}

/// Filter function used for sampling.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum FilterMode {
  Nearest,
  Linear,
}

impl FilterMode {
  pub(crate) fn to_wgpu(self) -> wgpu::FilterMode {
    return match self {
      FilterMode::Nearest => wgpu::FilterMode::Nearest,
      FilterMode::Linear => wgpu::FilterMode::Linear,
    };
  }
}

/// Texture addressing mode when sampling outside the [0,1] range.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum AddressMode {
  ClampToEdge,
  Repeat,
  MirrorRepeat,
}

impl AddressMode {
  pub(crate) fn to_wgpu(self) -> wgpu::AddressMode {
    return match self {
      AddressMode::ClampToEdge => wgpu::AddressMode::ClampToEdge,
      AddressMode::Repeat => wgpu::AddressMode::Repeat,
      AddressMode::MirrorRepeat => wgpu::AddressMode::MirrorRepeat,
    };
  }
}

/// Supported color texture formats for sampling.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum TextureFormat {
  Rgba8Unorm,
  Rgba8UnormSrgb,
}

impl TextureFormat {
  /// Map to the corresponding `wgpu::TextureFormat`.
  pub(crate) fn to_wgpu(self) -> wgpu::TextureFormat {
    return match self {
      TextureFormat::Rgba8Unorm => wgpu::TextureFormat::Rgba8Unorm,
      TextureFormat::Rgba8UnormSrgb => wgpu::TextureFormat::Rgba8UnormSrgb,
    };
  }

  /// Number of bytes per pixel for tightly packed data.
  pub fn bytes_per_pixel(self) -> u32 {
    return match self {
      TextureFormat::Rgba8Unorm | TextureFormat::Rgba8UnormSrgb => 4,
    };
  }
}

/// Physical storage dimension of a texture.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum TextureDimension {
  TwoDimensional,
  ThreeDimensional,
}

impl TextureDimension {
  pub(crate) fn to_wgpu(self) -> wgpu::TextureDimension {
    return match self {
      TextureDimension::TwoDimensional => wgpu::TextureDimension::D2,
      TextureDimension::ThreeDimensional => wgpu::TextureDimension::D3,
    };
  }
}

/// View dimensionality exposed to shaders when sampling.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ViewDimension {
  TwoDimensional,
  ThreeDimensional,
}

impl ViewDimension {
  pub(crate) fn to_wgpu(self) -> wgpu::TextureViewDimension {
    return match self {
      ViewDimension::TwoDimensional => wgpu::TextureViewDimension::D2,
      ViewDimension::ThreeDimensional => wgpu::TextureViewDimension::D3,
    };
  }
}

#[derive(Debug)]
/// Wrapper around `wgpu::Texture` and its default `TextureView`.
pub struct Texture {
  pub(crate) raw: wgpu::Texture,
  pub(crate) view: wgpu::TextureView,
  pub(crate) label: Option<String>,
}

impl Texture {
  /// Borrow the underlying `wgpu::Texture`.
  pub fn raw(&self) -> &wgpu::Texture {
    return &self.raw;
  }

  /// Borrow the default full‑range `wgpu::TextureView`.
  pub fn view(&self) -> &wgpu::TextureView {
    return &self.view;
  }

  /// Optional debug label used during creation.
  pub fn label(&self) -> Option<&str> {
    return self.label.as_deref();
  }
}

/// Builder for creating a 2D sampled texture with optional initial data upload.
pub struct TextureBuilder {
  label: Option<String>,
  format: TextureFormat,
  dimension: TextureDimension,
  width: u32,
  height: u32,
  usage_texture_binding: bool,
  usage_copy_dst: bool,
  data: Option<Vec<u8>>, // tightly packed rows (width * bpp)
}

impl TextureBuilder {
  /// Construct a new 2D texture builder for a color format.
  pub fn new_2d(format: TextureFormat) -> Self {
    return Self {
      label: None,
      format,
      dimension: TextureDimension::TwoDimensional,
      width: 0,
      height: 0,
      usage_texture_binding: true,
      usage_copy_dst: true,
      data: None,
    };
  }

  /// Set the 2D texture size in pixels.
  pub fn with_size(mut self, width: u32, height: u32) -> Self {
    self.width = width;
    self.height = height;
    return self;
  }

  /// Provide tightly packed pixel data for level 0 upload.
  pub fn with_data(mut self, pixels: &[u8]) -> Self {
    self.data = Some(pixels.to_vec());
    return self;
  }

  /// Control usage flags. Defaults are suitable for sampling with initial upload.
  pub fn with_usage(mut self, texture_binding: bool, copy_dst: bool) -> Self {
    self.usage_texture_binding = texture_binding;
    self.usage_copy_dst = copy_dst;
    return self;
  }

  /// Attach a debug label.
  pub fn with_label(mut self, label: &str) -> Self {
    self.label = Some(label.to_string());
    return self;
  }

  /// Create the GPU texture and upload initial data if provided.
  pub fn build(
    self,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
  ) -> Result<Texture, TextureBuildError> {
    // Validate dimensions
    if self.width == 0 || self.height == 0 {
      return Err(TextureBuildError::InvalidDimensions {
        width: self.width,
        height: self.height,
      });
    }

    let size = wgpu::Extent3d {
      width: self.width,
      height: self.height,
      depth_or_array_layers: 1,
    };

    // Validate data length if provided
    if let Some(ref pixels) = self.data {
      let bpp = self.format.bytes_per_pixel() as usize;
      let expected = (self.width as usize)
        .checked_mul(self.height as usize)
        .and_then(|n| n.checked_mul(bpp))
        .ok_or(TextureBuildError::Overflow)?;
      if pixels.len() != expected {
        return Err(TextureBuildError::DataLengthMismatch {
          expected,
          actual: pixels.len(),
        });
      }
    }

    // Resolve usage flags
    let mut usage = wgpu::TextureUsages::empty();
    if self.usage_texture_binding {
      usage |= wgpu::TextureUsages::TEXTURE_BINDING;
    }
    if self.usage_copy_dst {
      usage |= wgpu::TextureUsages::COPY_DST;
    }

    let descriptor = wgpu::TextureDescriptor {
      label: self.label.as_deref(),
      size,
      mip_level_count: 1,
      sample_count: 1,
      dimension: self.dimension.to_wgpu(),
      format: self.format.to_wgpu(),
      usage,
      view_formats: &[],
    };

    let texture = device.create_texture(&descriptor);
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    if let Some(pixels) = self.data.as_ref() {
      // Compute 256-byte aligned bytes_per_row and pad rows if necessary.
      let bpp = self.format.bytes_per_pixel();
      let row_bytes = self
        .width
        .checked_mul(bpp)
        .ok_or(TextureBuildError::Overflow)?;
      let padded_row_bytes = align_up(row_bytes, 256);

      // Prepare a staging buffer with zeroed padding between rows.
      let total_bytes = (padded_row_bytes as u64)
        .checked_mul(self.height as u64)
        .ok_or(TextureBuildError::Overflow)? as usize;
      let mut staging = vec![0u8; total_bytes];

      let src_row_stride = row_bytes as usize;
      let dst_row_stride = padded_row_bytes as usize;
      for row in 0..(self.height as usize) {
        let src_off = row
          .checked_mul(src_row_stride)
          .ok_or(TextureBuildError::Overflow)?;
        let dst_off = row
          .checked_mul(dst_row_stride)
          .ok_or(TextureBuildError::Overflow)?;
        staging[dst_off..(dst_off + src_row_stride)]
          .copy_from_slice(&pixels[src_off..(src_off + src_row_stride)]);
      }

      let data_layout = wgpu::TexelCopyBufferLayout {
        offset: 0,
        bytes_per_row: Some(padded_row_bytes),
        rows_per_image: Some(self.height),
      };

      let copy_dst = wgpu::TexelCopyTextureInfo {
        texture: &texture,
        mip_level: 0,
        origin: wgpu::Origin3d { x: 0, y: 0, z: 0 },
        aspect: wgpu::TextureAspect::All,
      };

      queue.write_texture(copy_dst, &staging, data_layout, size);
    }

    return Ok(Texture {
      raw: texture,
      view,
      label: self.label,
    });
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn filter_mode_maps() {
    assert_eq!(FilterMode::Nearest.to_wgpu(), wgpu::FilterMode::Nearest);
    assert_eq!(FilterMode::Linear.to_wgpu(), wgpu::FilterMode::Linear);
  }

  #[test]
  fn address_mode_maps() {
    assert_eq!(
      AddressMode::ClampToEdge.to_wgpu(),
      wgpu::AddressMode::ClampToEdge
    );
    assert_eq!(AddressMode::Repeat.to_wgpu(), wgpu::AddressMode::Repeat);
    assert_eq!(
      AddressMode::MirrorRepeat.to_wgpu(),
      wgpu::AddressMode::MirrorRepeat
    );
  }

  #[test]
  fn texture_format_maps() {
    assert_eq!(
      TextureFormat::Rgba8Unorm.to_wgpu(),
      wgpu::TextureFormat::Rgba8Unorm
    );
    assert_eq!(
      TextureFormat::Rgba8UnormSrgb.to_wgpu(),
      wgpu::TextureFormat::Rgba8UnormSrgb
    );
  }

  #[test]
  fn bytes_per_pixel_is_correct() {
    assert_eq!(TextureFormat::Rgba8Unorm.bytes_per_pixel(), 4);
    assert_eq!(TextureFormat::Rgba8UnormSrgb.bytes_per_pixel(), 4);
  }

  #[test]
  fn dimensions_map() {
    assert_eq!(
      TextureDimension::TwoDimensional.to_wgpu(),
      wgpu::TextureDimension::D2
    );
    assert_eq!(
      TextureDimension::ThreeDimensional.to_wgpu(),
      wgpu::TextureDimension::D3
    );
  }

  #[test]
  fn view_dimensions_map() {
    assert_eq!(
      ViewDimension::TwoDimensional.to_wgpu(),
      wgpu::TextureViewDimension::D2
    );
    assert_eq!(
      ViewDimension::ThreeDimensional.to_wgpu(),
      wgpu::TextureViewDimension::D3
    );
  }

  #[test]
  fn align_up_computes_expected_values() {
    assert_eq!(super::align_up(0, 256), 0);
    assert_eq!(super::align_up(1, 256), 256);
    assert_eq!(super::align_up(255, 256), 256);
    assert_eq!(super::align_up(256, 256), 256);
    assert_eq!(super::align_up(300, 256), 512);
  }
}
