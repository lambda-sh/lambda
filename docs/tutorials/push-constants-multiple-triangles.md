---
title: "Push Constants: Draw Multiple 2D Triangles"
document_id: "push-constants-multiple-triangles-tutorial-2025-12-16"
status: "draft"
created: "2025-12-16T00:00:00Z"
last_updated: "2025-12-16T00:00:00Z"
version: "0.1.0"
engine_workspace_version: "2023.1.30"
wgpu_version: "26.0.1"
shader_backend_default: "naga"
winit_version: "0.29.10"
repo_commit: "797047468a927f1e4ba111b43381a607ac53c0d1"
owners: ["lambda-sh"]
reviewers: ["engine", "rendering"]
tags: ["tutorial", "graphics", "push-constants", "triangle", "rust", "wgpu"]
---

## Overview <a name="overview"></a>

Push constants provide a small block of per-draw data that is cheap to update
and does not require buffers or bind groups. This tutorial draws multiple 2D
triangles by looping over a set of push constant values and issuing one draw
per triangle.

Reference implementation: `crates/lambda-rs/examples/triangles.rs`.

## Table of Contents

- [Overview](#overview)
- [Goals](#goals)
- [Prerequisites](#prerequisites)
- [Requirements and Constraints](#requirements-and-constraints)
- [Data Flow](#data-flow)
- [Implementation Steps](#implementation-steps)
  - [Step 1 — Define the Push Constant Layout](#step-1)
  - [Step 2 — Shaders for Position, Scale, and Color](#step-2)
  - [Step 3 — Build a Pipeline with Push Constants](#step-3)
  - [Step 4 — Push Constants per Draw](#step-4)
  - [Step 5 — Input and Resize Handling](#step-5)
- [Validation](#validation)
- [Notes](#notes)
- [Conclusion](#conclusion)
- [Exercises](#exercises)
- [Changelog](#changelog)

## Goals <a name="goals"></a>

- Define a push constant block in GLSL and mirror it in Rust.
- Build a pipeline that declares a vertex-stage push constant range.
- Draw multiple triangles by pushing per-draw constants and issuing draws.

## Prerequisites <a name="prerequisites"></a>

- The workspace builds: `cargo build --workspace`.
- The `lambda-rs` crate examples run: `cargo run -p lambda-rs --example minimal`.

## Requirements and Constraints <a name="requirements-and-constraints"></a>

- Push constant layout MUST match between shader and Rust in size, alignment,
  and field order (`#[repr(C)]` is required on the Rust struct).
- The pipeline MUST declare a push constant range for the stage that reads it
  (`PipelineStage::VERTEX` in this example).
- The pushed byte slice length MUST match the pipeline’s declared push constant
  size.
- Back-face culling MUST be disabled or the triangle winding MUST be adjusted.
  Rationale: the example’s vertex positions are defined in clockwise order.

## Data Flow <a name="data-flow"></a>

- CPU constructs pipeline and render pass once in `on_attach`.
- CPU builds a list of per-triangle `PushConstant` values on each frame.
- CPU emits a loop of `PushConstants` → `Draw` inside a single render pass.

ASCII diagram

```
PushConstant (CPU) ──▶ RenderCommand::PushConstants ──▶ Vertex Shader
       │                                              │
       └────────────── per triangle draw ──────────────┘
```

## Implementation Steps <a name="implementation-steps"></a>

### Step 1 — Define the Push Constant Layout <a name="step-1"></a>

Define the push constant block in the vertex shader and mirror it in Rust.

```glsl
layout(push_constant) uniform PushConstant {
  vec4 color;
  vec2 pos;
  vec2 scale;
} pcs;
```

```rust
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct PushConstant {
  color: [f32; 4],
  pos: [f32; 2],
  scale: [f32; 2],
}
```

This layout provides per-draw color, translation, and scale.

### Step 2 — Shaders for Position, Scale, and Color <a name="step-2"></a>

The vertex shader uses `gl_VertexIndex` to select base positions, then applies
`scale` and `pos`. The fragment shader outputs the interpolated color.

Reference shader sources:
- `crates/lambda-rs/assets/shaders/triangles.vert`
- `crates/lambda-rs/assets/shaders/triangles.frag`

### Step 3 — Build a Pipeline with Push Constants <a name="step-3"></a>

Compute the push constant size and configure the pipeline to accept vertex-stage
push constants. Disable culling for consistent visibility.

```rust
let push_constants_size = std::mem::size_of::<PushConstant>() as u32;

let pipeline = pipeline::RenderPipelineBuilder::new()
  .with_culling(pipeline::CullingMode::None)
  .with_push_constant(PipelineStage::VERTEX, push_constants_size)
  .build(
    render_context.gpu(),
    render_context.surface_format(),
    render_context.depth_format(),
    &render_pass,
    &self.vertex_shader,
    Some(&self.triangle_vertex),
  );
```

The pipeline definition controls which stages can read the pushed bytes.

### Step 4 — Push Constants per Draw <a name="step-4"></a>

Build a list of `PushConstant` values, then emit a loop that pushes bytes and
issues a draw for each triangle.

```rust
for triangle in triangle_data {
  commands.push(RenderCommand::PushConstants {
    pipeline: render_pipeline.clone(),
    stage: PipelineStage::VERTEX,
    offset: 0,
    bytes: Vec::from(push_constants_to_bytes(triangle)),
  });
  commands.push(RenderCommand::Draw {
    vertices: 0..3,
    instances: 0..1,
  });
}
```

This produces multiple triangles without creating any GPU buffers.

### Step 5 — Input and Resize Handling <a name="step-5"></a>

Update component state from events:
- `WindowEvent::Resize` updates the stored width/height for viewport creation.
- `KeyW`, `KeyA`, `KeyS`, `KeyD` update the translation for one triangle.

These updates are reflected on the next `on_render` call.

## Validation <a name="validation"></a>

- Build: `cargo build --workspace`
- Run: `cargo run -p lambda-rs --example triangles`
- Expected behavior: a window opens and shows multiple colored triangles; the
  `W`, `A`, `S`, and `D` keys move one triangle.

## Notes <a name="notes"></a>

- Push constant limits
  - Push constants are device-limited; the declared size MUST remain within the
    GPU’s supported push constant range.
- Layout correctness
  - The Rust `PushConstant` type MUST remain `#[repr(C)]` and must not include
    padding-sensitive fields without validating the matching GLSL layout.
- Naming
  - The reference implementation stores the fragment shader in the field named
    `triangle_vertex`; treat it as the fragment shader when extending the code.

## Conclusion <a name="conclusion"></a>

This tutorial demonstrates per-draw customization using push constants by
looping over a set of constants and issuing repeated draws within one render
pass.

## Exercises <a name="exercises"></a>

- Exercise 1: Animate color or scale
  - Update `animation_scalar` each frame and modulate one triangle’s color or
    scale over time.
- Exercise 2: Add per-triangle rotation
  - Extend the push constant block with an angle and rotate positions in the
    vertex shader.
- Exercise 3: Enable back-face culling
  - Set `.with_culling(CullingMode::Back)` and update
    `crates/lambda-rs/assets/shaders/triangles.vert` to counter-clockwise
    winding.
- Exercise 4: Consolidate into instancing
  - Convert the per-triangle loop into one instanced draw and provide per-
    instance data via a vertex buffer.
- Exercise 5: Add a UI-controlled triangle count
  - Generate `triangle_data` dynamically from runtime state and draw an
    arbitrary number of triangles.

## Changelog <a name="changelog"></a>

- 0.1.0 (2025-12-16): Initial draft aligned with
  `crates/lambda-rs/examples/triangles.rs`.
