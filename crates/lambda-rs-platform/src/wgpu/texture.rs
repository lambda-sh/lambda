//! Texture and sampler enums and mappings for the platform layer.
//!
//! This module defines stable enums for texture formats, dimensions, view
//! dimensions, filtering, and addressing. It provides explicit mappings to the
//! underlying `wgpu` types and basic helpers such as bytes‑per‑pixel.

use wgpu;

#[derive(Debug)]
/// Errors returned when building a texture or preparing its initial upload.
pub enum TextureBuildError {
  /// Width or height is zero or exceeds device limits.
  InvalidDimensions { width: u32, height: u32 },
  /// Provided data length does not match expected tightly packed size.
  DataLengthMismatch { expected: usize, actual: usize },
  /// Internal arithmetic overflow while computing sizes or paddings.
  Overflow,
}

/// Align `value` up to the next multiple of `alignment`.
///
/// `alignment` must be a power of two. This is used to compute
/// `bytes_per_row` for `Queue::write_texture` (256‑byte requirement).
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

/// Depth/stencil texture formats supported for render attachments.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum DepthFormat {
  Depth32Float,
  Depth24Plus,
  Depth24PlusStencil8,
}

impl DepthFormat {
  pub(crate) fn to_wgpu(self) -> wgpu::TextureFormat {
    return match self {
      DepthFormat::Depth32Float => wgpu::TextureFormat::Depth32Float,
      DepthFormat::Depth24Plus => wgpu::TextureFormat::Depth24Plus,
      DepthFormat::Depth24PlusStencil8 => {
        wgpu::TextureFormat::Depth24PlusStencil8
      }
    };
  }
}

#[derive(Debug)]
/// Wrapper for a depth (and optional stencil) texture used as a render attachment.
pub struct DepthTexture {
  pub(crate) raw: wgpu::Texture,
  pub(crate) view: wgpu::TextureView,
  pub(crate) label: Option<String>,
  pub(crate) format: DepthFormat,
}

impl DepthTexture {
  /// Borrow the underlying `wgpu::Texture`.
  pub fn raw(&self) -> &wgpu::Texture {
    return &self.raw;
  }

  /// Borrow the full‑range `wgpu::TextureView` for depth attachment.
  pub fn view(&self) -> &wgpu::TextureView {
    return &self.view;
  }

  /// The depth format used by this attachment.
  pub fn format(&self) -> DepthFormat {
    return self.format;
  }
}

/// Builder for a depth texture attachment sized to the current framebuffer.
pub struct DepthTextureBuilder {
  label: Option<String>,
  width: u32,
  height: u32,
  format: DepthFormat,
  sample_count: u32,
}

impl DepthTextureBuilder {
  /// Create a builder with no size and `Depth32Float` format.
  pub fn new() -> Self {
    return Self {
      label: None,
      width: 0,
      height: 0,
      format: DepthFormat::Depth32Float,
      sample_count: 1,
    };
  }

  /// Set the 2D attachment size in pixels.
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

  /// Configure multi‑sampling.
  pub fn with_sample_count(mut self, count: u32) -> Self {
    self.sample_count = count.max(1);
    return self;
  }

  /// Attach a debug label for the created texture.
  pub fn with_label(mut self, label: &str) -> Self {
    self.label = Some(label.to_string());
    return self;
  }

