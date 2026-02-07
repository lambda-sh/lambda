//! High‑level wrapper for render pass color attachments.
//!
//! This module provides `RenderColorAttachments`, a lightweight engine‑level
//! wrapper that maps to the platform `RenderColorAttachments` type without
//! exposing `wgpu` details at call sites.

use lambda_platform::wgpu as platform;

use super::targets::surface::TextureView;

#[derive(Debug, Default)]
/// High‑level color attachments collection used when beginning a render pass.
///
/// This type delegates to the platform `RenderColorAttachments` while keeping
/// the engine API stable and avoiding direct references to platform types in
/// higher‑level modules.
pub(crate) struct RenderColorAttachments<'view> {
  inner: platform::render_pass::RenderColorAttachments<'view>,
}

impl<'view> RenderColorAttachments<'view> {
  /// Create an empty color attachments list.
  pub(crate) fn new() -> Self {
    return Self {
      inner: platform::render_pass::RenderColorAttachments::new(),
    };
  }

  /// Append a color attachment targeting the provided texture view.
  ///
  /// Accepts a high-level `TextureView` and converts it internally to the
  /// platform type.
  pub(crate) fn push_color(&mut self, view: TextureView<'view>) {
    self.inner.push_color(view.to_platform());
  }

  /// Append a multi‑sampled color attachment with a resolve target view.
  ///
  /// The `msaa_view` is the multi-sampled render target, and `resolve_view`
  /// is the single-sample target that receives the resolved output.
  /// Both accept high-level `TextureView` types and convert internally.
  pub(crate) fn push_msaa_color(
    &mut self,
    msaa_view: TextureView<'view>,
    resolve_view: TextureView<'view>,
  ) {
    self
      .inner
      .push_msaa_color(msaa_view.to_platform(), resolve_view.to_platform());
  }

  /// Borrow the underlying platform attachments mutably for pass creation.
  pub(crate) fn as_platform_attachments_mut(
    &mut self,
  ) -> &mut platform::render_pass::RenderColorAttachments<'view> {
    return &mut self.inner;
  }

  /// Build color attachments for a surface-backed render pass.
  ///
  /// This helper configures single-sample or multi-sample color attachments
  /// targeting the presentation surface. The MSAA view is optional and should
  /// be provided when multi-sampling is enabled.
  ///
  /// # Arguments
  /// * `uses_color` - Whether the render pass uses color output.
  /// * `sample_count` - The MSAA sample count (1 for no MSAA).
  /// * `msaa_view` - Optional high-level MSAA texture view (required when
  ///   `sample_count > 1`).
  /// * `surface_view` - The high-level surface texture view (resolve target
  ///   for MSAA, or direct target for single-sample).
  pub(crate) fn for_surface_pass(
    uses_color: bool,
    sample_count: u32,
    msaa_view: Option<TextureView<'view>>,
    surface_view: TextureView<'view>,
  ) -> Self {
    let mut attachments = RenderColorAttachments::new();

    if !uses_color {
      return attachments;
    }

    if sample_count > 1 {
      let msaa =
        msaa_view.expect("MSAA view must be provided when sample_count > 1");
      attachments.push_msaa_color(msaa, surface_view);
    } else {
      attachments.push_color(surface_view);
    }

    return attachments;
  }

  /// Build color attachments for an offscreen render pass.
  ///
  /// This helper configures single-sample or multi-sample color attachments
  /// targeting an offscreen resolve texture. When MSAA is enabled, the
  /// `msaa_view` is used as the multi-sampled render target and `resolve_view`
  /// receives the resolved output.
  pub(crate) fn for_offscreen_pass(
    uses_color: bool,
    sample_count: u32,
    msaa_view: Option<TextureView<'view>>,
    resolve_view: TextureView<'view>,
  ) -> Self {
    let mut attachments = RenderColorAttachments::new();

    if !uses_color {
      return attachments;
    }

    if sample_count > 1 {
      let msaa =
        msaa_view.expect("MSAA view must be provided when sample_count > 1");
      attachments.push_msaa_color(msaa, resolve_view);
    } else {
      attachments.push_color(resolve_view);
    }

    return attachments;
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::render::{
    gpu::{
      Gpu,
      GpuBuilder,
    },
    instance::InstanceBuilder,
    texture::{
      ColorAttachmentTextureBuilder,
      TextureBuilder,
      TextureFormat,
    },
  };

  fn create_test_gpu() -> Option<Gpu> {
    let instance = InstanceBuilder::new()
      .with_label("lambda-color-attachments-test-instance")
      .build();
    return GpuBuilder::new()
      .with_label("lambda-color-attachments-test-gpu")
      .build(&instance, None)
      .ok();
  }

  #[test]
  #[ignore = "requires a real GPU adapter"]
  fn for_surface_pass_returns_empty_when_color_disabled() {
    let gpu = create_test_gpu().expect("requires a real GPU adapter");

    let texture = TextureBuilder::new_2d(TextureFormat::Rgba8Unorm)
      .with_size(1, 1)
      .for_render_target()
      .build(&gpu)
      .expect("build resolve texture");

    let view = texture.view_ref();
    let mut attachments =
      RenderColorAttachments::for_surface_pass(false, 1, None, view);
    let _ = attachments.as_platform_attachments_mut();
  }

  #[test]
  #[ignore = "requires a real GPU adapter"]
  fn for_offscreen_pass_builds_single_sample_color_attachment() {
    let gpu = create_test_gpu().expect("requires a real GPU adapter");

    let texture = TextureBuilder::new_2d(TextureFormat::Rgba8Unorm)
      .with_size(4, 4)
      .for_render_target()
      .build(&gpu)
      .expect("build resolve texture");

    let view = texture.view_ref();
    let mut attachments =
      RenderColorAttachments::for_offscreen_pass(true, 1, None, view);
    let _ = attachments.as_platform_attachments_mut();
  }

  #[test]
  #[ignore = "requires a real GPU adapter"]
  fn for_offscreen_pass_builds_msaa_color_attachment() {
    let gpu = create_test_gpu().expect("requires a real GPU adapter");

    let resolve = TextureBuilder::new_2d(TextureFormat::Rgba8Unorm)
      .with_size(4, 4)
      .for_render_target()
      .build(&gpu)
      .expect("build resolve texture");

    let msaa = ColorAttachmentTextureBuilder::new(TextureFormat::Rgba8Unorm)
      .with_size(4, 4)
      .with_sample_count(4)
      .build(&gpu);

    let mut attachments = RenderColorAttachments::for_offscreen_pass(
      true,
      4,
      Some(msaa.view_ref()),
      resolve.view_ref(),
    );
    let _ = attachments.as_platform_attachments_mut();
  }

  #[test]
  #[ignore = "requires a real GPU adapter"]
  fn for_offscreen_pass_panics_when_msaa_view_missing() {
    let gpu = create_test_gpu().expect("requires a real GPU adapter");

    let resolve = TextureBuilder::new_2d(TextureFormat::Rgba8Unorm)
      .with_size(1, 1)
      .for_render_target()
      .build(&gpu)
      .expect("build resolve texture");

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
      let _ = RenderColorAttachments::for_offscreen_pass(
        true,
        4,
        None,
        resolve.view_ref(),
      );
    }));
    assert!(result.is_err());
  }
}
