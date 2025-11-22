//! Render pass wrapper and builder for starting a pass on a command encoder.
//!
//! This module provides a small builder to describe the pass setup (label and
//! clear color) and a thin `RenderPass` wrapper around `wgpu::RenderPass<'_>`.
//!
//! Building a render pass implicitly begins the pass on the provided
//! `CommandEncoder` and texture view. The returned `RenderPass` borrows the
//! encoder and remains valid until dropped.

use wgpu::{
  self,
  RenderPassColorAttachment,
};

use super::{
  bind,
  buffer,
  command,
  pipeline,
  surface,
};

/// Color load operation for a render pass color attachment.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ColorLoadOp {
  /// Load the existing contents of the attachment.
  Load,
  /// Clear the attachment to the provided RGBA color.
  Clear([f64; 4]),
}

/// Store operation for a render pass attachment.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StoreOp {
  /// Store the results to the attachment at the end of the pass.
  Store,
  /// Discard the results at the end of the pass when possible.
  Discard,
}

/// Combined load and store operations for the color attachment.
#[derive(Clone, Copy, Debug)]
pub struct ColorOperations {
  pub load: ColorLoadOp,
  pub store: StoreOp,
}

impl Default for ColorOperations {
  fn default() -> Self {
    return Self {
      load: ColorLoadOp::Clear([0.0, 0.0, 0.0, 1.0]),
      store: StoreOp::Store,
    };
  }
}

/// Depth load operation for a depth attachment.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DepthLoadOp {
  /// Load the existing contents of the depth attachment.
  Load,
  /// Clear the depth attachment to the provided value in [0,1].
  Clear(f32),
}

/// Depth operations (load/store) for the depth attachment.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DepthOperations {
  pub load: DepthLoadOp,
  pub store: StoreOp,
}

impl Default for DepthOperations {
  fn default() -> Self {
    return Self {
      load: DepthLoadOp::Clear(1.0),
      store: StoreOp::Store,
    };
  }
}

/// Stencil load operation for a stencil attachment.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StencilLoadOp {
  /// Load the existing contents of the stencil attachment.
  Load,
  /// Clear the stencil attachment to the provided value.
  Clear(u32),
}

/// Stencil operations (load/store) for the stencil attachment.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StencilOperations {
  pub load: StencilLoadOp,
  pub store: StoreOp,
}

impl Default for StencilOperations {
  fn default() -> Self {
    return Self {
      load: StencilLoadOp::Clear(0),
      store: StoreOp::Store,
    };
  }
}

/// Configuration for beginning a render pass.
#[derive(Clone, Debug, Default)]
pub struct RenderPassConfig {
  pub label: Option<String>,
  pub color_operations: ColorOperations,
}

/// Wrapper around `wgpu::RenderPass<'_>` exposing the operations needed by the
/// engine without leaking raw `wgpu` types at the call sites.
#[derive(Debug)]
pub struct RenderPass<'a> {
  pub(super) raw: wgpu::RenderPass<'a>,
}

#[derive(Debug)]
struct RenderPassKeepAlive<'a> {
  color_attachments: [Option<wgpu::RenderPassColorAttachment<'a>>; 1],
  label: Option<String>,
}

impl<'a> RenderPass<'a> {
  /// Set the active render pipeline.
  pub fn set_pipeline(&mut self, pipeline: &pipeline::RenderPipeline) {
    self.raw.set_pipeline(pipeline.raw());
  }

  /// Apply viewport state.
  pub fn set_viewport(
    &mut self,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    min_depth: f32,
    max_depth: f32,
  ) {
    self
      .raw
      .set_viewport(x, y, width, height, min_depth, max_depth);
  }

  /// Apply scissor rectangle.
  pub fn set_scissor_rect(&mut self, x: u32, y: u32, width: u32, height: u32) {
    self.raw.set_scissor_rect(x, y, width, height);
  }

  /// Bind a group with optional dynamic offsets.
  pub fn set_bind_group(
    &mut self,
    set: u32,
    group: &bind::BindGroup,
    dynamic_offsets: &[u32],
  ) {
    self.raw.set_bind_group(set, group.raw(), dynamic_offsets);
  }

  /// Bind a vertex buffer slot.
  pub fn set_vertex_buffer(&mut self, slot: u32, buffer: &buffer::Buffer) {
    self.raw.set_vertex_buffer(slot, buffer.raw().slice(..));
  }

