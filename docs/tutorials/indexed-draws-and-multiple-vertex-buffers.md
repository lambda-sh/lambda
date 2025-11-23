---
title: "Indexed Draws and Multiple Vertex Buffers"
document_id: "indexed-draws-multiple-vertex-buffers-tutorial-2025-11-22"
status: "draft"
created: "2025-11-22T00:00:00Z"
last_updated: "2025-11-23T00:00:00Z"
version: "0.2.0"
engine_workspace_version: "2023.1.30"
wgpu_version: "26.0.1"
shader_backend_default: "naga"
winit_version: "0.29.10"
repo_commit: "db7fa78d143e5ff69028413fe86c948be9ba76ee"
owners: ["lambda-sh"]
reviewers: ["engine", "rendering"]
tags: ["tutorial", "graphics", "indexed-draws", "vertex-buffers", "rust", "wgpu"]
---

## Overview <a name="overview"></a>
This tutorial constructs a small scene rendered with indexed geometry and multiple vertex buffers. The example separates per-vertex positions from per-vertex colors and draws the result using the engine’s high-level buffer and command builders.

Reference implementation: `crates/lambda-rs/examples/indexed_multi_vertex_buffers.rs`.

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
Step 1 introduces the runtime and component that own the render context, pipeline, and buffers. The component implements the application lifecycle callbacks and records render commands, while the runtime creates the window and drives the main loop.

The example uses a `Component` implementation and an `ApplicationRuntimeBuilder` entry point:

```rust
use lambda::{
  component::Component,
  render::{
    command::RenderCommand,
    RenderContext,
    ResourceId,
  },
  runtime::start_runtime,
  runtimes::{
    application::ComponentResult,
    ApplicationRuntimeBuilder,
  },
};

pub struct IndexedMultiBufferExample {
  render_pass_id: Option<ResourceId>,
  render_pipeline_id: Option<ResourceId>,
  index_buffer_id: Option<ResourceId>,
  index_count: u32,
  width: u32,
  height: u32,
}

impl Component<ComponentResult, String> for IndexedMultiBufferExample {
  fn on_attach(
    &mut self,
    render_context: &mut RenderContext,
  ) -> Result<ComponentResult, String> {
    // Pipeline and buffer setup lives here.
    return Ok(ComponentResult::Success);
  }

  fn on_render(
    &mut self,
    _render_context: &mut RenderContext,
  ) -> Vec<RenderCommand> {
    // Commands are recorded here in later steps.
    return Vec::new();
  }
}

fn main() {
  let runtime =
    ApplicationRuntimeBuilder::new("Indexed Multi-Vertex-Buffer Example")
      .with_window_configured_as(move |window_builder| {
        return window_builder.with_dimensions(800, 600).with_name(
          "Indexed Multi-Vertex-Buffer Example",
        );
      })
      .with_component(move |runtime, example: IndexedMultiBufferExample| {
        return (runtime, example);
      })
      .build();

  start_runtime(runtime);
}
```

The runtime builds a windowed application and instantiates the component. The component stores identifiers for the render pass, pipeline, and buffers and uses the lifecycle callbacks to initialize resources and record commands.

### Step 2 — Vertex and Fragment Shaders <a name="step-2"></a>
Step 2 defines a shader interface that matches the vertex buffers used in the example. The vertex shader reads positions from one vertex buffer slot and colors from another slot. The fragment shader receives the interpolated color and writes it to the frame buffer.

The example uses GLSL (OpenGL Shading Language) with explicit locations:

```glsl
#version 450

layout (location = 0) in vec3 vertex_position;
layout (location = 1) in vec3 vertex_color;

layout (location = 0) out vec3 frag_color;

void main() {
  gl_Position = vec4(vertex_position, 1.0);
  frag_color = vertex_color;
}

// Fragment shader:

#version 450

layout (location = 0) in vec3 frag_color;
layout (location = 0) out vec4 fragment_color;

void main() {
  fragment_color = vec4(frag_color, 1.0);
}
```

Attributes at locations `0` and `1` align with vertex buffer layouts declared on the pipeline. The engine associates each vertex attribute location with elements in `VertexAttribute` lists configured in the pipeline builder.

### Step 3 — Vertex Data, Index Data, and Layouts <a name="step-3"></a>
Step 3 structures vertex and index data for a quad composed of two triangles. Positions live in one buffer and colors in a second buffer. A 16-bit index buffer references the vertices to avoid duplicating position and color data.

