---
title: "Physics 2D: Collision Events"
document_id: "physics-collision-events-2d-2026-04-01"
status: "draft"
created: "2026-04-01T00:00:00Z"
last_updated: "2026-04-01T00:00:00Z"
version: "0.2.0"
engine_workspace_version: "2023.1.30"
wgpu_version: "28.0.0"
shader_backend_default: "naga"
winit_version: "0.29.10"
repo_commit: "7273183d923e78273b77b7f924bc8d6abc734cb9"
owners: ["lambda-sh"]
reviewers: ["engine", "rendering"]
tags: ["tutorial", "physics", "2d", "collision-events", "fixed-timestep"]
---

## Overview <a name="overview"></a>

This tutorial builds the `physics_collision_events_2d` demo that now exists in
`demos/physics/src/bin/physics_collision_events_2d.rs`. The finished example
creates a static floor and a dynamic ball, advances a 2D physics world on a
fixed timestep, drains `PhysicsWorld2D::collision_events()` after each step,
and changes the ball tint while contact with the floor is active.

The tutorial focuses on the gameplay-facing side of the API. The rendering
path stays intentionally small and only exists to make the collision state
visible without adding UI or text rendering.

## Table of Contents

- [Overview](#overview)
- [Goals](#goals)
- [Prerequisites](#prerequisites)
- [Implementation Steps](#implementation-steps)
  - [Step 1 — Register the Demo Binary](#step-1)
  - [Step 2 — Define Constants, Shaders, and Uniforms](#step-2)
  - [Step 3 — Add Render and Gameplay State](#step-3)
  - [Step 4 — Add Geometry and Event Helpers](#step-4)
  - [Step 5 — Build the Default Physics Scene](#step-5)
  - [Step 6 — Attach Resources and Process Events](#step-6)
  - [Step 7 — Render the Bodies and Start the Runtime](#step-7)
- [Validation](#validation)
- [Notes](#notes)
- [Conclusion](#conclusion)
- [Exercises](#exercises)
- [Changelog](#changelog)

## Goals <a name="goals"></a>

- Build a dedicated 2D collision-events demo binary.
- Show a fixed-timestep update loop that drains collision events immediately
  after `PhysicsWorld2D::step()`.
- Demonstrate how to track contact state for one body pair without inferring it
  from transforms.
- Log representative `Started` contact data and handle `Ended` without contact
  payloads.
- Make the state transition visible by tinting the ball while floor contact is
  active.

## Prerequisites <a name="prerequisites"></a>

- The workspace builds with `cargo build --workspace`.
- The demos crate is available as `lambda-demos-physics`.
- The `physics-2d` feature is enabled for the physics demos crate.
- You are comfortable reading a `Component` implementation and a small amount
  of render setup code.

## Implementation Steps <a name="implementation-steps"></a>

### Step 1 — Register the Demo Binary <a name="step-1"></a>

Add a new binary entry to `demos/physics/Cargo.toml`. Keeping collision events
in a dedicated binary prevents the broader collider demo from becoming a second
physics tutorial with competing goals.

```toml
[[bin]]
name = "physics_collision_events_2d"
path = "src/bin/physics_collision_events_2d.rs"
required-features = ["physics-2d"]
```

This step gives Cargo a focused entry point for the tutorial. From this point
on, you can build and run the example independently from the other physics
demos.

### Step 2 — Define Constants, Shaders, and Uniforms <a name="step-2"></a>

Create `demos/physics/src/bin/physics_collision_events_2d.rs` and start with
the constants that define the scene and the small shader pair used to draw it.
The demo only needs a floor, a ball, and one uniform block with translation,
rotation, and tint.

```rust
const WINDOW_WIDTH: u32 = 1200;
const WINDOW_HEIGHT: u32 = 600;

const FLOOR_HALF_WIDTH: f32 = 0.88;
const FLOOR_HALF_HEIGHT: f32 = 0.05;
const FLOOR_Y: f32 = -0.82;

const BALL_RADIUS: f32 = 0.08;
const BALL_START_Y: f32 = 0.42;
const BALL_LAUNCH_IMPULSE_Y: f32 = 1.45;
```

Use a uniform type that matches the shader contract:

```rust
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct ContactDemoUniform {
  offset_rotation: [f32; 4],
  tint: [f32; 4],
}

unsafe impl lambda::pod::PlainOldData for ContactDemoUniform {}
```

The vertex shader should rotate and translate the mesh in clip space, and the
fragment shader should output the tinted color. The real demo uses inline GLSL
strings named `VERTEX_SHADER_SOURCE` and `FRAGMENT_SHADER_SOURCE`.

After this step, the file has the immutable scene dimensions and the minimal
GPU contract needed to draw collision state.

### Step 3 — Add Render and Gameplay State <a name="step-3"></a>

Define the render record for each body and the main component state. The key
idea is to keep collision-derived state explicit. The tutorial is about
responding to event transitions, so `ball_contact_active` should live alongside
the physics handles rather than being recomputed from positions.

```rust
struct RenderBody {
  body: RigidBody2D,
  vertices: Range<u32>,
  tint_idle: [f32; 4],
  tint_contact: [f32; 4],
  highlights_contact: bool,
  uniform_buffer: Buffer,
  bind_group_id: ResourceId,
}

pub struct CollisionEvents2DDemo {
  physics_world: PhysicsWorld2D,
  physics_accumulator_seconds: f32,
  pending_launch_impulse: bool,

  ball_body: RigidBody2D,
  floor_body: RigidBody2D,
  ball_contact_active: bool,

  vertex_shader: Shader,
  fragment_shader: Shader,
  mesh: Option<Mesh>,
  render_pipeline_id: Option<ResourceId>,
  render_pass_id: Option<ResourceId>,
  bodies: Vec<RenderBody>,

  width: u32,
  height: u32,
}
```

This step separates the three responsibilities in the demo:
- `physics_world`, body handles, and impulse state drive simulation.
- `ball_contact_active` stores gameplay state derived from events.
- `bodies`, shaders, and pipeline handles support rendering.

### Step 4 — Add Geometry and Event Helpers <a name="step-4"></a>

Add small helpers for mesh construction and collision-event handling. The demo
uses one combined mesh, so helper functions keep the geometry code small and
make the tutorial easier to follow.

Build triangles with these helpers:

```rust
fn push_vertex(mesh_builder: MeshBuilder, x: f32, y: f32) -> MeshBuilder {
  return mesh_builder.with_vertex(
    VertexBuilder::new()
      .with_position([x, y, 0.0])
      .with_normal([0.0, 0.0, 1.0])
      .with_color([1.0, 1.0, 1.0])
      .build(),
  );
}

fn append_rectangle(
  mesh_builder: MeshBuilder,
  vertex_count: &mut u32,
  half_width: f32,
  half_height: f32,
) -> (MeshBuilder, Range<u32>) { /* ... */ }

fn append_circle(
  mesh_builder: MeshBuilder,
  vertex_count: &mut u32,
  radius: f32,
  segments: u32,
) -> (MeshBuilder, Range<u32>) { /* ... */ }
```

Then add the event-specific helpers:

```rust
fn is_ball_floor_event(&self, event: CollisionEvent) -> bool {
  let is_direct_pair =
    event.body_a == self.ball_body && event.body_b == self.floor_body;
  let is_swapped_pair =
    event.body_a == self.floor_body && event.body_b == self.ball_body;

  return is_direct_pair || is_swapped_pair;
}

fn log_ball_floor_event(&mut self, event: CollisionEvent) {
  match event.kind {
    CollisionEventKind::Started => {
      self.ball_contact_active = true;
      // Print contact data when present.
    }
    CollisionEventKind::Ended => {
      self.ball_contact_active = false;
      println!("Collision Ended: ball left the floor");
    }
  }
}
```

The real implementation prints either a fully formatted `Started` message with
contact point, normal, and penetration, or a fallback line when the contact
payload is unavailable.

After this step, the file has the local helpers that make the rest of the
component implementation short and readable.

### Step 5 — Build the Default Physics Scene <a name="step-5"></a>

Implement `Default` for the component. This keeps `main()` small and makes the
physics setup explicit in one place. The scene uses one static floor and one
dynamic ball because a single body pair produces the cleanest event stream.

```rust
impl Default for CollisionEvents2DDemo {
  fn default() -> Self {
    let mut physics_world = PhysicsWorld2DBuilder::new()
      .with_gravity(0.0, -3.2)
      .with_substeps(4)
      .build()
      .expect("Failed to create PhysicsWorld2D");

    let floor_body = RigidBody2DBuilder::new(RigidBodyType::Static)
      .with_position(0.0, FLOOR_Y)
      .build(&mut physics_world)
      .expect("Failed to create floor body");

    Collider2DBuilder::rectangle(FLOOR_HALF_WIDTH, FLOOR_HALF_HEIGHT)
      .with_density(0.0)
      .with_friction(0.8)
      .with_restitution(0.0)
      .build(&mut physics_world, floor_body)
      .expect("Failed to create floor collider");

    let ball_body = RigidBody2DBuilder::new(RigidBodyType::Dynamic)
      .with_position(0.0, BALL_START_Y)
      .build(&mut physics_world)
      .expect("Failed to create ball body");

    Collider2DBuilder::circle(BALL_RADIUS)
      .with_density(100.0)
      .with_friction(0.45)
      .with_restitution(0.0)
      .build(&mut physics_world, ball_body)
      .expect("Failed to create ball collider");

    // Build the inline GLSL shaders here as well.
  }
}
```

The demo also constructs the vertex and fragment shaders in `default()`, then
initializes the remaining render fields to `None` or empty collections.

After this step, the simulation is reproducible before any rendering code runs.
The ball starts above the floor, settles into contact, and is ready to produce
its first `Started` event.

### Step 6 — Attach Resources and Process Events <a name="step-6"></a>

Implement the component lifecycle and fixed-update loop. `on_attach()` should
build the render pass, bind group layout, combined mesh, per-body uniform
buffers, and render pipeline. The implementation uses one mesh for both bodies
and stores the floor and ball vertex ranges separately.

Create the two render entries like this:

```rust
let render_bodies = [
  (
    self.floor_body,
    floor_vertices,
    [0.22, 0.22, 0.24, 1.0],
    [0.22, 0.22, 0.24, 1.0],
    false,
  ),
  (
    self.ball_body,
    ball_vertices,
    [0.22, 0.55, 0.95, 1.0],
    [0.95, 0.28, 0.22, 1.0],
    true,
  ),
];
```

Handle keyboard input in `on_keyboard_event()` and only arm the launch when
the ball is already touching the floor:

```rust
fn on_keyboard_event(&mut self, event: &Key) -> Result<(), String> {
  let Key::Pressed { virtual_key, .. } = event else {
    return Ok(());
  };

  if virtual_key != &Some(VirtualKey::Space) {
    return Ok(());
  }

  if !self.ball_contact_active {
    println!("Space ignored: wait until the ball is resting on the floor");
    return Ok(());
  }

  self.pending_launch_impulse = true;
  return Ok(());
}
```

Drive physics and collision events from `on_update()`:

```rust
fn on_update(
  &mut self,
  last_frame: &std::time::Duration,
) -> Result<ComponentResult, String> {
  self.physics_accumulator_seconds += last_frame.as_secs_f32();

  let timestep_seconds = self.physics_world.timestep_seconds();

  while self.physics_accumulator_seconds >= timestep_seconds {
    if self.pending_launch_impulse {
      self
        .ball_body
        .set_velocity(&mut self.physics_world, 0.0, 0.0)
        .map_err(|error| error.to_string())?;
      self
        .ball_body
        .apply_impulse(
          &mut self.physics_world,
          0.0,
          BALL_LAUNCH_IMPULSE_Y,
        )
        .map_err(|error| error.to_string())?;
      self.pending_launch_impulse = false;
      println!("Launch impulse applied");
    }

    self.physics_world.step();

    for event in self.physics_world.collision_events() {
      if self.is_ball_floor_event(event) {
        self.log_ball_floor_event(event);
      }
    }

    self.physics_accumulator_seconds -= timestep_seconds;
  }

  return Ok(ComponentResult::Success);
}
```

Resetting the ball velocity before applying the launch impulse keeps the
separation readable. Without that reset, the jump height depends on the exact
velocity the solver produced while the ball was settling.

After this step, the demo has its core behavior. The ball falls, emits a
single `Started` event when contact begins, ignores `Space` until grounded,
and emits `Ended` when the launch separates the pair.

### Step 7 — Render the Bodies and Start the Runtime <a name="step-7"></a>

Implement `on_render()` so each body reads its current transform from
`PhysicsWorld2D`, writes a fresh `ContactDemoUniform`, and draws its vertex
range. The ball is the only body that switches tint when
`ball_contact_active` is true.

```rust
let tint = if body.highlights_contact && self.ball_contact_active {
  body.tint_contact
} else {
  body.tint_idle
};

let uniform = ContactDemoUniform {
  offset_rotation: [position[0], position[1], rotation, 0.0],
  tint,
};

body
  .uniform_buffer
  .write_value(render_context.gpu(), 0, &uniform);
```

Finish the file with a small `main()` that creates the runtime and window:

```rust
fn main() {
  let runtime = ApplicationRuntimeBuilder::new(
    "Physics: 2D Collision Events",
  )
  .with_window_configured_as(move |window_builder| {
    return window_builder
      .with_dimensions(WINDOW_WIDTH, WINDOW_HEIGHT)
      .with_name("Physics: 2D Collision Events");
  })
  .with_component(move |runtime, demo: CollisionEvents2DDemo| {
    return (runtime, demo);
  })
  .build();

  start_runtime(runtime);
}
```

After this step, the tutorial matches the checked-in demo. You can build and
run the binary and observe the floor-ball collision events in the terminal and
on screen.

## Validation <a name="validation"></a>

Build the demo:

```bash
cargo build -p lambda-demos-physics --bin physics_collision_events_2d
```

Run the demo:

```bash
cargo run -p lambda-demos-physics --bin physics_collision_events_2d
```

Expected behavior:

- The terminal prints the controls hint when the component attaches.
- The ball falls onto the floor under gravity and eventually turns orange-red.
- The first contact prints `Collision Started:` with point, normal, and
  penetration values when the backend provides them.
- Pressing `Space` before the ball settles prints
  `Space ignored: wait until the ball is resting on the floor`.
- Pressing `Space` after the ball settles prints `Launch impulse applied`.
- When the ball leaves the floor, the terminal prints
  `Collision Ended: ball left the floor`.
- When the ball lands again, a new `Collision Started:` line appears rather
  than one line every frame.

## Notes <a name="notes"></a>

- The demo MUST drain `collision_events()` inside the fixed-update loop after
  `PhysicsWorld2D::step()`. Draining later makes the event timing harder to
  reason about.
- `CollisionEventKind::Started` SHOULD be treated as the place where contact
  data is available. The demo prints a fallback message if the payload is not
  present.
- `CollisionEventKind::Ended` MUST be handled without assuming a contact
  point, normal, or penetration value exists.
- The tutorial SHOULD keep the scene to one body pair. That makes event
  transitions easy to inspect while validating the API.
- The render path MAY stay minimal. The purpose of this demo is to show how
  gameplay code reacts to collision events, not to demonstrate advanced
  rendering patterns.

## Conclusion <a name="conclusion"></a>

You now have a complete collision-events reference demo that matches the code
checked into the repository. The example shows the intended pattern for
gameplay integration: use a fixed timestep, step the world, drain transition
events immediately, and derive simple game state from those transitions.

Because the scene stays small, the tutorial also serves as a clean starting
point for later experiments with multiple bodies, collision filters, and query
APIs.

## Exercises <a name="exercises"></a>

- Add a second ball and maintain separate contact state for each ball-floor
  pair.
- Add a wall collider and print separate messages for ball-wall contact.
- Replace the terminal logging with a small on-screen event history.
- Add a second collider to the ball body and confirm the demo still reacts to
  one body-pair event stream.
- Change the launch impulse based on how long the ball has been grounded.
- Extend the demo with a point query that highlights the ball when the mouse is
  over it.

## Changelog <a name="changelog"></a>

- 0.2.0 (2026-04-01): Rewrite the tutorial to match the implemented
  `physics_collision_events_2d` demo and document the real build sequence.
- 0.1.0 (2026-04-01): Initial tutorial for `physics_collision_events_2d`.
