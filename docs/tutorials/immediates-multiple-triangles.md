---
title: "Immediates: Draw Multiple 2D Triangles"
document_id: "immediates-multiple-triangles-tutorial-2025-12-16"
status: "draft"
created: "2025-12-16T00:00:00Z"
last_updated: "2026-01-16T00:00:00Z"
version: "0.2.3"
engine_workspace_version: "2023.1.30"
wgpu_version: "28.0.0"
shader_backend_default: "naga"
winit_version: "0.29.10"
repo_commit: "87aa423aca541823f271101e5bac390f5ca54c42"
owners: ["lambda-sh"]
reviewers: ["engine", "rendering"]
tags: ["tutorial", "graphics", "immediates", "triangle", "rust", "wgpu"]
---

## Overview <a name="overview"></a>

Immediates (formerly push constants in wgpu < v28) provide a small block of
per-draw data that is cheap to update and does not require buffers or bind
groups. This tutorial draws multiple 2D triangles by looping over a set of
immediate values and issuing one draw per triangle.

> **Note:** In wgpu v28, push constants were renamed to "immediates" and gated
> behind the `Features::IMMEDIATES` feature flag. The GLSL shaders still use
> `layout(push_constant)` syntax.

Reference implementation: `crates/lambda-rs/examples/triangles.rs`.

## Table of Contents

