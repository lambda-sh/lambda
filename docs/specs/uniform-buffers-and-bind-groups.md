---
title: "Uniform Buffers and Bind Groups"
document_id: "ubo-spec-2025-10-11"
status: "living"
created: "2025-10-11T00:00:00Z"
last_updated: "2026-02-05T23:05:40Z"
version: "0.5.1"
engine_workspace_version: "2023.1.30"
wgpu_version: "26.0.1"
shader_backend_default: "naga"
winit_version: "0.29.10"
repo_commit: "544444652b4dc3639f8b3e297e56c302183a7a0b"
owners: ["lambda-sh"]
reviewers: ["engine", "rendering"]
tags: ["spec", "rendering", "uniforms", "bind-groups", "wgpu"]
---

# Uniform Buffers and Bind Groups

Summary

- Specifies uniform buffer objects (UBOs) and bind groups for the
  wgpu‑backed renderer, preserving builder/command patterns and the separation
  between platform and high‑level layers.
- Rationale: Enables structured constants (for example, cameras, materials,
  per‑frame data) beyond push constants and supports dynamic offsets for
  batching many small records efficiently.

## Scope

### Goals

- Add first-class uniform buffers and bind groups.
- Maintain builder ergonomics consistent with buffers, pipelines, and passes.
- Integrate with the existing render command stream (inside a pass).
- Provide a portable, WGSL/GLSL-friendly layout model and validation.
- Expose dynamic uniform offsets (opt-in) with correct alignment handling.

### Non-Goals

- Storage buffers, textures/samplers, and compute are referenced but not
  implemented here; separate specs cover them.
- Descriptor set caching beyond wgpu’s internal caches.

## Terminology

- Uniform buffer object (UBO): Read‑only constant buffer accessed by shaders as
  `var<uniform>`.
- Bind group: A collection of bound resources used together by a pipeline.
- Bind group layout: The declared interface (bindings, types, visibility) for a
  bind group.
- Dynamic offset: A per‑draw offset applied to a uniform binding to select a
  different slice within a larger buffer.
- Visibility: Shader stage visibility for a binding (vertex, fragment, compute).

## Architecture Overview

- Platform (`lambda-rs-platform`)
  - Wrappers around `wgpu::BindGroupLayout` and `wgpu::BindGroup` with builder
    types that produce `wgpu` descriptors and perform validation against device
    limits.
  - The platform layer owns the raw `wgpu` handles and exposes them to the
    high-level layer as needed.

- High level (`lambda-rs`)
  - Public builders and types for bind group layouts and bind groups, aligned
    with existing `RenderPipelineBuilder` and `BufferBuilder` patterns.
  - `RenderPipelineBuilder` accepts bind group layouts and constructs a
    `wgpu::PipelineLayout` during build.
  - `RenderCommand` includes `SetBindGroup` to bind resources during a pass.
  - The public application programming interface avoids exposing `wgpu` types.
    Numeric limits and high-level wrappers are surfaced; raw handles live in the
    platform layer.

Data flow (one-time setup → per-frame):

```
BindGroupLayoutBuilder --> BindGroupLayout --+--> RenderPipelineBuilder (layouts)
                                             |
BufferBuilder (Usage::UNIFORM) --------------+--> BindGroupBuilder (uniform binding)

Per-frame commands: BeginRenderPass -> SetPipeline -> SetBindGroup -> Draw -> End
```

## Design

### API Surface

- Platform layer (`lambda-rs-platform`, module `lambda_platform::wgpu::bind`)
  - Types: `BindGroupLayout`, `BindGroup`, and `Visibility` (maps to
    `wgpu::ShaderStages`).
  - Builders: `BindGroupLayoutBuilder` and `BindGroupBuilder` for declaring
    uniform bindings (static and dynamic), setting labels, and creating
    resources.
- High-level layer (`lambda-rs`, module `lambda::render::bind`)
  - Types: high-level `BindGroupLayout` and `BindGroup` wrappers, and
    `BindingVisibility` enumeration.
  - Builders: mirror the platform builders; integrate with `RenderContext`.
- Pipeline integration: `RenderPipelineBuilder::with_layouts(&[&BindGroupLayout])`
  stores layouts and constructs a `wgpu::PipelineLayout` during `build`.
- Render commands: `RenderCommand::SetBindGroup { set, group, dynamic_offsets }`
  encodes `wgpu::RenderPass::set_bind_group` via `RenderContext`.
- Buffers: Uniform buffers MUST be created with `Usage::UNIFORM`. For frequently
  updated data, pair with CPU-visible properties. A typed `UniformBuffer<T>`
  provides `new(&mut rc, &T, label)`, `write(&rc, &T)`, and exposes `raw()`.

### Behavior

- Bind group layouts declare uniform bindings and their stage visibility. Layout
  indices correspond to set numbers; binding indices map one-to-one to shader
  `@binding(N)` declarations.
