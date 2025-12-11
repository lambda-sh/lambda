//! High‑level wrapper for render pass color attachments.
//!
//! This module provides `RenderColorAttachments`, a lightweight engine‑level
//! wrapper that maps to the platform `RenderColorAttachments` type without
//! exposing `wgpu` details at call sites.

use lambda_platform::wgpu as platform;

use super::{
  render_pass::RenderPass as RenderPassDesc,
  RenderContext,
};

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
  pub(crate) fn push_color(
    &mut self,
    view: platform::surface::TextureViewRef<'view>,
  ) {
    self.inner.push_color(view);
  }

  /// Append a multi‑sampled color attachment with a resolve target view.
  pub(crate) fn push_msaa_color(
    &mut self,
    msaa_view: platform::surface::TextureViewRef<'view>,
    resolve_view: platform::surface::TextureViewRef<'view>,
  ) {
    self.inner.push_msaa_color(msaa_view, resolve_view);
  }

  /// Borrow the underlying platform attachments mutably for pass creation.
  pub(crate) fn as_platform_attachments_mut(
    &mut self,
  ) -> &mut platform::render_pass::RenderColorAttachments<'view> {
    return &mut self.inner;
  }

  /// Build color attachments for a surface‑backed render pass.
  ///
  /// This helper encapsulates the logic for configuring single‑sample and
  /// multi‑sample color attachments targeting the presentation surface,
  /// including creation and reuse of the MSAA resolve target stored on the
  /// `RenderContext`.
  pub(crate) fn for_surface_pass(
    render_context: &mut RenderContext,
    pass: &RenderPassDesc,
    surface_view: platform::surface::TextureViewRef<'view>,
  ) -> Self {
    let mut attachments = RenderColorAttachments::new();
    if !pass.uses_color() {
      return attachments;
    }

    let sample_count = pass.sample_count();
    if sample_count > 1 {
      let need_recreate = match &render_context.msaa_color {
        Some(_existing) => render_context.msaa_sample_count != sample_count,
        None => true,
      };

      if need_recreate {
        render_context.msaa_color = Some(
          platform::texture::ColorAttachmentTextureBuilder::new(
            render_context.config.format,
          )
          .with_size(render_context.size.0.max(1), render_context.size.1.max(1))
          .with_sample_count(sample_count)
          .with_label("lambda-msaa-color")
          .build(render_context.gpu()),
        );
        render_context.msaa_sample_count = sample_count;
      }

      let msaa_view = render_context
        .msaa_color
        .as_ref()
        .expect("MSAA color attachment should be created")
        .view_ref();
      attachments.push_msaa_color(msaa_view, surface_view);
    } else {
      attachments.push_color(surface_view);
    }

    return attachments;
  }
}
