---
title: "Physics 2D: Falling Quad (Kinematic)"
document_id: "physics-falling-quad-kinematic-2026-02-07"
status: "draft"
created: "2026-02-07T00:00:00Z"
last_updated: "2026-02-07T00:00:00Z"
version: "0.1.0"
engine_workspace_version: "2023.1.30"
wgpu_version: "28.0.0"
shader_backend_default: "naga"
winit_version: "0.29.10"
repo_commit: "4b0c5abf6743788596177b3c10c3214db20ad6b1"
owners: ["lambda-sh"]
reviewers: ["engine", "rendering"]
tags: ["tutorial", "physics", "2d", "fixed-timestep", "uniform-buffers"]
---

## Overview <a name="overview"></a>

This tutorial builds a minimal physics-enabled render demo that shows a single
2D quad falling under gravity using a fixed-timestep update loop. The quad’s
motion is integrated with simple kinematics and written to a uniform buffer as
a 2D offset applied in the vertex shader.

The physics world is stepped each tick to validate fixed timestep and sub-step
behavior for an empty world. The demo does not define rigid bodies or collision
shapes.

Reference implementation: `demos/physics/src/bin/physics_falling_quad.rs`.

## Table of Contents

- [Overview](#overview)
- [Goals](#goals)
- [Prerequisites](#prerequisites)
- [Requirements and Constraints](#requirements-and-constraints)
- [Data Flow](#data-flow)
- [Implementation Steps](#implementation-steps)
  - [Step 1 — Create a Physics Demos Crate](#step-1)
  - [Step 2 — Runtime and Component Skeleton](#step-2)
  - [Step 3 — Shaders and Uniform Contract](#step-3)
  - [Step 4 — Quad Mesh and Render Pipeline](#step-4)
  - [Step 5 — Fixed-Timestep Update and Kinematics](#step-5)
  - [Step 6 — Uniform Writes and Draw Commands](#step-6)
  - [Step 7 — Resize Handling](#step-7)
- [Validation](#validation)
- [Notes](#notes)
- [Conclusion](#conclusion)
- [Exercises](#exercises)
- [Changelog](#changelog)

## Goals <a name="goals"></a>

- Create a new physics demos crate with physics enabled by default.
- Render a quad whose position is driven by a uniform offset.
- Implement a fixed-timestep accumulator loop and integrate a falling motion
  using configured gravity and timestep values.
- Step `PhysicsWorld2D` on each fixed tick and allow the world to exist without
  any bodies.

## Prerequisites <a name="prerequisites"></a>

- The workspace builds: `cargo build --workspace`.
- A machine capable of running the render demos (a compatible graphics adapter
  is required).

## Requirements and Constraints <a name="requirements-and-constraints"></a>

- The demo crate MUST enable `lambda-rs` feature `physics-2d` to access
  `PhysicsWorld2D`.
- The update loop MUST step the simulation using a fixed timestep, independent
  of the render frame rate. Rationale: reduces variance across machines.
- The uniform structure in Rust MUST match the shader uniform block in size and
  alignment.
- The physics world MUST be constructed via `PhysicsWorld2DBuilder`.

## Data Flow <a name="data-flow"></a>

- CPU fixed ticks integrate `velocity_y` and `position_y`.
- CPU writes the quad’s Y offset into a uniform buffer.
- The vertex shader translates the quad by the uniform offset.

ASCII diagram

```
Frame time (variable)
  │
  ▼
Accumulator (seconds)
  │  while >= fixed_dt
  ▼
Fixed tick:
  ├─ PhysicsWorld2D::step()  (empty world)
  ├─ v += g * dt
  └─ y += v * dt
         │
         ▼
Uniform buffer write (offset.y)
         │
         ▼
Vertex shader translation
```

## Implementation Steps <a name="implementation-steps"></a>

### Step 1 — Create a Physics Demos Crate <a name="step-1"></a>

Create a new crate under `demos/physics/` and add it to the workspace. This
crate isolates physics-oriented demos and defaults to having physics enabled so
examples compile and run without extra feature flags.

Create `demos/physics/Cargo.toml`:

```toml
[package]
name = "lambda-demos-physics"
version = "0.1.0"
edition = "2021"
publish = false

[dependencies]
lambda-rs = { path = "../../crates/lambda-rs" }
lambda-rs-logging = { path = "../../crates/lambda-rs-logging" }

[features]
default = ["physics-2d"]
physics-2d = ["lambda-rs/physics-2d"]

[[bin]]
name = "physics_falling_quad"
path = "src/bin/physics_falling_quad.rs"
required-features = ["physics-2d"]
```

Add `demos/physics` to the workspace members in the root `Cargo.toml`.

After this step, `cargo build -p lambda-demos-physics` SHOULD succeed.

### Step 2 — Runtime and Component Skeleton <a name="step-2"></a>

Create `demos/physics/src/bin/physics_falling_quad.rs` with a minimal runtime
and a `Component` that stores both physics and render state. The component owns
an accumulator for fixed timestep stepping, the quad kinematics variables, and
placeholders for GPU resources that will be initialized in `on_attach`.

```rust
use lambda::{
  component::Component,
  events::{EventMask, WindowEvent},
  physics::{PhysicsWorld2D, PhysicsWorld2DBuilder},
  render::{
    buffer::Buffer,
    mesh::Mesh,
    shader::Shader,
    ResourceId,
  },
  runtime::start_runtime,
  runtimes::{
    application::ComponentResult,
    ApplicationRuntimeBuilder,
  },
};

pub struct FallingQuadDemo {
  physics_world: PhysicsWorld2D,
  physics_accumulator_seconds: f32,
  quad_position_y: f32,
  quad_velocity_y: f32,
  floor_y: f32,
  restitution: f32,

  vertex_shader: Option<Shader>,
  fragment_shader: Option<Shader>,
  mesh: Option<Mesh>,
  render_pipeline_id: Option<ResourceId>,
  render_pass_id: Option<ResourceId>,
  bind_group_id: Option<ResourceId>,
  uniform_buffer: Option<Buffer>,

  width: u32,
  height: u32,
}

impl Default for FallingQuadDemo {
  fn default() -> Self {
    let physics_world = PhysicsWorld2DBuilder::new()
      .with_gravity(0.0, -1.5)
      .build()
      .expect("Failed to create PhysicsWorld2D");

    return Self {
      physics_world,
      physics_accumulator_seconds: 0.0,
      quad_position_y: 0.8,
      quad_velocity_y: 0.0,
      floor_y: -0.8,
      restitution: 0.6,

      vertex_shader: None,
      fragment_shader: None,
      mesh: None,
      render_pipeline_id: None,
      render_pass_id: None,
      bind_group_id: None,
      uniform_buffer: None,

      width: 1200,
      height: 600,
    };
  }
}

impl Component<ComponentResult, String> for FallingQuadDemo {
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
    return Vec::new();
  }
}

fn main() {
  let runtime = ApplicationRuntimeBuilder::new("Physics: Falling Quad (Kinematic)")
    .with_renderer_configured_as(|render_context_builder| {
      return render_context_builder.with_render_timeout(1_000_000_000);
    })
    .with_window_configured_as(|window_builder| {
      return window_builder
        .with_dimensions(1200, 600)
        .with_name("Physics Falling Quad");
    })
    .with_component(|runtime, demo: FallingQuadDemo| {
      return (runtime, demo);
    })
    .build();

  start_runtime(runtime);
}
```

This step builds and starts the runtime. Rendering and motion are added in later
steps.

### Step 3 — Shaders and Uniform Contract <a name="step-3"></a>

Define shader stages that render a colored quad and apply a translation from a
uniform block at set 0, binding 0. The uniform contains a `vec4 offset` where
`offset.xy` shifts the quad in clip space.

```glsl
// Vertex (GLSL 450)
#version 450

layout (location = 0) in vec3 vertex_position;
layout (location = 1) in vec3 vertex_normal;
layout (location = 2) in vec3 vertex_color;

layout (location = 0) out vec3 frag_color;

layout (set = 0, binding = 0) uniform QuadGlobals {
  vec4 offset;
} globals;

void main() {
  vec2 translated = vertex_position.xy + globals.offset.xy;
  gl_Position = vec4(translated, vertex_position.z, 1.0);
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

Mirror the uniform block in Rust:

```rust
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct QuadGlobalsUniform {
  pub offset: [f32; 4],
}

unsafe impl lambda::pod::PlainOldData for QuadGlobalsUniform {}
```

Embed the shader sources as `&str` constants and build `Shader` objects. The
demo stores shader modules in component state so pipeline creation can borrow
them during `on_attach`.

```rust
use lambda::render::shader::{ShaderBuilder, ShaderKind, VirtualShader};

const VERTEX_SHADER_SOURCE: &str = r#"
#version 450

layout (location = 0) in vec3 vertex_position;
layout (location = 1) in vec3 vertex_normal;
layout (location = 2) in vec3 vertex_color;

layout (location = 0) out vec3 frag_color;

layout (set = 0, binding = 0) uniform QuadGlobals {
  vec4 offset;
} globals;

void main() {
  vec2 translated = vertex_position.xy + globals.offset.xy;
  gl_Position = vec4(translated, vertex_position.z, 1.0);
  frag_color = vertex_color;
}

"#;

const FRAGMENT_SHADER_SOURCE: &str = r#"
#version 450

layout (location = 0) in vec3 frag_color;
layout (location = 0) out vec4 fragment_color;

void main() {
  fragment_color = vec4(frag_color, 1.0);
}

"#;

fn build_shaders() -> (lambda::render::shader::Shader, lambda::render::shader::Shader) {
  let mut shader_builder = ShaderBuilder::new();

  let vertex_shader = shader_builder.build(VirtualShader::Source {
    source: VERTEX_SHADER_SOURCE.to_string(),
    kind: ShaderKind::Vertex,
    entry_point: "main".to_string(),
    name: "physics-falling-quad".to_string(),
  });

  let fragment_shader = shader_builder.build(VirtualShader::Source {
    source: FRAGMENT_SHADER_SOURCE.to_string(),
    kind: ShaderKind::Fragment,
    entry_point: "main".to_string(),
    name: "physics-falling-quad".to_string(),
  });

  return (vertex_shader, fragment_shader);
}
```

Update `Default` to initialize `vertex_shader` and `fragment_shader`:

```rust
let (vertex_shader, fragment_shader) = build_shaders();

return Self {
  // ...
  vertex_shader: Some(vertex_shader),
  fragment_shader: Some(fragment_shader),
  // ...
};
```

This establishes a single, per-draw uniform and a stable shader contract that
the pipeline will satisfy.

### Step 4 — Quad Mesh and Render Pipeline <a name="step-4"></a>

Build a quad mesh as two triangles and construct a render pipeline that binds
the uniform at set 0. The pipeline uses three vertex attributes (position,
normal, color) and disables culling to avoid winding issues during iteration.

```rust
use lambda::render::{
  bind::{BindGroupBuilder, BindGroupLayoutBuilder, BindingVisibility},
  buffer::{Buffer, BufferBuilder, Properties, Usage},
  command::RenderCommand,
  mesh::{Mesh, MeshBuilder},
  pipeline::{CullingMode, RenderPipelineBuilder},
  render_pass::RenderPassBuilder,
  vertex::{ColorFormat, VertexAttribute, VertexBuilder, VertexElement},
  viewport::ViewportBuilder,
  ResourceId,
};

fn on_attach(
  &mut self,
  render_context: &mut lambda::render::RenderContext,
) -> Result<ComponentResult, String> {
  let render_pass = RenderPassBuilder::new()
    .with_label("physics-falling-quad-pass")
    .build(
      render_context.gpu(),
      render_context.surface_format(),
      render_context.depth_format(),
    );

  let quad_half_size = 0.08_f32;
  let vertices = [
    VertexBuilder::new()
      .with_position([-quad_half_size, -quad_half_size, 0.0])
      .with_normal([0.0, 0.0, 1.0])
      .with_color([1.0, 0.3, 0.2])
      .build(),
    VertexBuilder::new()
      .with_position([quad_half_size, -quad_half_size, 0.0])
      .with_normal([0.0, 0.0, 1.0])
      .with_color([0.2, 1.0, 0.3])
      .build(),
    VertexBuilder::new()
      .with_position([quad_half_size, quad_half_size, 0.0])
      .with_normal([0.0, 0.0, 1.0])
      .with_color([0.2, 0.3, 1.0])
      .build(),
    VertexBuilder::new()
      .with_position([-quad_half_size, -quad_half_size, 0.0])
      .with_normal([0.0, 0.0, 1.0])
      .with_color([1.0, 0.3, 0.2])
      .build(),
    VertexBuilder::new()
      .with_position([quad_half_size, quad_half_size, 0.0])
      .with_normal([0.0, 0.0, 1.0])
      .with_color([0.2, 0.3, 1.0])
      .build(),
    VertexBuilder::new()
      .with_position([-quad_half_size, quad_half_size, 0.0])
      .with_normal([0.0, 0.0, 1.0])
      .with_color([1.0, 1.0, 0.2])
      .build(),
  ];

  let mut mesh_builder = MeshBuilder::new();
  vertices.iter().for_each(|vertex| {
    mesh_builder.with_vertex(*vertex);
  });

  let mesh = mesh_builder
    .with_attributes(vec![
      VertexAttribute {
        location: 0,
        offset: 0,
        element: VertexElement {
          format: ColorFormat::Rgb32Sfloat,
          offset: 0,
        },
      },
      VertexAttribute {
        location: 1,
        offset: 0,
        element: VertexElement {
          format: ColorFormat::Rgb32Sfloat,
          offset: 12,
        },
      },
      VertexAttribute {
        location: 2,
        offset: 0,
        element: VertexElement {
          format: ColorFormat::Rgb32Sfloat,
          offset: 24,
        },
      },
    ])
    .build();

  let layout = BindGroupLayoutBuilder::new()
    .with_uniform(0, BindingVisibility::Vertex)
    .build(render_context.gpu());

  let initial_uniform = QuadGlobalsUniform {
    offset: [0.0, self.quad_position_y, 0.0, 0.0],
  };

  let uniform_buffer = BufferBuilder::new()
    .with_length(std::mem::size_of::<QuadGlobalsUniform>())
    .with_usage(Usage::UNIFORM)
    .with_properties(Properties::CPU_VISIBLE)
    .with_label("quad-globals")
    .build(render_context.gpu(), vec![initial_uniform])
    .map_err(|error| error.to_string())?;

  let bind_group = BindGroupBuilder::new()
    .with_layout(&layout)
    .with_uniform(0, &uniform_buffer, 0, None)
    .build(render_context.gpu());

  let vertex_shader = self.vertex_shader.as_ref().expect("vertex shader missing");
  let fragment_shader = self.fragment_shader.as_ref().expect("fragment shader missing");

  let pipeline = RenderPipelineBuilder::new()
    .with_label("physics-falling-quad-pipeline")
    .with_culling(CullingMode::None)
    .with_layouts(&[&layout])
    .with_buffer(
      BufferBuilder::build_from_mesh(&mesh, render_context.gpu())
        .map_err(|error| error.to_string())?,
      mesh.attributes().to_vec(),
    )
    .build(
      render_context.gpu(),
      render_context.surface_format(),
      render_context.depth_format(),
      &render_pass,
      vertex_shader,
      Some(fragment_shader),
    );

  self.render_pass_id = Some(render_context.attach_render_pass(render_pass));
  self.render_pipeline_id = Some(render_context.attach_pipeline(pipeline));
  self.mesh = Some(mesh);
  self.uniform_buffer = Some(uniform_buffer);
  self.bind_group_id = Some(render_context.attach_bind_group(bind_group));

  return Ok(ComponentResult::Success);
}

fn on_render(
  &mut self,
  render_context: &mut lambda::render::RenderContext,
) -> Vec<RenderCommand> {
  let viewport = ViewportBuilder::new().build(self.width, self.height);

  let render_pass = self.render_pass_id.expect("render pass missing");
  let render_pipeline = self.render_pipeline_id.expect("render pipeline missing");
  let bind_group = self.bind_group_id.expect("bind group missing");

  return vec![
    RenderCommand::BeginRenderPass {
      render_pass,
      viewport: viewport.clone(),
    },
    RenderCommand::SetPipeline {
      pipeline: render_pipeline,
    },
    RenderCommand::SetViewports {
      start_at: 0,
      viewports: vec![viewport.clone()],
    },
    RenderCommand::SetScissors {
      start_at: 0,
      viewports: vec![viewport.clone()],
    },
    RenderCommand::SetBindGroup {
      set: 0,
      group: bind_group,
      dynamic_offsets: Vec::new(),
    },
    RenderCommand::BindVertexBuffer {
      pipeline: render_pipeline,
      buffer: 0,
    },
    RenderCommand::Draw {
      vertices: 0..self.mesh.as_ref().unwrap().vertices().len() as u32,
      instances: 0..1,
    },
    RenderCommand::EndRenderPass,
  ];
}
```

After this step, `on_attach` constructs and attaches all GPU resources required
to draw a static quad.

### Step 5 — Fixed-Timestep Update and Kinematics <a name="step-5"></a>

Implement a fixed-timestep accumulator in `on_update` and integrate motion with
gravity. The fixed tick reads gravity and timestep from `PhysicsWorld2D` and
steps the physics world even though it is empty. The quad integration remains a
separate, explicit kinematic path consistent with the current physics scope.

```rust
fn on_update(
  &mut self,
  last_frame: &std::time::Duration,
) -> Result<ComponentResult, String> {
  self.physics_accumulator_seconds += last_frame.as_secs_f32();

  let timestep_seconds = self.physics_world.timestep_seconds();
  let gravity = self.physics_world.gravity();

  while self.physics_accumulator_seconds >= timestep_seconds {
    self.physics_world.step();

    self.quad_velocity_y += gravity[1] * timestep_seconds;
    self.quad_position_y += self.quad_velocity_y * timestep_seconds;

    if self.quad_position_y < self.floor_y {
      self.quad_position_y = self.floor_y;
      self.quad_velocity_y = -self.quad_velocity_y * self.restitution;
    }

    self.physics_accumulator_seconds -= timestep_seconds;
  }

  return Ok(ComponentResult::Success);
}
```

This step produces deterministic falling motion independent of render frame
rate. Rendering the updated position requires writing `quad_position_y` into the
uniform buffer as described in Step 6.

### Step 6 — Uniform Writes and Draw Commands <a name="step-6"></a>

Write the updated quad offset into the uniform buffer each frame. This connects
the fixed tick motion from Step 5 to the vertex shader translation.

Insert the following block near the top of `on_render` (after viewport creation
and before the command list is returned):

```rust
if let Some(ref uniform_buffer) = self.uniform_buffer {
  let value = QuadGlobalsUniform {
    offset: [0.0, self.quad_position_y, 0.0, 0.0],
  };
  uniform_buffer.write_value(render_context.gpu(), 0, &value);
}
```

After this step, the quad’s motion is visible because the vertex shader reads
the updated offset.

### Step 7 — Resize Handling <a name="step-7"></a>

Ensure `WindowEvent::Resize` updates `width` and `height` so the viewport and
scissor match the current surface size.

After this step, the quad remains visible and correctly scaled through window
resizes.

## Validation <a name="validation"></a>

- Build: `cargo build --workspace`
- Run: `cargo run -p lambda-demos-physics --bin physics_falling_quad`
- Expected behavior: a window opens and a small colored quad falls from the top
  of the screen and bounces on a floor.

## Notes <a name="notes"></a>

- Coordinate system: The demo treats clip-space Y as “up” and integrates a
  position directly in clip space. Gravity is set to a smaller magnitude than
  real-world meters-per-second-squared to keep motion readable.
- Backend: `PhysicsWorld2D` is enabled by `lambda-rs` feature `physics-2d`, which
  wires a platform backend (Rapier 0.32.0 in the current implementation).
- Empty world stepping: `PhysicsWorld2D::step()` advances the backend pipeline
  even with zero bodies and colliders. This validates fixed timestep and
  sub-step behavior without exposing rigid body APIs.
- Fixed timestep: The accumulator pattern SHOULD clamp extremely large frame
  deltas in production to avoid long catch-up loops.

## Conclusion <a name="conclusion"></a>

This tutorial created a physics-oriented demo crate and implemented a falling
quad demo that uses a fixed-timestep accumulator loop. The implementation
constructed a `PhysicsWorld2D` via its builder, stepped the physics world each
tick, integrated kinematic motion using configured gravity and timestep values,
and rendered motion by writing a uniform offset consumed by a vertex shader.

## Exercises <a name="exercises"></a>

- Exercise 1: Render a visible floor
  - Add a second, static quad or thin rectangle at `floor_y` and keep it fixed.
- Exercise 2: Visualize sub-steps
  - Add a second quad that uses a different sub-step count and compare motion.
- Exercise 3: Add horizontal drift
  - Integrate an X velocity and apply both components in the uniform offset.
- Exercise 4: Determinism check
  - Run the demo at different frame rates and verify the fixed tick path
    produces consistent motion.
- Exercise 5: Multiple quads
  - Render a grid of quads with different initial heights and velocities.
- Exercise 6: Accumulator clamping
  - Add a maximum catch-up time per frame and document the behavior trade-off.
- Exercise 7: Migration path
  - Replace the kinematic integration with rigid bodies once a body API exists.

## Changelog <a name="changelog"></a>

- 0.1.0 (2026-02-07): Initial draft aligned with
  `demos/physics/src/bin/physics_falling_quad.rs`.
