# Lambda RS: Immediate Feature Ideas + Example APIs

This document proposes high‑impact features to add next to the Lambda RS
rendering layer and shows what each would look like from user code.

The proposed APIs follow existing builder/command patterns and keep the
render‑pass‑centric model intact.

## 1) Uniform Buffers and Bind Groups

Why: Push constants are convenient but limited. Uniform buffers support larger
constant data, structured layouts, and compatibility with all adapters.

API sketch:

```rust
// New types
use lambda::render::{
  bind::{BindGroupLayoutBuilder, BindGroupBuilder, Binding},
  buffer::{BufferBuilder, Usage, Properties},
};

// Layout: set(0) has a uniform buffer at binding(0)
let layout = BindGroupLayoutBuilder::new()
  .with_uniform(binding = 0, visibility = PipelineStage::VERTEX)
  .build(&mut rc);

// Create and upload a uniform buffer
let ubo = BufferBuilder::new()
  .with_length(std::mem::size_of::<Globals>())
  .with_usage(Usage::UNIFORM)
  .with_properties(Properties::CPU_VISIBLE)
  .with_label("globals")
  .build(&mut rc, vec![initial_globals])?;

// Bind group that points the layout(0)@binding(0) to our UBO
let group0 = BindGroupBuilder::new()
  .with_layout(&layout)
  .with_uniform(binding = 0, &ubo)
  .build(&mut rc);

// Pipeline accepts optional bind group layouts
let pipe = RenderPipelineBuilder::new()
  .with_layouts(&[&layout])
  .with_buffer(vbo, attributes)
  .build(&mut rc, &pass, &vs, Some(&fs));

// Commands inside a render pass
RC::SetPipeline { pipeline: pipe_id },
RC::BindGroup { set: 0, group: group0_id, offsets: &[] },
RC::Draw { vertices: 0..3 },
```

## 2) Textures + Samplers (sampling in fragment shader)

Why: Texture rendering is foundational (images, sprites, materials).

API sketch:

```rust
use lambda::render::texture::{TextureBuilder, SamplerBuilder, TextureFormat};

let texture = TextureBuilder::new_2d(TextureFormat::Rgba8UnormSrgb)
  .with_size(512, 512)
  .with_data(&pixels)
  .with_label("albedo")
  .build(&mut rc);

let sampler = SamplerBuilder::new()
  .linear_clamp()
  .build(&mut rc);

// Layout: binding(0) uniform buffer, binding(1) sampled texture, binding(2) sampler
let layout = BindGroupLayoutBuilder::new()
  .with_uniform(0, PipelineStage::VERTEX | PipelineStage::FRAGMENT)
  .with_sampled_texture(1)
  .with_sampler(2)
  .build(&mut rc);

let group = BindGroupBuilder::new()
  .with_layout(&layout)
  .with_uniform(0, &ubo)
  .with_texture(1, &texture)
  .with_sampler(2, &sampler)
  .build(&mut rc);

RC::BindGroup { set: 0, group: group_id, offsets: &[] },
```

## 3) Depth/Stencil and MSAA

Why: 3D scenes and high‑quality rasterization.

API sketch:

```rust
use lambda_platform::wgpu::types as wgpu;

let pass = RenderPassBuilder::new()
  .with_clear_color(wgpu::Color::BLACK)
  .with_depth_stencil(
     depth_format = wgpu::TextureFormat::Depth32Float,
     depth_clear = 1.0,
     depth_write = true,
     depth_compare = wgpu::CompareFunction::Less,
  )
  .with_msaa(samples = 4)
  .build(&rc);

let pipe = RenderPipelineBuilder::new()
  .with_msaa(samples = 4)
  .with_depth_format(wgpu::TextureFormat::Depth32Float)
  .build(&mut rc, &pass, &vs, Some(&fs));
```

## 4) Indexed Draw + Multiple Vertex Buffers

Why: Standard practice for meshes.

