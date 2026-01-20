---
title: "Textured Cube: 3D Immediates + 2D Sampling"
document_id: "textured-cube-tutorial-2025-11-10"
status: "draft"
created: "2025-11-10T00:00:00Z"
last_updated: "2026-01-19T00:00:00Z"
version: "0.3.3"
engine_workspace_version: "2023.1.30"
wgpu_version: "28.0.0"
shader_backend_default: "naga"
winit_version: "0.29.10"
repo_commit: "d0abc736e9d7308fdae80b2d0b568c4614f5a642"
owners: ["lambda-sh"]
reviewers: ["engine", "rendering"]
tags: ["tutorial", "graphics", "3d", "immediates", "textures", "samplers", "rust", "wgpu"]
---

## Overview <a name="overview"></a>

This tutorial builds a spinning 3D cube that uses immediates to provide model‑view‑projection (MVP) and model matrices to the vertex shader, and samples a 2D checkerboard texture in the fragment shader. Depth testing and back‑face culling are enabled so hidden faces do not render.

Reference implementation: `crates/lambda-rs/examples/textured_cube.rs`.

## Table of Contents

- [Overview](#overview)
- [Goals](#goals)
- [Prerequisites](#prerequisites)
- [Requirements and Constraints](#requirements-and-constraints)
- [Data Flow](#data-flow)
- [Implementation Steps](#implementation-steps)
  - [Step 1 — Runtime and Component Skeleton](#step-1)
  - [Step 2 — Shaders with Immediates](#step-2)
  - [Step 3 — Cube Mesh and Vertex Layout](#step-3)
  - [Step 4 — Build a 2D Checkerboard Texture](#step-4)
  - [Step 5 — Create a Sampler](#step-5)
  - [Step 6 — Bind Group Layout and Bind Group](#step-6)
  - [Step 7 — Render Pipeline with Depth and Culling](#step-7)
  - [Step 8 — Per‑Frame Camera and Transforms](#step-8)
  - [Step 9 — Record Draw Commands with Immediates](#step-9)
  - [Step 10 — Handle Window Resize](#step-10)
- [Validation](#validation)
- [Notes](#notes)
- [Conclusion](#conclusion)
- [Putting It Together](#putting-it-together)
- [Exercises](#exercises)
- [Changelog](#changelog)

## Goals <a name="goals"></a>

- Render a rotating cube with correct occlusion using depth testing and back‑face culling.
- Pass model‑view‑projection (MVP) and model matrices via immediates to the vertex stage.
- Sample a 2D texture in the fragment stage using a separate sampler, and apply simple Lambert lighting to emphasize shape.

## Prerequisites <a name="prerequisites"></a>

- Workspace builds: `cargo build --workspace`.
- Run a quick example: `cargo run --example minimal`.

## Requirements and Constraints <a name="requirements-and-constraints"></a>

- Immediate data size MUST match the shader declaration. This example sends 128 bytes (two `mat4`).
- The immediate data byte order MUST match the shader's expected matrix layout. This example transposes matrices before upload to match column‑major multiplication in GLSL.
- Face winding MUST be counter‑clockwise (CCW) for back‑face culling to work with `CullingMode::Back`.
- The model matrix MUST NOT include non‑uniform scale if normals are transformed with `mat3(model)`. Rationale: non‑uniform scale skews normals; either avoid it or compute a proper normal matrix.
- Binding indices MUST match between Rust and shaders: set 0, binding 1 is the 2D texture; set 0, binding 2 is the sampler.
- Acronyms: central processing unit (CPU), graphics processing unit (GPU), model‑view‑projection (MVP).

## Data Flow <a name="data-flow"></a>

```
CPU (mesh + pixels, elapsed time)
   │ build cube, checkerboard
   ▼
TextureBuilder (2D sRGB) + SamplerBuilder (linear clamp)
   │
   ▼
BindGroup(set0): binding1=texture2D, binding2=sampler
   │                                 ▲
   ▼                                 │
Render Pipeline (vertex: immediates, fragment: sampling)
   │  MVP + model (immediates)       │
   ▼                                 │
Render Pass (depth enabled, back‑face culling) → Draw 36 vertices
```

## Implementation Steps <a name="implementation-steps"></a>

### Step 1 — Runtime and Component Skeleton <a name="step-1"></a>

Create the application runtime and a `Component` that stores shader handles, GPU resource identifiers, window size, and elapsed time for animation.

```rust
use lambda::{
  component::Component,
  runtime::start_runtime,
  runtimes::{
    application::ComponentResult,
    ApplicationRuntimeBuilder,
  },
};

pub struct TexturedCubeExample {
  shader_vs: lambda::render::shader::Shader,
  shader_fs: lambda::render::shader::Shader,
  mesh: Option<lambda::render::mesh::Mesh>,
  render_pipeline: Option<lambda::render::ResourceId>,
  render_pass: Option<lambda::render::ResourceId>,
  bind_group: Option<lambda::render::ResourceId>,
  width: u32,
  height: u32,
  elapsed: f32,
}

impl Default for TexturedCubeExample {
  fn default() -> Self {
    use lambda::render::shader::{ShaderBuilder, ShaderKind, VirtualShader};
    let mut builder = ShaderBuilder::new();
    let shader_vs = builder.build(VirtualShader::Source {
      source: VERTEX_SHADER_SOURCE.to_string(),
      kind: ShaderKind::Vertex,
      entry_point: "main".to_string(),
      name: "textured-cube".to_string(),
    });
    let shader_fs = builder.build(VirtualShader::Source {
      source: FRAGMENT_SHADER_SOURCE.to_string(),
      kind: ShaderKind::Fragment,
      entry_point: "main".to_string(),
      name: "textured-cube".to_string(),
    });

    return Self {
      shader_vs,
      shader_fs,
      mesh: None,
      render_pipeline: None,
      render_pass: None,
      bind_group: None,
      width: 800,
      height: 600,
      elapsed: 0.0,
    };
  }
}

fn main() {
  let runtime = ApplicationRuntimeBuilder::new("Textured Cube Example")
    .with_window_configured_as(|b| b.with_dimensions(800, 600).with_name("Textured Cube"))
    .with_component(|runtime, example: TexturedCubeExample| { return (runtime, example); })
    .build();

  start_runtime(runtime);
}
```

This scaffold establishes the runtime and stores component state required to create resources and animate the cube.

### Step 2 — Shaders with Immediates <a name="step-2"></a>

Define GLSL 450 shaders. The vertex shader declares an immediate data block (using `push_constant` syntax) with two `mat4` values: `mvp` and `model`. The fragment shader samples a 2D texture using a separate sampler and applies simple Lambert lighting for shape definition.

> **Note:** In wgpu v28, push constants were renamed to "immediates" and require the `Features::IMMEDIATES` feature. The GLSL syntax remains `push_constant`.

```glsl
// Vertex (GLSL 450)
#version 450

layout (location = 0) in vec3 vertex_position;
layout (location = 1) in vec3 vertex_normal;
layout (location = 2) in vec3 vertex_color; // unused

layout (location = 0) out vec3 v_model_pos;
layout (location = 1) out vec3 v_model_normal;
layout (location = 2) out vec3 v_world_normal;

layout ( push_constant ) uniform Push {
  mat4 mvp;
  mat4 model;
} pc;

void main() {
  gl_Position = pc.mvp * vec4(vertex_position, 1.0);
  v_model_pos = vertex_position;
  v_model_normal = vertex_normal;
  // Rotate normals into world space using the model matrix (no scale/shear).
  v_world_normal = mat3(pc.model) * vertex_normal;
}
```

```glsl
// Fragment (GLSL 450)
#version 450

layout (location = 0) in vec3 v_model_pos;
layout (location = 1) in vec3 v_model_normal;
layout (location = 2) in vec3 v_world_normal;

layout (location = 0) out vec4 fragment_color;

layout (set = 0, binding = 1) uniform texture2D tex;
layout (set = 0, binding = 2) uniform sampler samp;

// Project model-space position to 2D UVs based on the dominant normal axis.
vec2 project_uv(vec3 p, vec3 n) {
  vec3 a = abs(n);
  if (a.x > a.y && a.x > a.z) {
    return p.zy * 0.5 + 0.5; // +/-X faces: map Z,Y
  } else if (a.y > a.z) {
    return p.xz * 0.5 + 0.5; // +/-Y faces: map X,Z
  } else {
    return p.xy * 0.5 + 0.5; // +/-Z faces: map X,Y
  }
}

void main() {
  vec3 N_model = normalize(v_model_normal);
  vec2 uv = clamp(project_uv(v_model_pos, N_model), 0.0, 1.0);
  vec3 base = texture(sampler2D(tex, samp), uv).rgb;

  // Simple Lambert lighting to emphasize shape
  vec3 N = normalize(v_world_normal);
  vec3 L = normalize(vec3(0.4, 0.7, 1.0));
  float diff = max(dot(N, L), 0.0);
  vec3 color = base * (0.25 + 0.75 * diff);
  fragment_color = vec4(color, 1.0);
}
```

Compile these as `VirtualShader::Source` instances using `ShaderBuilder` during `on_attach` or `Default`. Keep the binding indices in the shader consistent with the Rust side.

### Step 3 — Cube Mesh and Vertex Layout <a name="step-3"></a>

Build a unit cube centered at the origin. The following snippet uses a helper to add a face as two triangles with a shared normal. Attribute layout matches the shaders: location 0 = position, 1 = normal, 2 = color (unused).

```rust
use lambda::render::{
  mesh::{Mesh, MeshBuilder},
  vertex::{ColorFormat, Vertex, VertexAttribute, VertexBuilder, VertexElement},
};

let mut verts: Vec<Vertex> = Vec::new();
let mut add_face = |nx: f32, ny: f32, nz: f32, corners: [(f32, f32, f32); 4]| {
  let n = [nx, ny, nz];
  let v = |p: (f32, f32, f32)| {
    return VertexBuilder::new()
      .with_position([p.0, p.1, p.2])
      .with_normal(n)
      .with_color([0.0, 0.0, 0.0])
      .build();
  };
  // CCW winding: (0,1,2) and (0,2,3)
  let p0 = v(corners[0]);
  let p1 = v(corners[1]);
  let p2 = v(corners[2]);
  let p3 = v(corners[3]);
  verts.extend([p0, p1, p2, p0, p2, p3]);
};

let h = 0.5f32;
// +X, -X, +Y, -Y, +Z, -Z (all CCW from the outside)
add_face( 1.0,  0.0,  0.0, [( h, -h, -h), ( h,  h, -h), ( h,  h,  h), ( h, -h,  h)]);
add_face(-1.0,  0.0,  0.0, [(-h, -h, -h), (-h, -h,  h), (-h,  h,  h), (-h,  h, -h)]);
add_face( 0.0,  1.0,  0.0, [(-h,  h,  h), ( h,  h,  h), ( h,  h, -h), (-h,  h, -h)]);
add_face( 0.0, -1.0,  0.0, [(-h, -h, -h), ( h, -h, -h), ( h, -h,  h), (-h, -h,  h)]);
add_face( 0.0,  0.0,  1.0, [(-h, -h,  h), ( h, -h,  h), ( h,  h,  h), (-h,  h,  h)]);
add_face( 0.0,  0.0, -1.0, [( h, -h, -h), (-h, -h, -h), (-h,  h, -h), ( h,  h, -h)]);

let mut mesh_builder = MeshBuilder::new();
verts.into_iter().for_each(|v| { mesh_builder.with_vertex(v); });

let mesh: Mesh = mesh_builder
  .with_attributes(vec![
    VertexAttribute { // position @ location 0
      location: 0, offset: 0,
      element: VertexElement { format: ColorFormat::Rgb32Sfloat, offset: 0 },
    },
    VertexAttribute { // normal @ location 1
      location: 1, offset: 0,
      element: VertexElement { format: ColorFormat::Rgb32Sfloat, offset: 12 },
    },
    VertexAttribute { // color (unused) @ location 2
      location: 2, offset: 0,
      element: VertexElement { format: ColorFormat::Rgb32Sfloat, offset: 24 },
    },
  ])
  .build();
```

This produces 36 vertices (6 faces × 2 triangles × 3 vertices) with CCW winding and per‑face normals.

### Step 4 — Build a 2D Checkerboard Texture <a name="step-4"></a>

Generate a simple grayscale checkerboard and upload it as an sRGB 2D texture.

```rust
use lambda::render::texture::{TextureBuilder, TextureFormat};

let tex_w = 64u32;
let tex_h = 64u32;
let mut pixels = vec![0u8; (tex_w * tex_h * 4) as usize];
for y in 0..tex_h {
  for x in 0..tex_w {
    let i = ((y * tex_w + x) * 4) as usize;
    let checker = ((x / 8) % 2) ^ ((y / 8) % 2);
    let c: u8 = if checker == 0 { 40 } else { 220 };
    pixels[i + 0] = c; // R
    pixels[i + 1] = c; // G
    pixels[i + 2] = c; // B
    pixels[i + 3] = 255; // A
  }
}

let texture2d = TextureBuilder::new_2d(TextureFormat::Rgba8UnormSrgb)
  .with_size(tex_w, tex_h)
  .with_data(&pixels)
  .with_label("checkerboard")
  .build(render_context.gpu())
  .expect("Failed to create 2D texture");
```

Using `Rgba8UnormSrgb` ensures sampling converts from sRGB to linear space before shading.

### Step 5 — Create a Sampler <a name="step-5"></a>

Create a linear filtering sampler with clamp‑to‑edge addressing.

```rust
use lambda::render::texture::SamplerBuilder;

let sampler = SamplerBuilder::new()
  .linear_clamp()
  .build(render_context.gpu());
```

### Step 6 — Bind Group Layout and Bind Group <a name="step-6"></a>

Declare the layout and bind the texture and sampler at set 0, bindings 1 and 2.

```rust
use lambda::render::bind::{BindGroupBuilder, BindGroupLayoutBuilder};

let layout = BindGroupLayoutBuilder::new()
  .with_sampled_texture(1)
  .with_sampler(2)
  .build(render_context.gpu());

let bind_group = BindGroupBuilder::new()
  .with_layout(&layout)
  .with_texture(1, &texture2d)
  .with_sampler(2, &sampler)
  .build(render_context.gpu());
```

### Step 7 — Render Pipeline with Depth and Culling <a name="step-7"></a>

Enable depth, back‑face culling, and declare a vertex buffer built from the mesh. Add an immediate data range for the shaders.

```rust
use lambda::render::{
  buffer::BufferBuilder,
  pipeline::{RenderPipelineBuilder, CullingMode},
  render_pass::RenderPassBuilder,
};

let render_pass = RenderPassBuilder::new()
  .with_label("textured-cube-pass")
  .with_depth()
  .build(
    render_context.gpu(),
    render_context.surface_format(),
    render_context.depth_format(),
  );

let immediate_data_size = std::mem::size_of::<ImmediateData>() as u32;

let pipeline = RenderPipelineBuilder::new()
  .with_culling(CullingMode::Back)
  .with_depth()
  .with_immediate_data(immediate_data_size)
  .with_buffer(
    BufferBuilder::build_from_mesh(&mesh, render_context.gpu())
      .expect("Failed to create vertex buffer"),
    mesh.attributes().to_vec(),
  )
  .with_layouts(&[&layout])
  .build(
    render_context.gpu(),
    render_context.surface_format(),
    render_context.depth_format(),
    &render_pass,
    &self.shader_vs,
    Some(&self.shader_fs),
  );

// Attach to obtain ResourceId handles
self.render_pass = Some(render_context.attach_render_pass(render_pass));
self.render_pipeline = Some(render_context.attach_pipeline(pipeline));
self.bind_group = Some(render_context.attach_bind_group(bind_group));
```

### Step 8 — Per‑Frame Camera and Transforms <a name="step-8"></a>

Compute yaw and pitch from elapsed time, build `model`, `view`, and perspective `projection`, then combine to an MVP matrix. Update `elapsed` in `on_update`.

```rust
use lambda::render::scene_math::{compute_perspective_projection, compute_view_matrix, SimpleCamera};

// on_update
self.elapsed += last_frame.as_secs_f32();

// on_render
let camera = SimpleCamera {
  position: [0.0, 0.0, 2.2],
  field_of_view_in_turns: 0.24,
  near_clipping_plane: 0.1,
  far_clipping_plane: 100.0,
};

let angle_y_turns = 0.15 * self.elapsed; // yaw
let angle_x_turns = 0.10 * self.elapsed; // pitch

let mut model = lambda::math::matrix::identity_matrix(4, 4);
model = lambda::math::matrix::rotate_matrix(model, [0.0, 1.0, 0.0], angle_y_turns)
  .expect("rotation axis must be a unit axis vector");
model = lambda::math::matrix::rotate_matrix(model, [1.0, 0.0, 0.0], angle_x_turns)
  .expect("rotation axis must be a unit axis vector");

let view = compute_view_matrix(camera.position);
let projection = compute_perspective_projection(
  camera.field_of_view_in_turns,
  self.width.max(1),
  self.height.max(1),
  camera.near_clipping_plane,
  camera.far_clipping_plane,
);
let mvp = projection.multiply(&view).multiply(&model);
```

This multiplication order produces clip‑space positions as `mvp * vec4(position, 1)`. The final upload transposes matrices to match GLSL column‑major layout.

### Step 9 — Record Draw Commands with Immediates <a name="step-9"></a>

Define an immediate data struct and a helper to reinterpret it as `[u32]`. Record commands to begin the pass, set pipeline state, bind the texture and sampler, set immediates, and draw 36 vertices.

```rust
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ImmediateData {
  mvp: [[f32; 4]; 4],
  model: [[f32; 4]; 4],
}

pub fn immediate_data_to_bytes(immediate_data: &ImmediateData) -> &[u32] {
  unsafe {
    let size_in_bytes = std::mem::size_of::<ImmediateData>();
    let size_in_u32 = size_in_bytes / std::mem::size_of::<u32>();
    let ptr = immediate_data as *const ImmediateData as *const u32;
    return std::slice::from_raw_parts(ptr, size_in_u32);
  }
}

use lambda::render::{command::RenderCommand, viewport::ViewportBuilder};

let viewport = ViewportBuilder::new().build(self.width, self.height);
let pipeline = self.render_pipeline.expect("pipeline not set");
let group = self.bind_group.expect("bind group not set");
let mesh_len = self.mesh.as_ref().unwrap().vertices().len() as u32;

let commands = vec![
  RenderCommand::BeginRenderPass {
    render_pass: self.render_pass.expect("render pass not set"),
    viewport: viewport.clone(),
  },
  RenderCommand::SetPipeline { pipeline },
  RenderCommand::SetViewports { start_at: 0, viewports: vec![viewport.clone()] },
  RenderCommand::SetScissors { start_at: 0, viewports: vec![viewport.clone()] },
  RenderCommand::SetBindGroup { set: 0, group, dynamic_offsets: vec![] },
  RenderCommand::BindVertexBuffer { pipeline, buffer: 0 },
  RenderCommand::Immediates {
    pipeline,
    offset: 0,
    bytes: Vec::from(immediate_data_to_bytes(&ImmediateData {
      mvp: mvp.transpose(),
      model: model.transpose(),
    })),
  },
  RenderCommand::Draw { vertices: 0..mesh_len, instances: 0..1 },
  RenderCommand::EndRenderPass,
];
```

### Step 10 — Handle Window Resize <a name="step-10"></a>

Track window size from events so the projection and viewport use current dimensions.

```rust
use lambda::events::{EventMask, WindowEvent};

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

## Validation <a name="validation"></a>

- Build the workspace: `cargo build --workspace`
- Run the example: `cargo run -p lambda-rs --example textured_cube`
- Expected behavior: a spinning cube shows a gray checkerboard on all faces, shaded by a directional light. Hidden faces do not render due to back‑face culling and depth testing.

## Notes <a name="notes"></a>

- Immediate data limits: total size MUST be within the device's immediate data limit. This example uses 128 bytes, which fits common defaults. wgpu v28 requires `Features::IMMEDIATES` to be enabled.
- Matrix layout: GLSL multiplies column‑major by default; transposing on upload aligns memory layout and multiplication order.
- Normal transform: `mat3(model)` is correct when the model matrix contains only rotations and uniform scale. For non‑uniform scale, compute the normal matrix as the inverse‑transpose of the upper‑left 3×3.
- Texture color space: use `Rgba8UnormSrgb` for color images so sampling returns linear values.
- Winding and culling: keep face winding CCW to work with `CullingMode::Back`. Toggle to `CullingMode::None` when debugging geometry.
- Indices: the cube uses non‑indexed vertices for clarity. An index buffer SHOULD be used for efficiency in production code.

## Conclusion <a name="conclusion"></a>

This tutorial delivered a rotating, textured cube with depth testing and
back‑face culling. It compiled shaders that use a vertex immediate data block
for model‑view‑projection and model matrices, built a cube mesh and vertex
layout, created an sRGB texture and sampler, and constructed a pipeline with
depth and culling. Per‑frame transforms were computed and uploaded via immediates,
and draw commands were recorded. The result demonstrates immediates for per‑draw
transforms alongside 2D sampling in a 3D render path.

## Putting It Together <a name="putting-it-together"></a>

- Full reference: `crates/lambda-rs/examples/textured_cube.rs`
- The example includes logging in `on_attach` and uses the same builders and commands shown here.

## Exercises <a name="exercises"></a>

- Exercise 1: Add roll
  - Add a Z‑axis rotation to the model matrix and verify culling remains correct.
- Exercise 2: Nearest filtering
  - Replace `linear_clamp()` with nearest filtering and observe pixelated edges.
- Exercise 3: Image texture
  - Load a PNG or JPEG into RGBA bytes and upload with `TextureBuilder::new_2d`.
- Exercise 4: Normal matrix
  - Add non‑uniform scale and implement a proper normal matrix for correct lighting.
- Exercise 5: Index buffer
  - Replace the non‑indexed mesh with an indexed mesh and draw with an index buffer.
- Exercise 6: Phong or Blinn‑Phong
  - Extend the fragment shader with specular highlights for shinier faces.
- Exercise 7: Multiple materials
  - Bind two textures and blend per face based on projected UVs.

## Changelog <a name="changelog"></a>

- 0.3.2 (2026-01-16): Replace `on_event` resize handling with `event_mask()` and `on_window_event`.
- 0.3.1 (2026-01-07): Remove stage usage from immediates API examples.
- 0.3.0 (2026-01-05): Migrate from push constants to immediates for wgpu v28; update all code examples and terminology.
- 0.2.0 (2025-12-15): Update builder API calls to use `render_context.gpu()` and add `surface_format`/`depth_format` parameters to `RenderPassBuilder` and `RenderPipelineBuilder`.
- 0.1.1 (2025-11-10): Add Conclusion section summarizing outcomes; update metadata and commit.
- 0.1.0 (2025-11-10): Initial draft aligned with `crates/lambda-rs/examples/textured_cube.rs` including immediates, depth, culling, and projected UV sampling.
