---
title: "Offscreen Render Targets and Multipass Rendering"
document_id: "offscreen-render-targets-2025-11-25"
status: "draft"
created: "2025-11-25T00:00:00Z"
last_updated: "2025-11-25T00:00:00Z"
version: "0.1.0"
engine_workspace_version: "2023.1.30"
wgpu_version: "26.0.1"
shader_backend_default: "naga"
winit_version: "0.29.10"
repo_commit: "1cca6ebdf7cb0b786b3c46561b60fa2e44eecea4"
owners: ["lambda-sh"]
reviewers: ["engine", "rendering"]
tags: ["spec", "rendering", "offscreen", "multipass"]
---

# Offscreen Render Targets and Multipass Rendering

Summary
- Introduces offscreen render targets as first-class resources in `lambda-rs`
  for render-to-texture workflows (post-processing, shadow maps, UI
  composition).
- Defines multipass rendering semantics and API changes so passes can write to
  and sample from offscreen targets without exposing `wgpu` types.
- Preserves existing builder and command patterns while extending
  `lambda-rs-platform` to support textures that are both render attachments and
  sampled resources.

## Scope

### Goals

- Add first-class offscreen render targets with color and optional depth
  attachments in `lambda-rs`.
- Allow render passes to target either the presentation surface or an offscreen
  render target.
- Enable multipass workflows where later passes sample from textures produced
  by earlier passes.
- Provide validation and feature flags for render-target compatibility, sample
  count and format mismatches, and common configuration pitfalls.

### Non-Goals

- Multiple simultaneous color attachments (MRT) per pass; a single color
  attachment per pass remains the default in this specification.
- Compute pipelines, storage textures, and general framegraph scheduling;
  separate specifications cover these areas.
- Headless contexts without a presentation surface; this specification assumes
  a window-backed `RenderContext`.
- Vendor-specific optimizations beyond what `wgpu` exposes via limits and
  capabilities.

## Terminology

- Offscreen render target: A 2D color texture with an optional depth attachment
  that can be bound as a render attachment but is not presented directly to the
  window surface.
- Render target: Either the default presentation surface or an offscreen render
  target.
- Multipass rendering: A sequence of two or more render passes in a single
  frame where later passes consume the results of earlier passes (for example,
  post-processing or shadow map sampling).
- Default render target: The swapchain-backed surface associated with a
  `RenderContext`.
- Ping-pong target: A pair of offscreen render targets alternated between read
  and write roles across passes.

## Architecture Overview

- High-level (`lambda-rs`)
  - Introduces `RenderTarget` and `RenderTargetBuilder` in
    `lambda::render::target` to construct offscreen color (and optional depth)
    attachments sized independently of the window.
  - Extends `RenderPassBuilder` so a pass can declare a target: the default
    surface or a specific `RenderTarget`.
  - Extends `RenderPipelineBuilder` so pipelines can declare the expected color
    format independently of the surface while still aligning sample counts and
    depth formats with the active pass.
- Platform (`lambda-rs-platform`)
  - Extends `TextureBuilder` to create textures that include
    `RENDER_ATTACHMENT` usage in addition to sampling usage.
  - Reuses the existing render pass and pipeline builders to bind offscreen
    texture views as color attachments and to configure color target formats
    from texture formats instead of only surface formats.

Data flow (setup → per-frame multipass):
```
RenderTargetBuilder
  --> RenderTarget { color_texture, depth_format, sample_count }
        └── bound into bind groups for sampling

RenderPassBuilder::new()
  .with_target(&offscreen)         // or default surface
  .with_depth_clear(1.0)          // optional depth ops
  .with_multi_sample(1 | 2 | 4 | 8)
  --> RenderPass
        └── RenderContext::attach_render_pass(...)

RenderPipelineBuilder::new()
  .with_color_format(TextureFormat::Rgba8UnormSrgb)
  .with_depth_format(DepthFormat::Depth32Float)
  .with_multi_sample(...)
  --> RenderPipeline

Per-frame commands:
  BeginRenderPass { pass_id, viewport }  // surface or offscreen target
    SetPipeline / SetBindGroup / Draw...
  EndRenderPass
  (repeat for additional passes)
```

## Design

### API Surface

#### High-level layer (`lambda-rs`)

