---
title: "Instanced Rendering: Grid of Colored Quads"
document_id: "instanced-quads-tutorial-2025-11-25"
status: "draft"
created: "2025-11-25T00:00:00Z"
last_updated: "2025-12-15T00:00:00Z"
version: "0.2.0"
engine_workspace_version: "2023.1.30"
wgpu_version: "26.0.1"
shader_backend_default: "naga"
winit_version: "0.29.10"
repo_commit: "71256389b9efe247a59aabffe9de58147b30669d"
owners: ["lambda-sh"]
reviewers: ["engine", "rendering"]
tags: ["tutorial", "graphics", "instancing", "vertex-buffers", "rust", "wgpu"]
---

## Overview <a name="overview"></a>
This tutorial builds an instanced rendering example using the `lambda-rs` crate. The final application renders a grid of 2D quads that all share the same geometry but read per-instance offsets and colors from a second vertex buffer. The example demonstrates how to configure per-vertex and per-instance buffers, construct an instanced render pipeline, and issue draw commands with a multi-instance range.

Reference implementation: `crates/lambda-rs/examples/instanced_quads.rs`.

## Goals <a name="goals"></a>

- Build an instanced rendering example that draws a grid of quads using shared geometry and per-instance data.
- Understand how per-vertex and per-instance attributes are described on `RenderPipelineBuilder`.
- Learn how `RenderCommand::DrawIndexed` uses an instance range to control how many instances are rendered.
- Reinforce correct usage flags and buffer types for vertex and index data.

## Prerequisites <a name="prerequisites"></a>

- The workspace builds successfully: `cargo build --workspace`.
- Familiarity with the `lambda-rs` runtime and component model, for example from the indexed draws and uniform buffer tutorials.
- Ability to run examples:
  - `cargo run -p lambda-rs --example minimal`
  - `cargo run -p lambda-rs --example instanced_quads`

## Requirements and Constraints <a name="requirements-and-constraints"></a>

- Per-vertex and per-instance vertex attribute layouts on the pipeline MUST match shader `location` qualifiers and data formats.
- The instance buffer MUST be bound to the same slot that `with_instance_buffer` configures on the pipeline before issuing draw commands that rely on per-instance data.
- Draw calls that use instancing MUST provide an `instances` range where `start` is less than or equal to `end`. Rationale: `RenderContext` validation rejects inverted ranges.
- The instance buffer MAY be updated over time to animate positions or colors; this tutorial uses a static grid for clarity.

## Data Flow <a name="data-flow"></a>

- Quad geometry (positions) and instance data (offsets and colors) are constructed in Rust.
- Two vertex buffers (one per-vertex and one per-instance) and an index buffer are created using `BufferBuilder`.
- A `RenderPipeline` associates slot `0` with per-vertex positions and slot `1` with per-instance data.
- At render time, commands bind the pipeline, vertex buffers, and index buffer, then issue `DrawIndexed` with an instance range that covers the grid.

ASCII diagram

```
quad vertices, instance offsets, colors
   │  upload via BufferBuilder
   ▼
Vertex Buffer (slot 0, per-vertex)   Vertex Buffer (slot 1, per-instance)
   │                                  │
   └──────────────┬───────────────────┘
                  ▼
RenderPipeline (per-vertex + per-instance layouts)
   │
RenderCommand::{BindVertexBuffer, BindIndexBuffer, DrawIndexed}
   │
Render Pass
```

## Implementation Steps <a name="implementation-steps"></a>

### Step 1 — Shaders and Attribute Layout <a name="step-1"></a>
Step 1 defines the vertex and fragment shaders for instanced quads. The vertex shader consumes per-vertex positions and per-instance offsets and colors, and the fragment shader writes the interpolated color.

