---
title: "Uniform Buffers: Build a Spinning Triangle"
document_id: "uniform-buffers-tutorial-2025-10-17"
status: "draft"
created: "2025-10-17T00:00:00Z"
last_updated: "2025-10-17T00:15:00Z"
version: "0.2.0"
engine_workspace_version: "2023.1.30"
wgpu_version: "26.0.1"
shader_backend_default: "naga"
winit_version: "0.29.10"
repo_commit: "00aababeb76370ebdeb67fc12ab4393aac5e4193"
owners: ["lambda-sh"]
reviewers: ["engine", "rendering"]
tags: ["tutorial", "graphics", "uniform-buffers", "rust", "wgpu"]
---

Uniform buffer objects (UBOs) are a standard mechanism to pass per‑frame or per‑draw constants to shaders. This document demonstrates a minimal 3D spinning triangle that uses a UBO to provide a model‑view‑projection matrix to the vertex shader.

Reference implementation: `crates/lambda-rs/examples/uniform_buffer_triangle.rs`.

Goals
- Build a spinning triangle that reads a model‑view‑projection matrix from a uniform buffer.
- Learn how to define a uniform block in shaders and mirror it in Rust.
- Learn how to create a bind group layout, allocate a uniform buffer, and write per‑frame data.
- Learn how to construct a render pipeline and issue draw commands using Lambda’s builders.

Prerequisites
- Rust toolchain installed and the workspace builds: `cargo build --workspace`.
- Familiarity with basic Rust and the repository’s example layout.
- Ability to run examples: `cargo run --example minimal` verifies setup.

Requirements and constraints
- The uniform block layout in the shader and the Rust structure MUST match in size, alignment, and field order.
- The bind group layout in Rust MUST match the shader `set` and `binding` indices.
- Matrices MUST be provided in the order expected by the shader (column‑major in this example). Rationale: prevents implicit driver conversions and avoids incorrect transforms.
- Acronyms MUST be defined on first use (e.g., uniform buffer object (UBO)).

Data flow
- CPU writes → UBO → bind group (set 0) → pipeline layout → vertex shader.
- A single UBO MAY be reused across multiple draws and pipelines.

Implementation Steps

1) Runtime and component skeleton
Before rendering, create a minimal application entry point and a `Component` that receives lifecycle callbacks. The engine routes initialization, input, updates, and rendering through the component interface, which provides the context needed to create GPU resources and submit commands.

```rust
use lambda::{
  component::Component,
  runtime::start_runtime,
  runtimes::{
    application::ComponentResult,
    ApplicationRuntimeBuilder,
  },
};

pub struct UniformBufferExample {
  elapsed_seconds: f32,
  width: u32,
  height: u32,
  // we will add resources here as we build
}

impl Default for UniformBufferExample {
  fn default() -> Self {
    return Self {
      elapsed_seconds: 0.0,
      width: 800,
      height: 600,
    };
  }
}

fn main() {
  let runtime = ApplicationRuntimeBuilder::new("3D Uniform Buffer Example")
    .with_window_configured_as(|w| w.with_dimensions(800, 600).with_name("3D Uniform Buffer Example"))
    .with_renderer_configured_as(|r| r.with_render_timeout(1_000_000_000))
    .with_component(|runtime, example: UniformBufferExample| { return (runtime, example); })
    .build();

  start_runtime(runtime);
}
```

2) Vertex and fragment shaders
Define shader stages next. The vertex shader declares three vertex attributes and a uniform block at set 0, binding 0. It multiplies the incoming position by the matrix stored in the UBO. The fragment shader returns the interpolated color. Declaring the uniform block now establishes the contract that the Rust side will satisfy via a matching bind group layout and buffer.

```glsl
// Vertex (GLSL 450)
#version 450
layout (location = 0) in vec3 vertex_position;
layout (location = 1) in vec3 vertex_normal;
layout (location = 2) in vec3 vertex_color;

layout (location = 0) out vec3 frag_color;

layout (set = 0, binding = 0) uniform Globals {
  mat4 render_matrix;
} globals;

void main() {
  gl_Position = globals.render_matrix * vec4(vertex_position, 1.0);
  frag_color = vertex_color;
}
```

```glsl
// Fragment (GLSL 450)
#version 450
layout (location = 0) in vec3 frag_color;
layout (location = 0) out vec4 fragment_color;

void main() {
  fragment_color = vec4(frag_color, 1.0);
}
```

Load these as `VirtualShader::Source` via `ShaderBuilder`:

```rust
use lambda::render::shader::{Shader, ShaderBuilder, ShaderKind, VirtualShader};

let vertex_virtual = VirtualShader::Source {
  source: VERTEX_SHADER_SOURCE.to_string(),
  kind: ShaderKind::Vertex,
  entry_point: "main".to_string(),
  name: "uniform_buffer_triangle".to_string(),
};
let fragment_virtual = VirtualShader::Source {
  source: FRAGMENT_SHADER_SOURCE.to_string(),
  kind: ShaderKind::Fragment,
  entry_point: "main".to_string(),
  name: "uniform_buffer_triangle".to_string(),
};
let mut shader_builder = ShaderBuilder::new();
let vertex_shader: Shader = shader_builder.build(vertex_virtual);
let fragment_shader: Shader = shader_builder.build(fragment_virtual);
```

