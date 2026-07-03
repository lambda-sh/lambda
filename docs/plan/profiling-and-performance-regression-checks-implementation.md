Temporary implementation plan for
`docs/specs/runtime/profiling-and-performance-regression-checks.md`.

Each step is intended to be independently testable and committable.
When executing this plan, implement exactly one step, run the listed
validation, and stop so the change can be reviewed and tested before moving to
the next step.

1) Add workspace, feature, and re-export scaffolding (no profiler behavior yet)
- Changes
  - Update `/Cargo.toml`:
    - Add `crates/lambda-rs-profiling` to the workspace members.
  - Add `crates/lambda-rs-profiling/Cargo.toml`.
  - Add `crates/lambda-rs-profiling/src/lib.rs` with a minimal crate skeleton.
  - Update `crates/lambda-rs/Cargo.toml`:
    - Add umbrella features:
      - `profiling`
      - `profiling-all`
    - Add granular features:
      - `profiling-cpu`
      - `profiling-gpu`
      - `profiling-export-summary`
      - `profiling-export-trace`
    - Wire the public features to `lambda-rs-profiling` and
      `lambda-rs-platform` using granular feature names only.
  - Update `crates/lambda-rs-platform/Cargo.toml`:
    - Add granular feature `profiling-gpu`.
  - Update `crates/lambda-rs/src/lib.rs`:
    - Add the gated `lambda::profiling` re-export.
  - Update `crates/lambda-rs-platform/src/lib.rs` only as needed so the new
    platform profiling module can be introduced in a later step without
    feature-gating churn.
- Validation
  - `cargo build -p lambda-rs-profiling`
  - `cargo build -p lambda-rs --features profiling-cpu`
  - `cargo build -p lambda-rs --features profiling-all`
- Commit message
  - `[add] profiling workspace and feature scaffolding.`

2) Add the public profiling API skeleton and no-op user macros
- Changes
  - Expand `crates/lambda-rs-profiling/src/lib.rs` into stable modules such as:
    - `builder.rs`
    - `profiler.rs`
    - `report.rs`
    - `error.rs`
    - `macros.rs`
  - Add the public placeholder types required by the specification:
    - `ProfilerBuilder`
    - `Profiler`
    - `ScopeGuard`
    - `FrameArtifacts`
    - `SummaryReport`
    - `TraceReport`
    - `ComparisonReport`
    - `ProfilerBuildError`
    - `ProfilerInstallError`
    - `ProfilerFlushError`
    - `ExportError`
    - `CompareError`
  - Export the required macros:
    - `profile_scope!`
    - `profile_function!`
    - `profile_counter!`
    - `profile_instant!`
  - Make macro expansion compile cleanly when the corresponding profiling
    feature is disabled.
  - Add Rustdoc on every public type and function introduced in this step.
  - Add unit tests in `crates/lambda-rs-profiling/src/lib.rs` or a dedicated
    test module that cover:
    - no-op macro compilation
    - `profile_scope!` returning a bindable guard value
    - `profile_function!` producing a stable string
- Validation
  - `cargo test -p lambda-rs-profiling -- --nocapture`
  - `cargo test -p lambda-rs --features profiling-cpu -- --nocapture`
- Commit message
  - `[add] profiling api skeleton and no-op macros.`

3) Implement builder validation and global installation semantics
- Changes
  - Complete `ProfilerBuilder` and `Profiler` setup behavior in
    `crates/lambda-rs-profiling`:
    - validate output directory handling
    - validate frame history limits
    - enforce first-writer-wins global installation
  - Keep collection/export behavior inert for now; focus on configuration,
    error handling, and global access.
  - Add targeted unit tests for:
    - invalid output directory
    - invalid frame history limit
    - repeated `install_global()` returning
      `ProfilerInstallError::AlreadyInstalled` or equivalent
    - `Profiler::global()` visibility after successful installation
- Validation
  - `cargo test -p lambda-rs-profiling builder -- --nocapture`
  - `cargo test -p lambda-rs-profiling install -- --nocapture`
- Commit message
  - `[add] profiling builder validation and global install.`

