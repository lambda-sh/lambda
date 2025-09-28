# WGPU Support Overview

Lambda now ships with an experimental, opt-in rendering path implemented on top of [`wgpu`](https://github.com/gfx-rs/wgpu). This document walks through the changes introduced, how to enable the new feature, and provides code samples for building wgpu-powered experiences.

## Feature Summary
- Added `wgpu` and `pollster` as optional dependencies of `lambda-rs-platform`, plus new feature flags (`with-wgpu`, `with-wgpu-vulkan`, `with-wgpu-metal`, `with-wgpu-dx12`, `with-wgpu-gl`).
- Introduced `lambda_platform::wgpu`, a thin wrapper that mirrors the existing gfx-hal builders (instance, surface, GPU, frame acquisition) and exposes the `wgpu` types when the feature is enabled.
- Exposed `lambda::render::wgpu`, providing a high-level `ContextBuilder` and `RenderContext` so applications can render by issuing closures instead of manually managing swapchains.
- Added `lambda::runtimes::wgpu::WgpuRuntimeBuilder`, a runtime implementation that integrates the new context with Lambda's existing event loop and component model.
- Shipped a `wgpu_clear` example demonstrating a minimal runtime-driven wgpu application.

## Enabling WGPU
1. Opt into the feature on the `lambda` crate:
   ```bash
   cargo run --example wgpu_clear --features lambda-rs/with-wgpu
   ```
   You can also add the feature in your own `Cargo.toml`:
   ```toml
   [dependencies]
   lambda = { version = "2023.1.30", features = ["with-wgpu"] }
   ```
2. Choose a backend feature if you need more control (e.g. `with-wgpu-metal` on macOS).
3. Recent versions of `wgpu` depend on building `shaderc`; ensure CMake ≥ 3.5 and Ninja are installed. On macOS:
   ```bash
   brew install cmake ninja
   ```

## Key APIs
### Platform Builders
```rust
use lambda_platform::wgpu::{InstanceBuilder, SurfaceBuilder, GpuBuilder};

let instance = InstanceBuilder::new()
  .with_label("Lambda Instance")
  .build();

let mut surface = SurfaceBuilder::new()
  .with_label("Lambda Surface")
  .build(&instance, window_handle)?;

let gpu = GpuBuilder::new()
  .with_label("Lambda Device")
  .build(&instance, Some(&surface))?;
```

### Render Context
```rust
use lambda::render::wgpu::ContextBuilder;

let mut context = ContextBuilder::new()
  .with_present_mode(wgpu::types::PresentMode::Fifo)
  .with_texture_usage(wgpu::types::TextureUsages::RENDER_ATTACHMENT)
  .build(&window)?;

context.render(|device, queue, view, encoder| {
  let mut pass = encoder.begin_render_pass(&wgpu::types::RenderPassDescriptor {
    label: Some("lambda-wgpu-pass"),
    color_attachments: &[Some(wgpu::types::RenderPassColorAttachment {
      view,
      resolve_target: None,
      ops: wgpu::types::Operations {
        load: wgpu::types::LoadOp::Clear(wgpu::types::Color::BLACK),
        store: true,
      },
    })],
    depth_stencil_attachment: None,
  });
  drop(pass);
});
```

### Runtime Integration
```rust
use lambda::{
  runtime::start_runtime,
  runtimes::wgpu::WgpuRuntimeBuilder,
};

let runtime = WgpuRuntimeBuilder::new("Lambda WGPU App")
  .with_window_configured_as(|builder| builder.with_dimensions(960, 600))
  .with_render_callback(|_, _, view, encoder| {
    let mut pass = encoder.begin_render_pass(&wgpu::types::RenderPassDescriptor {
      label: Some("lambda-wgpu-clear"),
      color_attachments: &[Some(wgpu::types::RenderPassColorAttachment {
        view,
        resolve_target: None,
        ops: wgpu::types::Operations {
          load: wgpu::types::LoadOp::Clear(wgpu::types::Color {
            r: 0.15,
            g: 0.25,
            b: 0.45,
            a: 1.0,
          }),
          store: true,
        },
      })],
      depth_stencil_attachment: None,
    });
    drop(pass);
    Ok(())
  })
  .build()?;

start_runtime(runtime);
```

## Testing & Examples
- Unit tests cover builder defaults and error conversions for both platform and render layers (`cargo test --features lambda-rs/with-wgpu`).
- Run the sample: `cargo run --example wgpu_clear --features lambda-rs/with-wgpu`.
- If `cargo` fails while compiling `shaderc`, install/update CMake ≥ 3.5 or use a prebuilt shaderc toolchain.

## Migration Notes
- Existing gfx-hal paths remain the default; no code changes are required for current applications unless opting into wgpu.
- The feature set targets wgpu 23.x to align with the current `shaderc` toolchain. Future upgrades may require revisiting dependency versions or enabling wgpu's native shader translation features.

For further questions or suggestions, please open an issue in the repository.
