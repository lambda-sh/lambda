---
title: "Textured Quad: Sample a 2D Texture"
document_id: "textured-quad-tutorial-2025-11-01"
status: "draft"
created: "2025-11-01T00:00:00Z"
last_updated: "2026-02-05T23:05:40Z"
version: "0.4.2"
engine_workspace_version: "2023.1.30"
wgpu_version: "26.0.1"
shader_backend_default: "naga"
winit_version: "0.29.10"
repo_commit: "544444652b4dc3639f8b3e297e56c302183a7a0b"
owners: ["lambda-sh"]
reviewers: ["engine", "rendering"]
tags: ["tutorial", "graphics", "textures", "samplers", "rust", "wgpu"]
---

## Overview <a name="overview"></a>

This tutorial builds a textured quad using a sampled 2D texture and sampler. It covers creating pixel data on the central processing unit (CPU), uploading to a graphics processing unit (GPU) texture, defining a sampler, wiring a bind group layout, and sampling the texture in the fragment shader.

Reference implementation: `demos/render/src/bin/textured_quad.rs`.

## Table of Contents

- [Overview](#overview)
- [Goals](#goals)
- [Prerequisites](#prerequisites)
- [Requirements and Constraints](#requirements-and-constraints)
- [Data Flow](#data-flow)
- [Implementation Steps](#implementation-steps)
  - [Step 1 — Runtime and Component Skeleton](#step-1)
  - [Step 2 — Vertex and Fragment Shaders](#step-2)
  - [Step 3 — Mesh Data and Vertex Layout](#step-3)
  - [Step 4 — Build a 2D Texture (Checkerboard)](#step-4)
  - [Step 5 — Create a Sampler](#step-5)
  - [Step 6 — Bind Group Layout and Bind Group](#step-6)
  - [Step 7 — Create the Render Pipeline](#step-7)
  - [Step 8 — Record Draw Commands](#step-8)
  - [Step 9 — Handle Window Resize](#step-9)
- [Validation](#validation)
- [Notes](#notes)
- [Conclusion](#conclusion)
- [Putting It Together](#putting-it-together)
- [Exercises](#exercises)
- [Changelog](#changelog)

## Goals <a name="goals"></a>

- Render a screen‑space quad textured with a procedurally generated checkerboard.
- Define GLSL shader stages that pass texture coordinates (UV) and sample a 2D texture with a sampler.
- Create a `Texture` and `Sampler`, bind them in a layout, and draw using Lambda’s builders.

## Prerequisites <a name="prerequisites"></a>

- Workspace builds: `cargo build --workspace`.
- Run the minimal demo to verify setup: `cargo run -p lambda-demos-minimal --bin minimal`.

## Requirements and Constraints <a name="requirements-and-constraints"></a>

- Binding indices MUST match between Rust and shaders: set 0, binding 1 is the 2D texture; set 0, binding 2 is the sampler.
- The example uses `TextureFormat::Rgba8UnormSrgb` so sampling converts from sRGB to linear space before shading. Rationale: produces correct color and filtering behavior for color images.
- The CPU pixel buffer length MUST equal `width * height * 4` bytes for `Rgba8*` formats.
- The vertex attribute at location 2 carries the UV in `.xy` (the example reuses the color field to pack UV for simplicity).

## Data Flow <a name="data-flow"></a>

```
CPU pixels -> TextureBuilder (2D, sRGB)
          -> GPU Texture + default view
SamplerBuilder -> GPU Sampler (linear, clamp)

BindGroupLayout (set 0): binding 1 = texture2D, binding 2 = sampler
BindGroup (set 0): attach texture + sampler

Render pass -> SetPipeline -> SetBindGroup -> Draw (fragment samples)
```

## Implementation Steps <a name="implementation-steps"></a>

### Step 1 — Runtime and Component Skeleton <a name="step-1"></a>

Create the application runtime and a `Component` that receives lifecycle
callbacks and a render context for resource creation and command submission.

```rust
use lambda::{
  component::Component,
  runtime::start_runtime,
  runtimes::{
    application::ComponentResult,
    ApplicationRuntimeBuilder,
  },
};

// Placement guidance
// - Resource creation (shaders, mesh, textures, pipeline): on_attach
// - Window resize handling: on_window_event
// - Rendering (record commands): on_render

pub struct TexturedQuadExample {
  // Shaders (created in Default and set again on attach)
  shader_vs: lambda::render::shader::Shader,
  shader_fs: lambda::render::shader::Shader,
  // GPU resources (attached later)
  mesh: Option<lambda::render::mesh::Mesh>,
  render_pipeline: Option<lambda::render::ResourceId>,
  render_pass: Option<lambda::render::ResourceId>,
  bind_group: Option<lambda::render::ResourceId>,
  // window state
  width: u32,
  height: u32,
}

impl Default for TexturedQuadExample {
  fn default() -> Self {
    use lambda::render::shader::{ShaderBuilder, ShaderKind, VirtualShader};
    let mut builder = ShaderBuilder::new();
    let shader_vs = builder.build(VirtualShader::Source {
      source: VERTEX_SHADER_SOURCE.to_string(),
      kind: ShaderKind::Vertex,
      entry_point: "main".to_string(),
      name: "textured-quad".to_string(),
    });
    let shader_fs = builder.build(VirtualShader::Source {
      source: FRAGMENT_SHADER_SOURCE.to_string(),
      kind: ShaderKind::Fragment,
      entry_point: "main".to_string(),
      name: "textured-quad".to_string(),
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
    };
  }
}

// Minimal component scaffold to place later steps
impl Component<ComponentResult, String> for TexturedQuadExample {
  fn on_attach(
    &mut self,
    _render_context: &mut lambda::render::RenderContext,
  ) -> Result<ComponentResult, String> {
    return Ok(ComponentResult::Success);
  }

  fn on_detach(
    &mut self,
    _render_context: &mut lambda::render::RenderContext,
  ) -> Result<ComponentResult, String> {
    return Ok(ComponentResult::Success);
  }

  fn on_update(
    &mut self,
    _last_frame: &std::time::Duration,
  ) -> Result<ComponentResult, String> {
    return Ok(ComponentResult::Success);
  }

  fn on_render(
    &mut self,
    _render_context: &mut lambda::render::RenderContext,
  ) -> Vec<lambda::render::command::RenderCommand> {
    return vec![];
  }
}

fn main() {
  let runtime = ApplicationRuntimeBuilder::new("Textured Quad Example")
    .with_window_configured_as(|builder| builder.with_dimensions(800, 600).with_name("Textured Quad"))
    .with_component(|runtime, example: TexturedQuadExample| { return (runtime, example); })
    .build();

  start_runtime(runtime);
}
```

This scaffold establishes the runtime entry point and a component that participates in the engine lifecycle. The struct stores shader handles and placeholders for GPU resources that will be created during attachment. The `Default` implementation compiles inline GLSL into `Shader` objects up front so pipeline creation can proceed deterministically. At this stage the window is created and ready; no rendering occurs yet.

### Step 2 — Vertex and Fragment Shaders <a name="step-2"></a>

Define GLSL 450 shaders. The vertex shader forwards UV to the fragment shader; the fragment samples `sampler2D(tex, samp)`.

Place these constants near the top of `textured_quad.rs`:

```rust
const VERTEX_SHADER_SOURCE: &str = r#"
#version 450

layout (location = 0) in vec3 vertex_position;
layout (location = 2) in vec3 vertex_color; // uv packed into .xy

layout (location = 0) out vec2 v_uv;

void main() {
  gl_Position = vec4(vertex_position, 1.0);
  v_uv = vertex_color.xy;
}
"#;

const FRAGMENT_SHADER_SOURCE: &str = r#"
#version 450

layout (location = 0) in vec2 v_uv;
layout (location = 0) out vec4 fragment_color;

layout (set = 0, binding = 1) uniform texture2D tex;
layout (set = 0, binding = 2) uniform sampler samp;

void main() {
  fragment_color = texture(sampler2D(tex, samp), v_uv);
}
"#;
```

These constants define the GPU programs. The vertex stage forwards texture coordinates by packing UV into the color attribute’s `.xy` at location 2; the fragment stage samples a 2D texture using a separate sampler bound at set 0, bindings 1 and 2. Keeping the sources inline makes binding indices explicit and co‑located with the Rust layout defined later.

Placement: on_attach (store in `self`).

```rust
use lambda::render::shader::{ShaderBuilder, ShaderKind, VirtualShader};

let mut shader_builder = ShaderBuilder::new();
let shader_vs = shader_builder.build(VirtualShader::Source {
  source: VERTEX_SHADER_SOURCE.to_string(),
  kind: ShaderKind::Vertex,
  entry_point: "main".to_string(),
  name: "textured-quad".to_string(),
});
let shader_fs = shader_builder.build(VirtualShader::Source {
  source: FRAGMENT_SHADER_SOURCE.to_string(),
  kind: ShaderKind::Fragment,
  entry_point: "main".to_string(),
  name: "textured-quad".to_string(),
});
// Keep local variables for pipeline creation and persist on the component.
self.shader_vs = shader_vs;
self.shader_fs = shader_fs;
```

This compiles the virtual shaders to SPIR‑V using the engine’s shader builder and stores the resulting `Shader` objects on the component. The shaders are now ready for pipeline creation; drawing will begin only after a pipeline and render pass are created and attached.

### Step 3 — Mesh Data and Vertex Layout <a name="step-3"></a>

Placement: on_attach.

Define two triangles forming a quad. Pack UV into the vertex attribute at
location 2 using the color slot’s `.xy` for simplicity.

```rust
use lambda::render::{
  mesh::{Mesh, MeshBuilder},
  vertex::{Vertex, VertexAttribute, VertexBuilder, VertexElement},
  ColorFormat,
};

let vertices: [Vertex; 6] = [
  VertexBuilder::new().with_position([-0.5, -0.5, 0.0]).with_normal([0.0, 0.0, 1.0]).with_color([0.0, 0.0, 0.0]).build(), // uv (0,0)
  VertexBuilder::new().with_position([ 0.5, -0.5, 0.0]).with_normal([0.0, 0.0, 1.0]).with_color([1.0, 0.0, 0.0]).build(), // uv (1,0)
  VertexBuilder::new().with_position([ 0.5,  0.5, 0.0]).with_normal([0.0, 0.0, 1.0]).with_color([1.0, 1.0, 0.0]).build(), // uv (1,1)
  VertexBuilder::new().with_position([-0.5, -0.5, 0.0]).with_normal([0.0, 0.0, 1.0]).with_color([0.0, 0.0, 0.0]).build(), // uv (0,0)
  VertexBuilder::new().with_position([ 0.5,  0.5, 0.0]).with_normal([0.0, 0.0, 1.0]).with_color([1.0, 1.0, 0.0]).build(), // uv (1,1)
  VertexBuilder::new().with_position([-0.5,  0.5, 0.0]).with_normal([0.0, 0.0, 1.0]).with_color([0.0, 1.0, 0.0]).build(), // uv (0,1)
];

let mut mesh_builder = MeshBuilder::new();
vertices.iter().for_each(|v| { mesh_builder.with_vertex(*v); });

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
    VertexAttribute { // color (uv.xy) @ location 2
      location: 2, offset: 0,
      element: VertexElement { format: ColorFormat::Rgb32Sfloat, offset: 24 },
    },
  ])
  .build();
// Persist on component state for later use
self.mesh = Some(mesh);
```

This builds a quad from two triangles and declares the vertex attribute layout that the shaders consume. Positions map to location 0, normals to location 1, and UVs are encoded in the color field at location 2. The mesh currently resides on the CPU; a vertex buffer is created when building the pipeline.

### Step 4 — Build a 2D Texture (Checkerboard) <a name="step-4"></a>

Placement: on_attach.

Generate a simple checkerboard and upload it as an sRGB 2D texture.

```rust
use lambda::render::texture::{TextureBuilder, TextureFormat};

let texture_width = 64u32;
let texture_height = 64u32;
let mut pixels = vec![0u8; (texture_width * texture_height * 4) as usize];
for y in 0..texture_height {
  for x in 0..texture_width {
    let i = ((y * texture_width + x) * 4) as usize;
    let checker = ((x / 8) % 2) ^ ((y / 8) % 2);
    let c = if checker == 0 { 40 } else { 220 };
    pixels[i + 0] = c; // R
    pixels[i + 1] = c; // G
    pixels[i + 2] = c; // B
    pixels[i + 3] = 255; // A
  }
}

let texture = TextureBuilder::new_2d(TextureFormat::Rgba8UnormSrgb)
  .with_size(texture_width, texture_height)
  .with_data(&pixels)
  .with_label("checkerboard")
  .build(render_context.gpu())
  .expect("Failed to create texture");
```

This produces a GPU texture in `Rgba8UnormSrgb` format containing a checkerboard pattern. The builder uploads the CPU byte buffer and returns a handle suitable for binding. Using an sRGB color format ensures correct linearization during sampling in the fragment shader.

### Step 5 — Create a Sampler <a name="step-5"></a>

Create a linear filtering sampler with clamp‑to‑edge addressing.

```rust
use lambda::render::texture::SamplerBuilder;

let sampler = SamplerBuilder::new()
  .linear_clamp()
  .with_label("linear-clamp")
  .build(render_context.gpu());
```

This sampler selects linear minification and magnification with clamp‑to‑edge addressing. Linear filtering smooths the checkerboard when scaled, while clamping prevents wrapping at the texture borders.

### Step 6 — Bind Group Layout and Bind Group <a name="step-6"></a>

Declare the layout and bind the texture and sampler at set 0, bindings 1 and 2.

```rust
use lambda::render::bind::{BindGroupLayoutBuilder, BindGroupBuilder};

let layout = BindGroupLayoutBuilder::new()
  .with_sampled_texture(1) // texture2D at binding 1
  .with_sampler(2)         // sampler   at binding 2
  .build(render_context.gpu());

let bind_group = BindGroupBuilder::new()
  .with_layout(&layout)
  .with_texture(1, &texture)
  .with_sampler(2, &sampler)
  .build(render_context.gpu());
```

The bind group layout declares the shader‑visible interface for set 0: a sampled `texture2D` at binding 1 and a `sampler` at binding 2. The bind group then binds the concrete texture and sampler objects to those indices so the fragment shader can sample them during rendering.

### Step 7 — Create the Render Pipeline <a name="step-7"></a>

Build a pipeline that consumes the mesh vertex buffer and the layout. Disable face culling for simplicity.

```rust
use lambda::render::{
  buffer::BufferBuilder,
  pipeline::{RenderPipelineBuilder, CullingMode},
  render_pass::RenderPassBuilder,
};

let render_pass = RenderPassBuilder::new()
  .with_label("textured-quad-pass")
  .build(
    render_context.gpu(),
    render_context.surface_format(),
    render_context.depth_format(),
  );

let mesh = self.mesh.as_ref().expect("mesh must be created");

let pipeline = RenderPipelineBuilder::new()
  .with_culling(CullingMode::None)
  .with_layouts(&[&layout])
  .with_buffer(
    BufferBuilder::build_from_mesh(mesh, render_context.gpu())
      .expect("Failed to create vertex buffer"),
    mesh.attributes().to_vec(),
  )
  .build(
    render_context.gpu(),
    render_context.surface_format(),
    render_context.depth_format(),
    &render_pass,
    &self.shader_vs,
    Some(&self.shader_fs),
  );

// Attach resources to obtain `ResourceId`s for rendering
self.render_pass = Some(render_context.attach_render_pass(render_pass));
self.render_pipeline = Some(render_context.attach_pipeline(pipeline));
self.bind_group = Some(render_context.attach_bind_group(bind_group));
```

The render pass targets the surface’s color attachment. The pipeline uses the compiled shaders, disables face culling for clarity, and declares a vertex buffer built from the mesh with attribute descriptors that match the shader locations. Attaching the pass, pipeline, and bind group to the render context yields stable `ResourceId`s that render commands will reference.

### Step 8 — Record Draw Commands <a name="step-8"></a>

Center a square viewport inside the window, bind pipeline, bind group, and draw six vertices.

```rust
use lambda::render::{command::RenderCommand, viewport::ViewportBuilder};

let win_w = self.width.max(1);
let win_h = self.height.max(1);
let side = u32::min(win_w, win_h);
let x = ((win_w - side) / 2) as i32;
let y = ((win_h - side) / 2) as i32;
let viewport = ViewportBuilder::new().with_coordinates(x, y).build(side, side);

let commands = vec![
  RenderCommand::BeginRenderPass {
    render_pass: self.render_pass.expect("render pass not set"),
    viewport,
  },
  RenderCommand::SetPipeline {
    pipeline: self.render_pipeline.expect("pipeline not set"),
  },
  RenderCommand::SetBindGroup {
    set: 0,
    group: self.bind_group.expect("bind group not set"),
    dynamic_offsets: vec![],
  },
  RenderCommand::BindVertexBuffer {
    pipeline: self.render_pipeline.expect("pipeline not set"),
    buffer: 0,
  },
  RenderCommand::Draw { vertices: 0..6, instances: 0..1 },
  RenderCommand::EndRenderPass,
];
```

These commands open a render pass with a centered square viewport, select the pipeline, bind the texture and sampler group at set 0, bind the vertex buffer at slot 0, draw six vertices, and end the pass. When submitted, they render a textured quad while preserving aspect ratio via the viewport.

### Step 9 — Handle Window Resize <a name="step-9"></a>

Track window size from events and recompute the centered square viewport.

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

This window event handler updates the stored window dimensions when a resize occurs. The render path uses these values to recompute the centered square viewport so the quad remains square and centered as the window changes size.

## Validation <a name="validation"></a>

- Build the workspace: `cargo build --workspace`
- Run the demo (workspace root): `cargo run -p lambda-demos-render --bin textured_quad`
  - If needed, specify the package: `cargo run -p lambda-demos-render --bin textured_quad`
- Expected behavior: a centered square quad shows a gray checkerboard. Resizing the window preserves square aspect ratio by letterboxing with the viewport. With linear filtering, downscaling appears smooth.

## Notes <a name="notes"></a>

- sRGB vs linear formats: `Rgba8UnormSrgb` SHOULD be used for color images so sampling converts to linear space automatically. Use non‑sRGB formats (for example, `Rgba8Unorm`) for data textures like normal maps.
- Binding indices: The `BindGroupLayout` and `BindGroup` indices MUST match shader `set` and `binding` qualifiers. Mismatches surface as validation errors.
- Vertex attributes: Packing UV into the color slot is a simplification for the example. Defining a dedicated UV attribute at its own location is RECOMMENDED for production code.
- Filtering and addressing: `linear_clamp` sets linear min/mag and clamp‑to‑edge. Pixel art MAY prefer `nearest_*`. Tiling textures SHOULD use `Repeat` address modes.
- Pipeline layout: Include all used layouts via `.with_layouts(...)` when creating the pipeline; otherwise binding state is incomplete at draw time.

## Conclusion <a name="conclusion"></a>

This tutorial implemented a complete 2D sampling path. It generated a
checkerboard on the CPU, uploaded it as an sRGB texture, created a
linear‑clamp sampler, and defined matching binding layouts. Shaders forwarded
UV and sampled the texture; a mesh and render pipeline were built; commands
were recorded using a centered viewport. The result renders a textured quad
with correct color space handling and filtering.

## Putting It Together <a name="putting-it-together"></a>

- Full reference: `demos/render/src/bin/textured_quad.rs`
- Minimal differences: the example includes empty `on_detach` and `on_update` hooks and a log line in `on_attach`.

## Exercises <a name="exercises"></a>

- Exercise 1: Nearest filtering
  - Replace `linear_clamp()` with `nearest_clamp()` and observe sharper scaling.
- Exercise 2: Repeat addressing
  - Change UVs to extend beyond `[0, 1]` and use repeat addressing to tile the checkerboard.
- Exercise 3: Load an image file
  - Decode a PNG into RGBA bytes (any Rust image decoder) and upload with `TextureBuilder::new_2d`.
- Exercise 4: Vertical flip
  - Flip UV.y to compare conventions; document expected orientation.
- Exercise 5: Dual‑texture blend
  - Bind two textures and blend them in the fragment shader based on UV.
- Exercise 6: Mipmap exploration (conceptual)
  - Discuss artifacts without mipmaps and how multiple levels would improve minification.

## Changelog <a name="changelog"></a>

- 0.4.2 (2026-02-05): Update demo commands and reference paths for `demos/`.
- 0.4.1 (2026-01-16): Replace `on_event` resize handling with `event_mask()` and `on_window_event`.
- 0.4.0 (2025-12-15): Update builder API calls to use `render_context.gpu()` and add `surface_format`/`depth_format` parameters to `RenderPassBuilder` and `RenderPipelineBuilder`.
- 0.3.3 (2025-11-10): Add Conclusion section summarizing outcomes; update metadata and commit.
- 0.3.2 (2025-11-10): Add narrative explanations after each code block; clarify lifecycle and binding flow.
- 0.3.1 (2025-11-10): Align with example; add shader constants; attach resources; fix variable names; add missing section.
- 0.3.0 (2025-11-01): Initial draft aligned with `demos/render/src/bin/textured_quad.rs`.
