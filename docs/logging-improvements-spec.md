---
title: "lambda-rs Logging: Design Review and Improvement Spec"
document_id: "logging-spec-2025-09-28"
status: "draft"
created: "2025-09-28T05:35:01Z"
last_updated: "2025-09-28T05:35:01Z"
version: "0.1.0"
engine_workspace_version: "2023.1.30"
wgpu_version: "26.0.1"
shader_backend_default: "naga"
winit_version: "0.29.10"
repo_commit: "964d58c0658a4c5d23a4cf33f4a8ecc12458dad6"
owners: ["lambda-sh"]
reviewers: ["engine", "rendering"]
tags: ["spec", "logging", "engine", "infra"]
---

**Summary**
- Evaluate current `lambda-rs-logging` design and implementation.
- Propose a thread-safe, low-overhead, extensible logging API with clear migration.

**Goals**
- Thread-safe global and instance loggers without `unsafe`.
- Low allocation and minimal formatting when disabled.
- Simple API for console/file; room for structured output.
- Clear configuration (builder + env vars) and sane defaults.
- Compatibility bridge with the broader Rust ecosystem (`log`/`tracing`).

**Current State**
- Files: `crates/lambda-rs-logging/src/lib.rs`, `crates/lambda-rs-logging/src/handler.rs`.
- API: `Logger { name, level, handlers }` with per-level methods that take `&mut self` and `String`.
- Global: `pub(crate) static mut LOGGER: Option<Logger>` initialized in `Logger::global()` via `unsafe`.
- Handlers: `ConsoleHandler`, `FileHandler`; both colorize; file buffers 10 messages then writes.
- Macros: `trace!/debug!/…` call `logging::Logger::global()` and eagerly `format!(...)`.

**Key Issues**
- Unsound global singleton
  - `static mut` + `&'static mut` return is not thread-safe and is UB under concurrency.
- Macro design and overhead
  - Always allocates due to `format!` before level check; path uses `logging::` instead of `$crate`.
- Concurrency and ergonomics
  - `Logger` methods require `&mut self`; `Handler` requires `&mut self`, preventing concurrent logging; no `Send + Sync` guarantees.
- Fatal behavior
  - `Logger::fatal` calls `std::process::exit(1)` in a library crate; surprising and hard to test.
- File handler
  - Color codes in files; buffered writes with no `Drop` flush; potential data loss at shutdown; no newline guarantee.
- Output streams
  - All levels write to stdout; `WARN+` should prefer stderr.
- Timestamps and format
  - Seconds-since-epoch, not human-friendly; no timezone; no module/file/line metadata.
- Documentation drift
  - README example does not match API (`Logger::new` signature); crate/lib naming is confusing.

**Proposed Design**

- Core types
  - `enum Level { Trace, Debug, Info, Warn, Error }` plus optional `Off`.
  - `struct Record<'a> { ts: SystemTime, level: Level, target: &'a str, message: std::fmt::Arguments<'a>, module_path: Option<&'static str>, file: Option<&'static str>, line: Option<u32> }`.

- Handler trait
  - `pub trait Handler: Send + Sync { fn enabled(&self, level: Level, target: &str) -> bool { true } fn log(&self, record: &Record); }`
  - Rationale: single entry point, shared across levels; concurrency via `&self` with interior mutability where needed.

- Logger internals
  - `pub struct Logger { name: String, level: AtomicLevel, handlers: RwLock<Vec<std::sync::Arc<dyn Handler>>> }`
  - `AtomicLevel` is a thin wrapper over `AtomicU8` with `Level` conversions.
  - `impl Logger` exposes `builder()`, `level()`, `set_level()`, `add_handler()`, `clear_handlers()`.

- Global initialization
  - `static LOGGER: std::sync::OnceLock<std::sync::Arc<Logger>>`.
  - `pub fn init(logger: Logger) -> Result<(), InitError>`; first caller wins; no `unsafe`.
  - `pub fn global() -> &'static std::sync::Arc<Logger>` returns initialized default (console info) if `init` not called.

