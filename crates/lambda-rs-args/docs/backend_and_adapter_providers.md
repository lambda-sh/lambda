# Implementing Backend and Adapter Providers

Status: Draft

This document shows practical, code‑level sketches for two context-aware providers proposed in the providers + interactive wizard spec: a Backend provider (chooses the best wgpu backend for the current OS/hardware) and a GPU Adapter provider (chooses among available adapters for the selected backend). These are designed to be feature-gated so core builds stay lean and headless CI remains deterministic.

## Table of Contents
- 1. Overview and Goals
- 2. Core Provider API (Recap)
- 3. BackendProvider (wgpu)
- 4. GpuAdapterProvider (wgpu)
- 5. Parser Wiring Examples
- 6. Interactive Wizard Hook
- 7. Validation and Diagnostics
- 8. Testing Strategy
- 9. Notes and Alternatives

---

## 1. Overview and Goals

- Enumerate valid choices from the real environment at parse time (no hard-coded lists).
- Prefer sensible defaults per platform (e.g., Metal on macOS, DX12 on Windows, Vulkan on Linux).
- Remain optional and non-intrusive: strict mode for CI, interactive wizard for newcomers.

## 2. Core Provider API (Recap)

Add these types to `lambda-rs-args` behind a `providers` feature (or a small companion crate) and wire them into `Argument`/`ArgumentParser` as described in the spec. For quick prototyping you can also call providers manually before `.parse()`.

```rust
pub struct ProviderContext<'a> {
  pub os: &'a str,                  // "macos" | "linux" | "windows" | ...
  pub subcommand: Option<&'a str>,
  pub env: &'a std::collections::HashMap<String, String>,
  pub cwd: std::path::PathBuf,
  pub args: &'a args::ParsedArgs,   // already-parsed values (partial)
}

#[derive(Clone, Debug)]
pub struct ProviderChoice<T> {
  pub value: T,
  pub label: String,
  pub description: Option<String>,
  pub score: i32,
  pub supported: bool,
  pub why: Option<String>,
}

pub trait ValueProvider<T>: Send + Sync {
  fn id(&self) -> &'static str;
  fn choices(&self, ctx: &ProviderContext) -> Result<Vec<ProviderChoice<T>>, String>;
  fn default(&self, ctx: &ProviderContext) -> Option<ProviderChoice<T>> {
    self.choices(ctx).ok().and_then(|mut v| {
      v.sort_by_key(|c| -c.score);
      v.into_iter().find(|c| c.supported)
    })
  }
  fn validate(&self, ctx: &ProviderContext, value: &str) -> Result<T, String>;
}
```

## 3. BackendProvider (wgpu)

Enumerates supported wgpu backends for the current OS and scores likely best picks. Values are canonical strings like `wgpu/metal`, `wgpu/vulkan`, `wgpu/dx12`, `wgpu/gl`.

```rust
#[cfg(feature = "with-wgpu")]
pub struct BackendProvider;

#[cfg(feature = "with-wgpu")]
impl ValueProvider<String> for BackendProvider {
  fn id(&self) -> &'static str { "backend" }

  fn choices(&self, ctx: &ProviderContext) -> Result<Vec<ProviderChoice<String>>, String> {
    use wgpu::{Backends, Instance, InstanceDescriptor};
    let candidates: Vec<(&str, Backends, i32)> = match ctx.os {
      "macos" => vec![("wgpu/metal", Backends::METAL, 100), ("wgpu/vulkan", Backends::VULKAN, 80), ("wgpu/gl", Backends::GL, 60)],
      "linux" => vec![("wgpu/vulkan", Backends::VULKAN, 100), ("wgpu/gl", Backends::GL, 60)],
      "windows" => vec![("wgpu/dx12", Backends::DX12, 100), ("wgpu/vulkan", Backends::VULKAN, 90), ("wgpu/gl", Backends::GL, 60)],
      _ => vec![("wgpu/gl", Backends::GL, 50)],
    };

    let mut out = Vec::new();
    for (label, backend_flag, score) in candidates {
      let instance = Instance::new(InstanceDescriptor { backends: backend_flag, ..Default::default() });
      let supported = !instance.enumerate_adapters(backend_flag).is_empty();
      out.push(ProviderChoice {
        value: label.to_string(),
        label: label.to_string(),
        description: Some(format!("backends={:?}", backend_flag)),
        score,
        supported,
        why: if supported { None } else { Some("no compatible adapter found".to_string()) },
      });
    }
    Ok(out)
  }

  fn validate(&self, _ctx: &ProviderContext, value: &str) -> Result<String, String> {
    let allowed = ["wgpu/metal", "wgpu/vulkan", "wgpu/dx12", "wgpu/gl"];
    if allowed.contains(&value) { Ok(value.to_string()) } else { Err(format!("unsupported backend: {}", value)) }
  }
}
```

## 4. GpuAdapterProvider (wgpu)

Enumerates adapters for a selected backend. Returns the adapter index as a string (e.g. `"0"`), which avoids ambiguity in case of duplicate adapter names.

