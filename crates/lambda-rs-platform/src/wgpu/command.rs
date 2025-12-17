//! Command encoding abstractions around `wgpu::CommandEncoder`, `wgpu::RenderPass`,
//! and `wgpu::CommandBuffer` that expose only the operations needed by the
//! engine while keeping raw `wgpu` types crate-internal.

use super::gpu;

/// Thin wrapper around `wgpu::CommandEncoder` with convenience helpers.
#[derive(Debug)]
pub struct CommandEncoder {
  raw: wgpu::CommandEncoder,
}

/// Wrapper around `wgpu::CommandBuffer` to avoid exposing raw types upstream.
#[derive(Debug)]
pub struct CommandBuffer {
  raw: wgpu::CommandBuffer,
}

impl CommandBuffer {
  /// Convert to the raw wgpu command buffer.
  pub(crate) fn into_raw(self) -> wgpu::CommandBuffer {
    self.raw
  }
}

impl CommandEncoder {
  /// Create a new command encoder with an optional label.
  pub fn new(gpu: &gpu::Gpu, label: Option<&str>) -> Self {
    let raw = gpu
      .device()
      .create_command_encoder(&wgpu::CommandEncoderDescriptor { label });
    return Self { raw };
  }

  /// Internal helper for beginning a render pass. Used by the render pass builder.
  pub(crate) fn begin_render_pass_raw<'pass>(
    &'pass mut self,
    desc: &wgpu::RenderPassDescriptor<'pass>,
  ) -> wgpu::RenderPass<'pass> {
    return self.raw.begin_render_pass(desc);
  }

  /// Finish recording and return the command buffer.
  pub fn finish(self) -> CommandBuffer {
    return CommandBuffer {
      raw: self.raw.finish(),
    };
  }
}

// RenderPass wrapper and its methods now live under `wgpu::render_pass`.