4) Implement the CPU event model and drop-based scope recording
- Changes
  - Add internal CPU collector modules under `crates/lambda-rs-profiling`,
    such as:
    - `collector/mod.rs`
    - `collector/cpu.rs`
    - `collector/thread_local.rs`
  - Implement:
    - span begin/end recording with `std::time::Instant`
    - `ScopeGuard` close-on-drop semantics
    - counter events
    - instant events
    - fixed-capacity thread-local event buffering
  - Keep all CPU names as `&'static str` in v1.
  - Add unit tests that cover:
    - `ScopeGuard` closing exactly once
    - nested scopes
    - counters and instants entering the per-thread buffer
    - buffer rollover without per-event heap allocation in the steady path
- Validation
  - `cargo test -p lambda-rs-profiling cpu -- --nocapture`
- Commit message
  - `[add] profiling cpu event collection.`

5) Add frame flushing, summary aggregation, and summary JSON export
- Changes
  - Implement `Profiler::flush_frame()` with CPU-side frame boundary handling.
  - Add aggregation logic for:
    - `count`
    - `total_ns`
    - `mean_ns`
    - `p50_ns`
    - `p95_ns`
    - `max_ns`
  - Add stable `SummaryReport` serialization behind the
    `profiling-export-summary` and `lambda-rs-profiling/export-summary`
    feature wiring.
  - Ensure top-level metadata is present and scope-level data remains stable
    for CI comparison.
  - Add unit tests for:
    - percentile calculations
    - empty and single-sample summaries
    - schema field stability
    - `flush_frame()` returning artifact paths only when export is enabled
- Validation
  - `cargo test -p lambda-rs-profiling --features export-summary -- --nocapture`
- Commit message
  - `[add] profiling summary aggregation and export.`

6) Add trace event export and frame artifact writing
- Changes
  - Implement `TraceReport` serialization behind
    `profiling-export-trace` and `lambda-rs-profiling/export-trace`.
  - Preserve ordering within the CPU clock domain during export.
  - Implement filesystem artifact writing for summary and trace outputs with
    actionable path-aware errors.
  - Extend `FrameArtifacts` so `flush_frame()` reports written paths.
  - Add unit tests for:
    - trace schema fields
    - `span_begin` and `span_end` ordering
    - counter and instant encoding
    - write failures returning actionable `ExportError` values
- Validation
  - `cargo test -p lambda-rs-profiling --features export-trace -- --nocapture`
  - `cargo test -p lambda-rs-profiling --features export-summary,export-trace -- --nocapture`
- Commit message
  - `[add] profiling trace export and artifact writing.`

7) Add summary comparison helpers and CPU-only regression fixtures
- Changes
  - Implement comparison helpers in `crates/lambda-rs-profiling`, including:
    - scope matching by `(name, clock_domain)`
    - relative and absolute tolerance checks
    - explicit reporting of the violating scope and metric
    - informational handling for scopes present only in the current run
  - Add unit tests for:
    - relative threshold only
    - absolute threshold only
    - combined threshold failure
    - missing baseline scope failure
    - extra current scope informational result
  - Add initial deterministic CPU regression fixtures under
    `crates/lambda-rs/tests/` that use user-authored instrumentation only and
    compare summary output against a checked-in baseline.
- Validation
  - `cargo test -p lambda-rs-profiling compare -- --nocapture`
  - `cargo test -p lambda-rs --features profiling profiling_regression -- --nocapture`
- Commit message
  - `[add] profiling comparison helpers and cpu regression fixtures.`

8) Add platform-side GPU timestamp capability and query plumbing
- Changes
  - Add a new internal module under `crates/lambda-rs-platform/src/wgpu/`,
    such as `profiling.rs`, that exposes vendor-free helpers for:
    - timestamp query capability detection
    - timestamp period access
    - query set allocation
    - query resolve
    - non-blocking readback resource management
  - Gate all of this behavior behind
    `lambda-rs-platform/profiling-gpu`.
  - Keep `wgpu` types contained inside `lambda-rs-platform`.
  - Add targeted tests in `crates/lambda-rs-platform` for capability probing
    and for non-panicking behavior when timestamp support is unavailable.
- Validation
  - `cargo test -p lambda-rs-platform --features profiling-gpu -- --nocapture`
- Commit message
  - `[add] platform gpu timestamp profiling support.`

