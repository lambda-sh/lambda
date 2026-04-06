---
title: "Profiling and Performance Regression Checks"
document_id: "profiling-performance-regression-2026-04-06"
status: "draft"
created: "2026-04-06T00:13:19Z"
last_updated: "2026-04-06T00:13:19Z"
version: "0.1.0"
engine_workspace_version: "2023.1.30"
wgpu_version: "26.0.1"
shader_backend_default: "naga"
winit_version: "0.29.10"
repo_commit: "4313b4be0ae9b36c16dcf5152265a087bf81e223"
owners: ["lambda-sh"]
reviewers: ["engine", "rendering"]
tags: ["spec", "runtime", "profiling", "performance"]
---

# Profiling and Performance Regression Checks

## Table of Contents
- [Summary](#summary)
- [Scope](#scope)
- [Terminology](#terminology)
- [Architecture Overview](#architecture-overview)
- [Design](#design)
  - [API Surface](#api-surface)
  - [Behavior](#behavior)
  - [Validation and Errors](#validation-and-errors)
  - [Cargo Features](#cargo-features)
- [Constraints and Rules](#constraints-and-rules)
- [Performance Considerations](#performance-considerations)
- [Requirements Checklist](#requirements-checklist)
- [Verification and Testing](#verification-and-testing)
- [Compatibility and Migration](#compatibility-and-migration)
- [Changelog](#changelog)

## Summary

- Add a first-class profiling subsystem that can be compiled out of release
  builds by default, enabled through granular Cargo features, and used by both
  Lambda internals and library users.
- Record CPU spans with `Instant` and GPU spans with `wgpu` timestamp queries
  when supported, while keeping the two timing domains explicit and separate.
- Export stable summary and trace artifacts so targeted performance scenarios
  can detect regressions in CI and still provide deeper diagnostic output when
  a regression occurs.

## Scope

### Goals

- Add a new workspace support crate, `lambda-rs-profiling`, and re-export it
  from `lambda-rs` as `lambda::profiling`.
- Provide no-op macros and guards when profiling features are disabled so
  users can instrument their own code without `cfg` boilerplate.
- Automatically instrument hot engine paths in `ApplicationRuntime` and the
  render path.
- Support CPU profiling in all environments where Lambda currently runs.
- Support GPU timestamp profiling on the `wgpu` backend when the selected
  adapter exposes the required feature.
- Export machine-readable summaries for regression checks and timeline traces
  for diagnosis.
- Define a CI comparison workflow based on summary baselines and tolerances.

### Non-Goals

- Cross-backend GPU profiling outside the current `wgpu` path.
- True wall-clock synchronization between CPU and GPU timestamps.
- Compute pass or copy pass profiling in v1.
- Statistical benchmarking infrastructure such as Criterion-based throughput
  benchmarks.
- Automatic online sampling, flamegraph generation, or remote trace upload.

## Terminology

- Span: a timed region with a start and end, recorded under a stable name.
- Scope guard: a drop-based guard that closes a span automatically.
- Counter: a numeric measurement recorded for a named event or frame.
- Trace artifact: a structured event stream that records ordered span and
  counter events.
- Summary artifact: an aggregated report that records counts and latency
  percentiles per named scope.
- Clock domain: the time source for a measurement. This specification defines
  `cpu` and `gpu` clock domains.
- Baseline: a checked-in or CI-provided summary artifact used as the expected
  reference when comparing a new run.
- Regression fixture: a deterministic test or runnable scenario that emits
  profiler artifacts for comparison.

## Architecture Overview

- Crate `lambda-rs-profiling`
  - Owns public profiling APIs, collector state, export formats, comparison
    helpers, and user-facing macros.
- Crate `lambda-rs`
  - Re-exports `lambda-rs-profiling` as `lambda::profiling`.
  - Emits automatic spans from `ApplicationRuntime`, `RenderContext`, and
    command submission/render pass paths.
- Crate `lambda-rs-platform`
  - Extends the `wgpu` wrapper with timestamp capability queries, timestamp
    period access, query allocation, resolve, and readback helpers.

Data flow

```
user code / lambda-rs internals
  └── profile_scope! / profiler APIs
        ├── CPU collector (thread-local span buffers)
        └── GPU collector (timestamp query ring, when supported)
              └── frame flush
                    ├── SummaryReport JSON
                    └── TraceReport JSON

CI fixture
  └── emits summary artifact
        └── compare against baseline + tolerances
              ├── pass
              └── fail + upload trace artifact for diagnosis
```

## Design

### API Surface

#### Public crate layout

- New crate: `crates/lambda-rs-profiling`
  - MUST expose a library name that can be re-exported cleanly from
    `lambda-rs`.
- Re-export:
  - `lambda-rs/src/lib.rs` MUST expose `pub use lambda_rs_profiling as profiling;`
    behind the relevant profiling feature gates.

#### Public profiling types

- Module `lambda::profiling`
  - `pub struct ProfilerBuilder`
    - `pub fn new(name: &str) -> Self`
    - `pub fn with_summary_export(self, enabled: bool) -> Self`
    - `pub fn with_trace_export(self, enabled: bool) -> Self`
    - `pub fn with_output_directory(self, path: &std::path::Path) -> Self`
    - `pub fn with_gpu_timestamps(self, enabled: bool) -> Self`
    - `pub fn with_frame_history_limit(self, frames: usize) -> Self`
    - `pub fn build(self) -> Result<Profiler, ProfilerBuildError>`
  - `pub struct Profiler`
    - `pub fn install_global(self) -> Result<(), ProfilerInstallError>`
    - `pub fn global() -> Option<&'static Profiler>`
    - `pub fn begin_scope(&self, name: &'static str) -> ScopeGuard`
    - `pub fn record_counter(&self, name: &'static str, value: u64)`
    - `pub fn record_instant(&self, name: &'static str)`
    - `pub fn flush_frame(&self) -> Result<FrameArtifacts, ProfilerFlushError>`
    - `pub fn export_summary(&self) -> Result<SummaryReport, ExportError>`
    - `pub fn export_trace(&self) -> Result<TraceReport, ExportError>`
  - `pub struct ScopeGuard`
    - Drop MUST close the associated span exactly once.
  - `pub struct FrameArtifacts`
    - `pub summary_path: Option<std::path::PathBuf>`
    - `pub trace_path: Option<std::path::PathBuf>`
  - `pub struct SummaryReport`
  - `pub struct TraceReport`
  - `pub struct ComparisonReport`
  - `pub enum ProfilerBuildError`
  - `pub enum ProfilerInstallError`
  - `pub enum ProfilerFlushError`
  - `pub enum ExportError`
  - `pub enum CompareError`

#### User-facing macros

- The public macros MUST be exported by `lambda-rs-profiling` and available
  through the `lambda::profiling` re-export path.
- Required macros:
  - `profile_scope!($name)`
  - `profile_function!()`
  - `profile_counter!($name, $value)`
  - `profile_instant!($name)`
- Macro behavior:
  - When the relevant profiling feature is disabled, each macro MUST compile
    to a no-op and MUST NOT require callers to add `cfg` guards.
  - `profile_scope!` MUST evaluate to a guard expression so users can bind or
    intentionally discard it.
  - `profile_function!` MUST expand to the enclosing function name or a stable
    module-qualified fallback string.

#### Export formats

- `SummaryReport` JSON MUST include:
  - top-level metadata:
    - `schema_version`
    - `run_name`
    - `crate_version`
    - `clock_domains`
    - `generated_at_utc`
  - per-scope summaries:
    - `name`
    - `clock_domain`
    - `count`
    - `total_ns`
    - `mean_ns`
    - `p50_ns`
    - `p95_ns`
    - `max_ns`
  - frame metrics:
    - `frame_count`
    - `frame_p50_ns`
    - `frame_p95_ns`
    - `frame_max_ns`
- `TraceReport` JSON MUST include:
  - top-level metadata:
    - `schema_version`
    - `run_name`
    - `generated_at_utc`
  - ordered events:
    - `name`
    - `kind` (`span_begin`, `span_end`, `counter`, `instant`)
    - `clock_domain`
    - `timestamp_ns`
    - `thread_id` for CPU events
    - `frame_index`
    - optional `value` for counters

#### Baseline comparison helpers

- `lambda::profiling` MUST expose comparison helpers that accept a current
  summary report, a baseline summary report, and explicit tolerances.
- Baseline comparison MUST match scopes by `(name, clock_domain)`.
- If a scope exists in the baseline and is absent from the current run, the
  comparison MUST fail for deterministic regression fixtures.
- If a scope exists in the current run and not in the baseline, the comparison
  MUST report it as informational and MUST NOT fail by default.
- Default tolerances for the built-in regression workflow:
  - CPU scopes fail when `p95_ns` regresses by more than `10%` and at least
    `250_000 ns`.
  - GPU scopes fail when `p95_ns` regresses by more than `10%` and at least
    `500_000 ns`.
  - Frame-level scopes compare `p95_ns` and `max_ns`.
  - Inner named scopes compare `mean_ns` and `p95_ns`.

### Behavior

#### Feature-off behavior

- Profiling MUST be opt-in for release builds.
- When profiling features are disabled:
  - user macros MUST be no-ops,
  - engine instrumentation MUST compile out,
  - no collector threads, buffers, files, or runtime environment parsing MAY
    be created.

#### Collector model

- CPU collection
  - CPU spans MUST use `std::time::Instant`.
  - Span names in v1 MUST be `&'static str`.
  - CPU events MUST be buffered per thread and aggregated on frame flush or
    buffer rollover.
- GPU collection
  - GPU profiling MUST use `wgpu` timestamp queries through the platform
    wrapper.
  - GPU timing MUST be treated as a separate `gpu` clock domain.
  - The profiler MUST use a multi-frame query/readback ring so result
    retrieval does not stall the render thread.
  - If timestamp queries are unsupported on the active adapter, GPU profiling
    MUST be disabled automatically and the profiler MUST continue in CPU-only
    mode.

#### Automatic engine instrumentation

- `ApplicationRuntime` MUST emit named CPU spans for:
  - runtime initialization
  - component attach and detach
  - event mapping
  - event dispatch
  - per-frame update
  - per-component `on_update`
  - per-frame render
  - per-component `on_render`
- `RenderContext` and render encoding paths MUST emit named spans for:
  - surface frame acquisition
  - command encoding
  - each render pass
  - queue submission
  - frame presentation
- Render pass and resource labels SHOULD be incorporated into span names when
  they are available and stable.

#### User instrumentation

- User code MAY call the profiler APIs directly or use the macros.
- User spans and counters MUST flow into the same active profiler instance used
  by engine spans.
- The global profiler installation model MUST be first-writer-wins, matching
  the existing global logger initialization pattern.

#### Artifact generation

- Summary export
  - MUST be stable and suitable for CI comparison.
  - MUST avoid embedding nondeterministic fields inside scope-level values
    beyond top-level metadata.
- Trace export
  - MUST preserve ordering within each clock domain.
  - MUST be intended for diagnosis, not as the canonical comparison input.
- CI workflow
  - Regression fixtures MUST emit summary artifacts on every profiling run.
  - CI MUST compare current summaries against a baseline artifact.
  - On comparison failure, CI SHOULD retain the trace artifact as a build
    artifact for diagnosis.

### Validation and Errors

- `ProfilerBuilder::build` MUST return an error instead of panicking when:
  - the output directory is invalid or cannot be created,
  - trace or summary export is enabled without a writable destination,
  - an invalid frame history limit is supplied.
- `Profiler::install_global` MUST return a deterministic
  `ProfilerInstallError::AlreadyInstalled` equivalent when installation is
  attempted more than once.
- GPU profiling setup MUST return actionable errors or disable GPU collection
  cleanly when:
  - the adapter does not support timestamp queries,
  - query allocation fails,
  - readback resources cannot be created.
- Export helpers MUST return actionable errors with filesystem context when
  artifact creation or write fails.
- Comparison helpers MUST report which scope and metric violated tolerance.
- Library code introduced by this work MUST avoid panics except where an
  invariant is internal and unrecoverable.

### Cargo Features

- New `lambda-rs` umbrella features:
  - `profiling`
    - MUST compose `profiling-cpu`, `profiling-export-summary`, and
      `profiling-export-trace`.
  - `profiling-all`
    - MUST compose `profiling` and `profiling-gpu`.
- New `lambda-rs` granular features:
  - `profiling-cpu`
  - `profiling-gpu`
  - `profiling-export-summary`
  - `profiling-export-trace`
- `profiling-gpu` MUST compose `profiling-cpu`.
- New `lambda-rs-profiling` granular features:
  - `gpu`
  - `export-summary`
  - `export-trace`
- New `lambda-rs-platform` granular feature:
  - `profiling-gpu`
- Feature rules:
  - All behavior in code MUST be gated by granular feature names together with
    `debug_assertions`, never umbrella names.
  - `profiling` and `profiling-all` MUST exist only as convenience umbrellas.
  - None of the profiling features MAY be enabled by default.
  - `lambda-rs` MUST map its public profiling features to
    `lambda-rs-profiling` and `lambda-rs-platform` internals without exposing
    platform details in the public feature names.
  - The implementation change that adds these features MUST update
    `docs/features.md` with the exact feature names, owning crate, default
    state, summary, and runtime cost.

## Constraints and Rules

- The profiler MUST NOT expose raw `wgpu` types through `lambda-rs`.
- The new profiling support crate MUST remain usable by `lambda-rs` users
  without requiring them to depend on `lambda-rs-platform`.
- CPU and GPU clock domains MUST remain explicit in exported artifacts and
  internal APIs.
- GPU profiling MUST NOT block frame submission waiting for query results.
- Stable comparison inputs MUST come from summary artifacts, not raw traces.
- Built-in engine span names MUST be stable across patch releases unless the
  spec and migration notes explicitly document a change.
- The profiler MUST be thread-safe for concurrent scope emission from user
  threads and runtime threads.

## Performance Considerations

- Recommendations
  - Profiling SHOULD remain disabled in default release builds.
  - CPU collection SHOULD use fixed-capacity thread-local buffers to reduce
    synchronization overhead.
  - GPU readback SHOULD use a frame ring so query resolution does not stall the
    active frame.
  - Regression fixtures SHOULD be deterministic and narrowly scoped.
- Rationale
  - Profiling is diagnostic infrastructure and MUST NOT impose production
    runtime cost by default.
  - Buffering reduces contention on hot instrumented paths.
  - Deferred GPU readback preserves the non-intrusive design goal.
  - Narrow fixtures produce more stable CI comparisons than broad end-to-end
    application runs.

## Requirements Checklist

- Functionality
  - [ ] `lambda-rs-profiling` crate added to the workspace.
  - [ ] `lambda::profiling` re-export added to `lambda-rs`.
  - [ ] CPU collector implemented.
  - [ ] GPU timestamp collector implemented with CPU-only fallback.
  - [ ] Automatic runtime and render instrumentation added.
  - [ ] User-facing macros implemented as no-ops when disabled.
- API Surface
  - [ ] Public builder, profiler, guards, export types, and comparison helpers
        implemented.
  - [ ] Public APIs documented with Rustdoc.
  - [ ] Feature names added to `lambda-rs` and documented in `docs/features.md`.
- Validation and Errors
  - [ ] Filesystem, install, export, and comparison errors are actionable.
  - [ ] GPU capability detection degrades cleanly without panicking.
  - [ ] Comparison failures identify the violating scope and metric.
- Performance
  - [ ] Collector hot paths avoid per-event heap allocation in steady state.
  - [ ] GPU query readback does not block the active frame.
  - [ ] Regression thresholds documented and implemented.
- Documentation and Examples
  - [ ] Profiling guide or example added for library users.
  - [ ] Regression fixture workflow documented.
  - [ ] `docs/specs/README.md` updated for this specification.

For each checked item, include a reference to a commit, pull request, or file
path that demonstrates the implementation.

## Verification and Testing

- Unit Tests
  - Macro expansion compiles and performs no work when profiling is disabled.
  - `ScopeGuard` closes spans exactly once on drop.
  - Summary aggregation computes `count`, `total_ns`, `mean_ns`, `p50_ns`,
    `p95_ns`, and `max_ns` correctly.
  - JSON export preserves the documented schema fields.
  - Comparison helpers fail only when both the relative and absolute tolerance
    thresholds are exceeded.
  - Commands: `cargo test -p lambda-rs-profiling -- --nocapture`

- Integration Tests
  - `ApplicationRuntime` emits automatic CPU spans for update, render, and
    event dispatch.
  - User component instrumentation is captured in the same report as engine
    spans.
  - CPU-only mode works when GPU timestamps are disabled or unsupported.
  - Supported adapters collect render-pass GPU timestamps without stalling the
    frame loop.
  - Commands:
    - `cargo test -p lambda-rs --features profiling -- --nocapture`
    - `cargo test -p lambda-rs --features profiling-all -- --nocapture`

- Regression Fixtures
  - Targeted fixtures under `crates/lambda-rs/tests/` MUST emit summary
    artifacts for comparison.
  - Generic CI MUST run CPU regression checks.
  - GPU regression checks MUST run only on stable runners with supported
    timestamp-query adapters. Unsupported runners MUST mark the GPU check as
    skipped rather than failed.

- Manual Checks
  - Enable `profiling-all` in a render demo and confirm that:
    - summary artifacts are written,
    - trace artifacts are written,
    - CPU spans are present,
    - GPU render-pass spans are present when supported,
    - the application still runs when GPU timestamps are unsupported.

## Compatibility and Migration

- Breaking changes
  - None. This work is additive and feature-gated.

- Migration steps
  - Existing applications MAY ignore the profiler entirely.
  - Applications that want instrumentation MUST enable the relevant profiling
    features explicitly.
  - User code MAY add `profile_scope!`, `profile_function!`,
    `profile_counter!`, and `profile_instant!` without adding feature-specific
    `cfg` guards.

## Changelog

- 2026-04-06 (v0.1.0) — Initial draft defining a feature-gated profiling
  subsystem, CPU and GPU timing behavior, export artifacts, and CI regression
  comparison rules.
