# Lambda Args: Context‑Aware Providers and Interactive Wizard Specification

Status: Draft

Author(s): lambda-rs team

Target Crate: `lambda-rs-args`

Version: 0.1.0

## Table of Contents
- 1. Purpose and Scope
- 2. Background and Motivation
- 3. Goals and Non‑Goals
- 4. Key Concepts and Terminology
- 5. Architecture Overview
- 6. API Specification
  - 6.1 Core Traits and Types
  - 6.2 Parser Integration
  - 6.3 Providers Library (Built‑ins)
  - 6.4 Configuration and Environment Integration
  - 6.5 JSON Help/Discovery Output
- 7. Interactive Wizard (TUI) Specification
  - 7.1 Modes & Triggers
  - 7.2 UX Flows & Keybindings
  - 7.3 Accessibility & i18n
- 8. Error Handling & Diagnostics
- 9. Security & Privacy Considerations
- 10. Performance and Caching
- 11. Compatibility, Versioning & Backwards Compatibility
- 12. Testing Strategy
- 13. Model Use Cases (lambda‑rs)
  - 13.1 Backend Selection
  - 13.2 GPU Adapter Selection
  - 13.3 Surface/Present Mode Selection
  - 13.4 Shader System Selection
  - 13.5 Asset Path Discovery
- 14. Milestones & Rollout Plan
- 15. Open Questions & Future Work

---

## 1. Purpose and Scope

This document specifies a unique feature set for the `lambda-rs-args` argument
parser that differentiates it from existing CLI parsers: Context‑Aware Value
Providers with an optional Interactive Wizard (TUI) fallback. The model enables
dynamic, environment‑driven argument validation and selection for Lambda’s
graphics and tooling workflows (e.g., wgpu/gfx backend choice, adapter
enumeration, present mode filtering, shader system selection), while still
supporting strict, non‑interactive CLI workflows for CI and automation.

Scope includes technical requirements, public APIs, internal architecture,
integration with `lambda-rs` use cases, UX specification for the wizard,
diagnostics, security, and test strategies.

## 2. Background and Motivation

Traditional parsers validate against static sets of values. Lambda’s domain is
hardware‑ and platform‑dependent: not all adapters, backends, present modes, or
shader systems are valid on a given machine. Hard‑coding choices leads to poor
UX and “works on my machine” pitfalls.

Context‑Aware Providers resolve choices at parse time from the actual runtime
environment and project state. When non‑interactive parsing fails or input is
incomplete, a small optional TUI can guide users through valid choices.

## 3. Goals and Non‑Goals

Goals
- Provide dynamic value providers for arguments that can:
  - Enumerate valid choices based on runtime environment and project state.
  - Compute smart defaults and explain the rationale.
  - Validate inputs and produce actionable errors/suggestions.
- Offer an optional Interactive Wizard (TUI) that activates on demand or upon
  parse errors to resolve missing/invalid arguments.
- Emit machine‑readable help (JSON) including provider choices and reasons to
  support GUIs and tools.
- Preserve strict, non‑interactive behavior for automation and CI.

Non‑Goals
- Replace existing configuration systems; the feature augments CLI parsing and
  integrates with existing `env/config` precedence.
- Depend on heavyweight GUI frameworks; the wizard is a lightweight TUI.

## 4. Key Concepts and Terminology

- Provider: A component that knows how to enumerate valid choices for an
  argument, compute defaults, and validate a value in context.
- Context: Structured information available to providers: OS, environment
  variables, previously parsed args, subcommand, project root, GPU/displays.
- Choice: A value plus metadata (label, score, why, supported/unsupported).
- Interactive Wizard: An optional TUI that queries providers and guides
  selection.

## 5. Architecture Overview

At parse time, the `ArgumentParser` builds a `Context` (OS, env, subcommand,
partial args). For arguments registered with a Provider:
1. If the user supplied a value, the parser asks the provider to validate it.
2. If missing and interactive mode is off, the provider may supply a default.
3. If missing/invalid and interactive mode is on, the wizard queries the
   provider for choices and guides the user to a valid selection.
4. The selection flows back into the parser’s result.

Providers are pure Rust types implementing a trait. They must be fast,
side‑effect free (aside from discovery), and cancellable.

## 6. API Specification

### 6.1 Core Traits and Types