3) Mesh data and vertex layout
Provide vertex data for a single triangle and describe how the pipeline reads it. Each vertex stores position, normal, and color as three `f32` values. The attribute descriptors specify locations and byte offsets so the pipeline can interpret the packed buffer consistently across platforms.

```rust
use lambda::render::{
  mesh::{Mesh, MeshBuilder},
  vertex::{VertexAttribute, VertexBuilder, VertexElement},
  ColorFormat,
};

let vertices = [
  VertexBuilder::new().with_position([ 1.0,  1.0, 0.0]).with_normal([0.0,0.0,0.0]).with_color([1.0,0.0,0.0]).build(),
  VertexBuilder::new().with_position([-1.0,  1.0, 0.0]).with_normal([0.0,0.0,0.0]).with_color([0.0,1.0,0.0]).build(),
  VertexBuilder::new().with_position([ 0.0, -1.0, 0.0]).with_normal([0.0,0.0,0.0]).with_color([0.0,0.0,1.0]).build(),
];

let mut mesh_builder = MeshBuilder::new();
vertices.iter().for_each(|v| { mesh_builder.with_vertex(v.clone()); });

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
    VertexAttribute { // color @ location 2
      location: 2, offset: 0,
      element: VertexElement { format: ColorFormat::Rgb32Sfloat, offset: 24 },
    },
  ])
  .build();
```

4) Uniform data layout in Rust
Mirror the shader’s uniform block with a Rust structure. Use `#[repr(C)]` so the memory layout is predictable. A `mat4` in the shader corresponds to a 4×4 `f32` array here. Many GPU interfaces expect column‑major matrices; transpose before upload if the local math library is row‑major. This avoids implicit driver conversions and prevents incorrect transforms.

```rust
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct GlobalsUniform {
  pub render_matrix: [[f32; 4]; 4],
}
```

5) Bind group layout at set 0
Create a bind group layout that matches the shader declaration. This layout says: at set 0, binding 0 there is a uniform buffer visible to the vertex stage. The pipeline layout will incorporate this, ensuring the shader and the bound resources agree at draw time.

```rust
use lambda::render::bind::{BindGroupLayoutBuilder, BindingVisibility};

let layout = BindGroupLayoutBuilder::new()
  .with_uniform(0, BindingVisibility::Vertex) // binding 0
  .build(render_context);
```

6) Create the uniform buffer and bind group
Allocate the uniform buffer, seed it with an initial matrix, and create a bind group using the layout. Mark the buffer usage as `UNIFORM` and properties as `CPU_VISIBLE` to permit direct per‑frame writes from the CPU. This is the simplest path for frequently updated data.

```rust
use lambda::render::buffer::{BufferBuilder, Usage, Properties};

let initial_uniform = GlobalsUniform { render_matrix: initial_matrix.transpose() };

let uniform_buffer = BufferBuilder::new()
  .with_length(std::mem::size_of::<GlobalsUniform>())
  .with_usage(Usage::UNIFORM)
  .with_properties(Properties::CPU_VISIBLE)
  .with_label("globals-uniform")
  .build(render_context, vec![initial_uniform])
  .expect("Failed to create uniform buffer");

use lambda::render::bind::BindGroupBuilder;

let bind_group = BindGroupBuilder::new()
  .with_layout(&layout)
  .with_uniform(0, &uniform_buffer, 0, None) // binding 0
  .build(render_context);
```

7) Build the render pipeline
Construct the render pipeline, supplying the bind group layouts, vertex buffer, and the shader pair. Disable face culling for simplicity so both sides of the triangle remain visible regardless of winding during early experimentation.

```rust
use lambda::render::{
  pipeline::RenderPipelineBuilder,
  render_pass::RenderPassBuilder,
};

let render_pass = RenderPassBuilder::new().build(render_context);

let pipeline = RenderPipelineBuilder::new()
  .with_culling(lambda::render::pipeline::CullingMode::None)
  .with_layouts(&[&layout])
  .with_buffer(
    BufferBuilder::build_from_mesh(&mesh, render_context).expect("Failed to create buffer"),
    mesh.attributes().to_vec(),
  )
  .build(render_context, &render_pass, &vertex_shader, Some(&fragment_shader));
```

8) Per‑frame update and write
Animate by recomputing the model‑view‑projection matrix each frame and writing it into the uniform buffer. The helper `compute_model_view_projection_matrix_about_pivot` maintains a correct aspect ratio using the current window dimensions and rotates the model around a chosen pivot.

