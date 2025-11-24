---
title: "Instanced Rendering"
document_id: "instanced-rendering-2025-11-23"
status: "draft"
created: "2025-11-23T00:00:00Z"
last_updated: "2025-11-23T00:00:00Z"
version: "0.1.0"
engine_workspace_version: "2023.1.30"
wgpu_version: "26.0.1"
shader_backend_default: "naga"
winit_version: "0.29.10"
repo_commit: "afabc0597de11b66124e937b4346923e25da3159"
owners: ["lambda-sh"]
reviewers: ["engine", "rendering"]
tags: ["spec", "rendering", "instancing", "vertex-input"]
---

# Instanced Rendering

## Table of Contents
- [Summary](#summary)
- [Scope](#scope)
- [Terminology](#terminology)
- [Architecture Overview](#architecture-overview)
- [Design](#design)
  - [API Surface](#api-surface)
  - [Behavior](#behavior)
  - [Validation and Errors](#validation-and-errors)
- [Constraints and Rules](#constraints-and-rules)
- [Performance Considerations](#performance-considerations)
- [Requirements Checklist](#requirements-checklist)
- [Verification and Testing](#verification-and-testing)
- [Compatibility and Migration](#compatibility-and-migration)
- [Changelog](#changelog)

## Summary

- Introduce instanced rendering as a first-class capability in the high-level
  rendering API, enabling efficient drawing of many copies of shared geometry
  with per-instance data while keeping `wgpu` types encapsulated in
  `lambda-rs-platform`.
- Extend vertex input descriptions and draw commands to support instance-rate
  vertex buffers and explicit instance ranges without breaking existing
  non-instanced pipelines or render paths.
- Provide validation and feature-gated checks so instanced rendering failures
  are actionable in development while imposing minimal overhead in release
  builds.

## Scope

### Goals

- Define instance-rate vertex buffer semantics in the high-level vertex input
  model using engine-level types.
- Allow per-instance data (for example, transforms and colors) to be supplied
  through buffers and consumed by vertex shaders using existing binding
  patterns.
- Clarify the semantics of the `instances: Range<u32>` field on draw commands
  and propagate instance ranges through the platform layer to `wgpu`.
- Add feature-gated validation for instance ranges, buffer usage, and
  configuration ordering that integrates with existing rendering validation
  features.
- Require at least one example and runnable scenario that demonstrates
  instanced rendering with a visible, repeatable outcome.

### Non-Goals

- GPU-driven rendering techniques such as indirect draw buffers, automatic
  culling, or multi-draw indirect command generation.
- New mesh or asset file formats; instanced rendering consumes existing vertex
  and index data representations.
- Per-instance bind group fan-out or per-instance pipeline specialization.
- Scene graph or batching abstractions; instanced rendering remains a low-level
  rendering primitive that higher-level systems MAY build upon separately.

## Terminology

- Instance: one logical copy of a drawable object emitted by a draw call.
- Instanced rendering: a draw technique where a single draw command emits many
  instances of the same geometry, each instance optionally using distinct
  per-instance data.
- Instance buffer: a vertex buffer whose attributes advance once per instance
  rather than once per vertex.
- Step mode: a configuration that determines whether vertex input attributes
  are stepped per vertex or per instance.
- Instance range: a `Range<u32>` that specifies the first instance index and
  the number of instances to draw.
- Instance index: the built-in shader input that identifies the current
  instance (for example, `@builtin(instance_index)` in `wgpu`-style shaders).

## Architecture Overview

- High-level layer (`lambda-rs`)
  - `RenderPipelineBuilder` declares vertex buffer layouts, including a step
    mode that describes whether a buffer is per-vertex or per-instance.
  - The render command stream uses existing `Draw` and `DrawIndexed`
    commands with an `instances: Range<u32>` field to control instance count
    and first instance.
  - Public types represent step modes and instance-aware vertex buffer layouts;
    backend-specific details remain internal to `lambda-rs-platform`.
- Platform layer (`lambda-rs-platform`)
  - Wraps `wgpu` vertex buffer layouts with an engine-level `VertexStepMode`
    and forwards instance ranges to `wgpu::RenderPass::draw` and
    `wgpu::RenderPass::draw_indexed`.
  - Exposes draw helpers that accept both vertex or index ranges and instance
    ranges, ensuring that engine-level commands can express instance counts and
    first instance indices.
- Data flow

```
App Code
  └── lambda-rs
        ├── BufferBuilder (BufferType::Vertex)
        ├── RenderPipelineBuilder::with_buffer(.., step_mode)
        └── RenderCommand::{BindVertexBuffer, Draw, DrawIndexed}
              └── RenderContext encoder
                    └── lambda-rs-platform (vertex layouts, draw calls)
                          └── wgpu::RenderPass::{set_vertex_buffer,
                                                 draw, draw_indexed}
```

## Design

### API Surface

- Platform layer (`lambda-rs-platform`)
  - Vertex input types (module `lambda_platform::wgpu::vertex`)
    - `enum VertexStepMode { Vertex, Instance }`
      - Maps directly to `wgpu::VertexStepMode::{Vertex, Instance}`.
    - `struct VertexBufferLayout { stride: u64, step_mode: VertexStepMode, /* attributes */ }`
      - The `step_mode` field controls whether attributes sourced from this
        buffer advance per vertex or per instance.
  - Render pass integration (module `lambda_platform::wgpu::render_pass`)
    - `fn draw(&mut self, vertices: Range<u32>, instances: Range<u32>)`.
    - `fn draw_indexed(&mut self, indices: Range<u32>, base_vertex: i32,
                       instances: Range<u32>)`.
      - These functions map `instances.start` to `first_instance` and
        `instances.end - instances.start` to `instance_count` on the underlying
        `wgpu::RenderPass`.

- High-level layer (`lambda-rs`)
  - Vertex buffer layouts (module `lambda::render::vertex`)
    - `enum VertexStepMode { PerVertex, PerInstance }`
      - High-level mirror of the platform `VertexStepMode`.
    - `struct VertexBufferLayout { stride: u64, step_mode: VertexStepMode,
                                   /* attributes */ }`
      - `step_mode` defaults to `PerVertex` when not explicitly set.
  - Pipeline builder (module `lambda::render::pipeline`)
    - `RenderPipelineBuilder::with_buffer(buffer, attributes) -> Self`
      - Existing function; continues to configure a per-vertex buffer and
        implicitly sets `step_mode` to `PerVertex`.
    - `RenderPipelineBuilder::with_buffer_step_mode(buffer, attributes,
                                                    step_mode: VertexStepMode)
       -> Self`
      - New builder method that configures a buffer with an explicit step mode.
      - MUST accept both `PerVertex` and `PerInstance` and attach the step mode
        to the vertex buffer layout for the given slot.
    - `RenderPipelineBuilder::with_instance_buffer(buffer, attributes) -> Self`
      - Convenience method equivalent to calling `with_buffer_step_mode` with
        `step_mode = VertexStepMode::PerInstance`.
  - Render commands (module `lambda::render::command`)
    - `enum RenderCommand { /* existing variants */, Draw { vertices:
      Range<u32>, instances: Range<u32> }, DrawIndexed { indices:
      Range<u32>, base_vertex: i32, instances: Range<u32> }, /* ... */ }`
      - `Draw` and `DrawIndexed` remain the single entry points for emitting
        primitives; instanced rendering is expressed entirely through the
        `instances` range.
      - The engine MUST treat `instances = 0..1` as the default single-instance
        behavior used by existing rendering paths.
  - Feature flags (`lambda-rs`)
    - `render-instancing-validation`
      - Owning crate: `lambda-rs`.
      - Default: disabled in release builds; MAY be enabled in debug builds or
        by opt-in.
      - Summary: enables additional validation of instance ranges, step modes,
        and buffer bindings for instanced draws.
      - Runtime cost: additional checks per draw command when instancing is
        used; no effect when instancing is unused.

### Behavior

- Step modes and vertex consumption
  - Buffers configured with `VertexStepMode::PerVertex` advance attribute
    indices once per vertex and are indexed by the vertex index.
  - Buffers configured with `VertexStepMode::PerInstance` advance attribute
    indices once per instance and are indexed by the instance index.
  - The pipeline vertex input layout MUST include exactly one step mode per
    buffer slot; a buffer slot cannot mix per-vertex and per-instance step
    modes.
- Draw commands and instance ranges
  - The `instances` range on `Draw` and `DrawIndexed` commands controls the
    number of instances emitted and the first instance index:
    - `first_instance = instances.start`.
    - `instance_count = instances.end - instances.start`.
  - When `instance_count == 0`, the engine SHOULD treat the draw as a no-op and
    MAY log a debug-level diagnostic when instancing validation is enabled.
  - When no buffers are configured with `PerInstance` step mode, instanced
    draws remain valid and expose the instance index only through the shader
    built-in.
  - Existing rendering paths that omit explicit `instances` ranges MUST
    continue to behave as single-instance draws by using `0..1`.
- Buffer bindings and slots
  - Buffer slots are shared between per-vertex and per-instance buffers; the
    step mode recorded on the pipeline layout determines how the backend steps
    each buffer slot.
  - A buffer bound to a slot whose layout uses `PerInstance` step mode is an
    instance buffer; a buffer bound to a slot whose layout uses `PerVertex`
    step mode is a vertex buffer.
  - The render context MUST bind all buffers required by the pipeline (vertex
    and instance) before issuing `Draw` or `DrawIndexed` commands that rely on
    those slots.
- Validation behavior
  - When `render-instancing-validation` and `render-validation-encoder` are
    enabled, the engine SHOULD:
    - Verify that all buffer slots used by per-instance attributes are bound
      before a draw that uses those attributes.
    - Emit a clear error when a draw is issued with an `instances` range whose
      upper bound exceeds engine-configured expectations for the instance
      buffer size, when this information is available.
    - Check that `instances.start <= instances.end` and treat negative-length
      ranges as configuration errors.

### Validation and Errors

- Command ordering
  - `BeginRenderPass` MUST precede any `SetPipeline`, `BindVertexBuffer`,
    `Draw`, or `DrawIndexed` commands.
  - `EndRenderPass` MUST terminate the pass; commands that require an active
    pass and are encoded after `EndRenderPass` SHOULD be rejected and logged as
    configuration errors.
- Vertex and instance buffer binding
  - `BindVertexBuffer` MUST reference a buffer created with `BufferType::Vertex`
    and a slot index that is less than the number of vertex buffer layouts
    declared on the pipeline.
  - When `render-instancing-validation` is enabled, the engine SHOULD:
    - Verify that the set of bound buffers covers all pipeline slots that
      declare per-instance attributes before a draw is issued.
    - Log an error if a draw is issued with a per-instance attribute whose slot
      has not been bound.
- Instance range validation
  - An `instances` range with `start > end` MUST be rejected as invalid, and
    the engine SHOULD log a clear diagnostic that includes the offending
    range.
  - For `start == end`, the draw SHOULD be treated as a no-op and MAY log a
    debug-level message when instancing validation is enabled.
  - Extremely large instance counts MAY be clamped or rejected based on device
    limits; see Constraints and Rules.

## Constraints and Rules

- Device support
  - Instanced rendering is a core capability of the `wgpu` backend; the engine
    MAY assume basic support for instancing on all supported devices.
  - If a backend without instancing support is ever introduced, instanced
    draws MUST fail fast at pipeline creation or command encoding with a clear
    diagnostic.
- Data layout
  - Per-instance attributes follow the same alignment and format rules as
    per-vertex attributes; attribute offsets MUST be within the buffer stride
    and aligned to the format size.
  - Buffers used for per-instance data MUST be created with vertex usage flags
    consistent with existing vertex buffers.
  - Instance buffer sizes SHOULD be chosen to accommodate the maximum expected
    instance count for the associated draw paths.
- Limits
  - The engine SHOULD respect and document any `wgpu` limits on maximum vertex
    buffer stride, attribute count, and instance count.
  - Instanced draws that exceed device limits MUST be rejected and logged
    rather than silently truncated.

## Performance Considerations

- Recommendations
  - Prefer instanced rendering over many small, identical draw calls when
    rendering repeated geometry.
    - Rationale: instancing reduces CPU overhead and command buffer size by
      amortizing state setup across many instances.
  - Pack frequently updated per-instance attributes into a small number of
    tightly packed instance buffers.
    - Rationale: fewer, contiguous buffers improve cache locality and reduce
      binding overhead.
  - Avoid using instanced rendering for very small numbers of instances when
    it complicates shader logic without measurable benefit.
    - Rationale: the complexity overhead MAY outweigh the performance gain for
      a handful of instances.
  - Use validation features only in development builds.
    - Rationale: instancing validation introduces per-draw checks that are
      valuable for debugging but unnecessary in production.

## Requirements Checklist

- Functionality
  - [ ] Instance-aware vertex buffer layouts defined in `lambda-rs` and
        `lambda-rs-platform`.
  - [ ] Draw helpers in `lambda-rs-platform` accept and forward instance
        ranges.
  - [ ] Existing draw paths continue to function with `instances = 0..1`.
- API Surface
  - [ ] `VertexStepMode` exposed at engine and platform layers.
  - [ ] `RenderPipelineBuilder` supports explicit per-instance buffers via
        `with_buffer_step_mode` and `with_instance_buffer`.
  - [ ] Instancing validation feature flag defined in `lambda-rs`.
- Validation and Errors
  - [ ] Command ordering checks cover instanced draws.
  - [ ] Instance range validation implemented and feature-gated.
  - [ ] Buffer binding diagnostics cover per-instance attributes.
- Performance
  - [ ] Critical instanced draw paths reasoned about or profiled.
  - [ ] Memory usage for instance buffers characterized for example scenes.
  - [ ] Performance recommendations documented for instanced rendering usage.
- Documentation and Examples
  - [ ] User-facing rendering docs updated to describe instanced rendering and
        usage patterns.
  - [ ] At least one example or runnable scenario added that demonstrates
        instanced rendering.
  - [ ] Any necessary migration notes captured in `docs/rendering.md` or
        related documentation.

For each checked item, include a reference to a commit, pull request, or file
path that demonstrates the implementation.

## Verification and Testing

- Unit Tests
  - Verify that vertex buffer layouts correctly map `VertexStepMode` from the
    engine layer to the platform layer and into `wgpu`.
  - Ensure that draw helpers forward instance ranges correctly and reject
    invalid ranges when validation is enabled.
  - Commands:
    - `cargo test -p lambda-rs-platform -- --nocapture`
    - `cargo test -p lambda-rs -- --nocapture`
- Integration Tests
  - Add or extend runnable scenarios in `crates/lambda-rs/tests/runnables.rs`
    to cover instanced rendering of simple primitives (for example, many
    cubes or quads sharing geometry with per-instance transforms).
  - Validate that renders behave consistently across supported platforms and
    backends.
  - Commands:
    - `cargo test --workspace -- --nocapture`
- Manual Checks
  - Run an example binary that uses instanced rendering, verify that many
    instances of a mesh render with distinct transforms or colors, and confirm
    that instance count and ranges behave as expected when tweaked.
  - Observe logs with instancing validation enabled to confirm that invalid
    ranges or missing bindings produce actionable diagnostics.

## Compatibility and Migration

- Public engine APIs
  - Adding `VertexStepMode` and step mode-aware buffer builders is designed to
    be backwards compatible; existing code that does not configure per-instance
    buffers continues to function unchanged.
  - The default step mode for existing `with_buffer` calls MUST remain
    per-vertex to avoid altering current behavior.
- Internal platform APIs
  - The updated draw helper signatures in `lambda-rs-platform` constitute an
    internal change; engine call sites MUST be updated in the same change set.
  - No user-facing migration is required unless external code depends directly
    on `lambda-rs-platform` internals, which is discouraged.
- Feature interactions
  - Instancing validation MUST compose with existing rendering validation
    features; enabling multiple validation features MUST NOT alter the
    semantics of valid rendering commands.
  - No new environment variables are introduced by this specification.

## Changelog

- 2025-11-23 (v0.1.0) — Initial draft of instanced rendering specification.