```glsl
#version 450

layout (location = 0) in vec3 vertex_position;
layout (location = 1) in vec3 instance_offset;
layout (location = 2) in vec3 instance_color;

layout (location = 0) out vec3 frag_color;

void main() {
  vec3 position = vertex_position + instance_offset;
  gl_Position = vec4(position, 1.0);
  frag_color = instance_color;
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

Attribute locations `0`, `1`, and `2` correspond to pipeline vertex attribute definitions for the per-vertex position and the per-instance offset and color. These locations will be matched by `VertexAttribute` entries when the render pipeline is constructed.

### Step 2 — Vertex and Instance Types and Component State <a name="step-2"></a>
Step 2 introduces the Rust vertex and instance structures and prepares the component state. The component stores compiled shaders and identifiers for the render pass, pipeline, and buffers.

```rust
use lambda::{
  component::Component,
  events::WindowEvent,
  logging,
  render::{
    buffer::{
      BufferBuilder,
      BufferType,
      Properties,
      Usage,
    },
    command::{
      IndexFormat,
      RenderCommand,
    },
    pipeline::{
      CullingMode,
      RenderPipelineBuilder,
    },
    render_pass::RenderPassBuilder,
    shader::{
      Shader,
      ShaderBuilder,
      ShaderKind,
      VirtualShader,
    },
    vertex::{
      ColorFormat,
      VertexAttribute,
      VertexElement,
    },
    viewport,
    RenderContext,
    ResourceId,
  },
  runtime::start_runtime,
  runtimes::{
    application::ComponentResult,
    ApplicationRuntime,
    ApplicationRuntimeBuilder,
  },
};

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct QuadVertex {
  position: [f32; 3],
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct InstanceData {
  offset: [f32; 3],
  color: [f32; 3],
}

pub struct InstancedQuadsExample {
  vertex_shader: Shader,
  fragment_shader: Shader,
  render_pass_id: Option<ResourceId>,
  render_pipeline_id: Option<ResourceId>,
  index_buffer_id: Option<ResourceId>,
  index_count: u32,
  instance_count: u32,
  width: u32,
  height: u32,
}

impl Default for InstancedQuadsExample {
  fn default() -> Self {
    let vertex_virtual_shader = VirtualShader::Source {
      source: VERTEX_SHADER_SOURCE.to_string(),
      kind: ShaderKind::Vertex,
      entry_point: "main".to_string(),
      name: "instanced_quads".to_string(),
    };

    let fragment_virtual_shader = VirtualShader::Source {
      source: FRAGMENT_SHADER_SOURCE.to_string(),
      kind: ShaderKind::Fragment,
      entry_point: "main".to_string(),
      name: "instanced_quads".to_string(),
    };

    let mut shader_builder = ShaderBuilder::new();
    let vertex_shader = shader_builder.build(vertex_virtual_shader);
    let fragment_shader = shader_builder.build(fragment_virtual_shader);

    return Self {
      vertex_shader,
      fragment_shader,
      render_pass_id: None,
      render_pipeline_id: None,
      index_buffer_id: None,
      index_count: 0,
      instance_count: 0,
      width: 800,
      height: 600,
    };
  }
}
```

The `QuadVertex` and `InstanceData` structures mirror the GLSL inputs as arrays of `f32`, and the component tracks resource identifiers and counts that are populated during attachment. The `Default` implementation constructs shader objects from the GLSL source so that the component is ready to build a pipeline when it receives a `RenderContext`.

### Step 3 — Render Pass, Geometry, Instances, and Buffers <a name="step-3"></a>
Step 3 implements the `on_attach` method for the component. This method creates the render pass, quad geometry, instance data, GPU buffers, and the render pipeline. It also records the number of indices and instances for use during rendering.

```rust
fn on_attach(
  &mut self,
  render_context: &mut RenderContext,
) -> Result<ComponentResult, String> {
  let render_pass = RenderPassBuilder::new().build(
    render_context.gpu(),
    render_context.surface_format(),
    render_context.depth_format(),
  );

  // Quad geometry in clip space centered at the origin.
  let quad_vertices: Vec<QuadVertex> = vec![
    QuadVertex {
      position: [-0.05, -0.05, 0.0],
    },
    QuadVertex {
      position: [0.05, -0.05, 0.0],
    },
    QuadVertex {
      position: [0.05, 0.05, 0.0],
    },
    QuadVertex {
      position: [-0.05, 0.05, 0.0],
    },
  ];

  // Two triangles forming a quad.
  let indices: Vec<u16> = vec![0, 1, 2, 2, 3, 0];
  let index_count = indices.len() as u32;

  // Build a grid of instance offsets and colors.
  let grid_size: u32 = 10;
  let spacing: f32 = 0.2;
  let start: f32 = -0.9;

  let mut instances: Vec<InstanceData> = Vec::new();
  for y in 0..grid_size {
    for x in 0..grid_size {
      let offset_x = start + (x as f32) * spacing;
      let offset_y = start + (y as f32) * spacing;

      // Simple color gradient across the grid.
      let color_r = (x as f32) / ((grid_size - 1) as f32);
      let color_g = (y as f32) / ((grid_size - 1) as f32);
      let color_b = 0.5;

      instances.push(InstanceData {
        offset: [offset_x, offset_y, 0.0],
        color: [color_r, color_g, color_b],
      });
    }
  }
  let instance_count = instances.len() as u32;

  // Build vertex, instance, and index buffers.
  let vertex_buffer = BufferBuilder::new()
    .with_usage(Usage::VERTEX)
    .with_properties(Properties::DEVICE_LOCAL)
    .with_buffer_type(BufferType::Vertex)
    .with_label("instanced-quads-vertices")
    .build(render_context.gpu(), quad_vertices)
    .map_err(|error| error.to_string())?;

  let instance_buffer = BufferBuilder::new()
    .with_usage(Usage::VERTEX)
    .with_properties(Properties::DEVICE_LOCAL)
    .with_buffer_type(BufferType::Vertex)
    .with_label("instanced-quads-instances")
    .build(render_context.gpu(), instances)
    .map_err(|error| error.to_string())?;

  let index_buffer = BufferBuilder::new()
    .with_usage(Usage::INDEX)
    .with_properties(Properties::DEVICE_LOCAL)
    .with_buffer_type(BufferType::Index)
    .with_label("instanced-quads-indices")
    .build(render_context.gpu(), indices)
    .map_err(|error| error.to_string())?;

  // Vertex attributes for per-vertex positions in slot 0.
  let vertex_attributes = vec![VertexAttribute {
    location: 0,
    offset: 0,
    element: VertexElement {
      format: ColorFormat::Rgb32Sfloat,
      offset: 0,
    },
  }];

  // Instance attributes in slot 1: offset and color.
  let instance_attributes = vec![
    VertexAttribute {
      location: 1,
      offset: 0,
      element: VertexElement {
        format: ColorFormat::Rgb32Sfloat,
        offset: 0,
      },
    },
    VertexAttribute {
      location: 2,
      offset: 0,
      element: VertexElement {
        format: ColorFormat::Rgb32Sfloat,
        offset: 12,
      },
    },
  ];

  let pipeline = RenderPipelineBuilder::new()
    .with_culling(CullingMode::Back)
    .with_buffer(vertex_buffer, vertex_attributes)
    .with_instance_buffer(instance_buffer, instance_attributes)
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
  self.instance_count = instance_count;

  logging::info!(
    "Instanced quads example attached with {} instances",
    self.instance_count
  );
  return Ok(ComponentResult::Success);
}
```

The first buffer created by `with_buffer` is treated as a per-vertex buffer in slot `0`, while `with_instance_buffer` registers the instance buffer in slot `1` with per-instance step mode. The `vertex_attributes` and `instance_attributes` vectors connect shader locations `0`, `1`, and `2` to their corresponding buffer slots and formats, and the component records index and instance counts for later draws. The effective byte offset of each attribute is computed as `attribute.offset + attribute.element.offset`. In this example `attribute.offset` is kept at `0` for all attributes, and the struct layout is expressed entirely through `VertexElement::offset` (for example, the `color` field in `InstanceData` starts 12 bytes after the `offset` field). More complex layouts MAY use a non-zero `attribute.offset` to reuse the same attribute description at different base positions within a vertex or instance element.

### Step 4 — Resize Handling and Updates <a name="step-4"></a>
Step 4 wires window resize events into the component and implements detach and update hooks. The resize handler keeps `width` and `height` in sync with the window so that the viewport matches the surface size.

```rust
fn on_detach(
  &mut self,
  _render_context: &mut RenderContext,
) -> Result<ComponentResult, String> {
  logging::info!("Instanced quads example detached");
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
  // This example uses static instance data; no per-frame updates required.
  return Ok(ComponentResult::Success);
}
```

The component does not modify instance data over time, so `on_update` is a no-op. The resize path is the only dynamic input and ensures that the viewport used during rendering matches the current window size.

### Step 5 — Render Commands and Runtime Entry Point <a name="step-5"></a>
Step 5 records the render commands that bind the pipeline, vertex buffers, and index buffer, then wires the component into the `lambda-rs` runtime as a windowed application.

```rust
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
      instances: 0..self.instance_count,
    },
    RenderCommand::EndRenderPass,
  ];
}