The example defines simple vertex types and arrays:

```rust
#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct PositionVertex {
  position: [f32; 3],
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct ColorVertex {
  color: [f32; 3],
}

let positions: Vec<PositionVertex> = vec![
  PositionVertex {
    position: [-0.5, -0.5, 0.0],
  },
  PositionVertex {
    position: [0.5, -0.5, 0.0],
  },
  PositionVertex {
    position: [0.5, 0.5, 0.0],
  },
  PositionVertex {
    position: [-0.5, 0.5, 0.0],
  },
];

let colors: Vec<ColorVertex> = vec![
  ColorVertex {
    color: [1.0, 0.0, 0.0],
  },
  ColorVertex {
    color: [0.0, 1.0, 0.0],
  },
  ColorVertex {
    color: [0.0, 0.0, 1.0],
  },
  ColorVertex {
    color: [1.0, 1.0, 1.0],
  },
];

let indices: Vec<u16> = vec![0, 1, 2, 2, 3, 0];
let index_count = indices.len() as u32;
```

Each index in the `indices` vector references a position and color at the same vertex slot. This layout allows indexed rendering to reuse vertices across triangles while keeping position and color data in separate buffers.

### Step 4 — Create Vertex and Index Buffers <a name="step-4"></a>
Step 4 uploads the CPU-side arrays into GPU buffers. Each vertex stream uses a buffer with `Usage::VERTEX` and `BufferType::Vertex`, while the indices use `Usage::INDEX` and `BufferType::Index`. All buffers are created with device-local properties suitable for static geometry.

```rust
use lambda::render::buffer::{
  BufferBuilder,
  BufferType,
  Properties,
  Usage,
};

let position_buffer = BufferBuilder::new()
  .with_usage(Usage::VERTEX)
  .with_properties(Properties::DEVICE_LOCAL)
  .with_buffer_type(BufferType::Vertex)
  .with_label("indexed-positions")
  .build(render_context, positions)
  .map_err(|error| error.to_string())?;

let color_buffer = BufferBuilder::new()
  .with_usage(Usage::VERTEX)
  .with_properties(Properties::DEVICE_LOCAL)
  .with_buffer_type(BufferType::Vertex)
  .with_label("indexed-colors")
  .build(render_context, colors)
  .map_err(|error| error.to_string())?;

let index_buffer = BufferBuilder::new()
  .with_usage(Usage::INDEX)
  .with_properties(Properties::DEVICE_LOCAL)
  .with_buffer_type(BufferType::Index)
  .with_label("indexed-indices")
  .build(render_context, indices)
  .map_err(|error| error.to_string())?;
```

Usage flags and buffer types ensure that buffers can be bound to the correct pipeline stages. Validation features rely on this metadata to verify bindings for vertex and index buffers during command recording.

### Step 5 — Build the Render Pipeline with Multiple Buffers <a name="step-5"></a>
Step 5 constructs a render pipeline that declares two vertex buffers: one for positions and one for colors. Each buffer is registered on the pipeline with a `VertexAttribute` list that matches the shader’s attribute locations and formats.

The example uses `RenderPipelineBuilder` with two `with_buffer` calls:

```rust
use lambda::render::{
  pipeline::{
    CullingMode,
    RenderPipelineBuilder,
  },
  shader::Shader,
  vertex::{
    ColorFormat,
    VertexAttribute,
    VertexElement,
  },
};

fn build_pipeline(
  render_context: &mut RenderContext,
  render_pass: &RenderPass,
  vertex_shader: &Shader,
  fragment_shader: &Shader,
  position_buffer: ResourceId,
  color_buffer: ResourceId,
) -> RenderPipeline {
  return RenderPipelineBuilder::new()
    .with_culling(CullingMode::Back)
    .with_buffer(
      position_buffer,
      vec![VertexAttribute {
        location: 0,
        offset: 0,
        element: VertexElement {
          format: ColorFormat::Rgb32Sfloat,
          offset: 0,
        },
      }],
    )
    .with_buffer(
      color_buffer,
      vec![VertexAttribute {
        location: 1,
        offset: 0,
        element: VertexElement {
          format: ColorFormat::Rgb32Sfloat,
          offset: 0,
        },
      }],
    )
    .build(
      render_context,
      render_pass,
      vertex_shader,
      Some(fragment_shader),
    );
}
```

