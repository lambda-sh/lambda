---
title: "Cargo Features Overview"
document_id: "features-2025-11-17"
status: "living"
created: "2025-11-17T23:59:00Z"
last_updated: "2025-11-25T00:00:00Z"
version: "0.1.1"
engine_workspace_version: "2023.1.30"
wgpu_version: "26.0.1"
shader_backend_default: "naga"
winit_version: "0.29.10"
repo_commit: "b1f0509d245065823dff2721f97e16c0215acc4f"
owners: ["lambda-sh"]
reviewers: ["engine", "rendering"]
tags: ["guide", "features", "validation", "cargo"]
---

## Overview
This document enumerates the primary Cargo features exposed by the workspace relevant to rendering and validation behavior. It defines defaults, relationships, and expected behavior in debug and release builds.

## Table of Contents
- [Overview](#overview)
- [Defaults](#defaults)
- [Rendering Backends](#rendering-backends)
- [Shader Backends](#shader-backends)
- [Render Validation](#render-validation)
- [Changelog](#changelog)

## Defaults
- Workspace defaults prefer `wgpu` on supported platforms and `naga` for shader compilation.
- Debug builds enable all validations unconditionally via `debug_assertions`.
- Release builds enable only cheap safety checks by default; validation logs and per-draw checks MUST be enabled explicitly via features.

## Rendering Backends
- `lambda-rs`
  - `with-wgpu` (default): enables the `wgpu` platform backend via `lambda-rs-platform`.
  - Platform specializations: `with-wgpu-vulkan`, `with-wgpu-metal`, `with-wgpu-dx12`, `with-wgpu-gl`.

## Shader Backends
- `lambda-rs-platform`
  - `shader-backend-naga` (default): uses `naga` for shader handling.
  - `shader-backend-shaderc`: uses `shaderc`; optional `shader-backend-shaderc-build-from-source`.

## Render Validation

Umbrella features (crate: `lambda-rs`)
- `render-validation`: enables common builder/pipeline validation logs (MSAA counts, depth clear advisories, stencil format upgrades).
- `render-validation-strict`: includes `render-validation` and enables per-draw SetPipeline-time compatibility checks.
- `render-validation-all`: superset of `render-validation-strict` and enables device-probing advisories.

Granular features (crate: `lambda-rs`)
- `render-validation-msaa`: validates/logs MSAA sample counts; logs pass/pipeline sample mismatches. Behavior:
  - Builder APIs clamp invalid MSAA counts to `1`.
  - Pipelines align `sample_count` to the pass `sample_count`.
- `render-validation-depth`: logs when clamping depth clear to `[0.0, 1.0]`; adds depth usage advisories when a pass has depth but the pipeline does not.
- `render-validation-stencil`: logs when enabling stencil requires upgrading the depth format to `Depth24PlusStencil8`; warns about stencil usage mismatches.
- `render-validation-pass-compat`: SetPipeline-time errors when color targets or depth/stencil expectations do not match the active pass.
- `render-validation-device`: device/format probing advisories (if available via the platform layer).
- `render-validation-encoder`: additional per-draw/encoder-time checks; highest runtime cost.
- `render-instancing-validation`: instance-range and per-instance buffer binding validation for `RenderCommand::Draw` and `RenderCommand::DrawIndexed`. Behavior:
  - Validates that `instances.start <= instances.end` and treats `start == end` as a no-op (draw is skipped).
  - Ensures that all vertex buffer slots marked as per-instance on the active pipeline have been bound in the current render pass.
  - Adds per-draw checks proportional to the number of instanced draws and per-instance slots; SHOULD be enabled only when diagnosing instancing issues.

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
- Enable only MSAA validation in release:
  - `cargo test -p lambda-rs --features render-validation-msaa`

## Changelog
- 0.1.1 (2025-11-25): Document `render-instancing-validation` behavior and update metadata.
- 0.1.0 (2025-11-17): Initial document introducing validation features and behavior by build type.