fn main() {
  let runtime: ApplicationRuntime =
    ApplicationRuntimeBuilder::new("Instanced Quads Example")
      .with_window_configured_as(|window_builder| {
        return window_builder
          .with_dimensions(800, 600)
          .with_name("Instanced Quads Example");
      })
      .with_renderer_configured_as(|render_builder| {
        return render_builder.with_render_timeout(1_000_000_000);
      })
      .with_component(|runtime, example: InstancedQuadsExample| {
        return (runtime, example);
      })
      .build();

  start_runtime(runtime);
}
```

The commands bind both vertex buffers and the index buffer before issuing `DrawIndexed`. The `instances: 0..self.instance_count` range enables instanced rendering, and the runtime builder configures the window and renderer and installs the component so that `lambda-rs` drives `on_attach`, `on_event`, `on_update`, and `on_render` each frame.

## Validation <a name="validation"></a>

- Commands:
  - `cargo run -p lambda-rs --example instanced_quads`
  - `cargo test -p lambda-rs -- --nocapture`
- Expected behavior:
  - A grid of small quads appears in the window, with colors varying smoothly across the grid based on instance indices.
  - Changing the grid size or instance count SHOULD change the number of quads rendered without altering the per-vertex geometry.

## Notes <a name="notes"></a>

- Vertex attribute locations in the shaders MUST match the `VertexAttribute` configuration for both the per-vertex and per-instance buffers.
- The instance buffer MUST be bound on the same slot that `with_instance_buffer` uses; binding a different slot will lead to incorrect or undefined attribute data during rendering.
- Instance ranges for `DrawIndexed` MUST remain within the logical count of instances created for the instance buffer; validation features such as `render-validation-instancing` SHOULD be enabled when developing new instanced render paths.
- Per-instance data MAY be updated each frame to animate offsets or colors; static data is sufficient for verifying buffer layouts and instance ranges.

## Conclusion <a name="conclusion"></a>

This tutorial demonstrates how the `lambda-rs` crate uses per-vertex and per-instance vertex buffers to render a grid of quads with shared geometry and per-instance colors. The `instanced_quads` example serves as a concrete reference for applications that require instanced rendering of repeated geometry with varying per-instance attributes.

## Exercises <a name="exercises"></a>

- Modify the grid dimensions and spacing to explore different layouts and densities of quads.
- Animate instance offsets over time in `on_update` to create a simple wave pattern across the grid.
- Introduce a uniform buffer that applies a global transform to all instances and combine it with per-instance offsets.
- Extend the shaders to include per-instance scale or rotation and add fields to `InstanceData` to drive those transforms.
- Add a second instanced draw call that uses the same geometry but a different instance buffer to render a second grid with an alternate color pattern.
- Experiment with validation features, such as `render-validation-instancing`, by intentionally omitting the instance buffer binding and observing how configuration errors are reported.

## Changelog <a name="changelog"></a>

- 2025-12-15 (v0.2.0) — Update builder API calls to use `render_context.gpu()` and add `surface_format`/`depth_format` parameters to `RenderPassBuilder` and `RenderPipelineBuilder`.
- 2025-11-25 (v0.1.1) — Align feature naming with `render-validation-instancing` and update metadata.
- 2025-11-25 (v0.1.0) — Initial instanced quads tutorial describing per-vertex and per-instance buffers and the `instanced_quads` example.
