---
title: "Cargo Features Overview"
document_id: "features-2025-11-17"
status: "living"
created: "2025-11-17T23:59:00Z"
last_updated: "2026-01-30T22:10:39Z"
version: "0.1.7"
engine_workspace_version: "2023.1.30"
wgpu_version: "26.0.1"
shader_backend_default: "naga"
winit_version: "0.29.10"
repo_commit: "939c07cf5c7ff583559c60ea7bb571961579f12b"
owners: ["lambda-sh"]
reviewers: ["engine", "rendering"]
tags: ["guide", "features", "validation", "cargo", "audio"]
---

## Overview
This document enumerates the primary Cargo features exposed by the workspace
relevant to rendering, validation, and audio behavior. It defines defaults,
relationships, and expected behavior in debug and release builds.

## Table of Contents
- [Overview](#overview)
- [Defaults](#defaults)
- [`lambda-rs`](#lambda-rs)
- [`lambda-rs-platform`](#lambda-rs-platform)
- [Changelog](#changelog)

## Defaults
- Workspace defaults prefer `wgpu` on supported platforms and `naga` for shader compilation.
- Debug builds enable all validations unconditionally via `debug_assertions`.
- Release builds enable only cheap safety checks by default; validation logs and per-draw checks MUST be enabled explicitly via features.
- Audio features are disabled by default and incur runtime cost only when an
  audio device is initialized and kept alive.

## `lambda-rs`

Rendering backends
- `with-wgpu` (default): enables the `wgpu` platform backend via
  `lambda-rs-platform/wgpu`.
- Platform specializations:
  - `with-wgpu-vulkan`: enables the Vulkan backend via
    `lambda-rs-platform/wgpu-with-vulkan`.
  - `with-wgpu-metal`: enables the Metal backend via
    `lambda-rs-platform/wgpu-with-metal`.
  - `with-wgpu-dx12`: enables the DirectX 12 backend via
    `lambda-rs-platform/wgpu-with-dx12`.
  - `with-wgpu-gl`: enables the OpenGL/WebGL backend via
    `lambda-rs-platform/wgpu-with-gl`.
- Convenience aliases:
  - `with-vulkan`: alias for `with-wgpu` and `with-wgpu-vulkan`.
  - `with-metal`: alias for `with-wgpu` and `with-wgpu-metal`.
  - `with-dx12`: alias for `with-wgpu` and `with-wgpu-dx12`.
  - `with-opengl`: alias for `with-wgpu` and `with-wgpu-gl`.
  - `with-dx11`: alias for `with-wgpu`.

Audio
- `audio` (umbrella, disabled by default): enables audio support by composing
  granular audio features. This umbrella includes `audio-output-device`.
- `audio-output-device` (granular, disabled by default): enables audio output
  device enumeration and callback-based audio output via `lambda::audio`. This
  feature enables `lambda-rs-platform/audio-device` internally. Expected
  runtime cost is proportional to the output callback workload and buffer size;
  no runtime cost is incurred unless an `AudioOutputDevice` is built and kept
  alive.

Render validation

Umbrella features (crate: `lambda-rs`)
- `render-validation`: enables common builder/pipeline validation logs (MSAA counts, depth clear advisories, stencil format upgrades, render-target compatibility) by composing granular validation features. This umbrella includes `render-validation-msaa`, `render-validation-depth`, `render-validation-stencil`, `render-validation-pass-compat`, and `render-validation-render-targets`.
- `render-validation-strict`: includes `render-validation` and enables per-draw SetPipeline-time compatibility checks by composing additional granular encoder features. This umbrella additionally enables `render-validation-encoder`.
- `render-validation-all`: superset of `render-validation-strict` and enables device-probing advisories and instancing validation. This umbrella includes all granular render-validation flags, including `render-validation-instancing`.

Granular features (crate: `lambda-rs`)
- `render-validation-msaa`: validates/logs MSAA sample counts; logs pass/pipeline sample mismatches. Behavior:
  - Builder APIs clamp invalid MSAA counts to `1`.
  - Pipelines align `sample_count` to the pass `sample_count`.
- `render-validation-depth`: logs when clamping depth clear to `[0.0, 1.0]`; adds depth usage advisories when a pass has depth but the pipeline does not.
- `render-validation-stencil`: logs when enabling stencil requires upgrading the depth format to `Depth24PlusStencil8`; warns about stencil usage mismatches.
- `render-validation-pass-compat`: SetPipeline-time errors when color targets or depth/stencil expectations do not match the active pass.
- `render-validation-device`: device/format probing advisories (if available via the platform layer).
- `render-validation-encoder`: additional per-draw/encoder-time checks; highest runtime cost.
- `render-validation-instancing`: instance-range and per-instance buffer binding validation for `RenderCommand::Draw` and `RenderCommand::DrawIndexed`. Behavior:
  - Validates that `instances.start <= instances.end` and treats `start == end` as a no-op (draw is skipped).
  - Ensures that all vertex buffer slots marked as per-instance on the active pipeline have been bound in the current render pass.
  - Adds per-draw checks proportional to the number of instanced draws and per-instance slots; SHOULD be enabled only when diagnosing instancing issues.
- `render-validation-render-targets`: validates compatibility between offscreen `RenderTarget`s, `RenderPass` descriptions, and `RenderPipeline`s. Behavior:
  - Verifies that pass and pipeline color formats and sample counts match the selected render target.
  - Emits configuration logs when a pass targets an offscreen surface with significantly different size from the presentation surface or when a target lacks the attachments implied by pass configuration.
  - Expected runtime cost is low to moderate; checks run at builder time and at the start of each pass, not per draw.

Always-on safeguards (debug and release)
- Clamp depth clear values to `[0.0, 1.0]`.
- Align pipeline `sample_count` to the active pass `sample_count`.
- Clamp invalid MSAA sample counts to `1`.

Behavior by build type
- Debug (`debug_assertions`): all validations active regardless of features.
- Release: validations are active only when the corresponding feature is enabled; safeguards above remain active.

Usage examples
- Enable common validations in release:
  - `cargo build -p lambda-rs --features render-validation`
- Enable strict compatibility checks in release:
  - `cargo run -p lambda-rs --features render-validation-strict`
- Enable all validations, including device advisories and instancing validation, in release:
  - `cargo test -p lambda-rs --features render-validation-all`
- Enable only MSAA validation in release:
  - `cargo test -p lambda-rs --features render-validation-msaa`

## `lambda-rs-platform`

This crate provides platform and dependency abstractions for `lambda-rs`.
Applications MUST NOT depend on `lambda-rs-platform` directly.

Rendering backend
- `wgpu` (default): enables the `wgpu` backend.
- `wgpu-with-vulkan`: enables Vulkan support.
- `wgpu-with-metal`: enables Metal support.
- `wgpu-with-dx12`: enables DirectX 12 support.
- `wgpu-with-gl`: enables OpenGL/WebGL support.

Shader backends
- `shader-backend-naga` (default): uses `naga` for shader handling.

Audio
- `audio` (umbrella, disabled by default): enables platform audio support by
  composing granular platform audio features. This umbrella includes
  `audio-device`.
- `audio-device` (granular, disabled by default): enables the internal audio
  backend module `lambda_platform::cpal` backed by `cpal =0.17.1`.

## Changelog
- 0.1.7 (2026-01-30): Group features by crate and document audio feature flags.
- 0.1.6 (2026-01-25): Remove the deprecated legacy shader backend
  documentation.
- 0.1.5 (2025-12-22): Align `lambda-rs` Cargo feature umbrella composition with
  the documented render-validation feature set, including `render-validation-pass-compat`
  and `render-validation-render-targets`.
- 0.1.4 (2025-11-25): Document `render-validation-render-targets`, record its inclusion in the `render-validation` umbrella feature, and update metadata.
- 0.1.3 (2025-11-25): Rename the instancing validation feature to `render-validation-instancing`, clarify umbrella composition, and update metadata.
- 0.1.2 (2025-11-25): Clarify umbrella versus granular validation features, record that `render-validation-all` includes `render-instancing-validation`, and update metadata.
- 0.1.1 (2025-11-25): Document `render-instancing-validation` behavior and update metadata.
- 0.1.0 (2025-11-17): Initial document introducing validation features and behavior by build type.
