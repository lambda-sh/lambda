---
title: "Depth/Stencil and Multi-Sample Rendering"
document_id: "depth-stencil-msaa-2025-11-11"
status: "draft"
created: "2025-11-11T00:00:00Z"
last_updated: "2025-11-11T00:00:00Z"
version: "0.1.0"
engine_workspace_version: "2023.1.30"
wgpu_version: "26.0.1"
shader_backend_default: "naga"
winit_version: "0.29.10"
repo_commit: "c37e14cfa5fe220557da5e62aa456e42f1d34383"
owners: ["lambda-sh"]
reviewers: ["engine", "rendering"]
tags: ["spec", "rendering", "depth", "stencil", "msaa"]
---

# Depth/Stencil and Multi-Sample Rendering

Summary
- Add configurable depth testing/writes and multi-sample anti-aliasing (MSAA)
  to the high-level rendering API via builders, without exposing `wgpu` types.
- Provide strict validation at build time and predictable defaults to enable
  3D scenes and higher-quality rasterization in example and production code.

## Scope

- Goals
  - Expose depth/stencil and multi-sample configuration on `RenderPassBuilder`
    and `RenderPipelineBuilder` using `lambda-rs` types only.
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
  configuration using engine-defined types.
- `lambda-rs-platform` translates those types into backend-specific
  representations for `wgpu` creation of textures, passes, and pipelines.

```
App Code
  └── lambda-rs (RenderPassBuilder / RenderPipelineBuilder)
        └── DepthStencil + MultiSample config (engine types)
              └── lambda-rs-platform (mapping/validation)
                    └── wgpu device/pipeline/pass
```

## Design

- API Surface
  - Types (engine-level)
    - `enum DepthFormat { Depth32Float, Depth24Plus, Depth24PlusStencil8 }`
    - `enum CompareFunction { Never, Less, LessEqual, Greater, GreaterEqual, Equal, NotEqual, Always }`
    - `struct DepthStencil { format: DepthFormat, clear_value: f32, write: bool, compare: CompareFunction, stencil: Option<StencilState> }`
    - `struct StencilState { read_mask: u32, write_mask: u32, reference: u32 }` (placeholder; operations MAY be extended in a follow-up)
    - `struct MultiSample { sample_count: u32 }` (MUST be >= 1 and supported)
  - Builders (selected functions)
    - `RenderPassBuilder::with_clear_color(Color) -> Self`
    - `RenderPassBuilder::with_depth_stencil(DepthStencil) -> Self`
    - `RenderPassBuilder::with_multi_sample(MultiSample) -> Self`
    - `RenderPipelineBuilder::with_depth_format(DepthFormat) -> Self`
    - `RenderPipelineBuilder::with_multi_sample(MultiSample) -> Self`
  - Example (engine types only)
    ```rust
    use lambda_rs::render::{Color, DepthFormat, CompareFunction, DepthStencil, MultiSample};

    let pass = RenderPassBuilder::new()
      .with_clear_color(Color::BLACK)
      .with_depth_stencil(DepthStencil {
        format: DepthFormat::Depth32Float,
        clear_value: 1.0,
        write: true,
        compare: CompareFunction::Less,
        stencil: None,
      })
      .with_multi_sample(MultiSample { sample_count: 4 })
      .build(&render_context)?;

    let pipeline = RenderPipelineBuilder::new()
      .with_multi_sample(MultiSample { sample_count: 4 })
      .with_depth_format(DepthFormat::Depth32Float)
      .build(&mut render_context, &pass, &vertex_shader, Some(&fragment_shader))?;
    ```
- Behavior
  - Defaults
    - If `with_depth_stencil` is not called, the pass MUST NOT create a depth
      attachment and depth testing is disabled.
    - `DepthStencil.clear_value` defaults to `1.0` (furthest depth).
    - `DepthStencil.compare` defaults to `CompareFunction::Less`.
    - `MultiSample.sample_count` defaults to `1` (no multi-sampling).
  - Attachment creation
    - When `with_depth_stencil` is provided, the pass MUST create a depth (and
      stencil, if the format includes stencil) attachment matching `format`.
    - The pass MUST clear the depth aspect to `clear_value` at the start of the
      pass. Stencil clear behavior is unspecified in this version and MAY be
      added when extended stencil operations are introduced.
  - Multi-sample semantics
    - When `sample_count > 1`, the pass MUST render into a multi-sampled color
      target and resolve to the single-sample swap chain target before present.
    - The pipeline `sample_count` MUST equal the pass `sample_count`.
  - Matching constraints
    - If a pipeline declares a depth format, it MUST equal the pass depth
      attachment format. Mismatches are errors at build time.

## Validation and Errors

- Validation is performed in `lambda-rs` during `build(...)` and by
  `lambda-rs-platform` against device limits.
- Error type: `RenderConfigurationError`
  - `UnsupportedMultiSampleCount { requested: u32, supported: Vec<u32> }`
  - `UnsupportedDepthFormat { format: DepthFormat }`
  - `DepthFormatMismatch { pass: DepthFormat, pipeline: DepthFormat }`
  - `InvalidDepthClearValue { value: f32 }` (MUST be in [0.0, 1.0])
  - `StencilUnsupported { format: DepthFormat }`
  - `DeviceLimitExceeded { detail: String }`

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
  - [ ] Feature flags defined (if applicable)
  - [ ] Core behavior implemented
  - [ ] Edge cases handled (unsupported sample counts, format mismatch, range checks)
- API Surface
  - [ ] Public types and builders implemented
  - [ ] Commands/entry points exposed
  - [ ] Backwards compatibility assessed
- Validation and Errors
  - [ ] Input validation implemented
  - [ ] Device/limit checks implemented
  - [ ] Error reporting specified and implemented
- Performance
  - [ ] Critical paths profiled or reasoned
  - [ ] Memory usage characterized
  - [ ] Recommendations documented
- Documentation and Examples
  - [ ] User-facing docs updated
  - [ ] Minimal example(s) added/updated
  - [ ] Migration notes (if applicable)

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

- 2025-11-11 (v0.1.0) — Initial draft.