The pipeline uses the order of `with_buffer` calls to assign vertex buffer slots. The first buffer occupies slot `0` and provides attributes at location `0`, while the second buffer occupies slot `1` and provides attributes at location `1`.

### Step 6 — Record Commands with BindVertexBuffer and BindIndexBuffer <a name="step-6"></a>
Step 6 records the render commands that draw the quad. The command sequence begins a render pass, sets the pipeline, configures the viewport and scissors, binds both vertex buffers, binds the index buffer, and then issues a `DrawIndexed` command.

```rust
use lambda::render::{
  command::{
    IndexFormat,
    RenderCommand,
  },
  viewport,
};

fn record_commands(
  render_pass_id: ResourceId,
  pipeline_id: ResourceId,
  index_buffer_id: ResourceId,
  index_count: u32,
  width: u32,
  height: u32,
) -> Vec<RenderCommand> {
  let viewport =
    viewport::ViewportBuilder::new().build(width.max(1), height.max(1));

  return vec![
    RenderCommand::BeginRenderPass {
      render_pass: render_pass_id,
      viewport: viewport.clone(),
    },
    RenderCommand::SetPipeline {
      pipeline: pipeline_id,
    },
    RenderCommand::SetViewports {
      start_at: 0,
      viewports: vec![viewport.clone()],
    },
    RenderCommand::SetScissors {
      start_at: 0,
      viewports: vec![viewport.clone()],
    },
    RenderCommand::BindVertexBuffer {
      pipeline: pipeline_id,
      buffer: 0,
    },
    RenderCommand::BindVertexBuffer {
      pipeline: pipeline_id,
      buffer: 1,
    },
    RenderCommand::BindIndexBuffer {
      buffer: index_buffer_id,
      format: IndexFormat::Uint16,
    },
    RenderCommand::DrawIndexed {
      indices: 0..index_count,
      base_vertex: 0,
      instances: 0..1,
    },
    RenderCommand::EndRenderPass,
  ];
}
```

The commands bind both vertex buffers and the index buffer before issuing `DrawIndexed`. Validation features can detect missing or mismatched bindings if the index buffer format or vertex buffer slots do not match the pipeline configuration.

### Step 7 — Add Simple Camera or Transform <a name="step-7"></a>
Step 7 is optional and can introduce transforms or a camera scheme on top of the indexed draw path. A uniform buffer or push constants can hold a model-view-projection matrix that affects vertex positions while leaving the vertex and index buffer layout unchanged.

The reference example keeps the quad static in clip space and does not add transforms. Applications that need motion can extend the pattern by adding uniforms without changing the indexing scheme or buffer bindings.

### Step 8 — Handle Resize and Resource Lifetime <a name="step-8"></a>
Step 8 handles window resizes and the lifetime of render resources. The example tracks the current width and height in the component and updates these values in the window event handler so that viewports and scissors can be recomputed.

Resources such as the render pass, pipeline, and buffers remain valid across resizes in this simple example. More advanced scenarios that depend on window size can rebuild passes or pipelines in response to resize events while keeping vertex and index buffers intact.

## Validation <a name="validation"></a>

- Commands:
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

This tutorial demonstrates how indexed draws and multiple vertex buffers combine to render geometry efficiently while keeping the engine’s high-level abstractions simple. The example in `crates/lambda-rs/examples/indexed_multi_vertex_buffers.rs` provides a concrete reference for applications that require indexed meshes or split vertex streams.

## Exercises <a name="exercises"></a>

- Extend the example to render multiple meshes that share the same index buffer but use different color data.
- Add a per-instance transform buffer and demonstrate instanced drawing by varying transforms while reusing positions and indices.
- Introduce a wireframe mode that uses the same vertex and index buffers but modifies pipeline state to emphasize edge connectivity.
- Experiment with `u16` versus `u32` indices and measure the effect on buffer size and performance for larger meshes.
- Add a debug mode that binds an incorrect index format intentionally and observe how validation features report the error.

## Changelog <a name="changelog"></a>

- 2025-11-23 (v0.2.0) — Filled in the implementation steps for the indexed draws and multiple vertex buffers tutorial and aligned the narrative with the `indexed_multi_vertex_buffers` example.
- 2025-11-22 (v0.1.0) — Initial skeleton for the indexed draws and multiple vertex buffers tutorial; content placeholders added for future implementation.
