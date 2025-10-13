---
title: "Uniform Buffers and Bind Groups"
document_id: "ubo-spec-2025-10-11"
status: "living"
created: "2025-10-11T00:00:00Z"
last_updated: "2025-10-13T00:00:00Z"
version: "0.2.0"
engine_workspace_version: "2023.1.30"
wgpu_version: "26.0.1"
shader_backend_default: "naga"
winit_version: "0.29.10"
repo_commit: "3e63f82b0a364bc52a40ae297a5300f998800518"
owners: ["lambda-sh"]
reviewers: ["engine", "rendering"]
tags: ["spec", "rendering", "uniforms", "bind-groups", "wgpu"]
---

# Uniform Buffers and Bind Groups

This spec defines uniform buffer objects (UBO) and bind groups for Lambda’s
wgpu-backed renderer. It follows the existing builder/command patterns and
splits responsibilities between the platform layer (`lambda-rs-platform`) and
the high-level API (`lambda-rs`).

The design enables larger, structured GPU constants (cameras, materials,
per-frame data) beyond push constants, with an ergonomic path to dynamic
offsets for batching many small uniforms in a single buffer.

## Goals

- Add first-class uniform buffers and bind groups.
- Maintain builder ergonomics consistent with buffers, pipelines, and passes.
- Integrate with the existing render command stream (inside a pass).
- Provide a portable, WGSL/GLSL-friendly layout model and validation.
- Expose dynamic uniform offsets (opt-in) with correct alignment handling.

## Non-Goals

- Storage buffers, textures/samplers, and compute are referenced but not
  implemented here; separate specs cover them.
- Descriptor set caching beyond wgpu’s internal caches.

## Background

Roadmap docs propose UBOs and bind groups to complement push constants and
unlock cameras/materials. This spec refines those sketches into concrete API
types, builders, commands, validation, and an implementation plan for both
layers of the workspace.

## Architecture Overview

- Platform (`lambda-rs-platform`)
  - Thin wrappers around `wgpu::BindGroupLayout` and `wgpu::BindGroup` with
    builder structs that produce concrete `wgpu` descriptors and perform
    validation against device limits.
  - Expose the raw `wgpu` handles for use by higher layers.

- High level (`lambda-rs`)
  - Public builders/types for bind group layouts and bind groups aligned with
    `RenderPipelineBuilder` and `BufferBuilder` patterns.
  - Extend `RenderPipelineBuilder` to accept bind group layouts, building a
    `wgpu::PipelineLayout` under the hood.
  - Extend `RenderCommand` with `SetBindGroup` to bind resources during a pass.
  - Avoid exposing `wgpu` types in the public API; surface numeric limits and
    high-level wrappers only, delegating raw handles to the platform layer.

Data flow (one-time setup → per-frame):
```
BindGroupLayoutBuilder --> BindGroupLayout --+--> RenderPipelineBuilder (layouts)
                                             |
BufferBuilder (Usage::UNIFORM) --------------+--> BindGroupBuilder (uniform binding)

Per-frame commands: BeginRenderPass -> SetPipeline -> SetBindGroup -> Draw -> End
```

## Platform API Design (lambda-rs-platform)

- Module: `lambda_platform::wgpu::bind`
  - `struct BindGroupLayout { raw: wgpu::BindGroupLayout, label: Option<String> }`
  - `struct BindGroup { raw: wgpu::BindGroup, label: Option<String> }`
  - `enum Visibility { Vertex, Fragment, Compute, VertexAndFragment, All }`
    - Maps to `wgpu::ShaderStages`.
  - `struct BindGroupLayoutBuilder { entries: Vec<wgpu::BindGroupLayoutEntry>, label: Option<String> }`
    - `fn new() -> Self`
    - `fn with_uniform(mut self, binding: u32, visibility: Visibility) -> Self`
    - `fn with_uniform_dynamic(mut self, binding: u32, visibility: Visibility) -> Self`
    - `fn with_label(mut self, label: &str) -> Self`
    - `fn build(self, device: &wgpu::Device) -> BindGroupLayout`
  - `struct BindGroupBuilder { layout: wgpu::BindGroupLayout, entries: Vec<wgpu::BindGroupEntry>, label: Option<String> }`
    - `fn new() -> Self`
    - `fn with_layout(mut self, layout: &BindGroupLayout) -> Self`
    - `fn with_uniform(mut self, binding: u32, buffer: &wgpu::Buffer, offset: u64, size: Option<NonZeroU64>) -> Self`
    - `fn with_label(mut self, label: &str) -> Self`
    - `fn build(self, device: &wgpu::Device) -> BindGroup`

