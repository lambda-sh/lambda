//! Texture and sampler enums and mappings for the platform layer.
//!
//! This module defines stable enums for texture formats, dimensions, view
//! dimensions, filtering, and addressing. It provides explicit mappings to the
//! underlying `wgpu` types and basic helpers such as bytes‑per‑pixel.

use super::types as wgpu;

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
}