- Bind groups bind a buffer (with optional size slice) to a binding declared in
  the layout. When a binding is dynamic, the actual offset is supplied at draw
  time using `dynamic_offsets`.
- Pipelines reference one or more bind group layouts; all render passes that use
  that pipeline MUST supply compatible bind groups at the expected sets.

### Validation and Errors

- Uniform binding ranges MUST NOT exceed
  `limits.max_uniform_buffer_binding_size`.
- Dynamic uniform offsets MUST be aligned to
  `limits.min_uniform_buffer_offset_alignment` and the count MUST match the
  number of dynamic bindings set.
- The number of bind group layouts in a pipeline MUST be ≤ `limits.max_bind_groups`.
- Violations surface as wgpu validation errors during resource creation or when
  encoding `set_bind_group`. Helper functions validate alignment and counts.

## Constraints and Rules

- WGSL/std140-like layout for uniform buffers (as enforced by wgpu):
  - Scalars 4 B; `vec2` 8 B; `vec3/vec4` 16 B; matrices 16 B column alignment.
  - Struct members rounded up to their alignment; struct size rounded up to the
    max alignment of its fields.
- Rust-side structs used as UBOs MUST be `#[repr(C)]` and plain old data. Using
  `bytemuck::{Pod, Zeroable}` in examples is recommended for safety.
- Dynamic offsets must be multiples of
  `limits.min_uniform_buffer_offset_alignment`.
- Respect `limits.max_uniform_buffer_binding_size` when slicing UBOs.
- Matrices are column‑major in GLSL/WGSL. If CPU math constructs row‑major
  matrices, transpose before uploading or mark GLSL uniform
  blocks with `layout(row_major)` to avoid unexpected transforms.

## Example Usage

Rust (high level)

```rust
use lambda::render::{
  bind::{BindGroupLayoutBuilder, BindGroupBuilder, BindingVisibility},
  buffer::{BufferBuilder, Usage, Properties},
  pipeline::RenderPipelineBuilder,
  command::RenderCommand as RC,
};

#[repr(C)]
#[derive(Copy, Clone)]
struct Globals { view_proj: [[f32; 4]; 4] }

// Layout: set(0)@binding(0) uniform visible to vertex stage
let layout = BindGroupLayoutBuilder::new()
  .with_uniform(0, BindingVisibility::Vertex)
  .build(rc.gpu());

// Create UBO
let ubo = BufferBuilder::new()
  .with_length(core::mem::size_of::<Globals>())
  .with_usage(Usage::UNIFORM)
  .with_properties(Properties::CPU_VISIBLE)
  .with_label("globals-ubo")
  .build(rc.gpu(), vec![Globals { view_proj }])?;

// Bind group that points binding(0) at our UBO
let group0 = BindGroupBuilder::new()
  .with_layout(&layout)
  .with_uniform(0, &ubo, 0, None)
  .build(rc.gpu());

// Pipeline includes the layout
let pipe = RenderPipelineBuilder::new()
  .with_layouts(&[&layout])
  .with_buffer(vbo, attributes)
  .build(
    rc.gpu(),
    rc.surface_format(),
    rc.depth_format(),
    &pass,
    &vs,
    Some(&fs),
  );

// Encode commands
let cmds = vec![
  RC::BeginRenderPass { render_pass: pass_id, viewport },
  RC::SetPipeline { pipeline: pipe_id },
  RC::SetBindGroup { set: 0, group: group0_id, dynamic_offsets: vec![] },
  RC::Draw { vertices: 0..3 },
  RC::EndRenderPass,
];
rc.render(cmds);
```

WGSL snippet

```wgsl
struct Globals { view_proj: mat4x4<f32>; };
@group(0) @binding(0) var<uniform> globals: Globals;

@vertex
fn vs_main(in_pos: vec3<f32>) -> @builtin(position) vec4<f32> {
  return globals.view_proj * vec4<f32>(in_pos, 1.0);
}
```

GLSL snippet (via naga -> SPIR-V)

```glsl
layout(set = 0, binding = 0) uniform Globals { mat4 view_proj; } globals;
void main() { gl_Position = globals.view_proj * vec4(in_pos, 1.0); }
```

Dynamic offsets

```rust
let dyn_layout = BindGroupLayoutBuilder::new()
  .with_uniform_dynamic(0, BindingVisibility::Vertex)
  .build(rc.gpu());

let align = rc.limit_min_uniform_buffer_offset_alignment() as u64;
let size = core::mem::size_of::<Globals>() as u64;
let stride = lambda::render::validation::align_up(size, align);
let offsets = vec![0u32, stride as u32, (2*stride) as u32];
RC::SetBindGroup { set: 0, group: dyn_group_id, dynamic_offsets: offsets };
```

## Performance Considerations

