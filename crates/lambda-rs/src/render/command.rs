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
  /// Issue a non‑indexed draw for the provided vertex range.
  Draw { vertices: Range<u32> },
}
