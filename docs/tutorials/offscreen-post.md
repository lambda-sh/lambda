---
title: "Offscreen Post: Render to a Texture and Sample to the Surface"
document_id: "offscreen-post-tutorial-2025-12-29"
status: "draft"
created: "2025-12-29T00:00:00Z"
last_updated: "2025-12-29T00:00:00Z"
version: "0.1.0"
engine_workspace_version: "2023.1.30"
wgpu_version: "26.0.1"
shader_backend_default: "naga"
winit_version: "0.29.10"
repo_commit: "bc2ca687922db601998e7e5a0c0b2e870c857be1"
owners: ["lambda-sh"]
reviewers: ["engine", "rendering"]
tags: ["tutorial", "graphics", "offscreen", "render-targets", "multipass", "post-processing", "texture", "sampler", "wgpu", "rust"]
---

## Overview <a name="overview"></a>

This tutorial renders a triangle into an offscreen render target, then samples
that target in a second pass to present the result on the window surface. The
implementation demonstrates multi-pass rendering, bind groups for texture
sampling, and resource replacement during window resize.

Reference implementation: `crates/lambda-rs/examples/offscreen_post.rs`.

## Table of Contents

- [Overview](#overview)
- [Goals](#goals)
- [Prerequisites](#prerequisites)
- [Requirements and Constraints](#requirements-and-constraints)
- [Data Flow](#data-flow)
- [Implementation Steps](#implementation-steps)
  - [Step 1 — Runtime and Component Skeleton](#step-1)
  - [Step 2 — Post Shaders: Fullscreen Sample](#step-2)
  - [Step 3 — Build and Attach an Offscreen Target](#step-3)
  - [Step 4 — Passes and Pipelines](#step-4)
  - [Step 5 — Sampling Bind Group](#step-5)
  - [Step 6 — Fullscreen Quad Mesh and Vertex Buffer](#step-6)
  - [Step 7 — Render Commands and Resize Replacement](#step-7)
- [Validation](#validation)
- [Notes](#notes)
- [Conclusion](#conclusion)
- [Exercises](#exercises)
- [Changelog](#changelog)

## Goals <a name="goals"></a>

- Render into an offscreen color texture using `RenderDestination::Offscreen`.
- Sample the offscreen result in a second pass using a bind group.
- Replace the offscreen target and dependent bind group on window resize.

## Prerequisites <a name="prerequisites"></a>

- The workspace builds: `cargo build --workspace`.
- The `lambda-rs` crate examples run:
  `cargo run -p lambda-rs --example minimal`.

## Requirements and Constraints <a name="requirements-and-constraints"></a>

- The offscreen target color texture MUST be created with both render-attachment
  and sampled usage. Use `OffscreenTargetBuilder` to ensure correct usage.
- The offscreen pass/pipeline color format MUST match the offscreen target
  format. This example uses `render_context.surface_format()` for both.
- The bind group layout bindings MUST match the shader declarations:
  `layout (set = 0, binding = 1)` for the texture and `binding = 2` for the
  sampler.
- Replacing an offscreen target MUST also replace any bind groups that reference
  the previous target’s texture view.
- Acronyms: graphics processing unit (GPU), central processing unit (CPU),
  texture coordinates (UV).

## Data Flow <a name="data-flow"></a>

```
Component::on_attach
  ├─ OffscreenTargetBuilder → OffscreenTarget (attached)
  ├─ RenderPassBuilder → offscreen pass + post pass (attached)
  ├─ RenderPipelineBuilder → offscreen pipeline + post pipeline (attached)
  └─ BindGroupLayout/BindGroup → sample offscreen color texture

Component::on_render (each frame)
  Pass A (Offscreen): draw triangle → offscreen color texture
  Pass B (Surface): sample offscreen texture → fullscreen quad
```

## Implementation Steps <a name="implementation-steps"></a>

### Step 1 — Runtime and Component Skeleton <a name="step-1"></a>

Create an `ApplicationRuntime` and register a component that stores shader
handles and resource IDs for two passes.

```rust
pub struct OffscreenPostExample {
  triangle_vs: Shader,
  triangle_fs: Shader,
  post_vs: Shader,
  post_fs: Shader,

  offscreen_pass: Option<ResourceId>,
  offscreen_pipeline: Option<ResourceId>,
  offscreen_target: Option<ResourceId>,

  post_pass: Option<ResourceId>,
  post_pipeline: Option<ResourceId>,
  post_bind_group: Option<ResourceId>,
  post_layout: Option<BindGroupLayout>,

  width: u32,
  height: u32,
}
```

The component builds GPU resources in `on_attach`, emits commands in `on_render`,
and updates the stored dimensions in `on_event`.

### Step 2 — Post Shaders: Fullscreen Sample <a name="step-2"></a>

Define a post vertex shader that passes UV coordinates and a fragment shader
that samples a `texture2D` with a `sampler`.

```glsl
layout (set = 0, binding = 1) uniform texture2D tex;
layout (set = 0, binding = 2) uniform sampler samp;

void main() {
  fragment_color = texture(sampler2D(tex, samp), v_uv);
}
```

The shader interface defines the bind group layout requirements for Step 5.

### Step 3 — Build and Attach an Offscreen Target <a name="step-3"></a>

Build an offscreen target sized to the current surface and attach it to the
`RenderContext`.

```rust
let (width, height) = render_context.surface_size();
let offscreen_target = OffscreenTargetBuilder::new()
  .with_color(render_context.surface_format(), width, height)
  .with_label("offscreen-post-target")
  .build(render_context.gpu())
  .map_err(|e| format!("Failed to build offscreen target: {:?}", e))?;

let offscreen_target_id =
  render_context.attach_offscreen_target(offscreen_target);
```

The attached ID is used later with `RenderDestination::Offscreen`.

### Step 4 — Passes and Pipelines <a name="step-4"></a>

Create two passes: one for offscreen rendering and one for the surface. Build
one pipeline per pass.

```rust
let offscreen_pass =
  RenderPassBuilder::new().with_label("offscreen-pass").build(
    render_context.gpu(),
    render_context.surface_format(),
    render_context.depth_format(),
  );

let offscreen_pipeline = RenderPipelineBuilder::new()
  .with_label("offscreen-pipeline")
  .with_culling(CullingMode::None)
  .build(
    render_context.gpu(),
    render_context.surface_format(),
    render_context.depth_format(),
    &offscreen_pass,
    &self.triangle_vs,
    Some(&self.triangle_fs),
  );
```

The post pipeline adds a layout and a vertex buffer in Step 5 and Step 6.

### Step 5 — Sampling Bind Group <a name="step-5"></a>

Build a bind group layout that matches the post fragment shader and create a
sampler and bind group that reference the offscreen target’s color texture.

```rust
let post_layout = BindGroupLayoutBuilder::new()
  .with_sampled_texture(1)
  .with_sampler(2)
  .build(render_context.gpu());

let sampler = SamplerBuilder::new()
  .linear_clamp()
  .with_label("offscreen-post-sampler")
  .build(render_context.gpu());

let offscreen_ref =
  render_context.get_offscreen_target(offscreen_target_id);
let post_bind_group = BindGroupBuilder::new()
  .with_layout(&post_layout)
  .with_texture(1, offscreen_ref.color_texture())
  .with_sampler(2, &sampler)
  .build(render_context.gpu());
```

The post pipeline uses `.with_layouts(&[&post_layout])` so set `0` is defined.

### Step 6 — Fullscreen Quad Mesh and Vertex Buffer <a name="step-6"></a>

Build a fullscreen quad (two triangles) and pack UV coordinates into the
`Vertex` color attribute at location `2` to match the post vertex shader.

```rust
VertexBuilder::new()
  .with_position([-1.0, -1.0, 0.0])
  .with_normal([0.0, 0.0, 1.0])
  .with_color([0.0, 0.0, 0.0])
  .build();
```

Upload the mesh to a vertex buffer and attach it to the post pipeline:
`BufferBuilder::build_from_mesh(&quad_mesh, render_context.gpu())`.

### Step 7 — Render Commands and Resize Replacement <a name="step-7"></a>

Emit two passes per frame. Use `BeginRenderPassTo` with an offscreen destination
for the first pass, then sample the result to the surface in the second pass.

```rust
RenderCommand::BeginRenderPassTo {
  render_pass: offscreen_pass_id,
  viewport,
  destination: RenderDestination::Offscreen(offscreen_target_id),
},
// draw triangle ...
RenderCommand::BeginRenderPass { render_pass: post_pass_id, viewport },
// set bind group + bind vertex buffer + draw quad ...
```

When the window resizes, rebuild the offscreen target and replace both the
target and the post bind group.

```rust
if target_size != surface_size {
  let new_target = OffscreenTargetBuilder::new()
    .with_color(render_context.surface_format(), surface_size.0, surface_size.1)
    .with_label("offscreen-post-target")
    .build(render_context.gpu())?;

  render_context.replace_offscreen_target(offscreen_id, new_target)?;
  // Rebuild bind group with the new target’s `color_texture()`.
}
```

The reference implementation performs this replacement in
`ensure_offscreen_matches_surface`.

## Validation <a name="validation"></a>

- Build: `cargo build --workspace`
- Run: `cargo run -p lambda-rs --example offscreen_post`
- Expected behavior:
  - A window opens and shows a solid-color triangle.
  - Resizing the window preserves the rendering without stretching artifacts.

## Notes <a name="notes"></a>

- Format matching
  - The offscreen target and the offscreen pass/pipeline MUST agree on the
    color format. Use `render_context.surface_format()` to match the window.
- Bindings
  - `BindGroupLayoutBuilder::with_sampled_texture(1)` MUST match
    `layout (set = 0, binding = 1)` in the fragment shader.
  - The sampler binding index MUST also match (`binding = 2`).
- Resize
  - Replacing the offscreen target invalidates the previous texture view.
    Rebuild the bind group after calling
    `render_context.replace_offscreen_target`.

## Conclusion <a name="conclusion"></a>

This tutorial demonstrates a minimal multi-pass post path in `lambda-rs`:
render into an offscreen texture, then sample that texture to the surface using
a fullscreen quad and a bind group.

## Exercises <a name="exercises"></a>

- Exercise 1: Apply a post effect
  - Modify the post fragment shader to invert colors or apply a grayscale
    conversion before writing `fragment_color`.
- Exercise 2: Render offscreen at half resolution
  - Create the offscreen target at `width / 2`, `height / 2` and adjust UVs or
    sampling to upsample to the surface.
- Exercise 3: Add a debug border
  - Draw a second quad in the post pass that outlines the viewport to validate
    scissor and viewport behavior.
- Exercise 4: Add MSAA to the offscreen target
  - Enable multi-sampling on the offscreen target and ensure the pipeline and
    pass use the same sample count.
- Exercise 5: Add a second post pass
  - Render the first offscreen result into a second offscreen target, then
    sample the second target to the surface.
- Exercise 6: Sample with nearest filtering
  - Replace `.linear_clamp()` with nearest sampling and compare the result when
    rendering offscreen at reduced resolution.

## Changelog <a name="changelog"></a>

- 0.1.0 (2025-12-29): Initial draft aligned with
  `crates/lambda-rs/examples/offscreen_post.rs`.
