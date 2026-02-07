---
title: "Offscreen Render Targets and Multipass Rendering"
document_id: "offscreen-render-targets-2025-11-25"
status: "draft"
created: "2025-11-25T00:00:00Z"
last_updated: "2026-02-07T00:00:00Z"
version: "0.2.6"
engine_workspace_version: "2023.1.30"
wgpu_version: "26.0.1"
shader_backend_default: "naga"
winit_version: "0.29.10"
repo_commit: "544444652b4dc3639f8b3e297e56c302183a7a0b"
owners: ["lambda-sh"]
reviewers: ["engine", "rendering"]
tags: ["spec", "rendering", "offscreen", "multipass"]
---

# Offscreen Render Targets and Multipass Rendering

Summary

- Defines an offscreen render-to-texture resource that produces a sampleable
  color texture.
- Extends the command-driven renderer so a pass begin selects a render
  destination: the presentation surface or an offscreen target.
- Defines the MSAA resolve model for offscreen targets so later passes sample a
  single-sample resolve texture.

## Table of Contents

- [Scope](#scope)
- [Terminology](#terminology)
- [Architecture Overview](#architecture-overview)
- [Design](#design)
- [Behavior](#behavior)
- [Validation and Errors](#validation-and-errors)
- [Constraints and Rules](#constraints-and-rules)
- [Performance Considerations](#performance-considerations)
- [Requirements Checklist](#requirements-checklist)
- [Verification and Testing](#verification-and-testing)
- [Compatibility and Migration](#compatibility-and-migration)
- [Changelog](#changelog)

## Scope

- Goals
  - Add a first-class offscreen target resource with one color output and
    optional depth.
  - Allow a pass begin command to select a destination: the surface or a
    specific offscreen target.
  - Enable multipass workflows where later passes sample from textures
    produced by earlier passes.
  - Provide validation and feature flags for render-target compatibility,
    sample count and format mismatches, and common configuration pitfalls.
- Non-Goals
  - Multiple render targets (MRT) per pass; a single color attachment per pass
    remains the default in this document.
  - A full framegraph scheduler; ordering remains the explicit command
    sequence.
  - Headless contexts without a presentation surface; the current design
    requires a window-backed `RenderContext`.
  - Vendor-specific optimizations beyond what `wgpu` exposes via limits and
    capabilities.

## Terminology

- Multi-sample anti-aliasing (MSAA): rasterization technique that stores
  multiple coverage samples per pixel and resolves them to a single color.
- Multiple render targets (MRT): rendering to more than one color attachment
  within a single pass.
- Presentation render target: window-backed render target that acquires and
  presents swapchain frames (see
  `lambda::render::targets::surface::WindowSurface`).
- Offscreen target: persistent resource that owns textures for render-to-
  texture workflows and exposes a sampleable color texture.
- Render destination: destination selected when beginning a render pass:
  the presentation surface or a specific offscreen target.
- Resolve texture: single-sample color texture produced by resolving an
  MSAA color attachment; this is the texture sampled by later passes.
- Multipass rendering: sequence of two or more render passes in a single
  frame where later passes consume the results of earlier passes (for example,
  post-processing or shadow map sampling).
- Ping-pong target: pair of offscreen render targets alternated between read
  and write roles across passes.

## Architecture Overview

`lambda-rs` exposes two render target concepts:

- `lambda::render::targets::surface::RenderTarget`: trait for acquiring and
  presenting frames from a window-backed surface.
- `lambda::render::targets::offscreen::OffscreenTarget`: persistent render-to-
  texture resource that owns textures and exposes a sampleable resolve color.

Terminology in this document:

- "Render target" refers to `lambda::render::targets::surface::RenderTarget`.
- The offscreen resource is `OffscreenTarget`.

Implementation notes:

- `RenderTarget`, `RenderTargetBuilder`, and `RenderTargetError` in the offscreen
  module are deprecated aliases for `OffscreenTarget`, `OffscreenTargetBuilder`,
  and `OffscreenTargetError` and MUST NOT be used in new code.

Data flow (setup → per-frame multipass):

```
RenderPassBuilder::new()
  .with_multi_sample(1 | 2 | 4 | 8)
  --> RenderPass
        └── RenderContext::attach_render_pass(...)

OffscreenTargetBuilder
  --> OffscreenTarget { resolve_texture, msaa_texture?, depth_texture? }
        └── RenderContext::attach_offscreen_target(...)

RenderPipelineBuilder::new()
  .with_multi_sample(...)
  --> RenderPipeline (built for a specific color format)

Per-frame commands:
  BeginRenderPassTo { render_pass, viewport, destination } // surface or offscreen
    SetPipeline / SetBindGroup / Draw...
  EndRenderPass
  (repeat for additional passes)
```

## Design

### API Surface

#### High-level layer (`lambda-rs`)

- Module `lambda::render::targets::offscreen` (offscreen resource)
  - `pub struct OffscreenTarget`
    - Represents a 2D offscreen destination with a single color output and
      optional depth attachment.
    - `OffscreenTarget::color_texture()` MUST return the single-sample resolve
      texture (even when MSAA is enabled on the destination).
  - `pub struct OffscreenTargetBuilder`
    - `pub fn new() -> Self`
    - `pub fn with_color(self, format: texture::TextureFormat, width: u32, height: u32) -> Self`
    - `pub fn with_depth(self, format: texture::DepthFormat) -> Self`
    - `pub fn with_multi_sample(self, samples: u32) -> Self`
    - `pub fn with_label(self, label: &str) -> Self`
    - `pub fn build(self, gpu: &Gpu) -> Result<OffscreenTarget, OffscreenTargetError>`
    - Defaults:
      - Offscreen targets MUST NOT auto-resize; applications rebuild targets
        when their desired size changes.
  - `pub enum OffscreenTargetError`
    - `MissingColorAttachment`
    - `InvalidSize { width: u32, height: u32 }`
    - `UnsupportedSampleCount { requested: u32 }`
    - `UnsupportedFormat { message: String }`
    - `DeviceError(String)`
  - Note: Deprecated aliases (`RenderTarget`, `RenderTargetBuilder`,
    `RenderTargetError`) exist for short-term source compatibility.

- Module `lambda::render::command`
  - Add explicit destination selection for pass begins:
    - `pub enum RenderDestination { Surface, Offscreen(ResourceId) }`
    - `RenderCommand::BeginRenderPassTo { render_pass, viewport, destination }`
    - `RenderCommand::BeginRenderPass { render_pass, viewport }` MUST remain
      and be equivalent to `BeginRenderPassTo { destination: Surface, ... }`.

- Module `lambda::render::RenderContext`
  - Add an offscreen target registry:
    - `pub fn attach_offscreen_target(&mut self, target: OffscreenTarget) -> ResourceId`
    - `pub fn get_offscreen_target(&self, id: ResourceId) -> &OffscreenTarget`

- Module `lambda::render::render_pass`
  - The pass description remains destination-agnostic (clear/load/store,
    depth/stencil ops, sample count, and `uses_color`).
  - Destination selection occurs in `BeginRenderPassTo`, not in the pass
    builder.

- Module `lambda::render::pipeline`
  - Pipelines with a fragment stage are built for one color target format.
  - `RenderPipelineBuilder::build` MUST treat its `surface_format` parameter as
    the active color target format:
    - Surface passes pass `RenderContext::surface_format()`.
    - Offscreen passes pass `OffscreenTarget::color_format()`.

- Module `lambda::render::texture`
  - `TextureBuilder::for_render_target` MUST create textures with usage flags
    suitable for both sampling and render attachments.

#### Platform layer (`lambda-rs-platform`)

- Module `lambda_platform::wgpu::texture`
  - Offscreen resolve textures MUST support both `RENDER_ATTACHMENT` and
    `TEXTURE_BINDING` usage.
  - Offscreen MSAA attachment textures MUST support `RENDER_ATTACHMENT` usage.
- Module `lambda_platform::wgpu::pipeline`
  - Pipelines use `RenderPipelineBuilder::with_color_target` to declare the
    active color target format.
- Module `lambda_platform::wgpu::render_pass`
  - Existing `RenderColorAttachments` already supports arbitrary texture views,
    including MSAA attachments with resolve views.

## Behavior

### Offscreen target creation and lifetime

- Creation
  - `OffscreenTargetBuilder::build` MUST fail when:
    - `with_color` was never called.
    - Width or height is zero.
    - The requested sample count is unsupported for the chosen color format.
    - The requested sample count is unsupported for the chosen depth format
      when depth is enabled.
- MSAA resolve model
  - When `sample_count == 1`, the destination owns a single-sample color
    texture that is both rendered into and sampled by later passes.
  - When `sample_count > 1`, the destination MUST own:
    - A multi-sampled color attachment texture used only as the render
      attachment.
    - A single-sample resolve texture used as the resolve destination and later
      sampled.
  - `OffscreenTarget::color_texture()` MUST return the single-sample resolve
    texture in both cases.
- Lifetime
  - When an offscreen target is attached to a `RenderContext` and referenced by
    id, the application MUST keep the target attached for as long as any
    commands reference that id.

### Render pass destination semantics

- Destination selection occurs in `RenderCommand::BeginRenderPassTo`.
- `RenderCommand::BeginRenderPass` is equivalent to `RenderDestination::Surface`.
- `RenderDestination::Surface`
  - Color attachment is the swapchain view (with optional MSAA resolve).
  - Depth attachment is the `RenderContext`-managed depth texture.
- `RenderDestination::Offscreen(target_id)`
  - Color attachment is the offscreen target:
    - When `sample_count == 1`, the resolve texture view.
    - When `sample_count > 1`, the MSAA attachment view with resolve to the
      resolve texture view.
  - Depth attachment is the offscreen depth texture view when present.
  - When the offscreen target has no depth attachment, depth and stencil
    operations MUST be rejected as configuration errors.
- Sample count
  - The pass sample count MUST equal the destination sample count.
  - The pipeline sample count MUST equal the pass sample count.

### Multipass flows

- Command ordering
  - Multipass rendering is expressed as multiple `BeginRenderPass` /
    `BeginRenderPassTo` / `EndRenderPass` pairs in a single command list.
  - Nested passes remain invalid and MUST be rejected by `RenderContext::render`.
- Data dependencies
  - Passes that render into an offscreen destination produce resolve textures
    that MAY be sampled in subsequent passes.
- Hazards
  - Sampling from a resolve texture while writing to that resolve texture in
    the same pass is undefined behavior and MUST NOT be supported.

### Pipeline and destination compatibility

- Color format
  - Pipelines with a fragment stage MUST be built for the destination color
    format:
    - Surface destinations use `RenderContext::surface_format()`.
    - Offscreen destinations use `OffscreenTarget::color_format()`.
- Depth format
  - When the pass requests stencil operations, the destination depth format
    MUST include a stencil aspect.
  - For offscreen destinations with a depth format, pipeline depth format MUST
    match the destination depth format.
- Sample count
  - Pipelines MUST match the pass sample count, and the pass sample count MUST
    match the destination sample count.

## Validation and Errors

### Always-on safeguards

- Reject zero-sized offscreen targets at build time.
- Treat `sample_count == 0` as `1` in builder APIs.

### Feature-gated validation

Crate: `lambda-rs`

- Granular feature:
  - `render-validation-render-targets`
    - Validates compatibility between:
      - `RenderDestination` selection at `BeginRenderPassTo`.
      - Offscreen target attachments (color + optional depth).
      - The active `RenderPass` description (sample count, depth/stencil ops).
      - The active `RenderPipeline` (color target presence, format, and sample
        count).
    - Checks MUST occur at pass begin and at `SetPipeline` time, not per draw.
    - Logs SHOULD include:
      - Missing depth attachment when depth or stencil ops are requested.
      - Color format mismatches between destination and pipeline.
    - Expected runtime cost is low to moderate.

Umbrella composition (crate: `lambda-rs`)

- `render-validation` MUST include `render-validation-render-targets`.

Build-type behavior

- Debug builds (`debug_assertions`) MAY enable offscreen validation.
- Release builds MUST keep offscreen validation disabled by default and enable
  it only via `render-validation-render-targets` (or umbrellas that include it).

Gating requirements

- Offscreen validation MUST be gated behind
  `cfg(any(debug_assertions, feature = "render-validation-render-targets"))`.
- Offscreen validation MUST NOT be gated behind umbrella feature names.

## Constraints and Rules

- Offscreen target constraints
  - Width and height MUST be strictly positive.
  - A destination produces exactly one color output.
  - Color formats MUST be limited to formats supported by `texture::TextureFormat`.
  - Depth formats MUST be limited to `texture::DepthFormat`.
  - Sample counts MUST be supported by the device for the chosen color and
    depth formats; the initial spec assumes {1, 2, 4, 8}.
  - When `sample_count > 1`, the destination MUST provide a single-sample
    resolve texture for sampling.
- Pass constraints
  - Each `BeginRenderPassTo` MUST select exactly one destination.
  - Nested `BeginRenderPass`/`BeginRenderPassTo`/`EndRenderPass` sequences
    remain invalid.
  - Viewport and scissor rectangles are expressed in destination-relative
    coordinates when an offscreen destination is selected.
- Pipeline constraints
  - Pipelines used with destinations that have color output MUST declare a
    color target (a fragment stage must be present).
  - Pipelines MUST match destination format and sample count.

## Performance Considerations

- Use reduced-resolution offscreen targets for expensive post-processing
  effects (for example, half-resolution bloom).
  - Rationale: Smaller render targets reduce fill-rate and bandwidth demands
    while preserving acceptable visual quality for blurred or combined passes.
- Reuse offscreen targets across frames instead of recreating them.
  - Rationale: Repeated allocation and destruction of GPU textures can fragment
    memory and increase driver overhead; long-lived targets amortize setup
    costs.
- Prefer sample count `1` for intermediate post-processing passes and limit
  multi-sampling to geometry passes.
  - Rationale: MSAA increases memory bandwidth and shader cost; geometric
    passes benefit most, while post-process passes typically do not.
- Pack related passes that use the same offscreen destination close together in
  the command stream.
  - Rationale: Grouping passes reduces state changes and keeps relevant
    resources warm in caches and descriptor pools.

## Requirements Checklist

- Functionality
  - [x] Offscreen target resource exists in
        `crates/lambda-rs/src/render/targets/offscreen.rs`.
  - [x] Rename public API to `OffscreenTarget` to avoid collision with
        `lambda::render::targets::surface::RenderTarget`.
  - [x] Add `RenderDestination` and `RenderCommand::BeginRenderPassTo`.
  - [x] Add `RenderContext::{attach,get}_offscreen_target`.
  - [x] Support offscreen destinations in `RenderContext::render`.
  - [x] Implement offscreen MSAA resolve textures (render to MSAA, resolve to
        single-sample, sample resolve).
  - [x] Ensure offscreen depth sample count matches destination sample count.
- API Surface
  - [x] Platform pipeline supports explicit color targets.
  - [x] Engine `TextureBuilder::for_render_target` sets attachment-capable usage.
- Validation and Errors
  - [x] `render-validation-render-targets` feature implemented and composed
        into umbrella validation features.
  - [x] Pass/pipeline/destination compatibility checks implemented.
  - [x] `docs/features.md` updated to list the feature, default state, and cost.
- Documentation and Examples
  - [x] Minimal render-to-texture example added under `demos/render/src/bin/offscreen_post.rs`.
  - [ ] Rendering guide updated to include an offscreen multipass walkthrough.
  - [ ] Migration notes added for consumers adopting destination-based passes.

## Verification and Testing

- Unit tests
  - Offscreen target builder validation:
    - Invalid sizes.
    - Unsupported sample counts for color and depth formats.
    - Resolve texture usage flags suitable for attachment and sampling.
  - Destination validation:
    - Surface versus offscreen attachment selection at `BeginRenderPassTo`.
    - Sample count mismatch handling (destination, pass, pipeline).
    - Depth/stencil requested with no offscreen depth attachment.
  - Commands: `cargo test --workspace`
- Integration tests and examples
  - Render-to-texture example:
    - Pass 1: scene → offscreen destination.
    - Pass 2: fullscreen quad sampling `offscreen.color_texture()` → surface.
  - Commands: `cargo run -p lambda-demos-render --bin offscreen_post`
- Manual checks
  - Visual confirmation that:
    - Offscreen-only passes do not produce visible output until sampled.
    - Misconfigured formats or sample counts emit actionable validation logs
      when validation features are enabled.

## Compatibility and Migration

- Existing surface-only command streams remain valid:
  - `RenderCommand::BeginRenderPass` continues to target the surface.
  - Pipelines built against `RenderContext::surface_format()` remain compatible.
- Migration path
  - Create and attach one offscreen target.
  - Render to it using `RenderCommand::BeginRenderPassTo` with
    `RenderDestination::Offscreen(target_id)`.
  - Sample `offscreen.color_texture()` in a later surface pass.
- Naming migration
  - `RenderTarget` (offscreen resource) is a deprecated alias for
    `OffscreenTarget` and SHOULD remain available until a major version bump.

## Changelog

- 2026-02-05 (v0.2.6) — Update demo and example references for `demos/`.
- 2025-12-29 (v0.2.5) — Remove references to `lambda::render::target` and
  `lambda::render::render_target` compatibility shims; document
  `lambda::render::targets::{surface,offscreen}` as the canonical module
  layout.
- 2025-12-25 (v0.2.4) — Decouple `OffscreenTargetBuilder::build` from
  `RenderContext` by requiring an explicit size and a `Gpu`.
- 2025-12-22 (v0.2.3) — Document `lambda::render::targets::{surface,offscreen}`
  as the canonical module structure and note compatibility shims.
- 2025-12-22 (v0.2.2) — Update checklist and implementation notes to reflect
  destination-based offscreen passes, MSAA resolve targets, and validation
  feature wiring.
- 2025-12-17 (v0.2.1) — Polish language for style consistency, clarify MSAA
  terminology and builder safeguards, and specify validation gating
  requirements.
- 2025-12-17 (v0.2.0) — Align terminology with
  `lambda::render::targets::surface::RenderTarget`,
  specify destination-based pass targeting, define the offscreen MSAA resolve
  model, and define feature-gated validation requirements.
- 2025-11-25 (v0.1.1) — Updated requirements checklist to reflect implemented
  engine texture builder helpers and aligned metadata with current workspace
  revision.
- 2025-11-25 (v0.1.0) — Initial draft specifying offscreen render targets,
  multipass semantics, high-level and platform API additions, validation
  behavior, and testing expectations.