9) Integrate GPU collection into `lambda-rs-profiling` with CPU-only fallback
- Changes
  - Add internal GPU collector support in `crates/lambda-rs-profiling`, such
    as `collector/gpu.rs`.
  - Implement:
    - GPU scope reservation and bookkeeping
    - separate `gpu` clock domain handling
    - a multi-frame query/readback ring
    - automatic CPU-only fallback when GPU timestamps are unsupported
  - Ensure GPU setup errors are actionable and do not panic.
  - Add targeted tests for:
    - fallback behavior when GPU timestamps are unavailable
    - query ring advancement without blocking the active frame
    - stable export labeling for `gpu` events
- Validation
  - `cargo test -p lambda-rs-profiling --features gpu -- --nocapture`
- Commit message
  - `[add] profiling gpu collector with cpu fallback.`

10) Add automatic CPU spans to `ApplicationRuntime`
- Changes
  - Update `crates/lambda-rs/src/runtimes/application.rs` to emit automatic
    CPU spans for:
    - runtime initialization
    - component attach
    - component detach
    - event mapping
    - event dispatch
    - per-frame update
    - per-component `on_update`
    - per-frame render
    - per-component `on_render`
  - Use stable span names and gate all behavior with granular profiling
    feature names only.
  - Add or extend integration tests under `crates/lambda-rs/tests/` to verify
    that runtime spans appear in the same active profiler instance as
    user-authored spans.
- Validation
  - `cargo test -p lambda-rs --features profiling-cpu runtime -- --nocapture`
- Commit message
  - `[add] runtime cpu profiling spans.`

11) Add automatic CPU spans to the render path
- Changes
  - Update the relevant render modules in `crates/lambda-rs/src/render/` to
    emit automatic CPU spans for:
    - surface frame acquisition
    - command encoder creation and command encoding
    - each render pass
    - queue submission
    - frame presentation
  - Reuse stable render pass labels in span names when available.
  - Ensure this instrumentation flows through the same installed profiler.
  - Add focused integration coverage for render-path summary content in
    `crates/lambda-rs/tests/`.
- Validation
  - `cargo test -p lambda-rs --features profiling-cpu render -- --nocapture`
- Commit message
  - `[add] render path cpu profiling spans.`

12) Add GPU render-pass instrumentation and export integration
- Changes
  - Wire render-pass begin/end points to the GPU collector using the new
    platform timestamp helpers.
  - Record render-pass GPU durations without treating them as synchronized
    wall-clock CPU times.
  - Ensure `flush_frame()` resolves completed GPU query batches and exports
    them under the `gpu` clock domain when supported.
  - Add integration coverage that:
    - verifies CPU-only fallback on unsupported adapters
    - verifies GPU render-pass spans on supported adapters
    - does not fail when support is absent
- Validation
  - `cargo test -p lambda-rs --features profiling-all -- --nocapture`
- Commit message
  - `[add] gpu render pass profiling integration.`

13) Add user-facing profiling docs, examples, and feature documentation
- Changes
  - Update `docs/features.md` with the exact profiling feature names, crate
    ownership, default state, umbrella composition, and expected runtime cost.
  - Add a short profiling guide or example that shows:
    - global profiler installation
    - user-authored `profile_scope!`
    - user-authored `profile_counter!`
    - frame flushing and artifact inspection
  - Cross-check all public Rustdoc in `crates/lambda-rs-profiling` and the
    `lambda::profiling` re-export surface.
- Validation
  - `cargo test -p lambda-rs --features profiling --doc`
  - `cargo test -p lambda-rs-profiling --doc`
- Commit message
  - `[docs] add profiling guide and feature documentation.`

14) Add end-to-end regression workflow coverage and final cleanup
- Changes
  - Finalize deterministic regression fixtures under
    `crates/lambda-rs/tests/` so CPU profiling always runs in generic CI and
    GPU profiling can be skipped cleanly on unsupported runners.
  - Add or refine any remaining integration tests for:
    - automatic runtime spans
    - user-authored spans in the same report
    - summary export
    - trace export
    - comparison failure messaging
  - Reconcile final symbol names and behavior against
    `docs/specs/runtime/profiling-and-performance-regression-checks.md`.
  - Run formatting and lint cleanup across the workspace.
- Validation
  - `cargo +nightly fmt --all`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo test -p lambda-rs-profiling --all-features -- --nocapture`
  - `cargo test -p lambda-rs --features profiling -- --nocapture`
  - `cargo test -p lambda-rs --features profiling-all -- --nocapture`
  - `cargo test --workspace`
- Commit message
  - `[docs] finalize profiling and regression workflow implementation.`