```rust
use lambda::render::scene_math::{compute_model_view_projection_matrix_about_pivot, SimpleCamera};

const ROTATION_TURNS_PER_SECOND: f32 = 0.12;

fn update_uniform_each_frame(
  elapsed_seconds: f32,
  width: u32,
  height: u32,
  render_context: &mut lambda::render::RenderContext,
  uniform_buffer: &lambda::render::buffer::Buffer,
) {
  let camera = SimpleCamera {
    position: [0.0, 0.0, 3.0],
    field_of_view_in_turns: 0.25,
    near_clipping_plane: 0.1,
    far_clipping_plane: 100.0,
  };

  let angle_in_turns = ROTATION_TURNS_PER_SECOND * elapsed_seconds;
  let model_view_projection_matrix = compute_model_view_projection_matrix_about_pivot(
    &camera,
    width.max(1),
    height.max(1),
    [0.0, -1.0 / 3.0, 0.0], // pivot
    [0.0, 1.0, 0.0],        // axis
    angle_in_turns,
    0.5,                    // scale
    [0.0, 1.0 / 3.0, 0.0],  // translation
  );

  let value = GlobalsUniform { render_matrix: model_view_projection_matrix.transpose() };
  uniform_buffer.write_value(render_context, 0, &value);
}
```

9) Issue draw commands
Record commands in the order the GPU expects: begin the render pass, set the pipeline, configure viewport and scissors, bind the vertex buffer and the uniform bind group, draw the vertices, then end the pass. This sequence describes the full state required for a single draw.

```rust
use lambda::render::{
  command::RenderCommand,
  viewport::ViewportBuilder,
};

let viewport = ViewportBuilder::new().build(width, height);

let commands = vec![
  RenderCommand::BeginRenderPass { render_pass: render_pass_id, viewport: viewport.clone() },
  RenderCommand::SetPipeline { pipeline: pipeline_id },
  RenderCommand::SetViewports { start_at: 0, viewports: vec![viewport.clone()] },
  RenderCommand::SetScissors { start_at: 0, viewports: vec![viewport.clone()] },
  RenderCommand::BindVertexBuffer { pipeline: pipeline_id, buffer: 0 },
  RenderCommand::SetBindGroup { set: 0, group: bind_group_id, dynamic_offsets: vec![] },
  RenderCommand::Draw { vertices: 0..mesh.vertices().len() as u32 },
  RenderCommand::EndRenderPass,
];
```

10) Handle window resize
Track window dimensions and update the per‑frame matrix using the new aspect ratio. Forwarding resize events into stored `width` and `height` maintains consistent camera projection across resizes.

```rust
use lambda::events::{Events, WindowEvent};

fn on_event(&mut self, event: Events) -> Result<ComponentResult, String> {
  if let Events::Window { event, .. } = event {
    if let WindowEvent::Resize { width, height } = event {
      self.width = width;
      self.height = height;
    }
  }
  return Ok(ComponentResult::Success);
}
```

Validation
- Build the workspace: `cargo build --workspace`
- Run the example: `cargo run --example uniform_buffer_triangle`

Notes
- Layout matching: The Rust `GlobalsUniform` MUST match the shader block layout. Keep `#[repr(C)]` and follow alignment rules.
- Matrix order: The shader expects column‑major matrices, so the uploaded matrix MUST be transposed if the local math library uses row‑major.
- Binding indices: The Rust bind group layout and `.with_uniform(0, ...)`, plus the shader `set = 0, binding = 0`, MUST be consistent.
- Update strategy: `CPU_VISIBLE` buffers SHOULD be used for per‑frame updates; device‑local memory MAY be preferred for static data.
- Pipeline layout: All bind group layouts used by the pipeline MUST be included via `.with_layouts(...)`.

Exercises

- Exercise 1: Time‑based fragment color
  - Implement a second UBO at set 0, binding 1 with a `float time_seconds`.
  - Modify the fragment shader to modulate color with a sine of time.
  - Hint: add `.with_uniform(1, BindingVisibility::Fragment)` and a second binding.

- Exercise 2: Camera orbit control
  - Implement an orbiting camera around the origin and update the uniform each frame.
  - Add input to adjust orbit speed.

- Exercise 3: Two objects with dynamic offsets
  - Pack two `GlobalsUniform` matrices into one UBO and issue two draws with different dynamic offsets.
  - Use `dynamic_offsets` in `RenderCommand::SetBindGroup`.

- Exercise 4: Basic Lambert lighting
  - Extend shaders to compute diffuse lighting.
  - Provide a lighting UBO at binding 2 with light position and color.

- Exercise 5: Push constants comparison
  - Port to push constants (see `crates/lambda-rs/examples/push_constants.rs`) and compare trade‑offs.

- Exercise 6: Per‑material uniforms
  - Split per‑frame and per‑material data; use a shared frame UBO and a per‑material UBO (e.g., tint color).

- Exercise 7: Shader hot‑reload (stretch)
  - Rebuild shaders on file changes and re‑create the pipeline while preserving UBOs and bind groups.

Changelog
- 0.2.0 (2025‑10‑17): Added goals and book‑style step explanations; expanded rationale before code blocks; refined validation and notes.
- 0.1.0 (2025‑10‑17): Initial draft aligned with `crates/lambda-rs/examples/uniform_buffer_triangle.rs`.