  /// Create the depth texture on the device.
  pub fn build(self, device: &wgpu::Device) -> DepthTexture {
    let size = wgpu::Extent3d {
      width: self.width.max(1),
      height: self.height.max(1),
      depth_or_array_layers: 1,
    };
    let format = self.format.to_wgpu();
    let raw = device.create_texture(&wgpu::TextureDescriptor {
      label: self.label.as_deref(),
      size,
      mip_level_count: 1,
      sample_count: self.sample_count,
      dimension: wgpu::TextureDimension::D2,
      format,
      usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
      view_formats: &[],
    });
    let view = raw.create_view(&wgpu::TextureViewDescriptor {
      label: None,
      format: Some(format),
      dimension: Some(wgpu::TextureViewDimension::D2),
      aspect: match self.format {
        DepthFormat::Depth24PlusStencil8 => wgpu::TextureAspect::All,
        _ => wgpu::TextureAspect::DepthOnly,
      },
      base_mip_level: 0,
      mip_level_count: None,
      base_array_layer: 0,
      array_layer_count: None,
      usage: Some(wgpu::TextureUsages::RENDER_ATTACHMENT),
    });

    return DepthTexture {
      raw,
      view,
      label: self.label,
      format: self.format,
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
/// Wrapper around `wgpu::Sampler` that preserves a label.
pub struct Sampler {
  pub(crate) raw: wgpu::Sampler,
  pub(crate) label: Option<String>,
}

impl Sampler {
  /// Borrow the underlying `wgpu::Sampler`.
  pub fn raw(&self) -> &wgpu::Sampler {
    return &self.raw;
  }

  /// Optional debug label used during creation.
  pub fn label(&self) -> Option<&str> {
    return self.label.as_deref();
  }
}

/// Builder for creating a sampler.
pub struct SamplerBuilder {
  label: Option<String>,
  min_filter: FilterMode,
  mag_filter: FilterMode,
  mipmap_filter: FilterMode,
  address_u: AddressMode,
  address_v: AddressMode,
  address_w: AddressMode,
  lod_min: f32,
  lod_max: f32,
}

impl SamplerBuilder {
  /// Create a new builder with nearest filtering and clamp addressing.
  pub fn new() -> Self {
    return Self {
      label: None,
      min_filter: FilterMode::Nearest,
      mag_filter: FilterMode::Nearest,
      mipmap_filter: FilterMode::Nearest,
      address_u: AddressMode::ClampToEdge,
      address_v: AddressMode::ClampToEdge,
      address_w: AddressMode::ClampToEdge,
      lod_min: 0.0,
      lod_max: 32.0,
    };
  }

  /// Set both min and mag filter to nearest.
  pub fn nearest(mut self) -> Self {
    self.min_filter = FilterMode::Nearest;
    self.mag_filter = FilterMode::Nearest;
    return self;
  }

  /// Set both min and mag filter to linear.
  pub fn linear(mut self) -> Self {
    self.min_filter = FilterMode::Linear;
    self.mag_filter = FilterMode::Linear;
    return self;
  }

  /// Convenience: nearest filtering with clamp-to-edge addressing.
  pub fn nearest_clamp(mut self) -> Self {
    self = self.nearest();
    self.address_u = AddressMode::ClampToEdge;
    self.address_v = AddressMode::ClampToEdge;
    self.address_w = AddressMode::ClampToEdge;
    return self;
  }

  /// Convenience: linear filtering with clamp-to-edge addressing.
  pub fn linear_clamp(mut self) -> Self {
    self = self.linear();
    self.address_u = AddressMode::ClampToEdge;
    self.address_v = AddressMode::ClampToEdge;
    self.address_w = AddressMode::ClampToEdge;
    return self;
  }

  /// Set address mode for U (x) coordinate.
  pub fn with_address_mode_u(mut self, mode: AddressMode) -> Self {
    self.address_u = mode;
    return self;
  }

  /// Set address mode for V (y) coordinate.
  pub fn with_address_mode_v(mut self, mode: AddressMode) -> Self {
    self.address_v = mode;
    return self;
  }

  /// Set address mode for W (z) coordinate.
  pub fn with_address_mode_w(mut self, mode: AddressMode) -> Self {
    self.address_w = mode;
    return self;
  }

  /// Set mipmap filtering mode.
  pub fn with_mip_filter(mut self, mode: FilterMode) -> Self {
    self.mipmap_filter = mode;
    return self;
  }

  /// Set minimum and maximum level-of-detail clamps.
  pub fn with_lod(mut self, min: f32, max: f32) -> Self {
    self.lod_min = min;
    self.lod_max = max;
    return self;
  }

  /// Attach a debug label.
  pub fn with_label(mut self, label: &str) -> Self {
    self.label = Some(label.to_string());
    return self;
  }

  fn to_descriptor(&self) -> wgpu::SamplerDescriptor<'_> {
    return wgpu::SamplerDescriptor {
      label: self.label.as_deref(),
      address_mode_u: self.address_u.to_wgpu(),
      address_mode_v: self.address_v.to_wgpu(),
      address_mode_w: self.address_w.to_wgpu(),
      mag_filter: self.mag_filter.to_wgpu(),
      min_filter: self.min_filter.to_wgpu(),
      mipmap_filter: self.mipmap_filter.to_wgpu(),
      lod_min_clamp: self.lod_min,
      lod_max_clamp: self.lod_max,
      ..Default::default()
    };
  }

  /// Create the sampler on the provided device.
  pub fn build(self, device: &wgpu::Device) -> Sampler {
    let desc = self.to_descriptor();
    let raw = device.create_sampler(&desc);
    return Sampler {
      raw,
      label: self.label,
    };
  }
}
#[derive(Debug)]
/// Wrapper around `wgpu::Texture` and its default `TextureView`.
///
/// The view covers the full resource with a view dimension that matches the
/// texture (D2/D3). The view usage mirrors the texture usage to satisfy wgpu
/// validation rules.
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

/// Builder for creating a sampled texture with optional initial data upload.
///
/// - 2D path: call `new_2d()` then `with_size(w, h)`.
/// - 3D path: call `new_3d()` then `with_size_3d(w, h, d)`.
///
/// The `with_data` payload is expected to be tightly packed (no row or image
/// padding). Row padding and `rows_per_image` are computed internally.
pub struct TextureBuilder {
  /// Optional debug label propagated to the created texture.
  label: Option<String>,
  /// Color format for the texture (filterable formats only).
  format: TextureFormat,
  /// Physical storage dimension (D2/D3).
  dimension: TextureDimension,
  /// Width in texels.
  width: u32,
  /// Height in texels.
  height: u32,
  /// Depth in texels (1 for 2D).
  depth: u32,
  /// Include `TEXTURE_BINDING` usage.
  usage_texture_binding: bool,
  /// Include `COPY_DST` usage when uploading initial data.
  usage_copy_dst: bool,
  /// Optional tightly‑packed pixel payload for level 0 (rows are `width*bpp`).
  data: Option<Vec<u8>>,
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
      depth: 1,
      usage_texture_binding: true,
      usage_copy_dst: true,
      data: None,
    };
  }

