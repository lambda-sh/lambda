---
title: "Specifications Index"
document_id: "specs-index-2026-02-07"
status: "living"
created: "2026-02-07T00:00:00Z"
last_updated: "2026-06-03T00:00:00Z"
version: "0.1.7"
engine_workspace_version: "2023.1.30"
wgpu_version: "29.0.0"
shader_backend_default: "naga"
winit_version: "0.29.15"
repo_commit: "929fba66d6a717a4de3f0a63407c3d9446ea1e27"
owners: ["lambda-sh"]
reviewers: ["engine"]
tags: ["index", "specs", "docs"]
---

## Rendering

- Uniform Buffers and Bind Groups — [rendering/uniform-buffers-and-bind-groups.md](rendering/uniform-buffers-and-bind-groups.md)
- Textures and Samplers — [rendering/textures-and-samplers.md](rendering/textures-and-samplers.md)
- Instanced Rendering — [rendering/instanced-rendering.md](rendering/instanced-rendering.md)
- Indexed Draws and Multiple Vertex Buffers — [rendering/indexed-draws-and-multiple-vertex-buffers.md](rendering/indexed-draws-and-multiple-vertex-buffers.md)
- Depth, Stencil, and MSAA — [rendering/depth-stencil-msaa.md](rendering/depth-stencil-msaa.md)
- Offscreen Render Targets and Multipass — [rendering/offscreen-render-targets-and-multipass.md](rendering/offscreen-render-targets-and-multipass.md)

## Audio

- Audio Devices — [audio/audio-devices.md](audio/audio-devices.md)
- Audio File Loading — [audio/audio-file-loading.md](audio/audio-file-loading.md)
- Sound Playback and Transport Controls — [audio/sound-playback.md](audio/sound-playback.md)
- Sound Instance Gain and Pitch Controls — [audio/sound-instance-gain-and-pitch.md](audio/sound-instance-gain-and-pitch.md)

## Runtime / Events

- Component Event Handling — [runtime/component-event-handling.md](runtime/component-event-handling.md)
- Profiling and Performance Regression Checks — [runtime/profiling-and-performance-regression-checks.md](runtime/profiling-and-performance-regression-checks.md)

## Physics

- 2D Physics World — [physics/physics-world-2d.md](physics/physics-world-2d.md)
- 2D Rigid Bodies — [physics/rigid-bodies-2d.md](physics/rigid-bodies-2d.md)
- 2D Colliders — [physics/colliders-2d.md](physics/colliders-2d.md)
- 2D Collision Queries and Events — [physics/collision-queries-and-events-2d.md](physics/collision-queries-and-events-2d.md)

## Templates

- Specification template — [_spec-template.md](_spec-template.md)

## Changelog

- 0.1.7 (2026-06-03): Refresh metadata after finalizing the profiling and
  performance regression checks implementation.
- 0.1.6 (2026-04-06): Add the profiling and performance regression checks
  specification and update metadata.
- 0.1.5 (2026-03-25): Add the 2D collision queries and events specification
  and align metadata with the long-lived documentation requirements.
