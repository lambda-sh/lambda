---
title: "Demo Crates"
document_id: "demo-crates-readme-2026-02-04"
status: "draft"
created: "2026-02-04T00:00:00Z"
last_updated: "2026-02-05T22:52:14Z"
version: "0.1.1"
engine_workspace_version: "2023.1.30"
wgpu_version: "26.0.1"
shader_backend_default: "naga"
winit_version: "0.29.10"
repo_commit: "544444652b4dc3639f8b3e297e56c302183a7a0b"
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

Minimal demo:

```bash
cargo run -p lambda-demos-minimal --bin minimal
```

Audio demos (audio support enabled by default):

```bash
cargo run -p lambda-demos-audio --bin sine_wave
cargo run -p lambda-demos-audio --bin sound_buffer
cargo run -p lambda-demos-audio --bin play_sound
```

Render demos:

```bash
cargo run -p lambda-demos-render --bin triangle
cargo run -p lambda-demos-render --bin triangles
cargo run -p lambda-demos-render --bin immediates
cargo run -p lambda-demos-render --bin indexed_multi_vertex_buffers
cargo run -p lambda-demos-render --bin instanced_quads
cargo run -p lambda-demos-render --bin textured_quad
cargo run -p lambda-demos-render --bin textured_cube
cargo run -p lambda-demos-render --bin uniform_buffer_triangle
cargo run -p lambda-demos-render --bin offscreen_post
cargo run -p lambda-demos-render --bin reflective_room
```

Render demos with validation enabled (feature passthrough to `lambda-rs`):

```bash
cargo run -p lambda-demos-render --bin triangle --features validation
cargo run -p lambda-demos-render --bin triangle --features validation-strict
cargo run -p lambda-demos-render --bin triangle --features validation-all
```

## Workspace Behavior

- `cargo build` at the repository root MUST NOT build demo crates.
- `cargo build -p <demo-package>` MUST build the selected demo crate.