- Module `lambda::render::target`
  - `pub struct RenderTarget`
    - Represents a 2D offscreen render target with a single color attachment
      and an optional depth attachment.
    - Encapsulates texture size, color format, depth format (if any), and
      sample count.
    - Exposes immutable accessors for binding in shaders and builders:
      - `pub fn size(&self) -> (u32, u32)`
      - `pub fn color_format(&self) -> texture::TextureFormat`
      - `pub fn depth_format(&self) -> Option<texture::DepthFormat>`
      - `pub fn sample_count(&self) -> u32`
      - `pub fn color_texture(&self) -> &texture::Texture`
    - Provides explicit destruction:
      - `pub fn destroy(self, render_context: &mut RenderContext)`
  - `pub struct RenderTargetBuilder`
    - Builder for constructing `RenderTarget` values.
    - API:
      - `pub fn new() -> Self`
      - `pub fn with_color(mut self, format: texture::TextureFormat, width: u32, height: u32) -> Self`
      - `pub fn with_depth(mut self, format: texture::DepthFormat) -> Self`
      - `pub fn with_multi_sample(mut self, samples: u32) -> Self`
      - `pub fn with_label(mut self, label: &str) -> Self`
      - `pub fn build(self, render_context: &mut RenderContext) -> Result<RenderTarget, RenderTargetError>`
    - Behavior:
      - Fails with `RenderTargetError::MissingColorAttachment` when no color
        attachment was configured.
      - Fails with `RenderTargetError::InvalidSize` when width or height is
        zero.
      - Defaults:
        - Size defaults to the current surface size when not explicitly
          provided.
        - Sample count defaults to `1` (no multi-sampling).
  - `pub enum RenderTargetError`
    - `InvalidSize { width: u32, height: u32 }`
    - `UnsupportedSampleCount { requested: u32 }`
    - `UnsupportedFormat { message: String }`
    - `DeviceError(String)` for device-level failures returned by the platform
      layer.

- Module `lambda::render::render_pass`
  - Extend `RenderPassBuilder` with target selection:
    - `pub fn with_target(mut self, target: &RenderTarget) -> Self`
      - Configures the pass to use the provided `RenderTarget` color and depth
        attachments instead of the default surface and context-managed depth
        texture.
      - The pass inherits the target size and sample count; explicit
        `with_multi_sample` on the pass MUST align with the target sample count
        (see Behavior).
    - Existing methods (for example, `with_clear_color`, `with_depth_clear`,
      `with_stencil_clear`, `with_multi_sample`) remain unchanged and apply to
      the selected target.
  - Extend `RenderPass` to expose its target:
    - `pub(crate) fn uses_default_surface(&self) -> bool`
    - `pub(crate) fn target(&self) -> Option<RenderTarget>`
      - Used internally by `RenderContext` to choose attachments when encoding
        passes.

- Module `lambda::render::pipeline`
  - Extend `RenderPipelineBuilder` to allow explicit color format selection:
    - `pub fn with_color_format(mut self, format: texture::TextureFormat) -> Self`
      - Declares the color format expected by the fragment stage.
      - When omitted:
        - For surface-backed passes, defaults to the current surface format.
        - For offscreen passes with a `RenderTarget`, defaults to the target
          color format.
  - `RenderPipelineBuilder::build` behavior changes:
    - Derives the color target format from:
      - Explicit `with_color_format`, when provided.
      - Otherwise from the associated `RenderPass` target:
        - `RenderContext::surface_format()` for the default surface.
        - `RenderTarget::color_format()` for offscreen targets.
    - Retains depth and sample count alignment rules:
      - Depth format continues to be derived from `with_depth_format` or the
        pass depth attachment, including stencil upgrades.
      - Sample count is aligned to the pass sample count, as in the MSAA spec.

- Module `lambda::render::texture`
  - Extend `TextureBuilder` to support render-target usage:
    - `pub fn for_render_target(mut self) -> Self`
      - Marks the texture for combined sampling and render-attachment usage.
      - Maps to `TEXTURE_BINDING | RENDER_ATTACHMENT | COPY_SRC` at the
        platform layer.
    - Existing uses that do not call `for_render_target` continue to produce
      sampled-only textures.