```rust
#[cfg(feature = "with-wgpu")]
pub struct GpuAdapterProvider { pub backend_arg: &'static str }

#[cfg(feature = "with-wgpu")]
impl ValueProvider<String> for GpuAdapterProvider {
  fn id(&self) -> &'static str { "gpu_adapter" }

  fn choices(&self, ctx: &ProviderContext) -> Result<Vec<ProviderChoice<String>>, String> {
    use wgpu::{Adapter, AdapterInfo, Backends, Instance, InstanceDescriptor, DeviceType};

    let backend = ctx
      .args
      .get_string(self.backend_arg)
      .unwrap_or_else(|| default_backend_for_os(ctx.os));

    let backends_flag = match backend.as_str() {
      "wgpu/metal" => Backends::METAL,
      "wgpu/vulkan" => Backends::VULKAN,
      "wgpu/dx12" => Backends::DX12,
      "wgpu/gl" => Backends::GL,
      other => return Err(format!("unknown backend: {}", other)),
    };

    let instance = Instance::new(InstanceDescriptor { backends: backends_flag, ..Default::default() });
    let adapters: Vec<Adapter> = instance.enumerate_adapters(backends_flag).into_iter().collect();

    let mut out = Vec::new();
    for (i, adapter) in adapters.iter().enumerate() {
      let info: AdapterInfo = adapter.get_info();
      let score = match info.device_type {
        DeviceType::DiscreteGpu => 100,
        DeviceType::IntegratedGpu => 80,
        DeviceType::VirtualGpu => 60,
        DeviceType::Cpu => 10,
        DeviceType::Other => 40,
      };
      let label = format!("{} ({:?})", info.name, info.device_type);
      out.push(ProviderChoice {
        value: i.to_string(),
        label,
        description: Some(format!("vendor=0x{:04x} device=0x{:04x}", info.vendor, info.device)),
        score,
        supported: true,
        why: None,
      });
    }
    out.sort_by_key(|c| -c.score);
    Ok(out)
  }

  fn validate(&self, ctx: &ProviderContext, value: &str) -> Result<String, String> {
    let idx: usize = value.parse().map_err(|_| format!("expected adapter index, got '{}'", value))?;

    let backend = ctx
      .args
      .get_string(self.backend_arg)
      .unwrap_or_else(|| default_backend_for_os(ctx.os));

    let bf = match backend.as_str() {
      "wgpu/metal" => wgpu::Backends::METAL,
      "wgpu/vulkan" => wgpu::Backends::VULKAN,
      "wgpu/dx12" => wgpu::Backends::DX12,
      "wgpu/gl" => wgpu::Backends::GL,
      _ => wgpu::Backends::empty(),
    };

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor { backends: bf, ..Default::default() });
    let adapters = instance.enumerate_adapters(bf);
    if idx >= adapters.len() {
      return Err(format!("adapter index {} out of range 0..{}", idx, adapters.len().saturating_sub(1)));
    }
    Ok(value.to_string())
  }
}

fn default_backend_for_os(os: &str) -> String {
  match os {
    "macos" => "wgpu/metal".into(),
    "linux" => "wgpu/vulkan".into(),
    "windows" => "wgpu/dx12".into(),
    _ => "wgpu/gl".into(),
  }
}
```

## 5. Parser Wiring Examples

Strict mode (no wizard), providers supply defaults and validation. If the provider hooks are not yet in `Argument`, you can run providers first to compute defaults and then pass them into your args before calling `.parse()`.

```rust
use args::{Argument, ArgumentParser, ArgumentType};

// Build a context builder for providers
let ctx_builder = || {
  use std::collections::HashMap;
  let env: HashMap<String, String> = std::env::vars().collect();
  args::ProviderContext {
    os: std::env::consts::OS,
    subcommand: None,
    env: &env,
    cwd: std::env::current_dir().unwrap(),
    args: &args::ParsedArgs { values: vec![], subcommand: None }, // or from a prior pass
  }
};

let parser = ArgumentParser::new("obj-loader")
  //.with_context_builder(ctx_builder)
  .with_argument(Argument::new("--backend").with_type(ArgumentType::String))
  .with_argument(Argument::new("--adapter").with_type(ArgumentType::String));
  // With hooks: .with_provider(Box::new(BackendProvider)) etc.

let argv: Vec<String> = std::env::args().collect();
match parser.parse(&argv) {
  Ok(parsed) => {
    let backend = parsed.get_string("--backend").unwrap_or_else(|| default_backend_for_os(std::env::consts::OS));
    let adapter = parsed.get_string("--adapter").unwrap_or_else(|| "0".into());
    println!("backend={}, adapter={}", backend, adapter);
  }
  Err(e) => eprintln!("{}", e),
}
```

## 6. Interactive Wizard Hook

Once `interactive_on_error(true)` and provider hooks exist, the parser can:
- Detect missing/invalid `--backend` or `--adapter`.
- Query provider `choices()` and launch a TUI list where unsupported items are disabled with a `why` tooltip.
- On selection, return normalized values to the parse result.

This wizard can live in a separate optional crate (e.g., `lambda-rs-args-wizard`) so core stays dependency-free.

## 7. Validation and Diagnostics

- Detailed errors from providers:
  - `unsupported backend: wgpu/foo (did you mean 'wgpu/vulkan'?)`
  - `adapter index 3 out of range 0..1`
- Discovery JSON (`--help --format json`) should include dynamic provider choices and default selection to enable GUI wrappers.
- `--trace-providers` can print how defaults/choices were computed and why some were disabled.

## 8. Testing Strategy

- Mock providers implementing `ValueProvider<String>` to return deterministic `choices()` and `default()`; test parser handling without graphics dependencies.
- Headless wizard: inject selection programmatically to bypass real TUI drawing in tests.
- Platform CI: for `wgpu` providers, either stub `wgpu::Instance`/`enumerate_adapters` or run on a known environment to exercise at least one real path.

## 9. Notes and Alternatives

- If provider hooks are not yet merged into `lambda-rs-args`, consumers can still “preflight” providers, compute defaults, and append them to argv/env before calling `.parse()`.
- The `Adapter` provider returns indexes as strings for simplicity; you can instead return a stable ID (vendor/device pair) if you prefer, at the cost of more complex validation.
- Everything above is feature-gated on `with-wgpu` to avoid pulling GPU deps into unrelated tools.

