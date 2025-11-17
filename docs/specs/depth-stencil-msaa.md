---
title: "Depth/Stencil and Multi-Sample Rendering"
document_id: "depth-stencil-msaa-2025-11-11"
status: "draft"
created: "2025-11-11T00:00:00Z"
last_updated: "2025-11-17T00:19:24Z"
version: "0.2.0"
engine_workspace_version: "2023.1.30"
wgpu_version: "26.0.1"
shader_backend_default: "naga"
winit_version: "0.29.10"
repo_commit: "ceaf345777d871912b2f92ae629a34b8e6f8654a"
owners: ["lambda-sh"]
reviewers: ["engine", "rendering"]
tags: ["spec", "rendering", "depth", "stencil", "msaa"]
---

# Depth/Stencil and Multi-Sample Rendering

Summary
- Add configurable depth testing/writes and multi-sample anti-aliasing (MSAA)
  to the high-level rendering API via builders, without exposing `wgpu` types.
- Provide validation and predictable defaults to enable 3D scenes and
  higher-quality rasterization in example and production code.

## Scope

- Goals
  - Expose depth/stencil and multi-sample configuration on `RenderPassBuilder`
    and `RenderPipelineBuilder` using engine/platform types; `wgpu` types are
    not exposed.
  - Validate device capabilities and configuration consistency at build time.
  - Define defaults for depth clear, compare operation, and sample count.
  - Map high-level configuration to `lambda-rs-platform` and `wgpu` internally.
- Non-Goals
  - Advanced per-draw depth bias configuration.
  - Post-process or temporal anti-aliasing techniques.
  - Vendor-specific tuning beyond standard device limits.

## Terminology

- Multi-sample anti-aliasing (MSAA): rasterization technique that stores
  multiple coverage samples per pixel and resolves them to a single color.
- Depth/stencil attachment: a `GPU` texture used for depth testing and optional
  stencil operations.
- Sample count: number of samples per pixel for targets and pipelines.
- Resolve: the operation that produces a single-sample color target from a
  multi-sampled color target at the end of a pass.

## Architecture Overview

- High-level builders in `lambda-rs` collect depth/stencil and multi-sample
  configuration using engine/platform types.
- `lambda-rs-platform` translates those types into backend-specific
  representations for `wgpu` creation of textures, passes, and pipelines.

```
App Code
  └── lambda-rs (RenderPassBuilder / RenderPipelineBuilder)
        └── DepthStencil + MultiSample config (engine/platform types)
              └── lambda-rs-platform (mapping/validation)
                    └── wgpu device/pipeline/pass
```

## Design

- API Surface
  - Types (engine-level)
    - `enum DepthFormat { Depth32Float, Depth24Plus, Depth24PlusStencil8 }`
    - `enum CompareFunction { Never, Less, LessEqual, Greater, GreaterEqual, Equal, NotEqual, Always }`
    - `struct MultiSample { sample_count: u32 }` (MUST be >= 1 and supported)
    - Stencil per-face state and operations exist at the platform layer and are exposed through `RenderPipelineBuilder::with_stencil(...)`.
  - Builders (selected functions)
    - `RenderPassBuilder::with_clear_color([f64; 4]) -> Self`
    - `RenderPassBuilder::with_depth() -> Self`
    - `RenderPassBuilder::with_depth_clear(f64) -> Self`
    - `RenderPassBuilder::with_stencil() -> Self`
    - `RenderPassBuilder::with_stencil_clear(u32) -> Self`
    - `RenderPassBuilder::with_multi_sample(u32) -> Self`
    - `RenderPipelineBuilder::with_depth_format(DepthFormat) -> Self`
    - `RenderPipelineBuilder::with_depth_compare(CompareFunction) -> Self`
    - `RenderPipelineBuilder::with_depth_write(bool) -> Self`
    - `RenderPipelineBuilder::with_stencil(StencilState) -> Self`
    - `RenderPipelineBuilder::with_multi_sample(u32) -> Self`
  - Example (engine types only)
    ```rust
    use lambda::render::render_pass::RenderPassBuilder;
    use lambda::render::pipeline::{RenderPipelineBuilder, CompareFunction};
    use lambda::render::texture::DepthFormat;

    let pass = RenderPassBuilder::new()
      .with_clear_color([0.0, 0.0, 0.0, 1.0])
      .with_depth_clear(1.0)
      .with_multi_sample(4)
      .build(&render_context);

    let pipeline = RenderPipelineBuilder::new()
      .with_multi_sample(4)
      .with_depth_format(DepthFormat::Depth32Float)
      .with_depth_compare(CompareFunction::Less)
      .build(&mut render_context, &pass, &vertex_shader, Some(&fragment_shader));
    ```
- Behavior
  - Defaults
    - If depth is not requested on the pass (`with_depth*`), the pass MUST NOT
      create a depth attachment and depth testing is disabled.
    - Depth clear defaults to `1.0` when depth is enabled on the pass and no
      explicit clear is provided.
    - Pipeline depth compare defaults to `CompareFunction::Less` when depth is
      enabled for a pipeline and no explicit compare is provided.
    - `MultiSample.sample_count` defaults to `1` (no multi-sampling).
  - Attachment creation
    - When depth is requested (`with_depth`/`with_depth_clear`), the pass MUST
      create a depth attachment. When stencil operations are requested on the
      pass (`with_stencil`/`with_stencil_clear`), the pass MUST attach a
      depth/stencil view and the depth format MUST include a stencil aspect.
    - If stencil is requested but the current depth format lacks a stencil
      aspect, the engine upgrades to `Depth24PlusStencil8` at pass build time
      or during encoding and logs an error.
    - The pass MUST clear the depth aspect to `1.0` by default (or the provided
      value) and clear/load stencil according to the requested ops.