- Module `lambda::render::command`
  - No new commands are required; multipass rendering continues to use
    `RenderCommand::BeginRenderPass` / `EndRenderPass` with different pass
    handles.
  - The semantics of `BeginRenderPass` change to:
    - If the referenced `RenderPass` uses the default surface, the pass writes
      to the swapchain (with optional MSAA resolve).
    - If the `RenderPass` references an offscreen `RenderTarget`, the pass
      writes to the target's color attachment (with optional depth).

- Module `lambda::render::RenderContext`
  - Extend internal state with an optional pool of offscreen resources owned by
    `RenderTarget`:
    - `render_targets` remains managed by application code via `RenderTarget`;
      `RenderContext` only borrows platform textures when encoding passes.
  - Expose the surface size for convenience:
    - `pub fn surface_size(&self) -> (u32, u32)`
      - Used by `RenderTargetBuilder::new()` as a default size when none is
        provided.

#### Platform layer (`lambda-rs-platform`)

- Module `lambda_platform::wgpu::texture`
  - Extend `TextureBuilder` usage flags:
    - Add internal `usage_render_attachment: bool` field.
    - Add `pub fn with_render_attachment_usage(mut self, enabled: bool) -> Self`
      - When enabled, include `wgpu::TextureUsages::RENDER_ATTACHMENT` in the
        created texture and its default view usage.
    - Offscreen color targets use:
      - `TEXTURE_BINDING | RENDER_ATTACHMENT | COPY_SRC` for flexible sampling
        and optional readback.
- Module `lambda_platform::wgpu::pipeline`
  - Extend `RenderPipelineBuilder`:
    - Add `pub fn with_color_target_format(mut self, format: texture::TextureFormat) -> Self`
      - Converts the texture format into a `wgpu::TextureFormat` and stores it
        as the color target format.
    - `with_surface_color_target` remains for surface-backed pipelines and is
      used when the color target should match the swapchain format.
- Module `lambda_platform::wgpu::render_pass`
  - No structural changes are required; existing `RenderColorAttachments`
    already accepts arbitrary `TextureView` references.
  - Offscreen passes provide `TextureViewRef` from `Texture` at pass-encode
    time.

### Behavior

#### RenderTarget creation and lifetime

- Creation
  - `RenderTargetBuilder::build` MUST fail when:
    - `with_color` was never called.
    - Width or height is zero.
    - The requested sample count is not supported by the device for the chosen
      format.
  - When no explicit size is set, the builder uses the current
    `RenderContext::surface_size()` as the color attachment size.
  - Depth is optional:
    - When `with_depth` is omitted, the target has no depth attachment.
    - When `with_depth` is provided, the target allocates a depth texture using
      `texture::DepthFormat`.
- Lifetime
  - `RenderTarget` owns its color (and optional depth) textures.
  - `RenderPassBuilder::with_target` clones the target handle; the application
    MUST keep the `RenderTarget` alive for as long as any attached passes and
    pipelines are used.
  - `RenderTarget::destroy` releases underlying resources; further use in
    passes is invalid and SHOULD be prevented by application code.

#### Render pass targeting semantics

- Default behavior (existing)
  - When `RenderPassBuilder` is used without `with_target`, the pass targets
    the presentation surface:
    - Color attachment: swapchain view (with optional MSAA resolve).
    - Depth attachment: `RenderContext`-managed depth texture.
- Offscreen behavior (new)
  - When `with_target(&offscreen)` is used:
    - Color attachment: offscreen target color texture view.
    - Depth attachment:
      - When the target has a depth format, a depth texture is allocated for
        the target and used as the pass depth attachment.
      - When the target has no depth format, depth is disabled unless the pass
        explicitly requests depth operations, in which case the pass MUST
        produce a configuration error.
    - Sample count:
      - The pass sample count MUST equal the target sample count.
      - When `with_multi_sample` is called with a different value, the pass
        aligns its sample count to `RenderTarget::sample_count()` and, under
        validation features, logs an error.
  - Color load/store operations, depth operations, and stencil operations apply
    to the offscreen attachments exactly as they apply to the surface-backed
    attachments.

#### Multipass flows

- Command ordering
  - Multipass rendering is expressed as multiple
    `BeginRenderPass`/`EndRenderPass` pairs in a single command list.
  - Nested passes remain invalid and MUST continue to be rejected by
    `RenderContext::encode_pass`.
