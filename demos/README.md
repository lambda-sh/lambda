---
title: "Demo Crates"
document_id: "demo-crates-readme-2026-02-04"
status: "draft"
created: "2026-02-04T00:00:00Z"
last_updated: "2026-02-04T00:00:00Z"
version: "0.1.0"
engine_workspace_version: "2023.1.30"
wgpu_version: "26.0.1"
shader_backend_default: "naga"
winit_version: "0.29.10"
repo_commit: "9031bcf7dd7fb60da09b04583de3a8f87743768f"
owners: ["lambda-sh"]
reviewers: ["engine", "rendering"]
tags: ["demos", "guide"]
---

# Demos

## Overview

This directory contains runnable demo crates that showcase `lambda-rs`.

Demo crates are included in the workspace `members` but excluded from
`default-members`. This keeps `cargo build` fast while keeping demos available
via the `-p <package>` flag.

Minimal reference examples remain in `crates/lambda-rs/examples/` for
documentation and rustdoc usage.

## Layout (Target)

```
demos/
  audio/    # Audio-focused demos (output device, decoding)
  minimal/  # Smallest window + render context demo
  render/   # Rendering demos (validation flags, advanced passes)
```

## Running Demos

Render demo:

```bash
cargo run -p lambda-demos-render --bin triangle
```

Render demo with validation:

```bash
cargo run -p lambda-demos-render --bin triangle --features validation
```

Audio demo:

```bash
cargo run -p lambda-demos-audio --bin sine_wave --features audio-output-device
```

## Workspace Behavior

- `cargo build` at the repository root MUST NOT build demo crates.
- `cargo build -p <demo-package>` MUST build the selected demo crate.
