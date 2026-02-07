---
title: "Indexed Draws and Multiple Vertex Buffers"
document_id: "indexed-draws-multiple-vertex-buffers-2025-11-22"
status: "draft"
created: "2025-11-22T00:00:00Z"
last_updated: "2026-02-07T00:00:00Z"
version: "0.2.1"
engine_workspace_version: "2023.1.30"
wgpu_version: "26.0.1"
shader_backend_default: "naga"
winit_version: "0.29.10"
repo_commit: "544444652b4dc3639f8b3e297e56c302183a7a0b"
owners: ["lambda-sh"]
reviewers: ["engine", "rendering"]
tags: ["spec", "rendering", "vertex-input", "indexed-draws"]
---

# Indexed Draws and Multiple Vertex Buffers

## Table of Contents

- [Summary](#summary)
- [Scope](#scope)
- [Terminology](#terminology)
- [Architecture Overview](#architecture-overview)
- [Design](#design)
  - [API Surface](#api-surface)
  - [Behavior](#behavior)
  - [Validation and Errors](#validation-and-errors)
- [Constraints and Rules](#constraints-and-rules)
- [Performance Considerations](#performance-considerations)
- [Requirements Checklist](#requirements-checklist)
- [Verification and Testing](#verification-and-testing)
- [Compatibility and Migration](#compatibility-and-migration)
- [Changelog](#changelog)

## Summary

- Specify indexed draw commands and index buffers as first-class concepts in the high-level rendering API while keeping `wgpu` types encapsulated in `lambda-rs-platform`.
- Define multiple vertex buffer support in `RenderPipelineBuilder` and the render command stream, enabling flexible vertex input layouts while preserving simple defaults.
- Rationale: Many scenes require indexed meshes and split vertex streams (for example, positions and per-instance data) for memory efficiency and performance; supporting these paths in a structured way aligns with modern GPU APIs.

## Scope

### Goals

- Expose index buffers and indexed draw commands on the high-level API without leaking backend types.
- Support multiple vertex buffers per pipeline and bind them by slot in the render command stream.
- Maintain compatibility with existing single-vertex-buffer and non-indexed draw workflows.
- Integrate with existing validation features for encoder- and pass-level safety checks.

### Non-Goals

- Multi-draw indirect and indirect command buffers.
- Per-instance rate vertex buffer layouts and advanced instancing patterns beyond simple instance ranges.
- New mesh or asset file formats; mesh containers may evolve separately.

## Terminology

- Vertex buffer: A GPU buffer containing per-vertex attribute streams consumed by the vertex shader.
- Vertex attribute: A contiguous region within a vertex buffer that feeds a shader input at a specific `location`.
- Vertex buffer slot: A numbered binding point (starting at zero) used to bind a vertex buffer layout and buffer to a pipeline.
- Index buffer: A GPU buffer containing compact integer indices that reference vertices in one or more vertex buffers.
- Index format: The integer width used to encode indices in an index buffer (16-bit or 32-bit unsigned).
- Indexed draw: A draw call that emits primitives by reading indices from an index buffer instead of traversing vertices linearly.

## Architecture Overview

- High-level layer (`lambda-rs`)
  - `RenderPipelineBuilder` declares one or more vertex buffers with associated `VertexAttribute` descriptions.
  - The render command stream includes commands to bind vertex buffers, bind an index buffer, and issue indexed or non-indexed draws.
  - Public types represent index formats and buffer classifications; backend-specific details remain internal to `lambda-rs-platform`.
- Platform layer (`lambda-rs-platform`)
  - Wraps `wgpu` vertex and index buffer usage, index formats, and render pass draw calls.
  - Provides `set_vertex_buffer`, `set_index_buffer`, `draw`, and `draw_indexed` helpers around the `wgpu::RenderPass`.
- Data flow

```
App Code
  └── lambda-rs
        ├── BufferBuilder (BufferType::Vertex / BufferType::Index)
        ├── RenderPipelineBuilder::with_buffer(..)
        └── RenderCommand::{BindVertexBuffer, BindIndexBuffer, Draw, DrawIndexed}
              └── RenderContext encoder
                    └── lambda-rs-platform (vertex/index binding, draw calls)
                          └── wgpu::RenderPass::{set_vertex_buffer, set_index_buffer, draw, draw_indexed}
```

## Design

### API Surface

- Platform layer (`lambda-rs-platform`, module `lambda_platform::wgpu::buffer`)
  - Types
    - `enum IndexFormat { Uint16, Uint32 }` (maps to `wgpu::IndexFormat`).
  - Render pass integration (module `lambda_platform::wgpu::render_pass`)
    - `fn set_vertex_buffer(&mut self, slot: u32, buffer: &buffer::Buffer)`.
    - `fn set_index_buffer(&mut self, buffer: &buffer::Buffer, format: buffer::IndexFormat)`.
    - `fn draw(&mut self, vertices: Range<u32>)`.
    - `fn draw_indexed(&mut self, indices: Range<u32>, base_vertex: i32)`.
- High-level layer (`lambda-rs`)
  - Buffer classification and creation (`lambda::render::buffer`)
    - `enum BufferType { Vertex, Index, Uniform, Storage }`.
    - `struct Buffer { buffer_type: BufferType, stride: u64, .. }`.
    - `struct Usage(platform_buffer::Usage)` with `Usage::VERTEX` and `Usage::INDEX` for vertex and index buffers.
    - `struct BufferBuilder`:
      - `fn with_usage(self, usage: Usage) -> Self`.
      - `fn with_buffer_type(self, buffer_type: BufferType) -> Self`.
      - `fn build<T: Copy>(self, render_context: &mut RenderContext, data: Vec<T>) -> Result<Buffer, Error>`.
      - `fn build_from_mesh(self, render_context: &mut RenderContext, mesh: Mesh) -> Result<Buffer, Error>` for convenience.
  - Vertex input definition (`lambda::render::vertex` and `lambda::render::pipeline`)
    - `struct VertexAttribute { location: u32, offset: u32, element: VertexElement }`. The effective byte offset of a vertex attribute is computed as `offset + element.offset`, where `offset` is a base offset within the buffer element and `element.offset` is the offset of the field within the logical vertex or instance struct.
    - `RenderPipelineBuilder::with_buffer(buffer: Buffer, attributes: Vec<VertexAttribute>) -> Self`:
      - Each call declares a vertex buffer slot with a stride and attribute list.
      - Slots are assigned in call order starting at zero.
  - Index format and commands (`lambda::render::command`)
    - Introduce an engine-level index format:
      - `enum IndexFormat { Uint16, Uint32 }`.
    - Render commands:
      - `BindVertexBuffer { pipeline: ResourceId, buffer: u32 }` binds a vertex buffer slot declared on the pipeline.
      - `BindIndexBuffer { buffer: ResourceId, format: IndexFormat }` binds an index buffer with a specific format.
      - `Draw { vertices: Range<u32>, instances: Range<u32> }` issues a non-indexed draw with an explicit instance range.
      - `DrawIndexed { indices: Range<u32>, base_vertex: i32, instances: Range<u32> }` issues an indexed draw with a signed base vertex and explicit instance range.

Example (high-level usage)

```rust
use lambda::render::{
  buffer::{BufferBuilder, BufferType, Usage},
  command::RenderCommand,
  pipeline::RenderPipelineBuilder,
};

let vertex_buffer_positions = BufferBuilder::new()
  .with_usage(Usage::VERTEX)
  .with_buffer_type(BufferType::Vertex)
  .with_label("positions")
  .build(render_context.gpu(), position_vertices)?;

let vertex_buffer_colors = BufferBuilder::new()
  .with_usage(Usage::VERTEX)
  .with_buffer_type(BufferType::Vertex)
  .with_label("colors")
  .build(render_context.gpu(), color_vertices)?;

let index_buffer = BufferBuilder::new()
  .with_usage(Usage::INDEX)
  .with_buffer_type(BufferType::Index)
  .with_label("indices")
  .build(render_context.gpu(), indices)?;

let pipeline = RenderPipelineBuilder::new()
  .with_buffer(vertex_buffer_positions, position_attributes)
  .with_buffer(vertex_buffer_colors, color_attributes)
  .build(
    render_context.gpu(),
    render_context.surface_format(),
    render_context.depth_format(),
    &render_pass,
    &vertex_shader,
    Some(&fragment_shader),
  );

let commands = vec![
  RenderCommand::BeginRenderPass { render_pass: render_pass_id, viewport },
  RenderCommand::SetPipeline { pipeline: pipeline_id },
  RenderCommand::BindVertexBuffer { pipeline: pipeline_id, buffer: 0 },
  RenderCommand::BindVertexBuffer { pipeline: pipeline_id, buffer: 1 },
  RenderCommand::BindIndexBuffer { buffer: index_buffer_id, format: IndexFormat::Uint16 },
  RenderCommand::DrawIndexed { indices: 0..index_count, base_vertex: 0, instances: 0..1 },
  RenderCommand::EndRenderPass,
];
```

### Behavior

- Vertex buffers and slots
  - `RenderPipelineBuilder` collects vertex buffer layouts in call order. Slot `0` corresponds to the first `with_buffer` call, slot `1` to the second, and so on.
  - A vertex buffer MUST have `BufferType::Vertex` and include at least one `VertexAttribute`.
  - Attributes in a slot MUST have distinct `location` values and their offsets MUST be within the buffer stride.
  - The render context uses the pipeline’s recorded buffer layouts to derive vertex buffer strides and attribute descriptors when building the platform pipeline.
- Index buffers and formats
  - Index buffers are created with `BufferType::Index` and `Usage::INDEX`. The element type of the index buffer data MUST match the `IndexFormat` passed in `BindIndexBuffer`.
  - Supported index formats are `Uint16` and `Uint32`. The engine MUST reject or log an error for any attempt to use unsupported formats.
  - At most one index buffer is considered active at a time for a render pass; subsequent `BindIndexBuffer` commands replace the previous binding.
- Draw commands
  - `Draw` uses the currently bound vertex buffers and does not require an index buffer. The `instances` range controls how many instances are emitted; a default of `0..1` preserves prior single-instance behavior.
  - `DrawIndexed` uses the currently bound index buffer and vertex buffers. If no index buffer is bound when `DrawIndexed` is encoded, the engine MUST treat this as a configuration error. The `instances` range controls how many instances are emitted.
  - `Draw` and `DrawIndexed` operate only inside an active render pass and after a pipeline has been set.
  - `base_vertex` shifts the vertex index computed from each index; this is forwarded to the platform layer and MUST be preserved exactly.
- Feature flags
  - Existing validation flags in `lambda-rs` apply:
    - `render-validation-pass-compat`: validates pipeline versus pass configuration (color/depth/stencil) and MAY be extended to ensure vertex buffer usage is compatible with the pass.
    - `render-validation-encoder`: enables per-command checks for correct binding order and buffer types.
  - Debug builds (`debug_assertions`) MAY enable additional checks regardless of feature flags.

### Validation and Errors

- Command ordering
  - `BeginRenderPass` MUST precede any `SetPipeline`, `BindVertexBuffer`, `BindIndexBuffer`, `Draw`, or `DrawIndexed` commands.
  - `EndRenderPass` MUST terminate the pass; commands after `EndRenderPass` that require a pass are considered invalid.
- Vertex buffer binding
  - The vertex buffer slot index in `BindVertexBuffer` MUST be less than the number of buffers declared on the pipeline.
  - When `render-validation-encoder` is enabled, the engine SHOULD:
    - Reject or log a configuration error if the bound buffer’s `BufferType` is not `Vertex`.
    - Emit diagnostics when a slot is bound more than once without being used for any draws.
- Index buffer binding
  - `BindIndexBuffer` MUST reference a buffer created with `BufferType::Index`. With `render-validation-encoder` enabled, the engine SHOULD validate this invariant and log a clear error when it is violated.
  - The `IndexFormat` passed in the command MUST match the element width used when creating the index data. Mismatches are undefined at the GPU level; the engine SHOULD provide validation where type information is available.
- Draw calls
  - With `render-validation-encoder` enabled, the engine SHOULD:
    - Validate that a pipeline and (for `DrawIndexed`) an index buffer are bound before encoding draw commands.
    - Validate that the index range does not exceed the logical count of indices provided at buffer creation when that length is tracked.
  - Errors SHOULD be reported with actionable messages, including the command index and pipeline label where available.

## Constraints and Rules

- Index buffers
  - Index data MUST be tightly packed according to the selected `IndexFormat` (no gaps or padding between indices).
  - Index buffers MUST be created with `Usage::INDEX`; additional usages MAY be combined when necessary (for example, `Usage::INDEX | Usage::STORAGE`).
  - Index ranges for `DrawIndexed` MUST be expressed in units of indices, not bytes.
- Vertex buffers
  - Each vertex buffer slot has exactly one stride in bytes, derived from the vertex type used at creation time.
  - Attribute offsets and formats MUST respect platform alignment requirements for the underlying GPU.
  - When multiple vertex buffers are in use, attributes for a given shader input `location` MUST appear in exactly one slot.
- Backend limits
  - The number of vertex buffer slots per pipeline MUST NOT exceed the device limit exposed through `lambda-rs-platform`.
  - The engine MUST clamp or reject configurations that exceed the maximum number of vertex buffers or vertex attributes per pipeline, logging errors when validation features are enabled.

## Performance Considerations

- Prefer 16-bit indices where possible.
  - Rationale: `Uint16` indices reduce index buffer size and memory bandwidth relative to `Uint32` when the vertex count permits it.
- Group data by update frequency across vertex buffers.
  - Rationale: Placing static geometry in one buffer and frequently updated per-instance data in another allows partial updates and reduces bandwidth.
- Avoid redundant buffer bindings.
  - Rationale: Rebinding the same buffer and slot between draws increases command traffic and validation cost without changing the GPU state.
- Use contiguous index ranges for cache-friendly access.
  - Rationale: Locality in index sequences improves vertex cache efficiency on the GPU and reduces redundant vertex shader invocations.

## Requirements Checklist

- Functionality
  - [x] Indexed draws (`DrawIndexed`) integrated with the render command stream.
  - [x] Index buffers (`BufferType::Index`, `Usage::INDEX`) created and bound through `BindIndexBuffer`.
  - [x] Multiple vertex buffers declared on `RenderPipelineBuilder` and bound via `BindVertexBuffer`.
  - [x] Edge cases handled for missing bindings and invalid ranges.
- API Surface
  - [x] Engine-level `IndexFormat` type exposed without leaking backend enums.
  - [x] Buffer builders support vertex and index usage/configuration.
  - [x] Render commands align with pipeline vertex buffer declarations.
- Validation and Errors
  - [x] Encoder ordering checks for pipeline, vertex buffers, and index buffers.
  - [x] Index range and buffer-type validation under `render-validation-encoder`.
  - [x] Device limit checks for vertex buffer slots and attributes.
- Performance
  - [x] Guidance documented in this section.
  - [ ] Indexed and non-indexed paths characterized for representative meshes.
- Documentation and Examples
  - [x] Example scene using indexed draws and multiple vertex buffers (for
  example, a mesh with separate position and color streams).

## Verification and Testing

- Unit Tests
  - Validate mapping from engine-level `IndexFormat` to platform index formats.
  - Validate command ordering rules and encoder-side checks when `render-validation-encoder` is enabled.
  - Validate vertex buffer slot bounds and buffer type checks in the encoder.
  - Commands: `cargo test -p lambda-rs -- --nocapture`
- Integration Tests and Examples
  - Example: an indexed mesh rendered with two vertex buffers (positions and colors) and a 16-bit index buffer.
  - Example: fall back to non-indexed draws for simple meshes to ensure both paths remain valid.
  - Commands:
    - `cargo run -p lambda-demos-render --bin indexed_multi_vertex_buffers`
    - `cargo test --workspace`
- Manual Checks (optional)
  - Render a mesh with and without indexed draws and visually confirm identical geometry.
  - Toggle between single and multiple vertex buffer configurations for the same mesh and confirm consistent output.

## Compatibility and Migration

- Existing pipelines that declare a single vertex buffer and use non-indexed draws remain valid; no code changes are required.
- Introducing an engine-level `IndexFormat` type for `BindIndexBuffer` is a source-compatible change when a re-export is provided for the previous platform type; call sites SHOULD migrate to the new type explicitly.
- Applications that currently emulate indexed draws by duplicating vertices MAY migrate to indexed meshes to reduce vertex buffer size and improve cache utilization.

## Changelog

- 2026-02-05 (v0.2.1) — Update demo and example references for `demos/`.
- 2025-12-15 (v0.2.0) — Update example code to use `render_context.gpu()` and add `surface_format`/`depth_format` parameters to `RenderPipelineBuilder`.
- 2025-11-22 (v0.1.0) — Initial draft specifying indexed draws and multiple vertex buffers, including API surface, behavior, validation hooks, performance guidance, and verification plan.
- 2025-11-22 (v0.1.1) — Added engine-level `IndexFormat`, instance ranges to `Draw`/`DrawIndexed`, encoder-side validation for pipeline and index buffer bindings, and updated requirements checklist.
- 2025-11-23 (v0.1.2) — Added index buffer stride and range validation, device limit checks for vertex buffer slots and attributes, an example scene with indexed draws and multiple vertex buffers, and updated the requirements checklist.