- Data dependencies
  - Passes that render into an offscreen target produce textures that MAY be
    sampled in subsequent passes:
    - Typical pattern:
      - Pass 1: scene → offscreen color (and depth).
      - Pass 2: fullscreen quad sampling offscreen.color → surface.
  - The specification does not introduce an explicit framegraph; ordering is
    determined solely by the command sequence.
- Hazards
  - Writing to a `RenderTarget` and sampling from the same texture in the same
    pass is undefined behavior and MUST NOT be supported; validation MAY detect
    obvious cases but cannot guarantee all hazards are caught.
  - Using the same `RenderTarget` as the destination for multiple passes in one
    frame is supported; the clear/load operations on each pass determine
    whether results accumulate or overwrite.

#### Pipeline and target compatibility

- Color format
  - For surface-backed passes:
    - When `with_color_format` is omitted, the pipeline color format is derived
      from `RenderContext::surface_format()` (existing behavior).
    - When `with_color_format` is provided and differs from the surface
      format, pipeline creation MUST fail under `render-validation-pass-compat`
      or debug assertions.
  - For offscreen passes:
    - When `with_color_format` is omitted, the pipeline color format is derived
      from `RenderTarget::color_format()`.
    - When `with_color_format` is provided and differs from the target format,
      pipeline creation MUST either:
      - Fail configuration-time validation, or
      - Align to the target format and log an error under
        `render-validation-render-targets`.
- Depth format
  - Depth behavior follows the depth/stencil specification:
    - When the pass requests stencil operations, the depth format MUST include
      a stencil aspect; otherwise, the engine upgrades to
      `Depth24PlusStencil8` and logs an error under
      `render-validation-stencil`.
    - For offscreen targets with a depth format, pipeline depth format MUST
      match the target depth format; mismatches are treated as configuration
      errors or aligned with logging, consistent with the depth spec.
- Sample count
  - Pass and pipeline sample counts are aligned as in the MSAA specification:
    - Pipeline sample count is aligned to the pass sample count.
    - For offscreen passes, both pass and pipeline sample counts MUST equal the
      target sample count; invalid sample counts are clamped to `1` with
      validation logs.

### Validation and Errors

- Builder-level validation (always on)
  - `RenderTargetBuilder::build` MUST:
    - Reject zero width or height.
    - Clamp sample counts less than `1` to `1`.
  - `RenderPassBuilder::with_target` MUST ensure that a target has a color
    attachment; targets without color are invalid for the current specification
    (depth-only targets MAY be added later).
  - `RenderPipelineBuilder::build` MUST:
    - Validate that the chosen color format is supported for render attachments
      on the device.
- Runtime validation (feature-gated)
  - New granular feature (crate: `lambda-rs`):
    - `render-validation-render-targets`
      - Validates compatibility between `RenderTarget`, `RenderPass`, and
        `RenderPipeline`:
        - Verifies that pass and pipeline color formats match the target color
          format.
        - Verifies that pass and pipeline sample counts equal the target sample
          count.
        - Logs when a pass references a `RenderTarget` whose size differs
          significantly from the surface size (for example, for debug
          visibility issues).
      - Expected runtime cost: low to moderate; checks occur at pass/pipeline
        build time and pass begin time, not per draw.
  - Existing granular features:
    - `render-validation-pass-compat` continues to enforce SetPipeline-time
      compatibility checks and MUST be updated to consider offscreen targets.
    - `render-validation-msaa`, `render-validation-depth`,
      `render-validation-stencil`, and `render-validation-device` remain
      unchanged but apply equally to offscreen passes.
- Build-type behavior
  - Debug builds (`debug_assertions`):
    - All render-target validations are active regardless of feature flags.
  - Release builds:
    - Only cheap size and sample-count clamps are always on.
    - Detailed compatibility logs require `render-validation-render-targets` or
      the appropriate umbrella features.

## Constraints and Rules

- RenderTarget constraints
  - Width and height MUST be strictly positive.
  - Color formats are limited to `TextureFormat::Rgba8Unorm` and
    `TextureFormat::Rgba8UnormSrgb` in the initial implementation.
  - Depth formats are limited to `DepthFormat::Depth32Float`,
    `DepthFormat::Depth24Plus`, and `DepthFormat::Depth24PlusStencil8`.
  - Sample counts MUST be one of the device-supported values; the initial
    spec assumes {1, 2, 4, 8}.
