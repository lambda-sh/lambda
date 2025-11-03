//! Render command definitions for lambda runtimes.

use std::ops::Range;

use super::{
  pipeline::PipelineStage,
  viewport::Viewport,
};

/// Commands recorded and executed by the `RenderContext` to produce a frame.
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
  /// Begin a render pass that targets the swapchain.
  BeginRenderPass {
    render_pass: super::ResourceId,
    viewport: Viewport,
  },
  /// End the current render pass.
  EndRenderPass,

  /// Upload push constants for the active pipeline/stage at `offset`.
  PushConstants {
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
    format: lambda_platform::wgpu::buffer::IndexFormat,
  },
  /// Issue a non‑indexed draw for the provided vertex range.
  Draw { vertices: Range<u32> },
  /// Issue an indexed draw for the provided index range.
  DrawIndexed {
    indices: Range<u32>,
    base_vertex: i32,
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
