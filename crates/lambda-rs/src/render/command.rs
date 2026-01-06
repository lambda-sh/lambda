//! Render command definitions for Lambda runtimes.
//!
//! A frame is described as a linear list of `RenderCommand`s. The sequence
//! MUST start a render pass before any draw‑related commands and MUST end the
//! pass explicitly. Resource and pipeline handles referenced by the commands
//! MUST have been attached to the active `RenderContext`.

use std::ops::Range;

use super::{
  pipeline::PipelineStage,
  viewport::Viewport,
};

/// Engine-level index format for indexed drawing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IndexFormat {
  Uint16,
  Uint32,
}

impl IndexFormat {
  pub(crate) fn to_platform(
    self,
  ) -> lambda_platform::wgpu::buffer::IndexFormat {
    match self {
      IndexFormat::Uint16 => lambda_platform::wgpu::buffer::IndexFormat::Uint16,
      IndexFormat::Uint32 => lambda_platform::wgpu::buffer::IndexFormat::Uint32,
    }
  }
}

/// Render destination selected when beginning a render pass.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderDestination {
  /// Target the presentation surface.
  Surface,
  /// Target a previously attached offscreen destination by id.
  Offscreen(super::ResourceId),
}

/// Commands recorded and executed by the `RenderContext` to produce a frame.
///
/// Order and validity are enforced by the encoder where possible. Invalid
/// sequences (e.g., nested passes or missing `EndRenderPass`) are reported as
/// configuration errors.
#[derive(Debug, Clone)]
pub enum RenderCommand {
  /// Set one or more viewports starting at `start_at` slot.
  SetViewports {
    start_at: u32,
    viewports: Vec<Viewport>,
  },
  /// Set one or more scissor rectangles matching the current viewports.
  SetScissors {
    start_at: u32,
    viewports: Vec<Viewport>,
  },
  /// Bind a previously attached graphics pipeline by id.
  SetPipeline { pipeline: super::ResourceId },
  /// Begin a render pass. When the pass is configured with color attachments,
  /// it targets the swapchain view (with optional MSAA resolve). Passes may
  /// also omit color to perform depth/stencil-only work.
  BeginRenderPass {
    render_pass: super::ResourceId,
    viewport: Viewport,
  },
  /// Begin a render pass targeting an explicit destination.
  ///
  /// `BeginRenderPass` remains valid and is equivalent to beginning a pass
  /// with `RenderDestination::Surface`.
  BeginRenderPassTo {
    render_pass: super::ResourceId,
    viewport: Viewport,
    destination: RenderDestination,
  },
  /// End the current render pass.
  EndRenderPass,

  /// Set the stencil reference value for the active pass.
  SetStencilReference { reference: u32 },

  /// Upload immediate data at `offset`.
  ///
  /// The byte vector is interpreted as tightly packed `u32` words; the
  /// encoder turns it into raw bytes when encoding. Both offset and data
  /// length must be multiples of 4 bytes. The GLSL syntax for declaring
  /// immediate data blocks remains `layout(push_constant)`.
  Immediates {
    pipeline: super::ResourceId,
    stage: PipelineStage,
    offset: u32,
    bytes: Vec<u32>,
  },
  /// Bind a vertex buffer by index as declared on the pipeline.
  BindVertexBuffer {
    pipeline: super::ResourceId,
    buffer: u32,
  },
  /// Bind an index buffer by resource id with format.
  BindIndexBuffer {
    /// Resource identifier returned by `RenderContext::attach_buffer`.
    buffer: super::ResourceId,
    /// Index format for this buffer.
    format: IndexFormat,
  },
  /// Issue a non‑indexed draw for the provided vertex range.
  Draw {
    vertices: Range<u32>,
    instances: Range<u32>,
  },
  /// Issue an indexed draw for the provided index range.
  DrawIndexed {
    indices: Range<u32>,
    base_vertex: i32,
    instances: Range<u32>,
  },

  /// Bind a previously created bind group to a set index with optional
  /// dynamic offsets. Dynamic offsets are counted in bytes and must obey the
  /// device's minimum uniform buffer offset alignment when using dynamic
  /// uniform bindings.
  SetBindGroup {
    /// The pipeline layout set index to bind this group to.
    set: u32,
    /// Resource identifier returned by `RenderContext::attach_bind_group`.
    group: super::ResourceId,
    /// Dynamic offsets in bytes to apply to bindings marked as dynamic.
    dynamic_offsets: Vec<u32>,
  },
}

#[cfg(test)]
mod tests {
  use super::IndexFormat;

  #[test]
  fn index_format_maps_to_platform() {
    let u16_platform = IndexFormat::Uint16.to_platform();
    let u32_platform = IndexFormat::Uint32.to_platform();

    assert_eq!(
      u16_platform,
      lambda_platform::wgpu::buffer::IndexFormat::Uint16
    );
    assert_eq!(
      u32_platform,
      lambda_platform::wgpu::buffer::IndexFormat::Uint32
    );
  }
}