  /// Bind an index buffer with the provided index format.
  pub fn set_index_buffer(
    &mut self,
    buffer: &buffer::Buffer,
    format: buffer::IndexFormat,
  ) {
    self
      .raw
      .set_index_buffer(buffer.raw().slice(..), format.to_wgpu());
  }

  /// Upload push constants.
  pub fn set_push_constants(
    &mut self,
    stages: pipeline::PipelineStage,
    offset: u32,
    data: &[u8],
  ) {
    self.raw.set_push_constants(stages.to_wgpu(), offset, data);
  }

  /// Issue a non-indexed draw over a vertex range.
  pub fn draw(
    &mut self,
    vertices: std::ops::Range<u32>,
    instances: std::ops::Range<u32>,
  ) {
    self.raw.draw(vertices, instances);
  }

  /// Issue an indexed draw with a base vertex applied.
  pub fn draw_indexed(
    &mut self,
    indices: std::ops::Range<u32>,
    base_vertex: i32,
    instances: std::ops::Range<u32>,
  ) {
    self.raw.draw_indexed(indices, base_vertex, instances);
  }
}

/// Wrapper for a variably sized list of color attachments passed into a render
/// pass. The attachments borrow `TextureView` references for the duration of
/// the pass.
#[derive(Debug, Default)]
pub struct RenderColorAttachments<'a> {
  attachments: Vec<Option<wgpu::RenderPassColorAttachment<'a>>>,
}

impl<'a> RenderColorAttachments<'a> {
  /// Create an empty attachments list.
  pub fn new() -> Self {
    return Self {
      attachments: Vec::new(),
    };
  }

  /// Append a color attachment targeting the provided `TextureView`.
  ///
  /// The load/store operations are initialized to load/store and will be
  /// configured by the render pass builder before beginning the pass.
  pub fn push_color(&mut self, view: surface::TextureViewRef<'a>) -> &mut Self {
    let attachment = wgpu::RenderPassColorAttachment {
      view: view.raw,
      resolve_target: None,
      depth_slice: None,
      ops: wgpu::Operations {
        load: wgpu::LoadOp::Load,
        store: wgpu::StoreOp::Store,
      },
    };
    self.attachments.push(Some(attachment));
    return self;
  }

  /// Append a multi-sampled color attachment with a resolve target view.
  ///
  /// The `msaa_view` MUST have a sample count > 1 and the `resolve_view` MUST
  /// be a single-sample view of the same format and size.
  pub fn push_msaa_color(
    &mut self,
    msaa_view: surface::TextureViewRef<'a>,
    resolve_view: surface::TextureViewRef<'a>,
  ) -> &mut Self {
    let attachment = wgpu::RenderPassColorAttachment {
      view: msaa_view.raw,
      resolve_target: Some(resolve_view.raw),
      depth_slice: None,
      ops: wgpu::Operations {
        load: wgpu::LoadOp::Load,
        store: wgpu::StoreOp::Store,
      },
    };
    self.attachments.push(Some(attachment));
    return self;
  }

  /// Apply the same operations to all color attachments.
  pub(crate) fn set_operations_for_all(
    &mut self,
    operations: wgpu::Operations<wgpu::Color>,
  ) {
    for attachment in &mut self.attachments {
      if let Some(ref mut a) = attachment {
        a.ops = operations;
      }
    }
  }

  pub(crate) fn as_slice(
    &self,
  ) -> &[Option<wgpu::RenderPassColorAttachment<'a>>] {
    return &self.attachments;
  }
}

#[derive(Debug, Default)]
/// Builder for beginning a render pass targeting a single color attachment.
///
/// Building a render pass implicitly begins one on the provided encoder and
/// returns a `RenderPass` wrapper that borrows the encoder.
pub struct RenderPassBuilder {
  config: RenderPassConfig,
}

impl RenderPassBuilder {
  /// Create a new builder. Defaults to clearing to opaque black.
  pub fn new() -> Self {
    Self {
      config: RenderPassConfig {
        label: None,
        color_operations: ColorOperations::default(),
      },
    }
  }

  /// Attach a debug label to the render pass.
  pub fn with_label(mut self, label: &str) -> Self {
    self.config.label = Some(label.to_string());
    return self;
  }

  /// Set the clear color for the color attachment.
  pub fn with_clear_color(mut self, color: [f64; 4]) -> Self {
    self.config.color_operations.load = ColorLoadOp::Clear(color);
    self.config.color_operations.store = StoreOp::Store;
    return self;
  }