```rust
pub struct ProviderContext<'a> {
  pub os: &'a str,                  // e.g., "macos", "linux", "windows"
  pub subcommand: Option<&'a str>,
  pub env: &'a std::collections::HashMap<String, String>,
  pub cwd: std::path::PathBuf,
  pub args: &'a crate::ParsedArgs,  // already‑parsed values (partial)
  // Optional domain info (feature‑gated):
  pub gpu_info: Option<GpuSnapshot>,
  pub displays: Option<Vec<DisplayInfo>>,
}

#[derive(Clone, Debug)]
pub struct ProviderChoice<T> {
  pub value: T,                     // canonical value to apply
  pub label: String,                // human‑readable label
  pub description: Option<String>,  // more details
  pub score: i32,                   // ordering hint: higher is better
  pub supported: bool,              // false => disabled choice in UI
  pub why: Option<String>,          // why supported/unsupported
}

pub trait ValueProvider<T>: Send + Sync {
  fn id(&self) -> &'static str;

  // Discover valid choices. Should be quick and cacheable.
  fn choices(&self, ctx: &ProviderContext) -> Result<Vec<ProviderChoice<T>>, String>;

  // Compute default choice if the user didn’t provide a value.
  fn default(&self, ctx: &ProviderContext) -> Option<ProviderChoice<T>> {
    self.choices(ctx)
      .ok()
      .and_then(|mut v| { v.sort_by_key(|c| -c.score); v.into_iter().find(|c| c.supported) })
  }

  // Validate a user‑provided value; return Ok with normalized T or Err(reason).
  fn validate(&self, ctx: &ProviderContext, value: &str) -> Result<T, String>;
}
```

Argument registration addition:

```rust
impl Argument {
  pub fn with_provider<T: 'static + Send + Sync>(
    self,
    provider: Box<dyn ValueProvider<T>>,
  ) -> Self { /* store provider handle */ }
}

impl ArgumentParser {
  pub fn interactive_on_error(mut self, enable: bool) -> Self { /* ... */ self }
  pub fn with_context_builder(
    mut self,
    builder: impl Fn() -> ProviderContext<'static> + Send + Sync + 'static,
  ) -> Self { /* ... */ self }
}
```

### 6.2 Parser Integration

- If an argument has a provider and the user supplies a value:
  - Parser calls `provider.validate(ctx, raw)`; on `Err`, either fails (strict)
    or, if interactive enabled, wizard prompts with choices.
- If missing value:
  - Parser calls `provider.default(ctx)`; if `Some`, apply.
  - If `None` and interactive enabled, wizard launches for selection.
- Parsed (and wizard‑resolved) values feed into `ParsedArgs`.

### 6.3 Providers Library (Built‑ins)

Initial built‑ins for lambda‑rs use cases (feature‑gated where applicable):

- `BackendProvider` (wgpu/gfx): enumerate supported backends (`wgpu/metal`,
  `wgpu/vulkan`, etc.). Score preferred backend per OS.
- `GpuAdapterProvider`: enumerate GPU adapters (Discrete/Integrated/Software),
  with VRAM, vendor, device id; filter unsupported.
- `PresentModeProvider`: compute supported present modes for selected backend /
  surface size / vsync; disable unsupported modes with `why`.
- `ShaderSystemProvider`: offer `naga` or `shaderc` based on platform/toolchain
  availability; explain disabled ones.
- `AssetProvider`: discover assets by glob patterns; show previews/metadata.

Providers should avoid long blocking calls; discoveries must be cancellable.

### 6.4 Configuration and Environment Integration

Precedence remains: CLI > Env > Config > Provider Default > Interactive.

Providers should respect existing parsed values (e.g., selected backend),
allowing dependent providers (e.g., present mode filtering) to compute choices
from partial context. `ProviderContext.args` gives access to already‑resolved
arguments.

### 6.5 JSON Help/Discovery Output

`--help --format json` returns the static help plus a `dynamic` field with
provider choices and reasons to facilitate GUIs.

Example schema (truncated):

```json
{
  "name": "obj-loader",
  "description": "Tool to render obj files",
  "options": [
    { "name": "--backend", "type": "string", "required": false },
    { "name": "--adapter", "type": "string", "required": false }
  ],
  "dynamic": {
    "--backend": {
      "provider": "backend",
      "choices": [
        { "value": "wgpu/metal", "label": "Metal", "score": 100, "supported": true },
        { "value": "wgpu/vulkan", "label": "Vulkan", "score": 80, "supported": false, "why": "Driver not present" }
      ],
      "default": "wgpu/metal"
    }
  }
}
```