Validation and limits
- High-level validation now checks common cases early:
  - Bind group uniform binding sizes are asserted to be ≤ `max_uniform_buffer_binding_size`.
  - Dynamic offset count and alignment are validated before encoding `SetBindGroup`.
  - Pipeline builder asserts the number of bind group layouts ≤ `max_bind_groups`.
  - Helpers are provided to compute aligned strides and to validate dynamic offsets.

Helpers
- High-level exposes small helpers:
  - `align_up(value, align)` to compute aligned uniform strides (for offsets).
  - `validate_dynamic_offsets(required, offsets, alignment, set)` used internally and testable.

## High-Level API Design (lambda-rs)

New module: `lambda::render::bind`
- `pub struct BindGroupLayout { /* holds Rc<wgpu::BindGroupLayout> */ }`
- `pub struct BindGroup { /* holds Rc<wgpu::BindGroup> */ }`
- `pub enum BindingVisibility { Vertex, Fragment, Compute, VertexAndFragment, All }`
- `pub struct BindGroupLayoutBuilder { /* mirrors platform builder */ }`
  - `pub fn new() -> Self`
  - `pub fn with_uniform(self, binding: u32, visibility: BindingVisibility) -> Self`
  - `pub fn with_uniform_dynamic(self, binding: u32, visibility: BindingVisibility) -> Self`
  - `pub fn with_label(self, label: &str) -> Self`
  - `pub fn build(self, rc: &RenderContext) -> BindGroupLayout`
- `pub struct BindGroupBuilder { /* mirrors platform builder */ }`
  - `pub fn new() -> Self`
  - `pub fn with_layout(self, layout: &BindGroupLayout) -> Self`
  - `pub fn with_uniform(self, binding: u32, buffer: &buffer::Buffer, offset: u64, size: Option<NonZeroU64>) -> Self`
  - `pub fn with_label(self, label: &str) -> Self`
  - `pub fn build(self, rc: &RenderContext) -> BindGroup`

Pipeline integration
- `RenderPipelineBuilder::with_layouts(&[&BindGroupLayout])` stores layouts and
  constructs a `wgpu::PipelineLayout` during `build(...)`.

Render commands
- Extend `RenderCommand` with:
  - `SetBindGroup { set: u32, group: super::ResourceId, dynamic_offsets: Vec<u32> }`
- `RenderContext::encode_pass` maps to `wgpu::RenderPass::set_bind_group`.

Buffers
- Continue using `buffer::BufferBuilder` with `Usage::UNIFORM` and CPU-visible
  properties for frequently updated UBOs.
- A typed `UniformBuffer<T>` wrapper is available with `new(&mut rc, &T, label)`
  and `write(&rc, &T)`, and exposes `raw()` to bind.

## Layout and Alignment Rules

- WGSL/std140-like layout for uniform buffers (via naga/wgpu):
  - Scalars 4 B; `vec2` 8 B; `vec3/vec4` 16 B; matrices 16 B column alignment.
  - Struct members rounded up to their alignment; struct size rounded up to the
    max alignment of its fields.
- Rust-side structs used as UBOs must be `#[repr(C)]` and plain-old-data.
  Recommend `bytemuck::{Pod, Zeroable}` in examples for safety.
- Dynamic offsets must be multiples of
  `limits.min_uniform_buffer_offset_alignment`.
- Respect `limits.max_uniform_buffer_binding_size` when slicing UBOs.
 - Matrices are column‑major in GLSL/WGSL. If your CPU math builds row‑major
   matrices, either transpose before uploading to the GPU or mark GLSL uniform
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
  .build(&mut rc);