- Multi-sample semantics
  - When `sample_count > 1`, the pass MUST render into a multi-sampled color
    target and resolve to the single-sample swap chain target before present.
    - The pipeline `sample_count` MUST equal the pass `sample_count`. If a
      mismatch is detected during pipeline build, the engine aligns the pipeline
      to the pass sample count and logs an error.
  - Matching constraints
    - If a pipeline declares a depth format, it MUST equal the pass depth
      attachment format. Mismatches are errors at build time. When a pipeline
      enables stencil, the engine upgrades its depth format to
      `Depth24PlusStencil8` to guarantee compatibility.

## Validation and Errors

- Validation is performed in `lambda-rs` during builder configuration and
  `build(...)`. Current behavior prefers logging and safe fallbacks over
  returning errors to preserve API stability.
- Multi-sample count validation
  - Allowed counts: 1, 2, 4, 8. Other values are rejected with an error log and
    clamped to `1` during `with_multi_sample(...)`.
  - On pipeline build, if the pipeline sample count differs from the pass, the
    engine aligns the pipeline to the pass and logs an error.
- Depth clear validation
  - Clear values outside `[0.0, 1.0]` SHOULD be rejected; current engine path
    relies on caller-provided sane values and `wgpu` validation. A strict check
    MAY be added in a follow-up.

## Constraints and Rules

- Multi-sample `sample_count` MUST be one of the device-supported counts. It is
  typically {1, 2, 4, 8}. Non-supported counts MUST be rejected.
- `Depth24Plus` and `Depth24PlusStencil8` MAY be emulated by the backend. The
  platform layer MUST query support before allocation.
- Depth clear values MUST be clamped to [0.0, 1.0] during validation.
- When the pass has no depth attachment, pipelines MUST behave as if depth
  testing and depth writes are disabled.

## Performance Considerations

- Use 4x multi-sampling by default for higher quality at moderate cost.
  - Rationale: 4x is widely supported and balances quality and performance.
- Prefer `Depth24Plus` for memory savings when stencil is not required.
  - Rationale: `Depth32Float` increases memory bandwidth and storage.
- Disable depth writes (`write = false`) for purely transparent or overlay
  passes.
  - Rationale: Skips unnecessary bandwidth and improves parallelism.

## Requirements Checklist

- Functionality
  - [x] Depth testing: enable/disable, clear, compare; depth write toggle
        (engine: `RenderPipelineBuilder::with_depth`, `.with_depth_clear`,
        `.with_depth_compare`, `.with_depth_write`)
  - [x] Stencil: clear/load/store, per-face ops, read/write mask, reference
        (platform stencil state; pass-level ops + `SetStencilReference`)
  - [x] MSAA: sample count selection, resolve path, depth sample matching
  - [x] Format selection: `Depth32Float`, `Depth24Plus`, `Depth24PlusStencil8`
  - [x] Edge cases: invalid sample counts (clamp/log), pass/pipeline sample
        mismatches (align/log); stencil implies stencil-capable format (upgrade)
- API Surface
  - [x] RenderPassBuilder: color ops, depth ops, stencil ops, MSAA
  - [x] RenderPipelineBuilder: depth format/compare, stencil state, depth write,
        MSAA
  - [x] Commands: set stencil reference; existing draw/bind/viewport remain
- Validation and Errors
  - [ ] Sample counts limited to {1,2,4,8}; invalid → log + clamp to 1
  - [ ] Pass/pipeline sample mismatch → align to pass + log
  - [ ] Depth clear in [0.0, 1.0] (SHOULD validate); device support (SHOULD)
- Performance
  - [ ] 4x MSAA guidance; memory trade-offs for `Depth32Float` vs `Depth24Plus`
  - [ ] Recommend disabling depth writes for overlays/transparency
- Documentation and Examples
  - [ ] Minimal MSAA + depth example
  - [ ] Reflective mirror (stencil) tutorial
  - [ ] Migration notes (none; additive API)

For each checked item, include a reference to a commit, pull request, or file
path that demonstrates the implementation.

## Verification and Testing

- Unit Tests
  - Validate mapping of engine types to platform/wgpu types.
  - Validate rejection of unsupported sample counts and format mismatches.
  - Commands: `cargo test -p lambda-rs -- --nocapture`
- Integration Tests
  - Render a depth-tested scene (e.g., overlapping cubes) at sample counts of 1
    and 4; verify occlusion and smoother edges when multi-sampling is enabled.
  - Commands: `cargo test --workspace`
- Manual Checks (if necessary)
  - Run `cargo run --example minimal` with a toggle for multi-sampling and
    observe aliasing reduction with 4x multi-sampling.

## Compatibility and Migration

- No breaking changes. New configuration is additive and does not expose `wgpu`
  types in the high-level API. Existing examples continue to render with
  defaults (no depth, no multi-sampling) unless explicitly configured.

## Changelog

- 2025-11-11 (v0.1.1) — Add MSAA validation in builders; align pipeline and
  pass sample counts; document logging-based fallback semantics.
- 2025-11-11 (v0.1.0) — Initial draft.
