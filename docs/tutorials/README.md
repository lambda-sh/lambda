---
title: "Tutorials Index"
document_id: "tutorials-index-2025-10-17"
status: "living"
created: "2025-10-17T00:20:00Z"
last_updated: "2026-02-07T00:00:00Z"
version: "0.8.0"
engine_workspace_version: "2023.1.30"
wgpu_version: "28.0.0"
shader_backend_default: "naga"
winit_version: "0.29.10"
repo_commit: "4b0c5abf6743788596177b3c10c3214db20ad6b1"
owners: ["lambda-sh"]
reviewers: ["engine", "rendering"]
tags: ["index", "tutorials", "docs"]
---

This index lists tutorials that teach specific `lambda-rs` tasks through complete, incremental builds.

## Rendering

### Basics

- Basic Triangle: Vertex‑Only Draw — [rendering/basics/basic-triangle.md](rendering/basics/basic-triangle.md)
- Immediates: Draw Multiple 2D Triangles — [rendering/basics/immediates-multiple-triangles.md](rendering/basics/immediates-multiple-triangles.md)

### GPU Resources

- Uniform Buffers: Build a Spinning Triangle — [rendering/resources/uniform-buffers.md](rendering/resources/uniform-buffers.md)
- Textured Quad: Sample a 2D Texture — [rendering/resources/textured-quad.md](rendering/resources/textured-quad.md)
- Textured Cube: 3D Immediates + 2D Sampling — [rendering/resources/textured-cube.md](rendering/resources/textured-cube.md)

### Techniques

- Indexed Draws and Multiple Vertex Buffers — [rendering/techniques/indexed-draws-and-multiple-vertex-buffers.md](rendering/techniques/indexed-draws-and-multiple-vertex-buffers.md)
- Instanced Rendering: Grid of Colored Quads — [rendering/techniques/instanced-quads.md](rendering/techniques/instanced-quads.md)
- Offscreen Post: Render to a Texture and Sample to the Surface — [rendering/techniques/offscreen-post.md](rendering/techniques/offscreen-post.md)
- Reflective Room: Stencil Masked Reflections with MSAA — [rendering/techniques/reflective-room.md](rendering/techniques/reflective-room.md)

Browse all tutorials under `rendering/`.

## Physics

### Basics

- Physics 2D: Falling Quad (Kinematic) — [physics/basics/falling-quad-kinematic.md](physics/basics/falling-quad-kinematic.md)

Browse all tutorials under `physics/`.

Changelog

- 0.8.0 (2026-02-07): Add physics tutorial section and first physics demo.
- 0.7.1 (2026-02-07): Group tutorials by feature area in the index.
- 0.7.0 (2026-01-05): Rename push constants tutorial to immediates for wgpu v28; update metadata.
- 0.6.0 (2025-12-29): Add offscreen post tutorial; update metadata and commit.
- 0.5.0 (2025-12-16): Add basic triangle and multi-triangle push constants tutorials; update metadata and commit.
- 0.4.0 (2025-11-25): Add Instanced Quads tutorial; update metadata and commit.
- 0.3.0 (2025-11-17): Add Reflective Room tutorial; update metadata and commit.
- 0.2.0 (2025-11-10): Add links for textured quad and textured cube; update metadata and commit.
- 0.1.0 (2025-10-17): Initial index with uniform buffers tutorial.
