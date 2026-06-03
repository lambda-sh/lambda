---
title: "Lambda RS Profiling Guide"
document_id: "profiling-guide-2026-06-03"
status: "living"
created: "2026-06-03T00:00:00Z"
last_updated: "2026-06-03T00:00:00Z"
version: "0.1.0"
engine_workspace_version: "2023.1.30"
wgpu_version: "29.0.0"
shader_backend_default: "naga"
winit_version: "0.29.15"
repo_commit: "929fba66d6a717a4de3f0a63407c3d9446ea1e27"
owners: ["lambda-sh"]
reviewers: ["engine", "rendering"]
tags: ["guide", "profiling", "performance", "features"]
---

# Lambda RS Profiling Guide

## Table of Contents
- [Overview](#overview)
- [Feature Flags](#feature-flags)
- [Basic Usage](#basic-usage)
- [Artifacts](#artifacts)
- [GPU Profiling](#gpu-profiling)
- [Regression Checks](#regression-checks)
- [Changelog](#changelog)

## Overview

`lambda-rs` exposes profiling through `lambda::profiling` when one of the
profiling features is enabled. The profiler records user-authored spans,
counters, and instants. It also records automatic CPU spans in runtime and
render paths, and GPU render-pass spans when GPU profiling is enabled and
supported by the active adapter.

Profiling is disabled by default so applications do not pay for diagnostics
they are not using. Enable the smallest feature set that fits the job, or use
the umbrella features while investigating performance locally.

## Feature Flags

- `profiling-cpu`: enables `lambda::profiling`, CPU event collection, user
  macros, and automatic CPU spans.
- `profiling-export-summary`: enables summary JSON export and composes
  `profiling-cpu`.
- `profiling-export-trace`: enables trace JSON export and composes
  `profiling-cpu`.
- `profiling`: enables CPU profiling plus summary and trace export.
- `profiling-gpu`: enables GPU render-pass timestamp collection and composes
  `profiling-cpu`.
- `profiling-all`: enables `profiling` plus `profiling-gpu`.

For CPU-only regression fixtures, start with `profiling`. For render
investigation on machines with timestamp-query support, use `profiling-all`.

## Basic Usage

Install one global profiler early in application startup, instrument relevant
user code with stable names, and flush once per frame or fixture iteration.

```rust
use std::{
  error::Error,
  path::Path,
};

use lambda::profiling::{
  profile_counter,
  profile_scope,
  Profiler,
  ProfilerBuilder,
};

fn install_profiler(output_directory: &Path) -> Result<(), Box<dyn Error>> {
  let profiler = ProfilerBuilder::new("my-performance-run")
    .with_output_directory(output_directory)
    .with_summary_export(true)
    .with_trace_export(true)
    .with_frame_history_limit(120)
    .build()?;

  profiler.install_global()?;

  return Ok(());
}

fn update_entities(entity_count: usize) -> Result<(), Box<dyn Error>> {
  let _scope = profile_scope!("app.update_entities");

  profile_counter!("app.entity_count", entity_count as u64);

  // Application update work goes here.

  if let Some(profiler) = Profiler::global() {
    let artifacts = profiler.flush_frame()?;

    if let Some(summary_path) = artifacts.summary_path {
      println!("summary: {}", summary_path.display());
    }
  }

  return Ok(());
}
```

Keep scope names stable across releases. Domain-qualified names such as
`app.update_entities` work better than names that include frame numbers, object
identifiers, or other high-cardinality values.

## Artifacts

Summary artifacts aggregate retained completed frames. They include frame
metrics and per-scope `count`, `total_ns`, `mean_ns`, `p50_ns`, `p95_ns`, and
`max_ns` values grouped by `clock_domain`.

Trace artifacts preserve ordered events for diagnosis. CPU events use the
`cpu` clock domain and include thread identifiers. GPU render-pass spans use
the `gpu` clock domain. Treat CPU and GPU timestamps as separate timelines
rather than synchronized wall-clock measurements.

`Profiler::flush_frame` writes enabled artifacts to the configured output
directory and returns their paths in `FrameArtifacts`.

## GPU Profiling

GPU profiling records render-pass durations with timestamp queries when the
adapter supports the required capabilities. If timestamp queries are not
available, the profiler continues with CPU-only profiling.

GPU profiling is best used as diagnostic instrumentation. It adds query
allocation, resolve, and non-blocking readback work, so keep it opt-in for
local investigation or targeted performance jobs.

## Regression Checks

Regression fixtures compare summary artifacts with checked-in baselines using
explicit tolerances. Comparisons match scopes by `(name, clock_domain)`.
Missing baseline scopes fail the comparison, while scopes present only in the
current run are reported as informational entries.

CPU-only fixtures are the most portable option for generic continuous
integration. GPU fixtures can still run on shared runners, but they should skip
GPU-specific assertions when timestamp queries are unsupported.

## Changelog

- 0.1.0 (2026-06-03): Add the initial user-facing profiling guide.
