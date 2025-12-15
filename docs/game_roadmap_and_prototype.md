---
title: "Lambda RS: Gaps, Roadmap, and Prototype Plan"
document_id: "game-roadmap-2025-09-24"
status: "living"
created: "2025-09-24T05:09:25Z"
last_updated: "2025-12-15T00:00:00Z"
version: "0.3.0"
engine_workspace_version: "2023.1.30"
wgpu_version: "26.0.1"
shader_backend_default: "naga"
winit_version: "0.29.10"
repo_commit: "71256389b9efe247a59aabffe9de58147b30669d"
owners: ["lambda-sh"]
reviewers: ["engine", "rendering"]
tags: ["roadmap","games","2d","3d","desktop"]
---

# Lambda RS: Gaps, Roadmap, and Prototype Plan

This document outlines current engine capabilities, the gaps to address for 2D/3D games and desktop apps, concrete API additions, and a step‑by‑step plan to deliver a playable 2D prototype first, followed by a small 3D scene. Code sketches follow Lambda’s existing builder/command style.

## Architecture Today

Key modules: windowing/events (winit), GPU (wgpu), render context, runtime loop, and GLSL→SPIR‑V shader compilation (naga).

Frame flow:
```
App Components --> ApplicationRuntime --> RenderContext --> wgpu (Device/Queue/Surface)
         |                 |                 |                  |
         |                 V                 V                  V
         |            Events/Loop      Pass/Pipeline        Adapter/Swapchain
         V
   RenderCommand stream per frame
```

Currently supported commands: Begin/EndRenderPass, SetPipeline, SetViewports, SetScissors, BindVertexBuffer, PushConstants, Draw.

## Gaps to Ship Games

- Bind groups + uniform/storage buffers (beyond push constants)
- Textures + samplers (images, sprites, materials)
- Depth/stencil and MSAA
- Index buffers + DrawIndexed; multiple vertex buffers; instancing
- Offscreen render targets (multipass)
- 2D layer: sprite batching, atlas loader, ortho camera, text, input mapping
- 3D layer: cameras, transforms, glTF load, basic materials/lighting
- Desktop apps: egui integration; dialogs/clipboard as needed

## Targeted API Additions (sketches)

Bind groups and uniforms (value: larger, structured GPU data; portable across adapters; enables cameras/materials):
```rust
// Layout with one uniform buffer at set(0) binding(0)
let layout = BindGroupLayoutBuilder::new()
  .with_uniform(0, PipelineStage::VERTEX)
  .build(rc.gpu());

let ubo = BufferBuilder::new()
  .with_length(std::mem::size_of::<Globals>())
  .with_usage(Usage::UNIFORM)
  .with_properties(Properties::CPU_VISIBLE)
  .build(rc.gpu(), vec![initial_globals])?;

let group = BindGroupBuilder::new(&layout)
  .with_uniform(0, &ubo)
  .build(rc.gpu());

let pipe = RenderPipelineBuilder::new()
  .with_layouts(&[&layout])
  .with_buffer(vbo, attrs)
  .build(
    rc.gpu(),
    rc.surface_format(),
    rc.depth_format(),
    &pass,
    &vs,
    Some(&fs),
  );

// Commands inside a pass
RC::SetPipeline { pipeline: pipe_id };
RC::SetBindGroup { set: 0, group: group_id, dynamic_offsets: vec![] };
RC::Draw { vertices: 0..3 };
```

Notes
- UBO vs push constants: UBOs scale to KBs and are supported widely; use for view/projection and per‑frame data.
- Dynamic offsets (optional later) let you pack many small structs into one UBO.

Textures and samplers (value: sprites, materials, UI images; sRGB correctness):
```rust
let tex = TextureBuilder::new_2d(TextureFormat::Rgba8UnormSrgb)
  .with_size(w, h)
  .with_data(&pixels)
  .build(rc.gpu());
let samp = SamplerBuilder::linear_clamp().build(rc.gpu());

let tex_layout = BindGroupLayoutBuilder::new()
  .with_sampled_texture(0)
  .with_sampler(1)
  .build(rc.gpu());
let tex_group = BindGroupBuilder::new(&tex_layout)
  .with_texture(0, &tex)
  .with_sampler(1, &samp)
  .build(rc.gpu());

// In fragment shader, sample with: sampler2D + UVs; ensure vertex inputs provide UVs.
// Upload path should convert source assets to sRGB formats when appropriate.
```

Index draw and instancing (value: reduce vertex duplication; batch many objects in one draw):
```rust
RC::BindVertexBuffer { pipeline: pipe_id, buffer: 0 };
RC::BindVertexBuffer { pipeline: pipe_id, buffer: 1 }; // instances
RC::BindIndexBuffer { buffer: ibo_id, format: IndexFormat::Uint16 };
RC::DrawIndexed { indices: 0..index_count, base_vertex: 0, instances: 0..instance_count };
```

Instance buffer attributes example
```rust
// slot 1: per-instance mat3x2 (2D) packed as 3x vec2, plus tint color
let instance_attrs = vec![
  // location 4..6 for rows
  VertexAttribute { location: 4, offset: 0,   element: VertexElement { format: ColorFormat::Rgb32Sfloat, offset: 0 }},
  VertexAttribute { location: 5, offset: 0,   element: VertexElement { format: ColorFormat::Rgb32Sfloat, offset: 8 }},
  VertexAttribute { location: 6, offset: 0,   element: VertexElement { format: ColorFormat::Rgb32Sfloat, offset: 16 }},
  // location 7 tint (RGBA8)
  VertexAttribute { location: 7, offset: 0,   element: VertexElement { format: ColorFormat::Rgba8Srgb,   offset: 24 }},
];
```

