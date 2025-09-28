# Lambda RS Rendering Guide (wgpu backend)

This guide shows how to build windows, compile shaders, create pipelines,
upload vertex data, and submit draw commands using Lambda RS with the wgpu
backend (default).

The examples below mirror the code in `crates/lambda-rs/examples/` and are
intended as a concise reference.

## Prerequisites

- Rust 1.70+
- Run `scripts/setup.sh` once (git hooks, git-lfs)
- Build: `cargo build --workspace`
- Test: `cargo test --workspace`

## Features and Backends

Lambda uses `wgpu` by default. You can select specific platform backends via
crate features:

- `with-wgpu` (default)
- `with-wgpu-vulkan`, `with-wgpu-metal`, `with-wgpu-dx12`, `with-wgpu-gl`
- Optional shader backends: `with-shaderc`,
  `with-shaderc-build-from-source`

In your `Cargo.toml` (consumer project):

```toml
[dependencies]
lambda-rs = { path = "crates/lambda-rs", features = ["with-wgpu-metal"] }
```

## Runtime + Window

Create a windowed runtime and start it:

```rust
use lambda::{runtime::start_runtime, runtimes::ApplicationRuntimeBuilder};

fn main() {
  let runtime = ApplicationRuntimeBuilder::new("Minimal App")
    .with_window_configured_as(|w| w.with_dimensions(800, 600).with_name("Window"))
    .build();

  start_runtime(runtime);
}
```

## Components: lifecycle and events

Implement `Component` to receive events and render access:

```rust
use lambda::{
  component::Component,
  events::{Events, WindowEvent},
  render::{command::RenderCommand, RenderContext},
};

#[derive(Default)]
struct MyComponent { width: u32, height: u32 }

impl Component<(), String> for MyComponent {
  fn on_attach(&mut self, _rc: &mut RenderContext) -> Result<(), String> { Ok(()) }
  fn on_detach(&mut self, _rc: &mut RenderContext) -> Result<(), String> { Ok(()) }

  fn on_event(&mut self, e: Events) -> Result<(), String> {
    if let Events::Window { event: WindowEvent::Resize { width, height }, .. } = e {
      self.width = width; self.height = height;
    }
    Ok(())
  }

  fn on_update(&mut self, _dt: &std::time::Duration) -> Result<(), String> { Ok(()) }

  fn on_render(&mut self, _rc: &mut RenderContext) -> Vec<RenderCommand> {
    Vec::new()
  }
}
```

Attach your component via the builder’s `with_component` method.

## Shaders (GLSL via Naga)

Compile GLSL into SPIR-V at runtime using the default Naga backend:

```rust
use lambda::render::shader::{Shader, ShaderBuilder, ShaderKind, VirtualShader};

let vs_src = VirtualShader::Source {
  source: include_str!("../assets/shaders/triangle.vert").to_string(),
  kind: ShaderKind::Vertex,
  name: "triangle".into(),
  entry_point: "main".into(),
};
let fs_src = VirtualShader::Source {
  source: include_str!("../assets/shaders/triangle.frag").to_string(),
  kind: ShaderKind::Fragment,
  name: "triangle".into(),
  entry_point: "main".into(),
};

let mut shader_builder = ShaderBuilder::new();
let vs: Shader = shader_builder.build(vs_src);
let fs: Shader = shader_builder.build(fs_src);
```

## Render Pass

Create a pass with a clear color:

```rust
use lambda::render::render_pass::RenderPassBuilder;
use lambda_platform::wgpu::types as wgpu;

let pass = RenderPassBuilder::new()
  .with_clear_color(wgpu::Color { r: 0.02, g: 0.02, b: 0.06, a: 1.0 })
  .build(&render_context);
```

## Vertex Data: Mesh and Buffer

Build a mesh and upload to GPU:

```rust
use lambda::render::{
  buffer::BufferBuilder,
  mesh::MeshBuilder,
  vertex::{VertexAttribute, VertexElement},
  ColorFormat,
};

let mut mesh = MeshBuilder::new();
// Push vertices by builder; positions/colors shown
// ... populate mesh ...

let attrs = vec![
  VertexAttribute { location: 0, offset: 0, element: VertexElement { format: ColorFormat::Rgb32Sfloat, offset: 0 }},
  VertexAttribute { location: 2, offset: 0, element: VertexElement { format: ColorFormat::Rgb32Sfloat, offset: 24}},
];
mesh = mesh.with_attributes(attrs).build();

let vbo = BufferBuilder::build_from_mesh(&mesh, &mut render_context)
  .expect("failed to create VBO");
```

## Pipeline + Push Constants

Create a pipeline, with optional push constants:

```rust
use lambda::render::pipeline::{RenderPipelineBuilder, PipelineStage};

let pipeline = RenderPipelineBuilder::new()
  .with_push_constant(PipelineStage::VERTEX, 64) // size in bytes
  .with_buffer(vbo, mesh.attributes().to_vec())
  .build(&mut render_context, &pass, &vs, Some(&fs));

let pipeline_id = render_context.attach_pipeline(pipeline);
let pass_id = render_context.attach_render_pass(pass);
```

## Viewport and Scissor

```rust
use lambda::render::viewport::ViewportBuilder;

let vp = ViewportBuilder::new().build(width, height);
```

## Submitting Draw Commands

Important: All state (pipeline, viewport, scissors, buffers, push constants,
draw) must be recorded inside a render pass.

```rust
use lambda::render::command::RenderCommand as RC;

let cmds = vec![
  RC::BeginRenderPass { render_pass: pass_id, viewport: vp.clone() },
  RC::SetPipeline { pipeline: pipeline_id },
  RC::SetViewports { start_at: 0, viewports: vec![vp.clone()] },
  RC::SetScissors  { start_at: 0, viewports: vec![vp.clone()] },
  RC::BindVertexBuffer { pipeline: pipeline_id, buffer: 0 },
  RC::PushConstants { pipeline: pipeline_id, stage: PipelineStage::VERTEX, offset: 0, bytes: vec![0u32; 16] },
  RC::Draw { vertices: 0..3 },
  RC::EndRenderPass,
];

render_context.render(cmds);
```

## Resizing

When you receive a window resize event, the runtime already reconfigures the
surface; your component can track width/height for viewport setup:

```rust
if let Events::Window { event: WindowEvent::Resize { width, height }, .. } = e {
  self.width = width; self.height = height;
}
```

## Notes and Tips

- The engine enables `wgpu::Features::PUSH_CONSTANTS` for convenience.
  If your adapter doesn’t support push constants, consider a uniform buffer
  fallback.
- Shaders are compiled from GLSL via Naga; you can switch to `shaderc` with
  the feature flag `with-shaderc`.
- All examples live under `crates/lambda-rs/examples/` and are runnable via:
  `cargo run -p lambda-rs --example <name>`