- [Overview](#overview)
- [Goals](#goals)
- [Prerequisites](#prerequisites)
- [Requirements and Constraints](#requirements-and-constraints)
- [Data Flow](#data-flow)
- [Implementation Steps](#implementation-steps)
  - [Step 1 — Define the Immediate Data Layout](#step-1)
  - [Step 2 — Shaders for Position, Scale, and Color](#step-2)
  - [Step 3 — Build a Pipeline with Immediates](#step-3)
  - [Step 4 — Immediates per Draw](#step-4)
  - [Step 5 — Input and Resize Handling](#step-5)
- [Validation](#validation)
- [Notes](#notes)
- [Conclusion](#conclusion)
- [Exercises](#exercises)
- [Changelog](#changelog)

## Goals <a name="goals"></a>

- Define an immediate data block in GLSL (using `push_constant`) and mirror it in Rust.
- Build a pipeline that declares an immediate data range.
- Draw multiple triangles by setting per-draw immediates and issuing draws.

## Prerequisites <a name="prerequisites"></a>

- The workspace builds: `cargo build --workspace`.
- The `lambda-rs` crate examples run: `cargo run -p lambda-rs --example minimal`.

## Requirements and Constraints <a name="requirements-and-constraints"></a>

- Immediate data layout MUST match between shader and Rust in size, alignment,
  and field order (`#[repr(C)]` is required on the Rust struct).
- The pipeline MUST declare an immediate data range that matches the shader
  immediate block size.
- The immediate byte slice length MUST match the pipeline's declared size.
- Back-face culling MUST be disabled or the triangle winding MUST be adjusted.
  Rationale: the example’s vertex positions are defined in clockwise order.

## Data Flow <a name="data-flow"></a>

- CPU constructs pipeline and render pass once in `on_attach`.
- CPU builds a list of per-triangle `ImmediateData` values on each frame.
- CPU emits a loop of `Immediates` → `Draw` inside a single render pass.

ASCII diagram

```
ImmediateData (CPU) ──▶ RenderCommand::Immediates ──▶ Vertex Shader
       │                                              │
       └────────────── per triangle draw ──────────────┘
```

## Implementation Steps <a name="implementation-steps"></a>

### Step 1 — Define the Immediate Data Layout <a name="step-1"></a>

Define the immediate data block in the vertex shader (using `push_constant`
syntax) and mirror it in Rust.

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
pub struct ImmediateData {
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

### Step 3 — Build a Pipeline with Immediates <a name="step-3"></a>

Compute the immediate data size and configure the pipeline to accept immediate
data. Disable culling for consistent visibility.

```rust
let immediate_size = std::mem::size_of::<ImmediateData>() as u32;

let pipeline = pipeline::RenderPipelineBuilder::new()
  .with_culling(pipeline::CullingMode::None)
  .with_immediate_data(immediate_size)
  .build(
    render_context.gpu(),
    render_context.surface_format(),
    render_context.depth_format(),
    &render_pass,
    &self.vertex_shader,
    Some(&self.triangle_vertex),
  );
```

The pipeline definition declares the byte range available as immediates.

### Step 4 — Set Immediates per Draw <a name="step-4"></a>

Build a list of `ImmediateData` values, then emit a loop that sets immediate
bytes and issues a draw for each triangle.

```rust
for triangle in triangle_data {
  commands.push(RenderCommand::Immediates {
    pipeline: render_pipeline.clone(),
    offset: 0,
    bytes: Vec::from(immediate_data_to_bytes(triangle)),
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

Implement input and resize handling using `event_mask()` and `on_*_event`
handlers.

```rust
use lambda::events::{EventMask, Key, VirtualKey, WindowEvent};

// Inside `impl Component<ComponentResult, String> for TrianglesExample`.
fn event_mask(&self) -> EventMask {
  return EventMask::WINDOW | EventMask::KEYBOARD;
}

fn on_window_event(&mut self, event: &WindowEvent) -> Result<(), String> {
  if let WindowEvent::Resize { width, height } = event {
    self.width = *width;
    self.height = *height;
  }
  return Ok(());
}

fn on_keyboard_event(&mut self, event: &Key) -> Result<(), String> {
  match event {
    Key::Pressed {
      scan_code: _,
      virtual_key: Some(VirtualKey::KeyW),
    } => {
      self.position.1 -= 0.01;
    }
    Key::Pressed {
      scan_code: _,
      virtual_key: Some(VirtualKey::KeyS),
    } => {
      self.position.1 += 0.01;
    }
    Key::Pressed {
      scan_code: _,
      virtual_key: Some(VirtualKey::KeyA),
    } => {
      self.position.0 -= 0.01;
    }
    Key::Pressed {
      scan_code: _,
      virtual_key: Some(VirtualKey::KeyD),
    } => {
      self.position.0 += 0.01;
    }
    _ => {}
  }
  return Ok(());
}
```

This setup declares interest in window and keyboard categories and avoids
per-component pattern matching over the full `Events` enum.

## Validation <a name="validation"></a>

- Build: `cargo build --workspace`
- Run: `cargo run -p lambda-rs --example triangles`
- Expected behavior: a window opens and shows multiple colored triangles; the
  `W`, `A`, `S`, and `D` keys move one triangle.

## Notes <a name="notes"></a>

- Immediate data limits
  - Immediate data is device-limited; the declared size MUST remain within the
    GPU's supported immediate data range. wgpu v28 requires the `Features::IMMEDIATES`
    feature to be enabled.
- Layout correctness
  - The Rust `ImmediateData` type MUST remain `#[repr(C)]` and must not include
    padding-sensitive fields without validating the matching GLSL layout.
- Naming
  - The reference implementation stores the fragment shader in the field named
    `triangle_vertex`; treat it as the fragment shader when extending the code.

## Conclusion <a name="conclusion"></a>

This tutorial demonstrates per-draw customization using immediates (wgpu's v28
replacement for push constants) by looping over a set of immediate values and
issuing repeated draws within one render pass.

## Exercises <a name="exercises"></a>

- Exercise 1: Animate color or scale
  - Update `animation_scalar` each frame and modulate one triangle’s color or
    scale over time.
- Exercise 2: Add per-triangle rotation
  - Extend the immediate data block with an angle and rotate positions in the
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

- 0.2.3 (2026-01-16): Normalize event handler terminology.
- 0.2.2 (2026-01-16): Add `event_mask()` and `on_*_event` handler examples.
- 0.2.1 (2026-01-07): Remove stage usage from immediates API examples.
- 0.2.0 (2026-01-05): Updated to use wgpu v28 immediates terminology.
- 0.1.0 (2025-12-16): Initial draft aligned with
  `crates/lambda-rs/examples/triangles.rs`.