  /// Set color load operation (load or clear with the previously set color).
  pub fn with_color_load_op(mut self, load: ColorLoadOp) -> Self {
    self.config.color_operations.load = load;
    return self;
  }

  /// Set color store operation (store or discard).
  pub fn with_store_op(mut self, store: StoreOp) -> Self {
    self.config.color_operations.store = store;
    return self;
  }

  /// Provide combined color operations.
  pub fn with_color_operations(mut self, operations: ColorOperations) -> Self {
    self.config.color_operations = operations;
    return self;
  }

  // Depth attachment is supplied at build time by the caller.

  /// Build (begin) the render pass on the provided encoder using the provided
  /// color attachments list. The attachments list MUST outlive the returned
  /// render pass value.
  pub fn build<'view>(
    &'view self,
    encoder: &'view mut command::CommandEncoder,
    attachments: &'view mut RenderColorAttachments<'view>,
    depth_view: Option<crate::wgpu::surface::TextureViewRef<'view>>,
    depth_ops: Option<DepthOperations>,
    stencil_ops: Option<StencilOperations>,
  ) -> RenderPass<'view> {
    let operations = match self.config.color_operations.load {
      ColorLoadOp::Load => wgpu::Operations {
        load: wgpu::LoadOp::Load,
        store: match self.config.color_operations.store {
          StoreOp::Store => wgpu::StoreOp::Store,
          StoreOp::Discard => wgpu::StoreOp::Discard,
        },
      },
      ColorLoadOp::Clear(color) => wgpu::Operations {
        load: wgpu::LoadOp::Clear(wgpu::Color {
          r: color[0],
          g: color[1],
          b: color[2],
          a: color[3],
        }),
        store: match self.config.color_operations.store {
          StoreOp::Store => wgpu::StoreOp::Store,
          StoreOp::Discard => wgpu::StoreOp::Discard,
        },
      },
    };

    // Apply operations to all provided attachments.
    attachments.set_operations_for_all(operations);

    // Optional depth/stencil attachment. Include depth or stencil ops only
    // when provided to avoid touching aspects that were not requested.
    let depth_stencil_attachment = depth_view.map(|v| {
      // Map depth ops only when explicitly provided; when `None`, preserve the
      // depth aspect, which is important for stencil-only passes.
      let mapped_depth_ops = depth_ops.map(|dop| match dop.load {
        DepthLoadOp::Load => wgpu::Operations {
          load: wgpu::LoadOp::Load,
          store: match dop.store {
            StoreOp::Store => wgpu::StoreOp::Store,
            StoreOp::Discard => wgpu::StoreOp::Discard,
          },
        },
        DepthLoadOp::Clear(value) => wgpu::Operations {
          load: wgpu::LoadOp::Clear(value),
          store: match dop.store {
            StoreOp::Store => wgpu::StoreOp::Store,
            StoreOp::Discard => wgpu::StoreOp::Discard,
          },
        },
      });

      // Map stencil ops only if explicitly provided.
      let mapped_stencil_ops = stencil_ops.map(|sop| match sop.load {
        StencilLoadOp::Load => wgpu::Operations {
          load: wgpu::LoadOp::Load,
          store: match sop.store {
            StoreOp::Store => wgpu::StoreOp::Store,
            StoreOp::Discard => wgpu::StoreOp::Discard,
          },
        },
        StencilLoadOp::Clear(value) => wgpu::Operations {
          load: wgpu::LoadOp::Clear(value),
          store: match sop.store {
            StoreOp::Store => wgpu::StoreOp::Store,
            StoreOp::Discard => wgpu::StoreOp::Discard,
          },
        },
      });

      wgpu::RenderPassDepthStencilAttachment {
        view: v.raw,
        depth_ops: mapped_depth_ops,
        stencil_ops: mapped_stencil_ops,
      }
    });

    let desc: wgpu::RenderPassDescriptor<'view> = wgpu::RenderPassDescriptor {
      label: self.config.label.as_deref(),
      color_attachments: attachments.as_slice(),
      depth_stencil_attachment,
      timestamp_writes: None,
      occlusion_query_set: None,
    };

    let pass = encoder.begin_render_pass_raw(&desc);
    return RenderPass { raw: pass };
  }
}

impl<'a> RenderPass<'a> {
  /// Set the stencil reference value used by the active pipeline's stencil test.
  pub fn set_stencil_reference(&mut self, reference: u32) {
    self.raw.set_stencil_reference(reference);
  }
}
