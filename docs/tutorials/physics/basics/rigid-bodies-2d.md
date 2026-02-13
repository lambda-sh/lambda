---
title: "Physics 2D: Rigid Bodies (No Collisions)"
document_id: "physics-rigid-bodies-2d-no-collisions-2026-02-13"
status: "draft"
created: "2026-02-13T00:00:00Z"
last_updated: "2026-02-13T00:00:00Z"
version: "0.1.0"
engine_workspace_version: "2023.1.30"
wgpu_version: "28.0.0"
shader_backend_default: "naga"
winit_version: "0.29.10"
repo_commit: "6a3b507eedddc39f568ed73cfadf34011d57b9a3"
owners: ["lambda-sh"]
reviewers: ["engine", "rendering"]
tags: ["tutorial", "physics", "2d", "rigid-bodies", "fixed-timestep", "uniform-buffers"]
---

## Overview <a name="overview"></a>

This tutorial builds a render demo that showcases 2D rigid bodies in
`PhysicsWorld2D`. The simulation does not define collision shapes, so bodies do
not collide. Instead, simple boundary rules clamp and bounce bodies within the
viewport to keep the demo visible.

Reference implementation: `demos/physics/src/bin/physics_rigid_bodies_2d.rs`.

## Table of Contents

- [Overview](#overview)
- [Goals](#goals)
- [Prerequisites](#prerequisites)
- [Requirements and Constraints](#requirements-and-constraints)
- [Data Flow](#data-flow)
- [Implementation Steps](#implementation-steps)
  - [Step 1 — Add a Demo Binary Entry](#step-1)
  - [Step 2 — Define Shader and Uniform Contract](#step-2)
  - [Step 3 — Define Component State](#step-3)
  - [Step 4 — Create the Physics World and Bodies](#step-4)
  - [Step 5 — Build GPU Resources and Per-Body Uniform Buffers](#step-5)
  - [Step 6 — Implement Fixed-Timestep Stepping and Controls](#step-6)
  - [Step 7 — Write Uniforms and Render Bodies](#step-7)
- [Validation](#validation)
- [Notes](#notes)
- [Conclusion](#conclusion)
- [Exercises](#exercises)
- [Changelog](#changelog)

## Goals <a name="goals"></a>

- Create three rigid body types:
  - Dynamic bodies (gravity, forces, impulses).
  - Kinematic body (user-provided velocity and rotation).
  - Static body (fixed reference).
- Step physics with a fixed timestep accumulator.
- Apply forces and impulses to dynamic bodies.
- Query position and rotation each frame and render via uniform buffers.

## Prerequisites <a name="prerequisites"></a>

- The workspace builds: `cargo build --workspace`.
- The physics demo crate builds: `cargo build -p lambda-demos-physics`.

## Requirements and Constraints <a name="requirements-and-constraints"></a>

- The demo MUST enable `lambda-rs` feature `physics-2d`.
- The update loop MUST step the simulation using a fixed timestep accumulator.
  Rationale: reduces variance across machines.
- The demo MUST NOT rely on collision shapes, collision detection, or collision
  response. Boundary behavior MUST be implemented in user code.
- Uniform structs in Rust MUST match shader uniform blocks in size and
  alignment.

## Data Flow <a name="data-flow"></a>

- Fixed ticks apply:
  - A constant wind force (dynamic bodies).
  - A pending impulse on input (dynamic bodies).
  - A kinematic rotation update and kinematic velocity (kinematic body).
  - One `PhysicsWorld2D::step()`.
  - Manual boundary bounce and clamp logic (dynamic and kinematic bodies).
- Each render frame queries rigid body transforms and writes them to per-body
  uniform buffers used by the vertex shader.

ASCII diagram

```
Variable frame time
  │
  ▼
Accumulator (seconds)
  │  while >= fixed_dt
  ▼
Fixed tick:
  ├─ apply_force / apply_impulse (dynamic)
  ├─ set_rotation (kinematic)
  ├─ PhysicsWorld2D::step()
  └─ boundary clamp + bounce (user code)
         │
         ▼
Per-frame:
  ├─ query body position/rotation
  ├─ write per-body uniforms
  └─ draw the same quad mesh per body
```

## Implementation Steps <a name="implementation-steps"></a>

### Step 1 — Add a Demo Binary Entry <a name="step-1"></a>

Add a new binary to the physics demos crate.

Update `demos/physics/Cargo.toml`:

```toml
[[bin]]
name = "physics_rigid_bodies_2d"
path = "src/bin/physics_rigid_bodies_2d.rs"
required-features = ["physics-2d"]
```

After this step, `cargo build -p lambda-demos-physics` SHOULD still succeed.

### Step 2 — Define Shader and Uniform Contract <a name="step-2"></a>

Define a vertex shader that:

- Rotates a quad in 2D using a uniform rotation in radians.
- Translates the quad by a uniform `(x, y)` offset.
- Applies a per-body tint color for readability.

Define a matching Rust uniform:

```rust
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct QuadGlobalsUniform {
  pub offset_rotation: [f32; 4],
  pub tint: [f32; 4],
}
```

After this step, the shader and uniform contract represent a complete per-body
transform and color payload.

### Step 3 — Define Component State <a name="step-3"></a>

Define a component that stores:

- A `PhysicsWorld2D`.
- A fixed timestep accumulator.
- `RigidBody2D` handles for each body in the demo.
- Basic input state (for an impulse trigger).
- GPU resources: shaders, mesh, pipeline, render pass.
- One uniform buffer and bind group per rigid body.

After this step, the demo has a concrete place to store both simulation state
and render state.

### Step 4 — Create the Physics World and Bodies <a name="step-4"></a>

Construct the physics world using `PhysicsWorld2DBuilder`, then create four
bodies:

- Two dynamic bodies with different masses.
- One kinematic body with an initial velocity.
- One static body as a fixed reference.

Example:

```rust
let mut physics_world = PhysicsWorld2DBuilder::new()
  .with_gravity(0.0, -1.6)
  .build()
  .expect("Failed to create PhysicsWorld2D");

let dynamic_light_body = RigidBody2DBuilder::new(RigidBodyType::Dynamic)
  .with_position(-0.35, 0.75)
  .with_dynamic_mass_kg(0.5)
  .build(&mut physics_world)
  .expect("Failed to create dynamic body (light)");
```

After this step, the demo has simulation objects whose state can be queried and
stepped.

### Step 5 — Build GPU Resources and Per-Body Uniform Buffers <a name="step-5"></a>

In `on_attach`:

- Build a quad mesh.
- Build a uniform bind group layout.
- For each rigid body:
  - Query initial position and rotation.
  - Create a CPU-visible uniform buffer with `QuadGlobalsUniform`.
  - Create a bind group for that buffer.
- Build a render pipeline using the shared layout and shared mesh buffer.

After this step, each body has a uniform buffer and bind group that can be
updated independently while sharing a single mesh and pipeline.

### Step 6 — Implement Fixed-Timestep Stepping and Controls <a name="step-6"></a>

Implement:

- Keyboard handling that sets an impulse flag when Space is pressed.
- A fixed timestep loop in `on_update`:
  - Apply a constant wind force to dynamic bodies each tick.
  - Apply an upward impulse when the impulse flag is set.
  - Increment and set a kinematic rotation value each tick.
  - Call `PhysicsWorld2D::step()`.
  - Apply manual boundary bounce and clamp logic:
    - Floor and ceiling bounce for dynamic bodies.
    - Left and right wall bounce for dynamic bodies.
    - Left and right wall clamp (and velocity flip) for the kinematic body.

After this step, the demo shows steady movement regardless of frame rate, and
input can inject instantaneous velocity changes into the dynamic bodies.

### Step 7 — Write Uniforms and Render Bodies <a name="step-7"></a>

In `on_render`:

- For each body:
  - Query position and rotation from the physics world.
  - Write `QuadGlobalsUniform` to the corresponding uniform buffer.
- Record draw commands:
  - Begin a render pass, set the pipeline, set viewport/scissor, bind the shared
    vertex buffer.
  - For each body, set its bind group and draw the quad.

After this step, the rendered quads track the simulated bodies each frame.

## Validation <a name="validation"></a>

Build and run:

```bash
cargo run -p lambda-demos-physics --bin physics_rigid_bodies_2d
```

Expected behavior:

- Two dynamic bodies fall under gravity and drift from a constant wind force.
- Dynamic bodies bounce off the floor, ceiling, and side walls.
- The kinematic body moves horizontally, rotates continuously, and clamps at
  the walls.
- The static body remains fixed.
- Pressing Space applies an upward impulse to both dynamic bodies.

## Notes <a name="notes"></a>

- This demo intentionally does not use collision shapes. Any “bouncing” behavior
  is implemented by clamping positions and mutating velocities in user code.
- `apply_force` and `apply_impulse` are intended for dynamic bodies. Calls on
  static or kinematic bodies SHOULD return an error.
- Fixed timestep stepping is implemented with an accumulator. The demo MUST NOT
  advance simulation directly from variable frame deltas.
- The rotation shown in the demo is explicitly set each tick. Angular dynamics
  are not required for this tutorial.

## Conclusion <a name="conclusion"></a>

This tutorial demonstrates how to create and step 2D rigid bodies using
`PhysicsWorld2D` and render them by writing per-body uniform buffers. The demo
uses dynamic, kinematic, and static bodies and applies forces and impulses to
validate basic rigid body integration without relying on collision shapes.

## Exercises <a name="exercises"></a>

- Add a configurable drag force that reduces dynamic body velocity over time.
- Replace constant wind with a time-varying sinusoidal force.
- Add a toggle key that enables or disables gravity at runtime.
- Render a simple velocity indicator (for example, a line in the direction of
  velocity) using additional geometry.
- Spawn N dynamic bodies at startup and randomize initial positions and masses.
- Apply a force proportional to body mass and observe acceleration differences.
- Add a soft “camera” offset uniform and pan the view with arrow keys.

## Changelog <a name="changelog"></a>

- 0.1.0 (2026-02-13): Initial tutorial for `physics_rigid_bodies_2d`.