// Create UBO
let ubo = BufferBuilder::new()
  .with_length(core::mem::size_of::<Globals>())
  .with_usage(Usage::UNIFORM)
  .with_properties(Properties::CPU_VISIBLE)
  .with_label("globals-ubo")
  .build(&mut rc, vec![Globals { view_proj }])?;

// Bind group that points binding(0) at our UBO
let group0 = BindGroupBuilder::new()
  .with_layout(&layout)
  .with_uniform(0, &ubo, 0, None)
  .build(&mut rc);

// Pipeline includes the layout
let pipe = RenderPipelineBuilder::new()
  .with_layouts(&[&layout])
  .with_buffer(vbo, attributes)
  .build(&mut rc, &pass, &vs, Some(&fs));

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
  .build(&mut rc);

let align = rc.limit_min_uniform_buffer_offset_alignment() as u64;
let size = core::mem::size_of::<Globals>() as u64;
let stride = lambda::render::validation::align_up(size, align);
let offsets = vec![0u32, stride as u32, (2*stride) as u32];
RC::SetBindGroup { set: 0, group: dyn_group_id, dynamic_offsets: offsets };
```

## Error Handling

- `BufferBuilder` already errors on zero length; keep behavior.
- Bind group and layout builders currently do not pre‑validate against device limits.
  Invalid sizes/offsets typically surface as `wgpu` validation errors during creation
  or when calling `set_bind_group`. Ensure dynamic offsets are aligned to device limits
  and uniform ranges respect `max_uniform_buffer_binding_size`.

## Performance Notes

- Prefer `Properties::DEVICE_LOCAL` for long-lived UBOs updated infrequently;
  otherwise CPU-visible + `Queue::write_buffer` for per-frame updates.
- Dynamic offsets reduce bind group churn; align and pack many objects per UBO.
- Group stable data (camera) separate from frequently changing data (object).

## Implementation Plan

Phase 0 (minimal, static UBO)
- Platform: add bind module, layout/bind builders, validation helpers.
- High level: expose `bind` module; add pipeline `.with_layouts`; extend
  `RenderCommand` and encoder with `SetBindGroup`.
- Update examples to use one UBO for a transform/camera.

Phase 1 (dynamic offsets)
- Done: `.with_uniform_dynamic` in layout builder, support for dynamic offsets, and
  validation of count/alignment before binding. Alignment helper implemented.

Phase 2 (ergonomics/testing)
- Done: `UniformBuffer<T>` wrapper with `.write(&T)` convenience.
- Added unit tests for alignment and dynamic offset validation; example animates a
  triangle with a UBO (integration test remains minimal).

File layout
- Platform: `crates/lambda-rs-platform/src/wgpu/bind.rs` (+ `mod.rs` re-export).
- High level: `crates/lambda-rs/src/render/bind.rs`, plus edits to
  `render/pipeline.rs`, `render/command.rs`, and `render/mod.rs` to wire in
  pipeline layouts and `SetBindGroup` encoding.

## Testing Plan

- Unit tests
  - Alignment helper (`align_up`) and dynamic offset validation logic.
  - Visibility enum mapping test (in-place).
- Integration
  - Example `uniform_buffer_triangle` exercises the full path; a fuller
    runnable test remains a future improvement.

## Open Questions

- Should we introduce a typed `Uniform<T>` handle now, or wait until there is a
  second typed resource (e.g., storage) to avoid a one-off abstraction?
- Do we want a tiny cache for bind groups keyed by buffer+offset for frequent
  reuse, or rely entirely on wgpu’s internal caches?

## Changelog

- 2025-10-13 (v0.1.1) — Synced spec to implementation: renamed visibility enum variant to `VertexAndFragment`; clarified that builders defer validation to `wgpu`; updated `with_uniform` size type to `Option<NonZeroU64>`; added note on GPU column‑major matrices and CPU transpose guidance; adjusted dynamic offset example.
- 2025-10-11 (v0.1.0) — Initial draft aligned to roadmap; specifies platform and high-level APIs, commands, validation, examples, and phased delivery.
- 2025-10-13 (v0.2.0) — Add validation for dynamic offsets (count/alignment),
  assert uniform binding sizes against device limits, assert max bind groups in
  pipeline builder; add `UniformBuffer<T>` wrapper; expose `align_up` helper; update examples.