- Macros (zero/low-cost when disabled)
  - Use `$crate` for paths and `format_args!` to avoid allocation:
    - `macro_rules! log { ($lvl:expr, $($arg:tt)*) => {{ if $crate::enabled($lvl) { $crate::log_args($lvl, module_path!(), file!(), line!(), format_args!($($arg)*)); } }} }`
    - Define `trace!/debug!/info!/warn!/error!` in terms of `log!`.
  - Early guard using an atomic global level to skip work before formatting.

- Console handler
  - Color only when writing to a TTY (use `atty` or equivalent); write `WARN+` to `stderr`.
  - Default format: `YYYY-MM-DDTHH:MM:SS.sssZ level target: message`.

- File handler
  - No color; use `BufWriter<File>` behind a `Mutex` to ensure thread-safety.
  - Flush on newline or on `Drop`; optional size-based rotation later.

- Configuration
  - Builder: `Logger::builder().name("lambda-rs").level(Level::Info).with_handler(Arc<ConsoleHandler>::default()).build()`.
  - Env var parser (opt-in): `LAMBDA_LOG="info,lambda_rs_render=debug"` to set per-target levels.
  - Feature flags: `color` (default on), `env` (enable env parsing), `log-bridge`, `tracing-bridge`.

- Ecosystem bridges (optional, but recommended)
  - `log` facade: implement `log::Log` for a thin adapter that forwards to our `Logger`; provide `init_log_bridge()`.
  - `tracing` bridge (future): implement a basic `tracing_subscriber::Layer` emitting to handlers.

- API surface (sketch)
  - `pub fn enabled(level: Level) -> bool`
  - `pub fn log_args(level: Level, target: &str, file: &'static str, line: u32, args: Arguments)`
  - `pub mod macros { pub use crate::{trace, debug, info, warn, error}; }`

**Migration Plan**
- Phase 1: Internals refactor (Record, Handler::log, OnceLock global); keep old methods/macros working via shims.
- Phase 2: Macro rewrite with `$crate` and early guard; mark old `String`-based methods `#[deprecated]`.
- Phase 3: Introduce builder and env config; update examples/README.
- Phase 4: Optional bridges (`log` facade) and additional handlers (JSON, rotating file).

**Backwards Compatibility**
- Maintain `trace!/debug!/…` macro names; forward to new implementation.
- Keep `Logger::{trace,debug,…}(String)` for one release, delegating to the new `Arguments`-based path; deprecate with guidance.
- Remove `fatal` auto-exit; add `fatal_and_exit!` macro behind a `danger-exit` feature flag for explicit opt-in.

**Testing**
- Unit tests: level filtering, macro early-exit (no formatting when disabled), handler invocation order, builder config.
- Concurrency tests: multi-threaded logging to console/file without panics or data races.
- Integration tests: env var parsing; bridge to `log` (if feature enabled).
- Snapshot tests for formatted output (normalize timestamps via injection).

**Documentation Updates**
- Fix README examples to match signatures and import paths; clarify crate name vs lib name.
- Add a usage guide with builder, env config, and handler customization.
- Document migration notes and deprecations.

**Risks & Trade-offs**
- Adding `OnceLock`, `RwLock`, and atomics slightly increases complexity but removes UB and makes logging robust under concurrency.
- Bridging to `log`/`tracing` introduces optional deps; gate behind features to keep core lightweight.

**Open Questions**
- Do we want hierarchical targets (e.g., `engine.render` inheritance) in v1, or keep flat target-level map?
- Should we default to integrating with `log` to reduce duplicate macros across crates?

**Changelog**
- 0.1.0 (draft): Initial spec, proposes thread-safe global, unified handler API, macro rewrite, configuration, and ecosystem bridges.