- Prefer `Properties::DEVICE_LOCAL` for long‑lived uniform buffers that are
  updated infrequently; otherwise use CPU‑visible memory with
  `Queue::write_buffer` for per‑frame updates.
  - Rationale: Device‑local memory provides higher bandwidth and lower latency
    for repeated reads. When updates are rare, the staging copy cost is
    amortized and the GPU benefits every frame. For small
    per‑frame updates, writing directly to CPU‑visible memory avoids additional
    copies and reduces driver synchronization. On integrated graphics, the hint
    still guides an efficient path and helps avoid stalls.
- Use dynamic offsets to reduce bind group churn; align and pack many objects in
  a single uniform buffer.
  - Rationale: Reusing one bind group and changing only a 32‑bit offset turns
    descriptor updates into a cheap command. This lowers CPU
    overhead, reduces driver validation and allocation, improves cache locality
    by keeping per‑object blocks contiguous, and reduces the number of bind
    groups created and cached. Align slices to
    `min_uniform_buffer_offset_alignment` to satisfy hardware requirements and
    avoid implicit padding or copies.
- Separate stable data (for example, camera) from frequently changing data (for
  example, per‑object).
  - Rationale: Bind stable data once per pass and vary only the hot set per
    draw. This reduces state changes, keeps descriptor caches warm, avoids
    rebinding large constant blocks when only small data changes, and lowers
    bandwidth while improving cache effectiveness.

## Requirements Checklist

- Functionality
  - [x] Core behavior implemented — crates/lambda-rs/src/render/bind.rs
  - [x] Dynamic offsets supported — crates/lambda-rs/src/render/command.rs
  - [x] Edge cases validated (alignment/size) — crates/lambda-rs/src/render/validation.rs
- API Surface
  - [x] Platform types and builders — crates/lambda-rs-platform/src/wgpu/bind.rs
  - [x] High-level wrappers and builders — crates/lambda-rs/src/render/bind.rs
  - [x] Pipeline layout integration — crates/lambda-rs/src/render/pipeline.rs
- Validation and Errors
  - [x] Uniform binding size checks — crates/lambda-rs/src/render/mod.rs
  - [x] Dynamic offset alignment/count checks — crates/lambda-rs/src/render/validation.rs
- Performance
  - [x] Recommendations documented (this section)
  - [x] Dynamic offsets example provided — docs/specs/uniform-buffers-and-bind-groups.md
- Documentation and Examples
  - [x] Spec updated (this document)
  - [x] Example added — demos/render/src/bin/uniform_buffer_triangle.rs

## Verification and Testing

- Unit tests
  - Alignment helper and dynamic offset validation — crates/lambda-rs/src/render/validation.rs
  - Visibility mapping — crates/lambda-rs-platform/src/wgpu/bind.rs
  - Command encoding satisfies device limits — crates/lambda-rs/src/render/command.rs
  - Command: `cargo test --workspace`
- Integration tests and examples
  - `uniform_buffer_triangle` exercises the full path — demos/render/src/bin/uniform_buffer_triangle.rs
  - Command: `cargo run -p lambda-demos-render --bin uniform_buffer_triangle`
- Manual checks (optional)
  - Validate dynamic offsets across multiple objects render correctly (no
    misaligned reads) by varying object counts and strides.

## Compatibility and Migration

- No breaking changes. The feature is additive. Existing pipelines without bind
  groups continue to function. New pipelines MAY specify layouts via
  `with_layouts` without impacting prior behavior.

## Changelog

- 2026-02-05 (v0.5.1) — Update demo and example references for `demos/`.
- 2025-12-15 (v0.5.0) — Update example code to use `rc.gpu()` and add `surface_format`/`depth_format` parameters to `RenderPipelineBuilder`.
- 2025-10-17 (v0.4.0) — Restructure to match spec template: add Summary, Scope,
  Terminology, Design (API/Behavior/Validation), Constraints and Rules,
  Requirements Checklist, Verification and Testing, and Compatibility. Remove
  Implementation Plan and Open Questions. No functional changes.
- 2025-10-17 (v0.3.0) — Edit for professional tone; adopt clearer normative
  phrasing; convert Performance Notes to concise rationale; no functional
  changes to the specification; update metadata.
- 2025-10-17 (v0.2.1) — Expand Performance Notes with rationale; update
  metadata (`last_updated`, `version`, `repo_commit`).
- 2025-10-13 (v0.1.1) — Synced spec to implementation: renamed visibility enum variant to `VertexAndFragment`; clarified that builders defer validation to `wgpu`; updated `with_uniform` size type to `Option<NonZeroU64>`; added note on GPU column‑major matrices and CPU transpose guidance; adjusted dynamic offset example.
- 2025-10-11 (v0.1.0) — Initial draft aligned to roadmap; specifies platform and high-level APIs, commands, validation, examples, and phased delivery.
- 2025-10-13 (v0.2.0) — Add validation for dynamic offsets (count/alignment),
  assert uniform binding sizes against device limits, assert max bind groups in
  pipeline builder; add `UniformBuffer<T>` wrapper; expose `align_up` helper; update examples.
