---
title: "Reflective Floor: Stencil‑Masked Planar Reflections"
document_id: "reflective-room-tutorial-2025-11-17"
status: "draft"
created: "2025-11-17T00:00:00Z"
last_updated: "2025-11-19T00:00:01Z"
version: "0.2.1"
engine_workspace_version: "2023.1.30"
wgpu_version: "26.0.1"
shader_backend_default: "naga"
winit_version: "0.29.10"
repo_commit: "bf0e90ae9ce653e1da2e1e594b22038094bada07"
owners: ["lambda-sh"]
reviewers: ["engine", "rendering"]
tags: ["tutorial", "graphics", "stencil", "depth", "msaa", "mirror", "3d", "push-constants", "wgpu", "rust"]
---

## Overview <a name="overview"></a>
This tutorial builds a reflective floor using the stencil buffer with an optional depth test and 4× multi‑sample anti‑aliasing (MSAA). The scene renders in four phases: a floor mask into stencil, a mirrored cube clipped by the mask, a translucent lit floor surface, and a normal cube above the plane. The camera looks down at a moderate angle so the reflection is clearly visible.

Reference implementation: `crates/lambda-rs/examples/reflective_room.rs`.

## Table of Contents
- [Overview](#overview)
- [Goals](#goals)
- [Prerequisites](#prerequisites)
- [Requirements and Constraints](#requirements-and-constraints)
- [Data Flow](#data-flow)
- [Implementation Steps](#implementation-steps)
  - [Step 1 — Runtime and Component Skeleton](#step-1)
  - [Step 2 — Shaders and Push Constants](#step-2)
  - [Step 3 — Meshes: Cube and Floor](#step-3)
  - [Step 4 — Render Passes: Mask and Color](#step-4)
  - [Step 5 — Pipeline: Floor Mask (Stencil Write)](#step-5)
  - [Step 6 — Pipeline: Reflected Cube (Stencil Test)](#step-6)
  - [Step 7 — Pipeline: Floor Visual (Tinted)](#step-7)
  - [Step 8 — Pipeline: Normal Cube](#step-8)
  - [Step 9 — Per‑Frame Transforms and Reflection](#step-9)
  - [Step 10 — Record Commands and Draw Order](#step-10)
  - [Step 11 — Input, MSAA/Depth/Stencil Toggles, and Resize](#step-11)
- [Validation](#validation)
- [Notes](#notes)
- [Conclusion](#conclusion)
- [Putting It Together](#putting-it-together)
- [Exercises](#exercises)
- [Changelog](#changelog)

## Goals <a name="goals"></a>

- Use the stencil buffer to restrict rendering to the floor area and show a mirrored reflection of a cube.
- Support depth testing and 4× MSAA to improve geometric correctness and edge quality.
- Drive transforms via push constants for model‑view‑projection (MVP) and model matrices.
- Provide runtime toggles for MSAA, stencil, and depth testing, plus camera pitch and visibility helpers.

## Prerequisites <a name="prerequisites"></a>
- Build the workspace: `cargo build --workspace`.
- Run an example to verify setup: `cargo run --example minimal`.

## Requirements and Constraints <a name="requirements-and-constraints"></a>
- A pipeline that uses stencil state MUST render into a pass with a depth‑stencil attachment. Use `DepthFormat::Depth24PlusStencil8`.
- The mask pass MUST disable depth writes and write stencil with `Replace` so the floor area becomes `1`.
- The reflected cube pipeline MUST test stencil `Equal` against reference `1` and SHOULD set stencil write mask to `0x00`.
- Mirroring across the floor plane flips face winding. Culling MUST be disabled for the reflected draw or the front‑face definition MUST be adjusted. This example culls front faces for the reflected cube.
- Push constant size and stage visibility MUST match the shader declaration. Two `mat4` values are sent to the vertex stage only (128 bytes total).
- Matrix order MUST match the shader’s expectation. The example transposes matrices before upload to match GLSL column‑major multiplication.
- The render pass and pipelines MUST use the same sample count when MSAA is enabled.
- Acronyms: graphics processing unit (GPU), central processing unit (CPU), multi‑sample anti‑aliasing (MSAA), model‑view‑projection (MVP).

## Data Flow <a name="data-flow"></a>

```
CPU (meshes, elapsed time, toggles)
  │
  ├─ Build/attach render passes (mask, color) with MSAA
  ├─ Build pipelines (mask → reflected → floor → normal)
  ▼
Pass 1: Depth/Stencil‑only (no color) — write stencil where floor covers
  │  stencil = 1 inside floor, 0 elsewhere; depth write off
  ▼
Pass 2: Color (with depth/stencil) — draw reflected cube with stencil test == 1
  │  culling front faces; depth compare configurable
  ▼
Pass 3: Color — draw tinted floor (alpha) to show reflection
  ▼
Pass 4: Color — draw normal cube above the floor
```

## Implementation Steps <a name="implementation-steps"></a>

### Step 1 — Runtime and Component Skeleton <a name="step-1"></a>
Define a `Component` that owns shaders, meshes, render passes, pipelines, window size, elapsed time, and user‑toggleable settings for MSAA, stencil, and depth testing.

```rust
use lambda::{
  component::Component,
  runtimes::{application::ComponentResult, ApplicationRuntimeBuilder},
};

pub struct ReflectiveRoomExample {
  shader_vs: lambda::render::shader::Shader,
  shader_fs_lit: lambda::render::shader::Shader,
  shader_fs_floor: lambda::render::shader::Shader,
  cube_mesh: Option<lambda::render::mesh::Mesh>,
  floor_mesh: Option<lambda::render::mesh::Mesh>,
  pass_id_mask: Option<lambda::render::ResourceId>,
  pass_id_color: Option<lambda::render::ResourceId>,
  pipe_floor_mask: Option<lambda::render::ResourceId>,
  pipe_reflected: Option<lambda::render::ResourceId>,
  pipe_floor_visual: Option<lambda::render::ResourceId>,
  pipe_normal: Option<lambda::render::ResourceId>,
  width: u32,
  height: u32,
  elapsed: f32,
  msaa_samples: u32,
  stencil_enabled: bool,
  depth_test_enabled: bool,
  needs_rebuild: bool,
}

impl Default for ReflectiveRoomExample { /* create shaders; set defaults */ }
impl Component<ComponentResult, String> for ReflectiveRoomExample { /* lifecycle */ }
```

Narrative: The component stores GPU handles and toggles. When settings change, mark `needs_rebuild = true` and rebuild pipelines/passes on the next frame.

### Step 2 — Shaders and Push Constants <a name="step-2"></a>
Use one vertex shader and two fragment shaders. The vertex shader expects push constants with two `mat4` values: the MVP and the model matrix, used to transform positions and rotate normals to world space. The floor fragment shader is lit and translucent so the reflection reads beneath it.

```glsl
// Vertex (GLSL 450)
layout (location = 0) in vec3 vertex_position;
layout (location = 1) in vec3 vertex_normal;
layout (location = 0) out vec3 v_world_normal;

layout ( push_constant ) uniform Push { mat4 mvp; mat4 model; } pc;

void main() {
  gl_Position = pc.mvp * vec4(vertex_position, 1.0);
  // Transform normals by the model matrix; sufficient for rigid + mirror.
  v_world_normal = mat3(pc.model) * vertex_normal;
}
```

```glsl
// Fragment (lit)
layout (location = 0) in vec3 v_world_normal;
layout (location = 0) out vec4 fragment_color;
void main() {
  vec3 N = normalize(v_world_normal);
  vec3 L = normalize(vec3(0.4, 0.7, 1.0));
  float diff = max(dot(N, L), 0.0);
  vec3 base = vec3(0.2, 0.6, 0.9);
  fragment_color = vec4(base * (0.25 + 0.75 * diff), 1.0);
}
```

```glsl
// Fragment (floor: lit + translucent)
layout (location = 0) in vec3 v_world_normal;
layout (location = 0) out vec4 fragment_color;
void main() {
  vec3 N = normalize(v_world_normal);
  vec3 L = normalize(vec3(0.4, 0.7, 1.0));
  float diff = max(dot(N, L), 0.0);
  vec3 base = vec3(0.10, 0.10, 0.11);
  vec3 color = base * (0.35 + 0.65 * diff);
  fragment_color = vec4(color, 0.15);
}
```

In Rust, pack push constants as 32‑bit words and transpose matrices before upload.

```rust
#[repr(C)]
pub struct PushConstant { pub mvp: [[f32; 4]; 4], pub model: [[f32; 4]; 4] }

pub fn push_constants_to_words(pc: &PushConstant) -> &[u32] {
  unsafe {
    let size = std::mem::size_of::<PushConstant>() / std::mem::size_of::<u32>();
    let ptr = pc as *const PushConstant as *const u32;
    return std::slice::from_raw_parts(ptr, size);
  }
}
```

### Step 3 — Meshes: Cube and Floor <a name="step-3"></a>
Build a unit cube (36 vertices) with per‑face normals and a large XZ floor quad at `y = 0`. Provide matching vertex attributes for position and normal at locations 0 and 1.

Reference: `crates/lambda-rs/examples/reflective_room.rs:740` and `crates/lambda-rs/examples/reflective_room.rs:807`.

### Step 4 — Render Passes: Mask and Color <a name="step-4"></a>
Create a depth/stencil‑only pass for the floor mask and a color pass for the scene. Use the same sample count on both.

```rust
use lambda::render::render_pass::RenderPassBuilder;

let pass_mask = RenderPassBuilder::new()
  .with_label("reflective-room-pass-mask")
  .with_depth_clear(1.0)
  .with_stencil_clear(0)
  .with_multi_sample(msaa_samples)
  .without_color() // no color target
  .build(ctx);

let pass_color = RenderPassBuilder::new()
  .with_label("reflective-room-pass-color")
  .with_multi_sample(msaa_samples)
  .with_depth_clear(1.0) // or .with_depth_load() when depth test is off
  .with_stencil_load()   // preserve mask from pass 1
  .build(ctx);
```

Rationale: pipelines that use stencil require a depth‑stencil attachment, even if depth testing is disabled.

### Step 5 — Pipeline: Floor Mask (Stencil Write) <a name="step-5"></a>
Draw the floor geometry to write `stencil = 1` where the floor covers. Do not write to color. Disable depth writes and set depth compare to `Always`.

```rust
use lambda::render::pipeline::{RenderPipelineBuilder, CompareFunction, StencilState, StencilFaceState, StencilOperation, PipelineStage};

let pipe_floor_mask = RenderPipelineBuilder::new()
  .with_label("floor-mask")
  .with_depth_format(lambda::render::texture::DepthFormat::Depth24PlusStencil8)
  .with_depth_write(false)
  .with_depth_compare(CompareFunction::Always)
  .with_push_constant(PipelineStage::VERTEX, std::mem::size_of::<PushConstant>() as u32)
  .with_buffer(floor_vertex_buffer, floor_attributes)
  .with_stencil(StencilState {
    front: StencilFaceState { compare: CompareFunction::Always, fail_op: StencilOperation::Keep, depth_fail_op: StencilOperation::Keep, pass_op: StencilOperation::Replace },
    back:  StencilFaceState { compare: CompareFunction::Always, fail_op: StencilOperation::Keep, depth_fail_op: StencilOperation::Keep, pass_op: StencilOperation::Replace },
    read_mask: 0xFF, write_mask: 0xFF,
  })
  .with_multi_sample(msaa_samples)
  .build(ctx, &pass_mask, &shader_vs, None);
```

### Step 6 — Pipeline: Reflected Cube (Stencil Test) <a name="step-6"></a>
Render the mirrored cube only where the floor mask is present. Mirroring flips the winding, so cull front faces for the reflected draw. Use `depth_compare = Always` and disable depth writes so the reflection remains visible; the stencil confines it to the floor.

```rust
let mut builder = RenderPipelineBuilder::new()
  .with_label("reflected-cube")
  .with_culling(lambda::render::pipeline::CullingMode::Front)
  .with_depth_format(lambda::render::texture::DepthFormat::Depth24PlusStencil8)
  .with_push_constant(PipelineStage::VERTEX, std::mem::size_of::<PushConstant>() as u32)
  .with_buffer(cube_vertex_buffer, cube_attributes)
  .with_stencil(StencilState {
    front: StencilFaceState { compare: CompareFunction::Equal, fail_op: StencilOperation::Keep, depth_fail_op: StencilOperation::Keep, pass_op: StencilOperation::Keep },
    back:  StencilFaceState { compare: CompareFunction::Equal, fail_op: StencilOperation::Keep, depth_fail_op: StencilOperation::Keep, pass_op: StencilOperation::Keep },
    read_mask: 0xFF, write_mask: 0x00,
  })
  .with_multi_sample(msaa_samples)
  .with_depth_write(false)
  .with_depth_compare(CompareFunction::Always);

let pipe_reflected = builder.build(ctx, &pass_color, &shader_vs, Some(&shader_fs_lit));
```

### Step 7 — Pipeline: Floor Visual (Tinted) <a name="step-7"></a>
Draw the floor surface with a translucent tint so the reflection remains visible beneath.

```rust
let mut floor_vis = RenderPipelineBuilder::new()
  .with_label("floor-visual")
  .with_push_constant(PipelineStage::VERTEX, std::mem::size_of::<PushConstant>() as u32)
  .with_buffer(floor_vertex_buffer, floor_attributes)
  .with_multi_sample(msaa_samples);

if depth_test_enabled || stencil_enabled {
  floor_vis = floor_vis
    .with_depth_format(lambda::render::texture::DepthFormat::Depth24PlusStencil8)
    .with_depth_write(false)
    .with_depth_compare(if depth_test_enabled { CompareFunction::LessEqual } else { CompareFunction::Always });
}

let pipe_floor_visual = floor_vis.build(ctx, &pass_color, &shader_vs, Some(&shader_fs_floor));
```

### Step 8 — Pipeline: Normal Cube <a name="step-8"></a>
Draw the unreflected cube above the floor using the lit fragment shader. Enable back‑face culling and depth testing when requested.

```rust
let mut normal = RenderPipelineBuilder::new()
  .with_label("cube-normal")
  .with_push_constant(PipelineStage::VERTEX, std::mem::size_of::<PushConstant>() as u32)
  .with_buffer(cube_vertex_buffer, cube_attributes)
  .with_multi_sample(msaa_samples);

if depth_test_enabled || stencil_enabled {
  normal = normal
    .with_depth_format(lambda::render::texture::DepthFormat::Depth24PlusStencil8)
    .with_depth_write(depth_test_enabled)
    .with_depth_compare(if depth_test_enabled { CompareFunction::Less } else { CompareFunction::Always });
}

let pipe_normal = normal.build(ctx, &pass_color, &shader_vs, Some(&shader_fs_lit));
```

### Step 9 — Per‑Frame Transforms and Reflection <a name="step-9"></a>
Compute camera, model rotation, and the mirror transform across the floor plane. The camera pitches downward and translates to a higher vantage point. Build the mirror using the plane‑reflection matrix `R = I − 2 n n^T` for a plane through the origin with unit normal `n` (for a flat floor, `n = (0,1,0)`).

```rust
use lambda::render::scene_math::{compute_perspective_projection, compute_view_matrix, SimpleCamera};

let camera = SimpleCamera { position: [0.0, 3.0, 4.0], field_of_view_in_turns: 0.24, near_clipping_plane: 0.1, far_clipping_plane: 100.0 };
// View = R_x(-pitch) * T(-position)
let pitch_turns = 0.10; // ~36 degrees downward
let rot_x = lambda::math::matrix::rotate_matrix(lambda::math::matrix::identity_matrix(4,4), [1.0,0.0,0.0], -pitch_turns);
let view = rot_x.multiply(&compute_view_matrix(camera.position));
let projection = compute_perspective_projection(camera.field_of_view_in_turns, width.max(1), height.max(1), camera.near_clipping_plane, camera.far_clipping_plane);

let angle_y = 0.12 * elapsed;
let mut model = lambda::math::matrix::identity_matrix(4, 4);
model = lambda::math::matrix::rotate_matrix(model, [0.0, 1.0, 0.0], angle_y);
model = model.multiply(&lambda::math::matrix::translation_matrix([0.0, 0.5, 0.0]));
let mvp = projection.multiply(&view).multiply(&model);

let n = [0.0f32, 1.0, 0.0];
let (nx, ny, nz) = (n[0], n[1], n[2]);
let mirror = [
  [1.0 - 2.0*nx*nx, -2.0*nx*ny,   -2.0*nx*nz,   0.0],
  [-2.0*ny*nx,       1.0 - 2.0*ny*ny, -2.0*ny*nz, 0.0],
  [-2.0*nz*nx,       -2.0*nz*ny, 1.0 - 2.0*nz*nz, 0.0],
  [0.0, 0.0, 0.0, 1.0],
];
let model_reflect = mirror.multiply(&model);
let mvp_reflect = projection.multiply(&view).multiply(&model_reflect);
```

### Step 10 — Record Commands and Draw Order <a name="step-10"></a>
Record commands in the following order. Set `viewport` and `scissor` to the window dimensions.

```rust
use lambda::render::command::RenderCommand;
use lambda::render::pipeline::PipelineStage;

let mut cmds: Vec<RenderCommand> = Vec::new();

// Pass 1: floor stencil mask
cmds.push(RenderCommand::BeginRenderPass { render_pass: pass_id_mask, viewport });
cmds.push(RenderCommand::SetPipeline { pipeline: pipe_floor_mask });
cmds.push(RenderCommand::SetStencilReference { reference: 1 });
cmds.push(RenderCommand::BindVertexBuffer { pipeline: pipe_floor_mask, buffer: 0 });
cmds.push(RenderCommand::PushConstants { pipeline: pipe_floor_mask, stage: PipelineStage::VERTEX, offset: 0, bytes: Vec::from(push_constants_to_words(&PushConstant { mvp: mvp_floor.transpose(), model: model_floor.transpose() })) });
cmds.push(RenderCommand::Draw { vertices: 0..floor_vertex_count });
cmds.push(RenderCommand::EndRenderPass);

// Pass 2: reflected cube (stencil test == 1)
cmds.push(RenderCommand::BeginRenderPass { render_pass: pass_id_color, viewport });
cmds.push(RenderCommand::SetPipeline { pipeline: pipe_reflected });
cmds.push(RenderCommand::SetStencilReference { reference: 1 });
cmds.push(RenderCommand::BindVertexBuffer { pipeline: pipe_reflected, buffer: 0 });
cmds.push(RenderCommand::PushConstants { pipeline: pipe_reflected, stage: PipelineStage::VERTEX, offset: 0, bytes: Vec::from(push_constants_to_words(&PushConstant { mvp: mvp_reflect.transpose(), model: model_reflect.transpose() })) });
cmds.push(RenderCommand::Draw { vertices: 0..cube_vertex_count });

// Pass 3: floor visual (tinted)
cmds.push(RenderCommand::SetPipeline { pipeline: pipe_floor_visual });
cmds.push(RenderCommand::BindVertexBuffer { pipeline: pipe_floor_visual, buffer: 0 });
cmds.push(RenderCommand::PushConstants { pipeline: pipe_floor_visual, stage: PipelineStage::VERTEX, offset: 0, bytes: Vec::from(push_constants_to_words(&PushConstant { mvp: mvp_floor.transpose(), model: model_floor.transpose() })) });
cmds.push(RenderCommand::Draw { vertices: 0..floor_vertex_count });

// Pass 4: normal cube
cmds.push(RenderCommand::SetPipeline { pipeline: pipe_normal });
cmds.push(RenderCommand::BindVertexBuffer { pipeline: pipe_normal, buffer: 0 });
cmds.push(RenderCommand::PushConstants { pipeline: pipe_normal, stage: PipelineStage::VERTEX, offset: 0, bytes: Vec::from(push_constants_to_words(&PushConstant { mvp: mvp.transpose(), model: model.transpose() })) });
cmds.push(RenderCommand::Draw { vertices: 0..cube_vertex_count });
cmds.push(RenderCommand::EndRenderPass);
```

### Step 11 — Input, MSAA/Depth/Stencil Toggles, and Resize <a name="step-11"></a>
Support runtime toggles to observe the impact of each setting:

- `M` toggles MSAA between `1×` and `4×`. Rebuild passes and pipelines when it changes.
- `S` toggles the stencil reflection. When disabled, the example skips the mask and reflected draw.
- `D` toggles depth testing. When disabled, set depth compare to `Always` and disable depth writes on pipelines.
- `F` toggles the floor overlay (mirror mode). When enabled, the reflection shows without the translucent floor surface.
- `I` and `K` adjust the camera pitch up/down in small steps.
- On window resize, update stored `width` and `height` and use them when computing the viewport and projection matrix.

Reference: `crates/lambda-rs/examples/reflective_room.rs:164`.

## Validation <a name="validation"></a>

- Build and run: `cargo run --example reflective_room`.
- Expected behavior:
  - A cube rotates above a reflective floor. The reflection appears only inside the floor area and shows correct mirroring.
  - Press `S` to toggle the reflection (stencil). The reflected cube disappears when stencil is off.
  - Press `F` to hide/show the floor overlay to see a clean mirror.
  - Press `I`/`K` to adjust camera pitch; ensure the reflection remains visible at moderate angles.
  - Press `D` to toggle depth testing. With depth off, the reflection still clips to the floor via stencil.
  - Press `M` to toggle MSAA. With `4×` MSAA, edges appear smoother.

## Notes <a name="notes"></a>

- Pipelines that use stencil MUST target a pass with a depth‑stencil attachment; otherwise, pipeline creation or draws will fail.
- Mirroring across a plane flips winding. Either disable culling or adjust front‑face winding for the reflected draw; do not leave back‑face culling enabled with mirrored geometry.
- This implementation culls front faces for the reflected pipeline to account for mirrored winding; the normal cube uses back‑face culling.
- The mask pass SHOULD clear stencil to `0` and write `1` where the floor renders. Use `Replace` and a write mask of `0xFF`.
- The reflected draw SHOULD use `read_mask = 0xFF`, `write_mask = 0x00`, and `reference = 1` to preserve the mask.
- When depth testing is disabled, set `depth_compare = Always` and `depth_write = false` to avoid unintended depth interactions.
- The pass and all pipelines in the pass MUST use the same MSAA sample count.
- Transpose matrices before uploading when GLSL expects column‑major multiplication.
- Metal (MSL) portability: avoid calling `inverse()` in shaders for normal transforms; compute the normal matrix on the CPU if needed. The example uses `mat3(model)` for rigid + mirror transforms.

## Conclusion <a name="conclusion"></a>

The reflective floor combines a simple stencil mask with an optional depth test and MSAA to produce a convincing planar reflection. The draw order and precise stencil state are critical: write the mask first, draw the mirrored geometry with a strict stencil test, render a translucent floor, and then render normal scene geometry.

## Putting It Together <a name="putting-it-together"></a>

- Full reference: `crates/lambda-rs/examples/reflective_room.rs`.

## Exercises <a name="exercises"></a>

- Replace the cube with a sphere and observe differences in mirrored normals.
- Move the floor plane to `y = k` and update the mirror transform accordingly.
- Add a blend mode change on the floor to experiment with different reflection intensities.
- Switch stencil reference values to render multiple reflective regions on the floor.
- Re‑enable back‑face culling for the reflected draw and adjust front‑face winding to match the mirrored transform.
- Add a checkerboard texture to the floor and render the reflection beneath it using the same mask.
- Extend the example to toggle a mirrored XZ room (two planes) using different reference values.

## Changelog <a name="changelog"></a>

- 0.2.0 (2025‑11‑19): Updated for camera pitch, front‑face culling on reflection, lit translucent floor, unmasked reflection debug toggle, floor overlay toggle, and Metal portability note.
- 0.1.0 (2025‑11‑17): Initial draft aligned with `crates/lambda-rs/examples/reflective_room.rs`, including stencil mask pass, reflected pipeline, and MSAA/depth toggles.
