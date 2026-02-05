---
title: "Basic Triangle: Vertex‑Only Draw"
document_id: "basic-triangle-tutorial-2025-12-16"
status: "draft"
created: "2025-12-16T00:00:00Z"
last_updated: "2026-02-05T23:05:40Z"
version: "0.2.4"
engine_workspace_version: "2023.1.30"
wgpu_version: "28.0.0"
shader_backend_default: "naga"
winit_version: "0.29.10"
repo_commit: "544444652b4dc3639f8b3e297e56c302183a7a0b"
owners: ["lambda-sh"]
reviewers: ["engine", "rendering"]
tags: ["tutorial", "graphics", "triangle", "rust", "wgpu"]
---

## Overview <a name="overview"></a>

This tutorial renders a single 2D triangle using a vertex shader that derives
positions from `gl_VertexIndex`. The implementation uses no vertex buffers and
demonstrates the minimal render pass, pipeline, and command sequence in
`lambda-rs`.

Reference implementation: `demos/render/src/bin/triangle.rs`.

## Table of Contents

- [Overview](#overview)
- [Goals](#goals)
- [Prerequisites](#prerequisites)
- [Requirements and Constraints](#requirements-and-constraints)
- [Data Flow](#data-flow)
- [Implementation Steps](#implementation-steps)
  - [Step 1 — Runtime and Component Skeleton](#step-1)
  - [Step 2 — Vertex and Fragment Shaders](#step-2)
  - [Step 3 — Compile Shaders with `ShaderBuilder`](#step-3)
  - [Step 4 — Build Render Pass and Pipeline](#step-4)
  - [Step 5 — Issue Render Commands](#step-5)
  - [Step 6 — Handle Window Resize](#step-6)
- [Validation](#validation)
- [Notes](#notes)
- [Conclusion](#conclusion)
- [Exercises](#exercises)
- [Changelog](#changelog)

## Goals <a name="goals"></a>

- Render a triangle with a vertex shader driven by `gl_VertexIndex`.
- Learn the minimal `RenderCommand` sequence for a draw.
- Construct a `RenderPass` and `RenderPipeline` using builder APIs.

## Prerequisites <a name="prerequisites"></a>

- The workspace builds: `cargo build --workspace`.
- The minimal demo runs: `cargo run -p lambda-demos-minimal --bin minimal`.

## Requirements and Constraints <a name="requirements-and-constraints"></a>

- Rendering commands MUST be issued inside an active render pass
  (`RenderCommand::BeginRenderPass` ... `RenderCommand::EndRenderPass`).
- The pipeline MUST be set before draw commands (`RenderCommand::SetPipeline`).
- The shader interface MUST match the pipeline configuration (no vertex buffers
  are declared for this example).
- Back-face culling MUST be disabled or the triangle winding MUST be adjusted.
  Rationale: the example’s vertex positions are defined in clockwise order.

## Data Flow <a name="data-flow"></a>

- CPU builds shaders and pipeline once in `on_attach`.
- CPU emits render commands each frame in `on_render`.
- The GPU generates vertex positions from `gl_VertexIndex` (no vertex buffers).

ASCII diagram

```
Component::on_attach
  ├─ ShaderBuilder → Shader modules
  ├─ RenderPassBuilder → RenderPass
  └─ RenderPipelineBuilder → RenderPipeline

Component::on_render (each frame)
  BeginRenderPass → SetPipeline → SetViewports/Scissors → Draw → EndRenderPass
```

## Implementation Steps <a name="implementation-steps"></a>

### Step 1 — Runtime and Component Skeleton <a name="step-1"></a>

Create an `ApplicationRuntime` and register a `Component` that receives
`on_attach`, `on_render`, and optional `on_*_event` callbacks.

```rust
fn main() {
  let runtime = ApplicationRuntimeBuilder::new("2D Triangle Demo")
    .with_window_configured_as(|window_builder| {
      return window_builder
        .with_dimensions(1200, 600)
        .with_name("2D Triangle Window");
    })
    .with_component(|runtime, demo: DemoComponent| {
      return (runtime, demo);
    })
    .build();

  start_runtime(runtime);
}
```

The runtime drives component lifecycle and calls `on_render` on each frame.

### Step 2 — Vertex and Fragment Shaders <a name="step-2"></a>

The vertex shader generates positions from `gl_VertexIndex` so the draw call
only needs a vertex count of `3`.

```glsl
vec2 positions[3];
positions[0] = vec2(0.0, -0.5);
positions[1] = vec2(-0.5, 0.5);
positions[2] = vec2(0.5, 0.5);

gl_Position = vec4(positions[gl_VertexIndex], 0.0, 1.0);
```

The fragment shader outputs a constant color.

### Step 3 — Compile Shaders with `ShaderBuilder` <a name="step-3"></a>

Load shader sources from `crates/lambda-rs/assets/shaders/` and compile them
using `ShaderBuilder`.

```rust
let triangle_vertex = VirtualShader::Source {
  source: include_str!("../assets/shaders/triangle.vert").to_string(),
  kind: ShaderKind::Vertex,
  name: String::from("triangle"),
  entry_point: String::from("main"),
};
```

The compiled `Shader` objects are stored in component state and passed to the
pipeline builder during `on_attach`.

### Step 4 — Build Render Pass and Pipeline <a name="step-4"></a>

Construct a `RenderPass` targeting the surface format, then build a pipeline.
Disable culling to ensure the triangle is visible regardless of winding.

```rust
let render_pass = render_pass::RenderPassBuilder::new().build(
  render_context.gpu(),
  render_context.surface_format(),
  render_context.depth_format(),
);

let pipeline = pipeline::RenderPipelineBuilder::new()
  .with_culling(pipeline::CullingMode::None)
  .build(
    render_context.gpu(),
    render_context.surface_format(),
    render_context.depth_format(),
    &render_pass,
    &self.vertex_shader,
    Some(&self.fragment_shader),
  );
```

Attach the created resources to the `RenderContext` and store their IDs.

### Step 5 — Issue Render Commands <a name="step-5"></a>

Emit a pass begin, bind the pipeline, set viewport/scissor, and issue a draw.

```rust
RenderCommand::Draw {
  vertices: 0..3,
  instances: 0..1,
}
```

This produces one triangle using three implicit vertices.

### Step 6 — Handle Window Resize <a name="step-6"></a>

Track `WindowEvent::Resize` and rebuild the `Viewport` each frame using the
stored dimensions.

The viewport and scissor MUST match the surface dimensions to avoid clipping or
undefined behavior when the window resizes.

Implement resize handling using `event_mask()` and `on_window_event`.

```rust
use lambda::events::{EventMask, WindowEvent};

// Inside `impl Component<ComponentResult, String> for DemoComponent`.
fn event_mask(&self) -> EventMask {
  return EventMask::WINDOW;
}

fn on_window_event(&mut self, event: &WindowEvent) -> Result<(), String> {
  if let WindowEvent::Resize { width, height } = event {
    self.width = *width;
    self.height = *height;
  }
  return Ok(());
}
```

This setup ensures the runtime only dispatches window events to the component,
and the component keeps a current width/height for viewport creation.

## Validation <a name="validation"></a>

- Build: `cargo build --workspace`
- Run: `cargo run -p lambda-demos-render --bin triangle`
- Expected behavior: a window opens and shows a solid-color triangle.

## Notes <a name="notes"></a>

- Culling and winding
  - This tutorial disables culling via `.with_culling(CullingMode::None)`.
  - If culling is enabled, the vertex order in
    `crates/lambda-rs/assets/shaders/triangle.vert` SHOULD be updated to
    counter-clockwise winding for a default `front_face = CCW` pipeline.
- Debugging
  - If the window is blank, verify that the pipeline is set inside the render
    pass and the draw uses `0..3` vertices.

## Conclusion <a name="conclusion"></a>

This tutorial demonstrates the minimal `lambda-rs` rendering path: compile
shaders, build a render pass and pipeline, and issue a draw using
`RenderCommand`s.

## Exercises <a name="exercises"></a>

- Exercise 1: Change the triangle color
  - Modify `crates/lambda-rs/assets/shaders/triangle.frag` to output a different
    constant color.
- Exercise 2: Enable back-face culling
  - Set `.with_culling(CullingMode::Back)` and update the vertex order in
    `crates/lambda-rs/assets/shaders/triangle.vert` to counter-clockwise.
- Exercise 3: Add a second triangle
  - Issue a second `Draw` and offset positions in the shader for one of the
    triangles.
- Exercise 4: Introduce immediates
  - Add an immediate data block for color and position and port the shader interface to
    match `demos/render/src/bin/triangles.rs`.
- Exercise 5: Replace `gl_VertexIndex` with a vertex buffer
  - Create a vertex buffer for positions and update the pipeline and shader
    inputs accordingly.

## Changelog <a name="changelog"></a>

- 0.2.4 (2026-02-05): Update demo commands and reference paths for `demos/`.
- 0.2.3 (2026-01-16): Normalize event handler terminology.
- 0.2.2 (2026-01-16): Add `event_mask()` and `on_window_event` resize example.
- 0.2.1 (2026-01-16): Replace deprecated `on_event` references with per-category handlers.
- 0.2.0 (2026-01-05): Update for wgpu v28; rename push constants to immediates in exercises.
- 0.1.0 (2025-12-16): Initial draft aligned with
  `demos/render/src/bin/triangle.rs`.