API sketch:

```rust
// New commands
RC::BindIndexBuffer { buffer: ibo_id, format: IndexFormat::Uint32 },
RC::DrawIndexed { indices: 0..index_count, base_vertex: 0, instances: 0..1 },

// Pipeline builder already accepts multiple vertex buffers; extend examples to show slot 1,2…
```

## 5) Multi‑pass: Offscreen (Render‑to‑Texture)

Why: Post‑processing, shadow maps, deferred rendering.

API sketch:

```rust
use lambda::render::target::{RenderTargetBuilder};

let offscreen = RenderTargetBuilder::new()
  .with_color(TextureFormat::Rgba8UnormSrgb, width, height)
  .with_depth(TextureFormat::Depth32Float)
  .with_label("offscreen")
  .build(&mut rc);

// Pass 1: draw scene into `offscreen`
let p1 = RenderPassBuilder::new().with_target(&offscreen).build(&rc);

// Pass 2: sample offscreen color into swapchain
let p2 = RenderPassBuilder::new().build(&rc);

// Commands
RC::BeginRenderPass { render_pass: p1_id, viewport },
// ... draw scene ...
RC::EndRenderPass,
RC::BeginRenderPass { render_pass: p2_id, viewport },
RC::BindGroup { set: 0, group: fullscreen_group, offsets: &[] },
RC::Draw { vertices: 0..3 },
RC::EndRenderPass,
```

## 6) Compute Pipelines

Why: GPU compute for general processing.

API sketch:

```rust
use lambda::render::compute::{ComputePipelineBuilder, ComputeCommand};

let cs = compile_shader("…", ShaderKind::Compute);
let pipe = ComputePipelineBuilder::new().build(&mut rc, &cs);

// Dispatch after binding resources
let cmds = vec![
  ComputeCommand::Begin,
  ComputeCommand::BindGroup { set: 0, group: g0, offsets: &[] },
  ComputeCommand::Dispatch { x: 64, y: 1, z: 1 },
  ComputeCommand::End,
];
```

## 7) WGSL Support in Shader Builder

Why: Native shader language for wgpu; fewer translation pitfalls.

API sketch:

```rust
let wgsl = VirtualShader::Source {
  source: include_str!("shaders/triangle.wgsl").into(),
  kind: ShaderKind::Vertex, // or a new `ShaderKind::Wgsl` variant
  name: "triangle".into(),
  entry_point: "vs_main".into(),
};
let vs = ShaderBuilder::new().build(wgsl);
```

## 8) Shader Hot‑Reloading

Why: Faster iteration during development.

API sketch:

```rust
let mut watcher = ShaderWatcher::new().watch_file("shaders/triangle.vert");
if watcher.changed() {
  let new_vs = shader_builder.build(VirtualShader::File { path: …, kind: …, name: …, entry_point: … });
  pipeline.replace_vertex_shader(&mut rc, new_vs);
}
```

## 9) Cameras and Transforms Helpers

Why: Common math boilerplate for 2D/3D.

API sketch:

```rust
let camera = Camera::perspective(
  fov_radians = 60f32.to_radians(), aspect = width as f32 / height as f32,
  near = 0.1, far = 100.0,
).look_at(eye, center, up);

let ubo = Globals { view_proj: camera.view_proj() };
```

## 10) Input Mapping Layer

Why: Stable input names independent of layouts.

API sketch:

```rust
if input.pressed(Action::MoveForward) { position.z -= speed; }

// Configure once
InputMap::new()
  .bind(Action::MoveForward, KeyCode::KeyW)
  .bind(Action::MoveLeft, KeyCode::KeyA)
  .install();
```

---

These proposals aim to keep Lambda’s surface area small while unlocking common
workflows (texturing, uniforms, depth/MSAA, compute, multipass). I can begin
implementing any of them next; uniform buffers/bind groups and depth/MSAA are
usually the quickest wins for examples and demos.

