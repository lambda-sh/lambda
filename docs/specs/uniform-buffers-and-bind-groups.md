---
title: "Uniform Buffers and Bind Groups"
document_id: "ubo-spec-2025-10-11"
status: "draft"
created: "2025-10-11T00:00:00Z"
last_updated: "2025-10-11T00:00:00Z"
version: "0.1.0"
engine_workspace_version: "2023.1.30"
wgpu_version: "26.0.1"
shader_backend_default: "naga"
winit_version: "0.29.10"
repo_commit: "0fdc489f5560acf809ca9cd8440f086baab7bad5"
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
  - `enum Visibility { Vertex, Fragment, Compute, BothVF, All }`
    - Maps to `wgpu::ShaderStages`.
  - `enum BindingKind { Uniform { dynamic: bool }, /* future: SampledTexture, Sampler, Storage*/ }`
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
- On `build`, validate against `wgpu::Limits`:
  - `max_uniform_buffer_binding_size` for explicit sizes.
  - `min_uniform_buffer_offset_alignment` for dynamic offsets (caller provides
    aligned `offset`; builder re-checks and errors if misaligned).
  - `max_bind_groups` when composing pipeline layouts (exposed via helper).
- Return detailed error strings mapped into high-level errors in `lambda-rs`.

Helpers
- `fn shader_stages(vis: Visibility) -> wgpu::ShaderStages`
- `fn align_up(value: u64, align: u64) -> u64`

## High-Level API Design (lambda-rs)

New module: `lambda::render::bind`
- `pub struct BindGroupLayout { /* holds Rc<wgpu::BindGroupLayout> */ }`
- `pub struct BindGroup { /* holds Rc<wgpu::BindGroup> */ }`
- `pub enum BindingVisibility { Vertex, Fragment, Compute, BothVF, All }`
- `pub struct BindGroupLayoutBuilder { /* mirrors platform builder */ }`
  - `pub fn new() -> Self`
  - `pub fn with_uniform(self, binding: u32, visibility: BindingVisibility) -> Self`
  - `pub fn with_uniform_dynamic(self, binding: u32, visibility: BindingVisibility) -> Self`
  - `pub fn with_label(self, label: &str) -> Self`
  - `pub fn build(self, rc: &RenderContext) -> BindGroupLayout`
- `pub struct BindGroupBuilder { /* mirrors platform builder */ }`
  - `pub fn new() -> Self`
  - `pub fn with_layout(self, layout: &BindGroupLayout) -> Self`
  - `pub fn with_uniform(self, binding: u32, buffer: &buffer::Buffer, offset: u64, size: Option<u64>) -> Self`
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
  properties for frequently updated UBOs. No new types are strictly required,
  but an optional typed helper improves ergonomics:

```rust
#[repr(C)]
#[derive(Copy, Clone)]
pub struct UniformBuffer<T> { inner: buffer::Buffer, _phantom: core::marker::PhantomData<T> }

impl<T> UniformBuffer<T> {
  pub fn raw(&self) -> &buffer::Buffer { &self.inner }
  pub fn write(&self, rc: &mut RenderContext, value: &T) where T: Copy {
    let bytes = unsafe {
      std::slice::from_raw_parts(
        (value as *const T) as *const u8,
        core::mem::size_of::<T>(),
      )
    };
    rc.queue().write_buffer(self.inner.raw(), 0, bytes);
  }
}
```

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

Dynamic offsets (optional)
```rust
let dyn_layout = BindGroupLayoutBuilder::new()
  .with_uniform_dynamic(0, BindingVisibility::Vertex)
  .build(&mut rc);

let stride = align_up(core::mem::size_of::<Globals>() as u64,
                      rc.device().limits().min_uniform_buffer_offset_alignment);
let offsets = vec![0u32, stride as u32, (2*stride) as u32];
RC::SetBindGroup { set: 0, group: dyn_group_id, dynamic_offsets: offsets };
```

## Error Handling

- `BufferBuilder` already errors on zero length; keep behavior.
- New bind errors returned during `build` with clear messages:
  - "uniform binding size exceeds max_uniform_buffer_binding_size"
  - "dynamic offset not aligned to min_uniform_buffer_offset_alignment"
  - "invalid binding index (duplicate or out of range)"
  - "pipeline layouts exceed device.max_bind_groups"

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
- Add `.with_uniform_dynamic` to layout builder and support offsets in
  `SetBindGroup`. Validate alignment vs device limits.
- Add small helper to compute aligned strides.

Phase 2 (ergonomics/testing)
- Optional `UniformBuffer<T>` wrapper with `.write(&T)` for convenience.
- Unit tests for builders and validation; integration test that animates a
  triangle with a camera UBO.

File layout
- Platform: `crates/lambda-rs-platform/src/wgpu/bind.rs` (+ `mod.rs` re-export).
- High level: `crates/lambda-rs/src/render/bind.rs`, plus edits to
  `render/pipeline.rs`, `render/command.rs`, and `render/mod.rs` to wire in
  pipeline layouts and `SetBindGroup` encoding.

## Testing Plan

- Unit tests
  - Layout builder produces expected `wgpu::BindGroupLayoutEntry` values.
  - Bind group builder rejects misaligned dynamic offsets.
  - Pipeline builder errors if too many layouts are provided.
- Integration tests (`crates/lambda-rs/tests/runnables.rs`)
  - Simple pipeline using a UBO to transform vertices; compare golden pixels or
    log successful draw on supported adapters.

## Open Questions

- Should we introduce a typed `Uniform<T>` handle now, or wait until there is a
  second typed resource (e.g., storage) to avoid a one-off abstraction?
- Do we want a tiny cache for bind groups keyed by buffer+offset for frequent
  reuse, or rely entirely on wgpu’s internal caches?

## Changelog

- 2025-10-11 (v0.1.0) — Initial draft aligned to roadmap; specifies platform
  and high-level APIs, commands, validation, examples, and phased delivery.
