---
title: "Indexed Draws and Multiple Vertex Buffers"
document_id: "indexed-draws-multiple-vertex-buffers-tutorial-2025-11-22"
status: "draft"
created: "2025-11-22T00:00:00Z"
last_updated: "2025-11-22T00:00:00Z"
version: "0.1.0"
engine_workspace_version: "2023.1.30"
wgpu_version: "26.0.1"
shader_backend_default: "naga"
winit_version: "0.29.10"
repo_commit: "80080b44b6c6b6c5ea8d796ea0f749608610753b"
owners: ["lambda-sh"]
reviewers: ["engine", "rendering"]
tags: ["tutorial", "graphics", "indexed-draws", "vertex-buffers", "rust", "wgpu"]
---

## Overview <a name="overview"></a>
This tutorial will construct a small scene rendered with indexed geometry and multiple vertex buffers. The example will separate per-vertex positions from per-instance colors and draw the result using the engine’s high-level buffer and command builders.

Reference implementation (planned): `crates/lambda-rs/examples/indexed_multi_vertex_buffers.rs`.

## Table of Contents
- [Overview](#overview)
- [Goals](#goals)
- [Prerequisites](#prerequisites)
- [Requirements and Constraints](#requirements-and-constraints)
- [Data Flow](#data-flow)
- [Implementation Steps](#implementation-steps)
  - [Step 1 — Runtime and Component Skeleton](#step-1)
  - [Step 2 — Vertex and Fragment Shaders](#step-2)
  - [Step 3 — Vertex Data, Index Data, and Layouts](#step-3)
  - [Step 4 — Create Vertex and Index Buffers](#step-4)
  - [Step 5 — Build the Render Pipeline with Multiple Buffers](#step-5)
  - [Step 6 — Record Commands with BindVertexBuffer and BindIndexBuffer](#step-6)
  - [Step 7 — Add Simple Camera or Transform](#step-7)
  - [Step 8 — Handle Resize and Resource Lifetime](#step-8)
- [Validation](#validation)
- [Notes](#notes)
- [Conclusion](#conclusion)
- [Exercises](#exercises)
- [Changelog](#changelog)

## Goals <a name="goals"></a>

- Render indexed geometry using an index buffer and `DrawIndexed` commands.
- Demonstrate multiple vertex buffers bound to a single pipeline (for example, positions in one buffer and colors in another).
- Show how the engine associates vertex buffer slots with shader locations and how those slots are bound via render commands.
- Reinforce correct buffer usage flags and buffer types for vertex and index data.

## Prerequisites <a name="prerequisites"></a>

- The workspace builds successfully: `cargo build --workspace`.
- Familiarity with the basics of the runtime and component model.
- Ability to run examples and tutorials:
  - `cargo run --example minimal`
  - `cargo run -p lambda-rs --example textured_quad`

## Requirements and Constraints <a name="requirements-and-constraints"></a>

- Vertex buffer layouts in the pipeline MUST match shader attribute `location` and format declarations.
- Index data MUST be tightly packed in the chosen index format (`u16` or `u32`) and the `IndexFormat` passed to the command MUST correspond to the element width.
- Vertex buffers used for geometry MUST be created with `Usage::VERTEX` and an appropriate `BufferType` value; index buffers MUST use `Usage::INDEX` and `BufferType::Index`.
- Draw commands that rely on indexed geometry MUST bind a pipeline, vertex buffers, and an index buffer inside an active render pass before issuing `DrawIndexed`.

## Data Flow <a name="data-flow"></a>

- CPU prepares vertex data (positions, colors) and index data.
- Buffers and pipeline layouts are constructed using the builder APIs.
- At render time, commands bind the pipeline, vertex buffers, and index buffer, then issue indexed draws.

ASCII diagram

```
CPU (positions, colors, indices)
   │  upload via BufferBuilder
   ▼
Vertex Buffers (slots 0, 1)      Index Buffer
   │                                   │
   ├───────────────┐                   │
   ▼               ▼                   ▼
RenderPipeline (vertex layouts)   RenderCommand::BindIndexBuffer
   │
RenderCommand::{BindVertexBuffer, DrawIndexed}
   │
Render Pass → wgpu::RenderPass::{set_vertex_buffer, set_index_buffer, draw_indexed}
```

## Implementation Steps <a name="implementation-steps"></a>

### Step 1 — Runtime and Component Skeleton <a name="step-1"></a>
Explain how to create a minimal runtime and component that will own the render context, pipelines, and buffers required for the tutorial. The code in this step will set up the application window, establish initial dimensions, and register a component that performs initialization and per-frame rendering.

Code placeholder:

```rust
// Entry point, runtime builder, and minimal component struct
// This code will mirror the patterns used in other tutorials
// and examples such as `uniform_buffer_triangle`.
```

Narrative placeholder:

Describe how the runtime and component are connected and why the component is a suitable place to allocate GPU resources and record render commands.

### Step 2 — Vertex and Fragment Shaders <a name="step-2"></a>
Describe the planned shader interface:
- Vertex shader: position from one vertex buffer slot and color (or instance data) from another slot.
- Fragment shader: pass-through of interpolated color or simple shading.

Code placeholder:

```glsl
// Vertex shader with attributes at multiple locations
// Fragment shader that outputs a color based on inputs
```

Narrative placeholder:

Explain how attribute locations map to vertex buffer layouts and how those layouts will be declared in the pipeline builder.

### Step 3 — Vertex Data, Index Data, and Layouts <a name="step-3"></a>
Define how vertex and index data will be structured:
- Vertex buffer 0: per-vertex positions.
- Vertex buffer 1: per-vertex or per-instance colors.
- Index buffer: indices referencing positions (and implicitly associated colors).

Code placeholder:

```rust
// Rust structures and arrays for positions, colors, and indices
// VertexAttribute lists for each buffer slot
```

Narrative placeholder:

Explain the relationship between index values and vertices and how the chosen layout enables indexed rendering.

### Step 4 — Create Vertex and Index Buffers <a name="step-4"></a>
Show how to convert the CPU-side arrays into GPU buffers using `BufferBuilder`:
- Vertex buffers using `Usage::VERTEX` and `BufferType::Vertex`.
- Index buffer using `Usage::INDEX` and `BufferType::Index`.

Code placeholder:

```rust
// BufferBuilder calls to create two vertex buffers and one index buffer
// with labels that identify their roles in debugging tools
```

Narrative placeholder:

Explain why usage flags and buffer types matter for correct binding and validation.

### Step 5 — Build the Render Pipeline with Multiple Buffers <a name="step-5"></a>
Demonstrate how to construct a pipeline that declares multiple vertex buffers:
- Use `RenderPipelineBuilder::with_buffer` once per vertex buffer.
- Ensure attribute lists map correctly to shader locations.

Code placeholder:

```rust
// RenderPipelineBuilder configuration, including with_buffer calls for
// each vertex buffer and attachment of compiled shaders
```

Narrative placeholder:

Clarify how the engine assigns vertex buffer slots and how those slots correspond to bind calls in the command stream.

### Step 6 — Record Commands with BindVertexBuffer and BindIndexBuffer <a name="step-6"></a>
Describe command recording for a frame:
- Begin a render pass and set the pipeline.
- Bind vertex buffers for each slot.
- Bind the index buffer with the correct `IndexFormat`.
- Issue one or more `DrawIndexed` commands.

Code placeholder:

```rust
// RenderCommand sequence using:
// BeginRenderPass, SetPipeline,
// BindVertexBuffer (for slots 0 and 1),
// BindIndexBuffer, DrawIndexed, EndRenderPass
```

Narrative placeholder:

Explain the ordering requirements for commands and highlight how incorrect ordering would be handled by validation features.

### Step 7 — Add Simple Camera or Transform <a name="step-7"></a>
Outline an optional addition of a simple transform or camera:
- Uniform buffer or push constants for a model-view-projection matrix.
- Per-frame update of the transform to demonstrate motion.

Code placeholder:

```rust
// Optional uniform buffer or push constant setup to animate the geometry
```

Narrative placeholder:

Describe how transforms and camera settings integrate with the indexed draw path without changing the vertex/index buffer setup.

### Step 8 — Handle Resize and Resource Lifetime <a name="step-8"></a>
Summarize how to:
- Respond to window resize events.
- Rebuild dependent resources if necessary (for example, passes or pipelines).
- Clean up buffers and pipelines when the component is destroyed.

Code placeholder:

```rust
// Skeleton handlers for resize and cleanup callbacks
```

Narrative placeholder:

Explain which resources are dependent on window size and which can persist across resizes.

## Validation <a name="validation"></a>

- Commands (planned once the example exists):
  - `cargo run -p lambda-rs --example indexed_multi_vertex_buffers`
  - `cargo test -p lambda-rs -- --nocapture`
- Expected behavior:
  - Indexed geometry renders correctly with distinct colors sourced from a second vertex buffer.
  - Switching between indexed and non-indexed paths SHOULD produce visually consistent geometry for the same mesh.

## Notes <a name="notes"></a>

- Vertex buffer slot indices MUST remain consistent between pipeline construction and binding commands.
- Index ranges for `DrawIndexed` MUST remain within the logical count of indices provided when the index buffer is created.
- Validation features such as `render-validation-encoder` SHOULD be enabled when developing new render paths to catch ordering and binding issues early.

## Conclusion <a name="conclusion"></a>

This tutorial will show how indexed draws and multiple vertex buffers combine to render geometry efficiently while keeping the engine’s high-level abstractions simple. The final example will provide a concrete reference for applications that require indexed meshes or split vertex streams.

## Exercises <a name="exercises"></a>

- Extend the example to render multiple meshes that share the same index buffer but use different color data.
- Add a per-instance transform buffer and demonstrate instanced drawing by varying transforms while reusing positions and indices.
- Introduce a wireframe mode that uses the same vertex and index buffers but modifies pipeline state to emphasize edge connectivity.
- Experiment with `u16` versus `u32` indices and measure the effect on buffer size and performance for larger meshes.
- Add a debug mode that binds an incorrect index format intentionally and observe how validation features report the error.

## Changelog <a name="changelog"></a>

- 2025-11-22 (v0.1.0) — Initial skeleton for the indexed draws and multiple vertex buffers tutorial; content placeholders added for future implementation.