Depth/MSAA (value: correct 3D visibility and improved edge quality):
```rust
let pass = RenderPassBuilder::new()
  .with_clear_color(wgpu::Color::BLACK)
  .with_depth_stencil(wgpu::TextureFormat::Depth32Float, 1.0, true, wgpu::CompareFunction::Less)
  .with_msaa(4)
  .build(
    rc.gpu(),
    rc.surface_format(),
    rc.depth_format(),
  );

let pipe = RenderPipelineBuilder::new()
  .with_depth_format(wgpu::TextureFormat::Depth32Float)
  .build(
    rc.gpu(),
    rc.surface_format(),
    rc.depth_format(),
    &pass,
    &vs,
    Some(&fs),
  );
```

Notes
- Use reversed‑Z (Greater) later for precision, but start with Less.
- MSAA sample count must match between pass and pipeline.

Offscreen render targets (value: post‑processing, shadow maps, UI composition, picking):
```rust
let offscreen = RenderTargetBuilder::new()
  .with_color(TextureFormat::Rgba8UnormSrgb, width, height)
  .with_depth(TextureFormat::Depth32Float)
  .build(rc.gpu());

let pass1 = RenderPassBuilder::new().with_target(&offscreen).build(
  rc.gpu(),
  rc.surface_format(),
  rc.depth_format(),
);
let pass2 = RenderPassBuilder::new().build(
  rc.gpu(),
  rc.surface_format(),
  rc.depth_format(),
); // backbuffer

// Pass 1: draw scene
RC::BeginRenderPass { render_pass: pass1_id, viewport };
// ... draw 3D scene ...
RC::EndRenderPass;

// Pass 2: fullscreen triangle sampling offscreen.color
RC::BeginRenderPass { render_pass: pass2_id, viewport };
RC::SetPipeline { pipeline: post_pipe };
RC::SetBindGroup { set: 0, group: offscreen_group, dynamic_offsets: vec![] };
RC::Draw { vertices: 0..3 };
RC::EndRenderPass;
```

WGSL support (value: first‑class wgpu shader language, fewer translation pitfalls):
```rust
let vs = VirtualShader::WgslSource { source: include_str!("shaders/quad.wgsl").into(), name: "quad".into(), entry_point: "vs_main".into() };
let fs = VirtualShader::WgslSource { source: include_str!("shaders/quad.wgsl").into(), name: "quad".into(), entry_point: "fs_main".into() };
```

Shader hot‑reload (value: faster iteration; no rebuild): watch file timestamps and recompile shaders when changed; swap pipeline modules safely between frames.

## 2D Prototype (Asteroids‑like)

Goals: sprite batching via instancing; atlas textures; ortho camera; input mapping; text HUD. Target 60 FPS with 10k sprites (mid‑range GPU).

Core draw:
```rust
RC::BeginRenderPass { render_pass: pass_id, viewport };
RC::SetPipeline { pipeline: pipe_id };
RC::SetBindGroup { set: 0, group: globals_gid, dynamic_offsets: vec![] };
RC::SetBindGroup { set: 1, group: atlas_gid, dynamic_offsets: vec![] };
RC::BindVertexBuffer { pipeline: pipe_id, buffer: 0 }; // quad
RC::BindVertexBuffer { pipeline: pipe_id, buffer: 1 }; // instances
RC::BindIndexBuffer { buffer: ibo_id, format: IndexFormat::Uint16 };
RC::DrawIndexed { indices: 0..6, base_vertex: 0, instances: 0..sprite_count };
RC::EndRenderPass;
```

Building instance data each frame (value: dynamic transforms with minimal overhead):
```rust
// CPU side: update transforms and pack into a Vec<Instance>
queue.write_buffer(instance_vbo.raw(), 0, bytemuck::cast_slice(&instances));
```

Text rendering options (value: legible UI/HUD):
- Bitmap font atlas: simplest path; pack glyphs into the sprite pipeline.
- glyphon/glyph_brush integration: high‑quality layout; more deps; implement later.

## 3D Prototype (Orbit Camera + glTF)

Goals: depth test/write, indexed mesh, textured material, simple lighting; orbit camera.

Core draw mirrors 2D but with depth enabled and mesh buffers.

Camera helpers (value: reduce boilerplate and bugs):
```rust
let proj = matrix::perspective_matrix(60f32.to_radians(), width as f32 / height as f32, 0.1, 100.0);
let view = matrix::translation_matrix([0.0, 0.0, -5.0]); // or look_at helper later
let view_proj = proj.multiply(&view);
```

glTF loading (value: standard asset path): map glTF meshes/materials/textures to VBO/IBO and bind groups; start with positions/normals/UVs and a single texture.

## Milestones & Estimates

- M1 Rendering Foundations (2–3 weeks): bind groups/UBO, textures/samplers, depth/MSAA, indexed draw, instancing.
- M2 2D Systems (2–3 weeks): sprite batching, atlas, 2D camera, input, text; ship prototype.
- M3 3D Systems (3–5 weeks): cameras, glTF, materials/lighting; ship scene.
- M4 Desktop UI (1–2 weeks): egui integration + example app.

## Changelog

- 2025-09-26 (v0.2.0) — Expanded examples, added value rationale per feature, offscreen/post, WGSL, and iteration tips.
- 2025-09-24 (v0.1.0) — Initial draft with gaps, roadmap, and prototype plan.
