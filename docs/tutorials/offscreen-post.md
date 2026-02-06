---
title: "Offscreen Post: Render to a Texture and Sample to the Surface"
document_id: "offscreen-post-tutorial-2025-12-29"
status: "draft"
created: "2025-12-29T00:00:00Z"
last_updated: "2026-02-05T23:05:40Z"
version: "0.2.2"
engine_workspace_version: "2023.1.30"
wgpu_version: "26.0.1"
shader_backend_default: "naga"
winit_version: "0.29.10"
repo_commit: "544444652b4dc3639f8b3e297e56c302183a7a0b"
owners: ["lambda-sh"]
reviewers: ["engine", "rendering"]
tags: ["tutorial", "graphics", "offscreen", "render-targets", "multipass", "post-processing", "texture", "sampler", "wgpu", "rust"]
---

## Overview <a name="overview"></a>

This tutorial renders a triangle into an offscreen render target, then samples
that target in a second pass to present the result on the window surface. The
implementation demonstrates multi-pass rendering, bind groups for texture
sampling, and resource replacement during window resize.

Reference implementation: `demos/render/src/bin/offscreen_post.rs`.

## Table of Contents

- [Overview](#overview)
- [Goals](#goals)
- [Prerequisites](#prerequisites)
- [Requirements and Constraints](#requirements-and-constraints)
- [Data Flow](#data-flow)
- [Implementation Steps](#implementation-steps)
  - [Step 1 — Imports and Shader Sources](#step-1)
  - [Step 2 — Component State](#step-2)
  - [Step 3 — Compile Shaders in `Default`](#step-3)
  - [Step 4 — Implement `Component` and Build Resources](#step-4)
  - [Step 5 — Fullscreen Quad Mesh](#step-5)
  - [Step 6 — Record Commands in `on_render`](#step-6)
  - [Step 7 — Resize Events and Resource Replacement](#step-7)
  - [Step 8 — Main Entry Point](#step-8)
- [Validation](#validation)
- [Notes](#notes)
- [Conclusion](#conclusion)
- [Exercises](#exercises)
- [Changelog](#changelog)

## Goals <a name="goals"></a>

- Render into an offscreen color texture using `RenderDestination::Offscreen`.
- Sample the offscreen result in a second pass using a bind group.
- Replace the offscreen target and dependent bind group on window resize.

## Prerequisites <a name="prerequisites"></a>

- The workspace builds: `cargo build --workspace`.
- The minimal demo runs:
  `cargo run -p lambda-demos-minimal --bin minimal`.

## Requirements and Constraints <a name="requirements-and-constraints"></a>

- The offscreen target color texture MUST be created with both render-attachment
  and sampled usage. Use `OffscreenTargetBuilder` to ensure correct usage.
- The offscreen pass/pipeline color format MUST match the offscreen target
  format. This example uses `render_context.surface_format()` for both.
- The render path MUST handle `0x0` sizes during resize. This example clamps
  viewport sizes via `width.max(1)` and `height.max(1)`.
- The bind group layout bindings MUST match the shader declarations:
  `layout (set = 0, binding = 1)` for the texture and `binding = 2` for the
  sampler.
- Replacing an offscreen target MUST also replace any bind groups that reference
  the previous target’s texture view.
- Acronyms: graphics processing unit (GPU), central processing unit (CPU),
  texture coordinates (UV).

## Data Flow <a name="data-flow"></a>

```
Default::default
  └─ ShaderBuilder → Shader handles

Component::on_attach
  ├─ OffscreenTargetBuilder → OffscreenTarget (attached)
  ├─ RenderPassBuilder → offscreen pass + post pass (attached)
  ├─ RenderPipelineBuilder → offscreen pipeline + post pipeline (attached)
  └─ BindGroupLayout/BindGroup → sample offscreen color texture

Component::on_render (each frame)
  Pass A (Offscreen): draw triangle → offscreen color texture
  Pass B (Surface): sample offscreen texture → fullscreen quad
```

## Implementation Steps <a name="implementation-steps"></a>

### Step 1 — Imports and Shader Sources <a name="step-1"></a>

Start with the imports and the embedded post shaders.

```rust
#![allow(clippy::needless_return)]

//! Example: Render to an offscreen target, then sample it to the surface.

use lambda::{
  component::Component,
  events::Events,
  logging,
  render::{
    bind::{
      BindGroupBuilder,
      BindGroupLayout,
      BindGroupLayoutBuilder,
    },
    buffer::BufferBuilder,
    command::{
      RenderCommand,
      RenderDestination,
    },
    mesh::{
      Mesh,
      MeshBuilder,
    },
    pipeline::{
      CullingMode,
      RenderPipelineBuilder,
    },
    render_pass::RenderPassBuilder,
    shader::{
      Shader,
      ShaderBuilder,
      ShaderKind,
      VirtualShader,
    },
    targets::offscreen::OffscreenTargetBuilder,
    texture::SamplerBuilder,
    vertex::{
      ColorFormat,
      Vertex,
      VertexAttribute,
      VertexBuilder,
      VertexElement,
    },
    viewport::ViewportBuilder,
    RenderContext,
    ResourceId,
  },
  runtime::start_runtime,
  runtimes::{
    application::ComponentResult,
    ApplicationRuntimeBuilder,
  },
};

const POST_VERTEX_SHADER_SOURCE: &str = r#"
#version 450

layout (location = 0) in vec3 vertex_position;
layout (location = 2) in vec3 vertex_color; // uv packed into .xy

layout (location = 0) out vec2 v_uv;

void main() {
  gl_Position = vec4(vertex_position, 1.0);
  v_uv = vertex_color.xy;
}
"#;

const POST_FRAGMENT_SHADER_SOURCE: &str = r#"
#version 450

layout (location = 0) in vec2 v_uv;
layout (location = 0) out vec4 fragment_color;

layout (set = 0, binding = 1) uniform texture2D tex;
layout (set = 0, binding = 2) uniform sampler samp;

void main() {
  fragment_color = texture(sampler2D(tex, samp), v_uv);
}
"#;
```

The offscreen pass uses `crates/lambda-rs/assets/shaders/triangle.vert` and
`crates/lambda-rs/assets/shaders/triangle.frag`.

### Step 2 — Component State <a name="step-2"></a>

Define the component state used by the example.

```rust
pub struct OffscreenPostExample {
  triangle_vs: Shader,
  triangle_fs: Shader,
  post_vs: Shader,
  post_fs: Shader,
  quad_mesh: Option<Mesh>,

  offscreen_pass: Option<ResourceId>,
  offscreen_pipeline: Option<ResourceId>,
  offscreen_target: Option<ResourceId>,

  post_pass: Option<ResourceId>,
  post_pipeline: Option<ResourceId>,
  post_bind_group: Option<ResourceId>,
  post_layout: Option<BindGroupLayout>,

  width: u32,
  height: u32,
}
```

This struct matches the example’s fields and keeps the shader handles alongside
the IDs returned by `RenderContext::attach_*`.

### Step 3 — Compile Shaders in `Default` <a name="step-3"></a>

Compile the triangle and post shaders in `Default`, matching the example.

```rust
impl Default for OffscreenPostExample {
  fn default() -> Self {
    let triangle_vertex = VirtualShader::Source {
      source: include_str!("../assets/shaders/triangle.vert").to_string(),
      kind: ShaderKind::Vertex,
      name: String::from("triangle"),
      entry_point: String::from("main"),
    };

    let triangle_fragment = VirtualShader::Source {
      source: include_str!("../assets/shaders/triangle.frag").to_string(),
      kind: ShaderKind::Fragment,
      name: String::from("triangle"),
      entry_point: String::from("main"),
    };

    let mut builder = ShaderBuilder::new();
    let triangle_vs = builder.build(triangle_vertex);
    let triangle_fs = builder.build(triangle_fragment);

    let post_vs = builder.build(VirtualShader::Source {
      source: POST_VERTEX_SHADER_SOURCE.to_string(),
      kind: ShaderKind::Vertex,
      entry_point: "main".to_string(),
      name: "offscreen-post".to_string(),
    });
    let post_fs = builder.build(VirtualShader::Source {
      source: POST_FRAGMENT_SHADER_SOURCE.to_string(),
      kind: ShaderKind::Fragment,
      entry_point: "main".to_string(),
      name: "offscreen-post".to_string(),
    });

    return OffscreenPostExample {
      triangle_vs,
      triangle_fs,
      post_vs,
      post_fs,
      quad_mesh: None,
      offscreen_pass: None,
      offscreen_pipeline: None,
      offscreen_target: None,
      post_pass: None,
      post_pipeline: None,
      post_bind_group: None,
      post_layout: None,
      width: 800,
      height: 600,
    };
  }
}
```

This keeps shader construction out of `on_attach` so the component can build
pipelines immediately from the stored `Shader` values.

### Step 4 — Implement `Component` and Build Resources <a name="step-4"></a>

Implement the component lifecycle. This example creates the offscreen target,
passes, pipelines, and bind group in `on_attach`, and records two render passes
each frame in `on_render`.

```rust
impl Component<ComponentResult, String> for OffscreenPostExample {
  fn on_attach(
    &mut self,
    render_context: &mut RenderContext,
  ) -> Result<ComponentResult, String> {
    logging::info!("Attaching OffscreenPostExample");

    let surface_size = render_context.surface_size();
    let offscreen_target = OffscreenTargetBuilder::new()
      .with_color(
        render_context.surface_format(),
        surface_size.0,
        surface_size.1,
      )
      .with_label("offscreen-post-target")
      .build(render_context.gpu())
      .map_err(|e| format!("Failed to build offscreen target: {:?}", e))?;
    let offscreen_target_id =
      render_context.attach_offscreen_target(offscreen_target);

    let offscreen_pass =
      RenderPassBuilder::new().with_label("offscreen-pass").build(
        render_context.gpu(),
        render_context.surface_format(),
        render_context.depth_format(),
      );

    let offscreen_pipeline = RenderPipelineBuilder::new()
      .with_label("offscreen-pipeline")
      .with_culling(CullingMode::None)
      .build(
        render_context.gpu(),
        render_context.surface_format(),
        render_context.depth_format(),
        &offscreen_pass,
        &self.triangle_vs,
        Some(&self.triangle_fs),
      );

    let post_pass = RenderPassBuilder::new().with_label("post-pass").build(
      render_context.gpu(),
      render_context.surface_format(),
      render_context.depth_format(),
    );

    let post_layout = BindGroupLayoutBuilder::new()
      .with_sampled_texture(1)
      .with_sampler(2)
      .build(render_context.gpu());

    let sampler = SamplerBuilder::new()
      .linear_clamp()
      .with_label("offscreen-post-sampler")
      .build(render_context.gpu());

    let offscreen_ref =
      render_context.get_offscreen_target(offscreen_target_id);
    let post_bind_group = BindGroupBuilder::new()
      .with_layout(&post_layout)
      .with_texture(1, offscreen_ref.color_texture())
      .with_sampler(2, &sampler)
      .build(render_context.gpu());

    let quad_mesh = Self::build_fullscreen_quad_mesh();
    let quad_vertex_buffer =
      BufferBuilder::build_from_mesh(&quad_mesh, render_context.gpu())
        .map_err(|e| format!("Failed to build quad vertex buffer: {:?}", e))?;

    let post_pipeline = RenderPipelineBuilder::new()
      .with_label("post-pipeline")
      .with_culling(CullingMode::None)
      .with_layouts(&[&post_layout])
      .with_buffer(quad_vertex_buffer, quad_mesh.attributes().to_vec())
      .build(
        render_context.gpu(),
        render_context.surface_format(),
        render_context.depth_format(),
        &post_pass,
        &self.post_vs,
        Some(&self.post_fs),
      );

    self.offscreen_pass =
      Some(render_context.attach_render_pass(offscreen_pass));
    self.offscreen_pipeline =
      Some(render_context.attach_pipeline(offscreen_pipeline));
    self.offscreen_target = Some(offscreen_target_id);

    self.post_pass = Some(render_context.attach_render_pass(post_pass));
    self.post_pipeline = Some(render_context.attach_pipeline(post_pipeline));
    self.post_bind_group =
      Some(render_context.attach_bind_group(post_bind_group));
    self.post_layout = Some(post_layout);
    self.quad_mesh = Some(quad_mesh);

    let (width, height) = render_context.surface_size();
    self.width = width;
    self.height = height;

    return Ok(ComponentResult::Success);
  }

  fn on_detach(
    &mut self,
    _render_context: &mut RenderContext,
  ) -> Result<ComponentResult, String> {
    return Ok(ComponentResult::Success);
  }

  fn event_mask(&self) -> lambda::events::EventMask {
    return lambda::events::EventMask::WINDOW;
  }

  fn on_window_event(
    &mut self,
    event: &lambda::events::WindowEvent,
  ) -> Result<(), String> {
    if let lambda::events::WindowEvent::Resize { width, height } = event {
      self.width = *width;
      self.height = *height;
    }
    return Ok(());
  }

  fn on_update(
    &mut self,
    _last_frame: &std::time::Duration,
  ) -> Result<ComponentResult, String> {
    return Ok(ComponentResult::Success);
  }

  fn on_render(
    &mut self,
    render_context: &mut RenderContext,
  ) -> Vec<RenderCommand> {
    self.ensure_offscreen_matches_surface(render_context);

    let offscreen_viewport =
      ViewportBuilder::new().build(self.width.max(1), self.height.max(1));
    let surface_viewport =
      ViewportBuilder::new().build(self.width.max(1), self.height.max(1));

    return vec![
      RenderCommand::BeginRenderPassTo {
        render_pass: self.offscreen_pass.expect("offscreen pass not set"),
        viewport: offscreen_viewport.clone(),
        destination: RenderDestination::Offscreen(
          self.offscreen_target.expect("offscreen target not set"),
        ),
      },
      RenderCommand::SetPipeline {
        pipeline: self.offscreen_pipeline.expect("offscreen pipeline not set"),
      },
      RenderCommand::SetViewports {
        start_at: 0,
        viewports: vec![offscreen_viewport.clone()],
      },
      RenderCommand::SetScissors {
        start_at: 0,
        viewports: vec![offscreen_viewport.clone()],
      },
      RenderCommand::Draw {
        vertices: 0..3,
        instances: 0..1,
      },
      RenderCommand::EndRenderPass,
      RenderCommand::BeginRenderPass {
        render_pass: self.post_pass.expect("post pass not set"),
        viewport: surface_viewport.clone(),
      },
      RenderCommand::SetPipeline {
        pipeline: self.post_pipeline.expect("post pipeline not set"),
      },
      RenderCommand::SetBindGroup {
        set: 0,
        group: self.post_bind_group.expect("post bind group not set"),
        dynamic_offsets: vec![],
      },
      RenderCommand::BindVertexBuffer {
        pipeline: self.post_pipeline.expect("post pipeline not set"),
        buffer: 0,
      },
      RenderCommand::SetViewports {
        start_at: 0,
        viewports: vec![surface_viewport.clone()],
      },
      RenderCommand::SetScissors {
        start_at: 0,
        viewports: vec![surface_viewport.clone()],
      },
      RenderCommand::Draw {
        vertices: 0..6,
        instances: 0..1,
      },
      RenderCommand::EndRenderPass,
    ];
  }
}
```

This produces two render passes: an offscreen triangle render and a post pass
that samples the offscreen color texture and draws a fullscreen quad.

### Step 5 — Fullscreen Quad Mesh <a name="step-5"></a>

Build the fullscreen quad mesh used by the post pass.

```rust
impl OffscreenPostExample {
  fn build_fullscreen_quad_mesh() -> Mesh {
    let vertices: [Vertex; 6] = [
      VertexBuilder::new()
        .with_position([-1.0, -1.0, 0.0])
        .with_normal([0.0, 0.0, 1.0])
        .with_color([0.0, 0.0, 0.0])
        .build(),
      VertexBuilder::new()
        .with_position([1.0, -1.0, 0.0])
        .with_normal([0.0, 0.0, 1.0])
        .with_color([1.0, 0.0, 0.0])
        .build(),
      VertexBuilder::new()
        .with_position([1.0, 1.0, 0.0])
        .with_normal([0.0, 0.0, 1.0])
        .with_color([1.0, 1.0, 0.0])
        .build(),
      VertexBuilder::new()
        .with_position([-1.0, -1.0, 0.0])
        .with_normal([0.0, 0.0, 1.0])
        .with_color([0.0, 0.0, 0.0])
        .build(),
      VertexBuilder::new()
        .with_position([1.0, 1.0, 0.0])
        .with_normal([0.0, 0.0, 1.0])
        .with_color([1.0, 1.0, 0.0])
        .build(),
      VertexBuilder::new()
        .with_position([-1.0, 1.0, 0.0])
        .with_normal([0.0, 0.0, 1.0])
        .with_color([0.0, 1.0, 0.0])
        .build(),
    ];

    let mut mesh_builder = MeshBuilder::new();
    for v in vertices {
      mesh_builder.with_vertex(v);
    }

    return mesh_builder
      .with_attributes(vec![
        VertexAttribute {
          location: 0,
          offset: 0,
          element: VertexElement {
            format: ColorFormat::Rgb32Sfloat,
            offset: 0,
          },
        },
        VertexAttribute {
          location: 1,
          offset: 0,
          element: VertexElement {
            format: ColorFormat::Rgb32Sfloat,
            offset: 12,
          },
        },
        VertexAttribute {
          location: 2,
          offset: 0,
          element: VertexElement {
            format: ColorFormat::Rgb32Sfloat,
            offset: 24,
          },
        },
      ])
      .build();
  }
}
```

The post vertex shader reads UV from `vertex_color.xy` at `location = 2`, which
is why the quad’s `VertexAttribute` for `location = 2` uses `offset: 24`.

### Step 6 — Record Commands in `on_render` <a name="step-6"></a>

`on_render` records two passes each frame. The offscreen pass targets
`RenderDestination::Offscreen` and draws `0..3` vertices. The post pass targets
the surface, binds set `0` and vertex buffer slot `0`, and draws `0..6`
vertices for the fullscreen quad.

### Step 7 — Resize Events and Resource Replacement <a name="step-7"></a>

`on_window_event` stores the new width/height and `ensure_offscreen_matches_surface`
rebuilds the offscreen target (and dependent bind group) when the sizes
diverge.

```rust
impl OffscreenPostExample {
  fn ensure_offscreen_matches_surface(
    &mut self,
    render_context: &mut RenderContext,
  ) {
    let offscreen_id = match self.offscreen_target {
      Some(id) => id,
      None => return,
    };
    let post_layout = match self.post_layout.as_ref() {
      Some(layout) => layout,
      None => return,
    };
    let bind_group_id = match self.post_bind_group {
      Some(id) => id,
      None => return,
    };

    let surface_size = render_context.surface_size();
    let target_size =
      render_context.get_offscreen_target(offscreen_id).size();
    if target_size == surface_size {
      return;
    }

    let new_target = match OffscreenTargetBuilder::new()
      .with_color(
        render_context.surface_format(),
        surface_size.0,
        surface_size.1,
      )
      .with_label("offscreen-post-target")
      .build(render_context.gpu())
    {
      Ok(target) => target,
      Err(error) => {
        logging::error!("Failed to rebuild offscreen target: {:?}", error);
        return;
      }
    };

    if let Err(error) =
      render_context.replace_offscreen_target(offscreen_id, new_target)
    {
      logging::error!("Failed to replace offscreen target: {}", error);
      return;
    }

    let offscreen_ref = render_context.get_offscreen_target(offscreen_id);
    let sampler = SamplerBuilder::new()
      .linear_clamp()
      .with_label("offscreen-post-sampler")
      .build(render_context.gpu());
    let new_bind_group = BindGroupBuilder::new()
      .with_layout(post_layout)
      .with_texture(1, offscreen_ref.color_texture())
      .with_sampler(2, &sampler)
      .build(render_context.gpu());

    if let Err(error) =
      render_context.replace_bind_group(bind_group_id, new_bind_group)
    {
      logging::error!("Failed to replace post bind group: {}", error);
    }
  }
}
```

This replacement path rebuilds both the offscreen target and the bind group so
the post pass samples the updated texture view after a resize.

### Step 8 — Main Entry Point <a name="step-8"></a>

Start the runtime using the example’s `main`.

```rust
fn main() {
  let runtime = ApplicationRuntimeBuilder::new("Offscreen Post Process")
    .with_window_configured_as(move |window_builder| {
      return window_builder
        .with_dimensions(1200, 600)
        .with_name("Offscreen Post Process");
    })
    .with_component(move |runtime, component: OffscreenPostExample| {
      return (runtime, component);
    })
    .build();

  start_runtime(runtime);
}
```

The resulting program opens a window, renders into an offscreen texture, and
presents the sampled result to the surface each frame.

## Validation <a name="validation"></a>

- Build: `cargo build --workspace`
- Run: `cargo run -p lambda-demos-render --bin offscreen_post`
- Expected behavior:
  - A window opens and shows a solid-color triangle.
  - Resizing the window preserves the rendering without stretching artifacts.

## Notes <a name="notes"></a>

- Format matching
  - The offscreen target and the offscreen pass/pipeline MUST agree on the
    color format. Use `render_context.surface_format()` to match the window.
- Bindings
  - `BindGroupLayoutBuilder::with_sampled_texture(1)` MUST match
    `layout (set = 0, binding = 1)` in the fragment shader.
  - The sampler binding index MUST also match (`binding = 2`).
- Resize
  - Replacing the offscreen target invalidates the previous texture view.
    Rebuild the bind group after calling
    `render_context.replace_offscreen_target`.
  - Viewports are built from `width.max(1)` and `height.max(1)` to avoid
    zero-size viewport creation during resize.

## Conclusion <a name="conclusion"></a>

This tutorial demonstrates a minimal multi-pass post path in `lambda-rs`:
render into an offscreen texture, then sample that texture to the surface using
a fullscreen quad and a bind group.

## Exercises <a name="exercises"></a>

- Exercise 1: Apply a post effect
  - Modify the post fragment shader to invert colors or apply a grayscale
    conversion before writing `fragment_color`.
- Exercise 2: Render offscreen at half resolution
  - Create the offscreen target at `width / 2`, `height / 2` and adjust UVs or
    sampling to upsample to the surface.
- Exercise 3: Add a debug border
  - Draw a second quad in the post pass that outlines the viewport to validate
    scissor and viewport behavior.
- Exercise 4: Add MSAA to the offscreen target
  - Enable multi-sampling on the offscreen target and ensure the pipeline and
    pass use the same sample count.
- Exercise 5: Add a second post pass
  - Render the first offscreen result into a second offscreen target, then
    sample the second target to the surface.
- Exercise 6: Sample with nearest filtering
  - Replace `.linear_clamp()` with nearest sampling and compare the result when
    rendering offscreen at reduced resolution.

## Changelog <a name="changelog"></a>

- 0.2.2 (2026-02-05): Update demo commands and reference paths for `demos/`.
- 0.2.1 (2026-01-16): Replace `on_event` resize handling with `event_mask()` and `on_window_event`.
- 0.2.0 (2025-12-31): Update the tutorial to match the example’s `Default`,
  `on_attach`, `on_render`, and resize replacement structure.
- 0.1.0 (2025-12-29): Initial draft aligned with
  `demos/render/src/bin/offscreen_post.rs`.