- Pass constraints
  - A pass with an offscreen target MUST not also target the surface; the
    target is exclusive.
  - Nested `BeginRenderPass`/`EndRenderPass` sequences remain invalid.
  - Viewport and scissor rectangles are expressed in target-relative
    coordinates when an offscreen target is selected.
- Pipeline constraints
  - Pipelines used with offscreen passes MUST declare a color target; vertex-
    only pipelines without a fragment stage are not compatible with offscreen
    color passes in this revision.

## Performance Considerations

- Use reduced-resolution offscreen targets for expensive post-processing
  effects (for example, half-resolution bloom).
  - Rationale: Smaller render targets reduce fill-rate and bandwidth demands
    while preserving acceptable visual quality for blurred or combined passes.
- Reuse `RenderTarget` instances across frames instead of recreating them.
  - Rationale: Repeated allocation and destruction of GPU textures can fragment
    memory and increase driver overhead; long-lived targets amortize setup
    costs.
- Prefer sample count `1` for intermediate post-processing passes and limit
  multi-sampling to geometry passes.
  - Rationale: MSAA increases memory bandwidth and shader cost; geometric
    passes benefit most, while post-process passes typically do not.
- Pack related passes that use the same `RenderTarget` close together in the
  command stream.
  - Rationale: Grouping passes reduces state changes and keeps relevant
    resources warm in caches and descriptor pools.

## Requirements Checklist

- Functionality
  - [ ] `RenderTarget` and `RenderTargetBuilder` implemented in
        `crates/lambda-rs/src/render/target.rs`.
  - [ ] Offscreen targeting added to `RenderPassBuilder` and `RenderPass`.
  - [ ] Offscreen targeting supported in `RenderContext::render`.
  - [ ] Edge cases handled (invalid size, unsupported sample count, missing
        depth when required).
- API Surface
  - [ ] High-level public types and builders added in `lambda-rs`.
  - [x] Platform texture usage for render targets implemented in
        `lambda-rs-platform`.
  - [ ] Pipeline color target changes implemented in `lambda-rs-platform`.
  - [ ] Backwards compatibility assessed for existing surface-backed paths.
- Validation and Errors
  - [ ] `render-validation-render-targets` feature implemented and composed
        into umbrella validation features.
  - [ ] Pass/pipeline/target compatibility checks implemented.
  - [ ] Device limit checks for offscreen formats/sample counts implemented.
- Performance
  - [ ] Critical render-target creation paths profiled or reasoned about.
  - [ ] Memory usage for long-lived render targets characterized.
  - [ ] Performance recommendations validated against representative examples.
- Documentation and Examples
  - [ ] Rendering guide updated to include offscreen/multipass examples.
  - [ ] Minimal render-to-texture example added under
        `crates/lambda-rs/examples/`.
  - [ ] Migration notes added for consumers adopting offscreen targets.

## Verification and Testing

- Unit tests
  - `RenderTargetBuilder` validation:
    - Invalid sizes and sample counts.
    - Mapping to platform texture builder usage flags.
  - Pipeline color format selection:
    - Surface-backed vs offscreen-backed passes.
  - Commands:
    - `cargo test --workspace`
- Integration tests and examples
  - Render-to-texture example:
    - Pass 1: scene → offscreen.
    - Pass 2: fullscreen quad sampling offscreen → surface.
  - Shadow-map-style example:
    - Depth-only offscreen target feeding a lighting pass.
  - Commands:
    - `cargo run -p lambda-rs --example offscreen_post`
- Manual checks
  - Visual confirmation that:
    - Offscreen-only passes do not produce visible output until sampled.
    - Misconfigured formats or sample counts emit actionable validation logs
      when validation features are enabled.

## Compatibility and Migration

- The offscreen render target and multipass API is additive:
  - Existing code that uses only surface-backed passes and pipelines continues
    to compile and render unchanged.
  - New APIs are exposed via `RenderTargetBuilder`, `RenderPassBuilder`, and
    `RenderPipelineBuilder` methods; existing method signatures remain
    compatible.
- Consumers MAY adopt offscreen targets incrementally:
  - Start with post-processing or UI composition that samples a single
    offscreen color target.
  - Extend to depth-based passes (for example, shadow maps) when depth
    attachment support is implemented.

## Changelog

- 2025-11-25 (v0.1.0) — Initial draft specifying offscreen render targets,
  multipass semantics, high-level and platform API additions, validation
  behavior, and testing expectations.