## 7. Interactive Wizard (TUI) Specification

### 7.1 Modes & Triggers

- `interactive_on_error(true)`: launch wizard when a required/provider argument
  is missing or invalid.
- `--wizard`: force wizard even if all args present; allow reviewing/resolving.
- `--no-wizard`: disable interactive fallback regardless of parser defaults.

### 7.2 UX Flows & Keybindings

Minimal flows per argument with a provider:
- List view: show `label`, `why` (if disabled), and highlight default.
- Confirm selection writes value back to parser.
- Allow “Explain unsupported” (toggle or key) to show `why`.
- Persist chosen defaults to a config file when the user opts in.

Keybindings (suggested):
- Up/Down (j/k): navigate choices.
- Enter: select.
- Space: toggle details.
- Esc/q: cancel wizard; return to parse result (error if still missing).

### 7.3 Accessibility & i18n
- Non‑color TUI fallback and high‑contrast mode.
- All strings sourced via a simple i18n layer to enable translation.

## 8. Error Handling & Diagnostics

- Rich errors: include provider `id`, value, and contextual hints.
- Trace mode: `--trace-providers` prints resolution order, defaults applied,
  unsupported options with reasons, and source of final value.
- Wizard gracefully handles provider errors (shows a message, offers retry).

## 9. Security & Privacy Considerations

- Providers should not exfiltrate system info; any network access must be
  opt‑in and documented (default providers operate offline).
- Respect sandboxing and permission boundaries.
- Do not collect PII; avoid writing outside project paths unless user confirms.

## 10. Performance and Caching

- Providers must return quickly; cache results per parse session.
- Expose an optional `ttl` on providers if longer‑lived caches are safe.
- Defer expensive provider queries until necessary (on demand in wizard).

## 11. Compatibility, Versioning & Backwards Compatibility

- Adding providers is backwards compatible.
- Existing arguments without providers continue to operate unchanged.
- Version gate provider API under a cargo feature `providers` (default on).

## 12. Testing Strategy

- Unit tests: mock providers to deterministically list choices/defaults and
  validate different error paths.
- Integration tests: launch parser in interactive off/on modes; simulate TUI
  flows by injecting selections (headless harness that bypasses UI draw loop).
- Snapshot tests for JSON help output.
- Lambda integration tests: ensure backend/adapter/present mode choices match
  platform capabilities in CI images (using stubbed platform layers).

## 13. Model Use Cases (lambda‑rs)

### 13.1 Backend Selection
Argument: `--backend`
Provider: Enumerate `wgpu` backends supported for OS. On macOS, score `wgpu/metal`
highest; on Linux, score Vulkan when available; disable others with reason.

### 13.2 GPU Adapter Selection
Argument: `--adapter`
Provider: Enumerate adapters (Discrete/Integrated/Software), label with vendor,
VRAM; filter based on required features. Default to the highest‑score supported.

### 13.3 Surface/Present Mode Selection
Argument: `--present-mode`
Provider: Validate against selected backend, surface, and vsync; show only
supported modes; annotate unsupported with reason (e.g., “Mailbox unsupported”).

### 13.4 Shader System Selection
Argument: `--shader-system`
Provider: Offer `naga` (default) and `shaderc` if toolchain available; explain
why `shaderc` may be disabled (cmake/ninja missing).

### 13.5 Asset Path Discovery
Argument: `--asset`
Provider: Glob assets under project `assets/`; show basic metadata and resolve
relative paths; default to a common asset if present.

## 14. Milestones & Rollout Plan

M1 (Core API)
- Add `ValueProvider` and `ProviderContext`.
- Implement parser hooks and precedence integration.
- Add JSON help dynamic section.

M2 (Built‑in Providers)
- Ship backend/adapter/present‑mode/shader‑system providers (feature‑gated).

M3 (Interactive Wizard)
- Add minimal TUI; wire `interactive_on_error(true)`. Persist choices to config
  upon opt‑in.

M4 (Tooling & Tests)
- Add mocks, headless wizard harness, and CI tests. Document providers in README.

## 15. Open Questions & Future Work

- Provider dependency graph: allow providers to declare hard dependencies and
  re‑compute choices when upstream values change.
- Network‑backed providers: opt‑in for asset registries or remote configs.
- GUI embedding: expose providers and choices via an API suitable for editors.
- Telemetry (opt‑in): aggregate anonymized reasons for unsupported options to
  improve defaults.