  /// Construct a new 3D texture builder for a color format.
  pub fn new_3d(format: TextureFormat) -> Self {
    return Self {
      label: None,
      format,
      dimension: TextureDimension::ThreeDimensional,
      width: 0,
      height: 0,
      depth: 0,
      usage_texture_binding: true,
      usage_copy_dst: true,
      data: None,
    };
  }

  /// Set the 2D texture size in pixels.
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
    if let TextureDimension::ThreeDimensional = self.dimension {
      if self.depth == 0 {
        return Err(TextureBuildError::InvalidDimensions {
          width: self.width,
          height: self.height,
        });
      }
    }

    let size = wgpu::Extent3d {
      width: self.width,
      height: self.height,
      depth_or_array_layers: match self.dimension {
        TextureDimension::TwoDimensional => 1,
        TextureDimension::ThreeDimensional => self.depth,
      },
    };

    // Validate data length if provided
    if let Some(ref pixels) = self.data {
      let bpp = self.format.bytes_per_pixel() as usize;
      let wh = (self.width as usize)
        .checked_mul(self.height as usize)
        .ok_or(TextureBuildError::Overflow)?;
      let expected = match self.dimension {
        TextureDimension::TwoDimensional => {
          wh.checked_mul(bpp).ok_or(TextureBuildError::Overflow)?
        }
        TextureDimension::ThreeDimensional => wh
          .checked_mul(self.depth as usize)
          .and_then(|n| n.checked_mul(bpp))
          .ok_or(TextureBuildError::Overflow)?,
      };
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
    let view_dimension = match self.dimension {
      TextureDimension::TwoDimensional => wgpu::TextureViewDimension::D2,
      TextureDimension::ThreeDimensional => wgpu::TextureViewDimension::D3,
    };
    let view = texture.create_view(&wgpu::TextureViewDescriptor {
      label: None,
      format: None,
      dimension: Some(view_dimension),
      aspect: wgpu::TextureAspect::All,
      base_mip_level: 0,
      mip_level_count: None,
      base_array_layer: 0,
      array_layer_count: None,
      usage: Some(usage),
    });

    if let Some(pixels) = self.data.as_ref() {
      // Compute 256-byte aligned bytes_per_row and pad rows if necessary.
      let bpp = self.format.bytes_per_pixel();
      let row_bytes = self
        .width
        .checked_mul(bpp)
        .ok_or(TextureBuildError::Overflow)?;
      let padded_row_bytes = align_up(row_bytes, 256);

      // Prepare a staging buffer with zeroed padding between rows (and images).
      let images = match self.dimension {
        TextureDimension::TwoDimensional => 1,
        TextureDimension::ThreeDimensional => self.depth,
      } as u64;
      let total_bytes = (padded_row_bytes as u64)
        .checked_mul(self.height as u64)
        .and_then(|n| n.checked_mul(images))
        .ok_or(TextureBuildError::Overflow)? as usize;
      let mut staging = vec![0u8; total_bytes];

      let src_row_stride = row_bytes as usize;
      let dst_row_stride = padded_row_bytes as usize;
      match self.dimension {
        TextureDimension::TwoDimensional => {
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
        }
        TextureDimension::ThreeDimensional => {
          let slice_stride = (self.height as usize)
            .checked_mul(src_row_stride)
            .ok_or(TextureBuildError::Overflow)?;
          let dst_image_stride = (self.height as usize)
            .checked_mul(dst_row_stride)
            .ok_or(TextureBuildError::Overflow)?;
          for z in 0..(self.depth as usize) {
            for y in 0..(self.height as usize) {
              let z_base_src = z
                .checked_mul(slice_stride)
                .ok_or(TextureBuildError::Overflow)?;
              let y_off_src = y
                .checked_mul(src_row_stride)
                .ok_or(TextureBuildError::Overflow)?;
              let src_off = z_base_src
                .checked_add(y_off_src)
                .ok_or(TextureBuildError::Overflow)?;

              let z_base_dst = z
                .checked_mul(dst_image_stride)
                .ok_or(TextureBuildError::Overflow)?;
              let y_off_dst = y
                .checked_mul(dst_row_stride)
                .ok_or(TextureBuildError::Overflow)?;
              let dst_off = z_base_dst
                .checked_add(y_off_dst)
                .ok_or(TextureBuildError::Overflow)?;

              staging[dst_off..(dst_off + src_row_stride)]
                .copy_from_slice(&pixels[src_off..(src_off + src_row_stride)]);
            }
          }
        }
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

  #[test]
  fn sampler_builder_defaults_map() {
    let b = SamplerBuilder::new();
    let d = b.to_descriptor();
    assert_eq!(d.address_mode_u, wgpu::AddressMode::ClampToEdge);
    assert_eq!(d.address_mode_v, wgpu::AddressMode::ClampToEdge);
    assert_eq!(d.address_mode_w, wgpu::AddressMode::ClampToEdge);
    assert_eq!(d.mag_filter, wgpu::FilterMode::Nearest);
    assert_eq!(d.min_filter, wgpu::FilterMode::Nearest);
    assert_eq!(d.mipmap_filter, wgpu::FilterMode::Nearest);
    assert_eq!(d.lod_min_clamp, 0.0);
    assert_eq!(d.lod_max_clamp, 32.0);
  }

  #[test]
  fn sampler_builder_linear_clamp_map() {
    let b = SamplerBuilder::new()
      .linear_clamp()
      .with_mip_filter(FilterMode::Linear);
    let d = b.to_descriptor();
    assert_eq!(d.address_mode_u, wgpu::AddressMode::ClampToEdge);
    assert_eq!(d.address_mode_v, wgpu::AddressMode::ClampToEdge);
    assert_eq!(d.address_mode_w, wgpu::AddressMode::ClampToEdge);
    assert_eq!(d.mag_filter, wgpu::FilterMode::Linear);
    assert_eq!(d.min_filter, wgpu::FilterMode::Linear);
    assert_eq!(d.mipmap_filter, wgpu::FilterMode::Linear);
  }
}
