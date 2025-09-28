---
title: "Lambda RS Logging Guide"
document_id: "logging-guide-2025-09-28"
status: "living"
created: "2025-09-28T21:17:44Z"
last_updated: "2025-09-28T21:17:44Z"
version: "0.2.0"
engine_workspace_version: "2023.1.30"
wgpu_version: "26.0.1"
shader_backend_default: "naga"
winit_version: "0.29.10"
repo_commit: "637b82305833ef0db6079bcae5f64777e847a505"
owners: ["lambda-sh"]
reviewers: ["engine", "rendering"]
tags: ["guide", "logging", "engine", "infra"]
---

Summary
- Thread-safe logging crate with global and instance loggers.
- Minimal-overhead macros, configurable level, and pluggable handlers.
- Console, file, JSON, and rotating-file handlers included.

Quick Start
- Global macros:
  - Add dependency (rename optional): `logging = { package = "lambda-rs-logging", version = "2023.1.30" }`
  - Use: `logging::info!("hello {}", 123);`
- Custom global from env:
  - `logging::env::init_global_from_env().ok(); // honors LAMBDA_LOG`
- Custom logger:
  - `let logger = logging::Logger::builder().name("app").level(logging::LogLevel::INFO).with_handler(Box::new(logging::handler::ConsoleHandler::new("app"))).build();`

Core Concepts
- Level filtering: `TRACE < DEBUG < INFO < WARN < ERROR < FATAL`.
- Early guard: macros check level before formatting message.
- Handlers: `Send + Sync` sinks implementing `fn log(&self, record: &Record)`.
- Record fields: timestamp, level, target(name), message, module/file/line.

Configuration
- Global init:
  - Default global created on first use (`TRACE`, console handler).
  - Override once via `Logger::init(logger)`.
- Builder:
  - `Logger::builder().name(..).level(..).with_handler(..).build()`.
- Environment:
  - `LAMBDA_LOG=trace|debug|info|warn|error|fatal`.
  - `logging::env::apply_env_level(&logger, Some("LAMBDA_LOG"));`
  - `logging::env::init_global_from_env()` creates a console logger and applies env.

Handlers
- ConsoleHandler
  - Colors only when stdout/stderr are TTYs.
  - Writes WARN/ERROR/FATAL to stderr; TRACE/DEBUG/INFO to stdout.
- FileHandler
  - Appends colored lines to a file (legacy behavior). Flushes every 10 messages.
- JsonHandler
  - Newline-delimited JSON (one object per line). Minimal string escaping.
  - Use: `with_handler(Box::new(logging::handler::JsonHandler::new("/path/log.jsonl".into())))`.
- RotatingFileHandler
  - Rotates when current file exceeds `max_bytes`.
  - Keeps `backups` files as `file.1`, `file.2`, ...
  - Use: `RotatingFileHandler::new("/path/app.log".into(), 1_048_576, 3)`.

Macros
- `trace!/debug!/info!/warn!/error!/fatal!`
  - Use `$crate` and `format_args!` internally for low overhead when disabled.
  - Attach module/file/line to records for handler formatting.

Examples (run from repo root)
- `cargo run -p lambda-rs-logging --example 01_global_macros`
- `cargo run -p lambda-rs-logging --example 02_custom_logger`
- `cargo run -p lambda-rs-logging --example 03_global_init`
- `cargo run -p lambda-rs-logging --example 04_builder_env`
- `cargo run -p lambda-rs-logging --example 05_json_handler`
- `cargo run -p lambda-rs-logging --example 06_rotating_file`

Changelog
- 0.2.0: Added builder, env config, JSON and rotating file handlers; improved console (colors on TTY, WARN+ to stderr); macro early-guard.
- 0.1.0: Initial spec and thread-safe global with unified handler API.
