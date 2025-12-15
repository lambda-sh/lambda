---
title: "Indexed Draws and Multiple Vertex Buffers"
document_id: "indexed-draws-multiple-vertex-buffers-tutorial-2025-11-22"
status: "draft"
created: "2025-11-22T00:00:00Z"
last_updated: "2025-12-15T00:00:00Z"
version: "0.3.0"
engine_workspace_version: "2023.1.30"
wgpu_version: "26.0.1"
shader_backend_default: "naga"
winit_version: "0.29.10"
repo_commit: "71256389b9efe247a59aabffe9de58147b30669d"
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
  - [Step 1 — Shaders and Vertex Types](#step-1)
  - [Step 2 — Component State and Shader Construction](#step-2)
  - [Step 3 — Render Pass, Vertex Data, Buffers, and Pipeline](#step-3)
  - [Step 4 — Resize Handling and Updates](#step-4)
  - [Step 5 — Render Commands and Runtime Entry Point](#step-5)
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

### Step 1 — Shaders and Vertex Types <a name="step-1"></a>
Step 1 defines the shader interface and vertex structures used by the example. The shaders consume positions and colors at locations `0` and `1`, and the vertex types store those attributes as three-component floating-point arrays.

```glsl
#version 450

layout (location = 0) in vec3 vertex_position;
layout (location = 1) in vec3 vertex_color;

layout (location = 0) out vec3 frag_color;

void main() {
  gl_Position = vec4(vertex_position, 1.0);
  frag_color = vertex_color;
}
```

```glsl
#version 450

layout (location = 0) in vec3 frag_color;
layout (location = 0) out vec4 fragment_color;

void main() {
  fragment_color = vec4(frag_color, 1.0);
}
```

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
```

The shader `location` qualifiers match the vertex buffer layouts declared on the pipeline, and the `PositionVertex` and `ColorVertex` types mirror the `vec3` inputs as `[f32; 3]` arrays in Rust.

### Step 2 — Component State and Shader Construction <a name="step-2"></a>
Step 2 introduces the `IndexedMultiBufferExample` component and its `Default` implementation, which builds shader objects from the GLSL source and initializes render-resource fields and window dimensions.

```rust
use lambda::render::{
  shader::{
    Shader,
    ShaderBuilder,
    ShaderKind,
    VirtualShader,
  },
  RenderContext,
  ResourceId,
};

pub struct IndexedMultiBufferExample {
  vertex_shader: Shader,
  fragment_shader: Shader,
  render_pass_id: Option<ResourceId>,
  render_pipeline_id: Option<ResourceId>,
  index_buffer_id: Option<ResourceId>,
  index_count: u32,
  width: u32,
  height: u32,
}

impl Default for IndexedMultiBufferExample {
  fn default() -> Self {
    let vertex_virtual_shader = VirtualShader::Source {
      source: VERTEX_SHADER_SOURCE.to_string(),
      kind: ShaderKind::Vertex,
      entry_point: "main".to_string(),
      name: "indexed_multi_vertex_buffers".to_string(),
    };

    let fragment_virtual_shader = VirtualShader::Source {
      source: FRAGMENT_SHADER_SOURCE.to_string(),
      kind: ShaderKind::Fragment,
      entry_point: "main".to_string(),
      name: "indexed_multi_vertex_buffers".to_string(),
    };

    let mut builder = ShaderBuilder::new();
    let vertex_shader = builder.build(vertex_virtual_shader);
    let fragment_shader = builder.build(fragment_virtual_shader);

    return Self {
      vertex_shader,
      fragment_shader,
      render_pass_id: None,
      render_pipeline_id: None,
      index_buffer_id: None,
      index_count: 0,
      width: 800,
      height: 600,
    };
  }
}
```

This `Default` implementation ensures that the component has valid shaders and initial dimensions before it attaches to the render context.

### Step 3 — Render Pass, Vertex Data, Buffers, and Pipeline <a name="step-3"></a>
Step 3 implements `on_attach` to create the render pass, vertex and index data, GPU buffers, and the render pipeline, then attaches them to the `RenderContext`.

```rust
use lambda::render::buffer::{
  BufferBuilder,
  BufferType,
  Properties,
  Usage,
};

use lambda::render::{
  pipeline::{
    CullingMode,
    RenderPipelineBuilder,
  },
  render_pass::RenderPassBuilder,
  vertex::{
    ColorFormat,
    VertexAttribute,
    VertexElement,
  },
};

fn on_attach(
  &mut self,
  render_context: &mut RenderContext,
) -> Result<ComponentResult, String> {
    let render_pass = RenderPassBuilder::new().build(
      render_context.gpu(),
      render_context.surface_format(),
      render_context.depth_format(),
    );

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

    let position_buffer = BufferBuilder::new()
      .with_usage(Usage::VERTEX)
      .with_properties(Properties::DEVICE_LOCAL)
      .with_buffer_type(BufferType::Vertex)
      .with_label("indexed-positions")
      .build(render_context.gpu(), positions)
      .map_err(|error| error.to_string())?;

    let color_buffer = BufferBuilder::new()
      .with_usage(Usage::VERTEX)
      .with_properties(Properties::DEVICE_LOCAL)
      .with_buffer_type(BufferType::Vertex)
      .with_label("indexed-colors")
      .build(render_context.gpu(), colors)
      .map_err(|error| error.to_string())?;

    let index_buffer = BufferBuilder::new()
      .with_usage(Usage::INDEX)
      .with_properties(Properties::DEVICE_LOCAL)
      .with_buffer_type(BufferType::Index)
      .with_label("indexed-indices")
      .build(render_context.gpu(), indices)
      .map_err(|error| error.to_string())?;

    let pipeline = RenderPipelineBuilder::new()
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
      render_context.gpu(),
      render_context.surface_format(),
      render_context.depth_format(),
      &render_pass,
      &self.vertex_shader,
      Some(&self.fragment_shader),
    );

    self.render_pass_id = Some(render_context.attach_render_pass(render_pass));
    self.render_pipeline_id = Some(render_context.attach_pipeline(pipeline));
    self.index_buffer_id = Some(render_context.attach_buffer(index_buffer));
    self.index_count = index_count;

    logging::info!("Indexed multi-vertex-buffer example attached");
    return Ok(ComponentResult::Success);
}
```

The pipeline uses the order of `with_buffer` calls to assign vertex buffer slots. The first buffer occupies slot `0` and provides attributes at location `0`, while the second buffer occupies slot `1` and provides attributes at location `1`. The component stores attached resource identifiers and the index count for use during rendering.

### Step 4 — Resize Handling and Updates <a name="step-4"></a>
Step 4 wires window resize events into the component and implements detach and update hooks. The resize handler keeps `width` and `height` in sync with the window so that the viewport matches the surface size.

```rust
fn on_detach(
  &mut self,
  _render_context: &mut RenderContext,
) -> Result<ComponentResult, String> {
  logging::info!("Indexed multi-vertex-buffer example detached");
  return Ok(ComponentResult::Success);
}

fn on_event(
  &mut self,
  event: lambda::events::Events,
) -> Result<ComponentResult, String> {
  match event {
    lambda::events::Events::Window { event, .. } => match event {
      WindowEvent::Resize { width, height } => {
        self.width = width;
        self.height = height;
        logging::info!("Window resized to {}x{}", width, height);
      }
      _ => {}
    },
    _ => {}
  }
  return Ok(ComponentResult::Success);
}

fn on_update(
  &mut self,
  _last_frame: &std::time::Duration,
) -> Result<ComponentResult, String> {
  return Ok(ComponentResult::Success);
}
```

The resize path is the only dynamic input in this example. The update hook is a no-op that keeps the component interface aligned with other examples.

### Step 5 — Render Commands and Runtime Entry Point <a name="step-5"></a>
Step 5 records the render commands that bind the pipeline, vertex buffers, and index buffer, and then wires the component into the runtime as a windowed application.

```rust
use lambda::render::{
  command::{
    IndexFormat,
    RenderCommand,
  },
  viewport,
};

fn on_render(
  &mut self,
  _render_context: &mut RenderContext,
) -> Vec<RenderCommand> {
  let viewport =
    viewport::ViewportBuilder::new().build(self.width, self.height);

  let render_pass_id = self
    .render_pass_id
    .expect("Render pass must be attached before rendering");
  let pipeline_id = self
    .render_pipeline_id
    .expect("Pipeline must be attached before rendering");
  let index_buffer_id = self
    .index_buffer_id
    .expect("Index buffer must be attached before rendering");

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
      indices: 0..self.index_count,
      base_vertex: 0,
      instances: 0..1,
    },
    RenderCommand::EndRenderPass,
  ];
}

use lambda::{
  component::Component,
  runtime::start_runtime,
  runtimes::application::ApplicationRuntimeBuilder,
};

fn main() {
  let runtime =
    ApplicationRuntimeBuilder::new("Indexed Multi-Vertex-Buffer Example")
      .with_window_configured_as(move |window_builder| {
        return window_builder
          .with_dimensions(800, 600)
          .with_name("Indexed Multi-Vertex-Buffer Example");
      })
      .with_renderer_configured_as(|renderer_builder| {
        return renderer_builder.with_render_timeout(1_000_000_000);
      })
      .with_component(move |runtime, example: IndexedMultiBufferExample| {
        return (runtime, example);
      })
      .build();

  start_runtime(runtime);
}
```

The commands bind both vertex buffers and the index buffer before issuing `DrawIndexed`. The runtime builder configures the window and renderer and installs the component so that the engine drives `on_attach`, `on_event`, `on_update`, and `on_render` each frame.

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

- 2025-12-15 (v0.3.0) — Update builder API calls to use `render_context.gpu()` and add `surface_format`/`depth_format` parameters to `RenderPassBuilder` and `RenderPipelineBuilder`.
- 2025-11-23 (v0.2.0) — Filled in the implementation steps for the indexed draws and multiple vertex buffers tutorial and aligned the narrative with the `indexed_multi_vertex_buffers` example.
- 2025-11-22 (v0.1.0) — Initial skeleton for the indexed draws and multiple vertex buffers tutorial; content placeholders added for future implementation.
