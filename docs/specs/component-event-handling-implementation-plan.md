---
title: "Component Event Handling Implementation Plan"
document_id: "component-event-handling-implementation-plan-2026-01-14"
status: "draft"
created: "2026-01-14T00:00:00Z"
last_updated: "2026-01-16T00:00:00Z"
version: "0.1.5"
engine_workspace_version: "2023.1.30"
wgpu_version: "26.0.1"
shader_backend_default: "naga"
winit_version: "0.29.10"
repo_commit: "9435ad1491b5930054117406abe08dd1c37f2102"
owners: ["lambda-sh"]
reviewers: ["engine", "rendering"]
tags: ["plan", "spec", "events", "components", "runtime"]
---

# Component Event Handling Implementation Plan

## Table of Contents

- [Summary](#summary)
- [Milestones](#milestones)
  - [Increment 1 — EventMask](#increment-1--eventmask)
  - [Increment 2 — Component handlers](#increment-2--component-handlers)
  - [Increment 3 — Runtime filtering](#increment-3--runtime-filtering)
  - [Increment 4 — Examples migration](#increment-4--examples-migration)
  - [Increment 5 — Docs updates](#increment-5--docs-updates)
- [Verification Strategy](#verification-strategy)
- [Risks and Mitigations](#risks-and-mitigations)
- [Changelog](#changelog)

## Summary

- Implement `docs/specs/component-event-handling.md` in commit-sized increments
  that preserve a buildable, testable workspace after each change.
- Implement the final API directly (no transitional `on_event` path).

## Milestones

Each increment MUST satisfy the verification commands listed in the increment
before it is committed.

### Increment 1 — EventMask

- Goal
  - Add `EventMask` and `Events::mask()` as specified, without changing runtime
    dispatch behavior.
- Changes
  - `crates/lambda-rs/src/events.rs`
    - Add `EventMask` newtype + category constants.
    - Add `EventMask::{contains, union}` (operator sugar optional).
    - Add `Events::mask(&self) -> EventMask` mapping existing variants.
    - Add unit tests for `contains`, `union`, and representative
      `Events::mask()` values.
- Verification
  - `cargo +nightly fmt --all`
  - `cargo test -p lambda-rs -- --nocapture`
  - `cargo clippy --workspace --all-targets -- -D warnings`
- Commit
  - Suggested message: `[add] EventMask and Events::mask.`
- Status
  - Completed (workspace).
  - Evidence: `crates/lambda-rs/src/events.rs`.

### Increment 2 — Component handlers

- Goal
  - Replace `Component::on_event` with opt-in granular handlers and
    `event_mask()`.
- Changes
  - `crates/lambda-rs/src/component.rs`
    - Add `event_mask(&self) -> EventMask` defaulting to `EventMask::NONE`.
    - Add default no-op `on_*_event` handlers returning `Result<(), E>`.
    - Remove `on_event` from the public API.
- Verification
  - `cargo +nightly fmt --all`
  - `cargo build --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`
- Commit
  - Suggested message: `[break] Replace Component::on_event with handlers.`
- Status
  - Completed (workspace).
  - Evidence: `crates/lambda-rs/src/component.rs`.

### Increment 3 — Runtime filtering

- Goal
  - Implement event-mask filtering in `ApplicationRuntime` and dispatch
    granular handlers, skipping uninterested components.
- Changes
  - `crates/lambda-rs/src/runtimes/application.rs`
    - Compute `event_mask = event.mask()` once per event.
    - Skip dispatch when `!component.event_mask().contains(event_mask)`.
    - Invoke the corresponding `on_*_event` handler with a borrowed payload.
    - Update handler error behavior:
      - When a granular handler returns `Err(e)`, publish
        `RuntimeEvent::ComponentPanic` with a message including `{:?}` for `e`.
    - Add unit tests for dispatch filtering using a small test component that
      records which handlers are invoked.
- Verification
  - `cargo +nightly fmt --all`
  - `cargo test -p lambda-rs -- --nocapture`
  - `cargo clippy --workspace --all-targets -- -D warnings`
- Commit
  - Suggested message: `[refactor] Filter event dispatch using EventMask.`
- Status
  - Completed (workspace).
  - Evidence: `crates/lambda-rs/src/runtimes/application.rs`.

### Increment 4 — Examples migration

- Goal
  - Update all examples to implement `event_mask()` and granular handlers.
- Changes
  - `crates/lambda-rs/examples/*.rs`
    - Add `event_mask()` values per example, based on used event categories.
    - Move event logic into `on_window_event`, `on_keyboard_event`,
      `on_mouse_event`, and `on_runtime_event` as appropriate.
- Verification
  - `cargo +nightly fmt --all`
  - `cargo build --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`
- Commit
  - Suggested message: `[update] Migrate examples to granular event handlers.`
- Status
  - Completed (workspace).
  - Evidence: `crates/lambda-rs/examples/`.

### Increment 5 — Docs updates

- Goal
  - Update user-facing docs and tutorials to match the new API and document
    migration steps, without changing runtime behavior further.
- Changes
  - `docs/rendering.md`
    - Replace `on_event` examples with `event_mask()` and `on_*_event`.
  - `docs/tutorials/*.md`
    - Update all references and code samples that use `on_event`.
    - Update tutorial metadata:
      - Bump `version` semantically.
      - Update `last_updated` and `repo_commit`.
      - Append a changelog entry.
  - `docs/specs/component-event-handling.md`
    - Update the Requirements Checklist with file paths and commit references
      for implemented items.
- Verification
  - `cargo +nightly fmt --all`
  - `cargo test --workspace`
  - `cargo clippy --workspace --all-targets -- -D warnings`
- Commit
  - Suggested message: `[docs] Update component event handling documentation.`
- Status
  - Completed (workspace).
  - Evidence: `docs/rendering.md`, `docs/tutorials/`,
    `docs/specs/component-event-handling.md`.

## Verification Strategy

- Per-increment gates
  - Each increment MUST pass its listed commands before it is committed.
- Local-only checks
  - When windowed examples are updated, a manual smoke run MAY be used to
    validate basic event wiring (for example, window resize still updates the
    render target).
- Error-path coverage
  - Unit tests SHOULD validate that:
    - `EventMask::NONE` excludes all categories.
    - `contains` and `union` behave as specified.
    - Dispatch invokes only handlers declared in `event_mask()`.

## Risks and Mitigations

- Risk: Partial migration leaves components not receiving events.
  - Mitigation: Ensure examples declare `event_mask()` for each handler they
    expect to run.
- Risk: Event dispatch refactor changes runtime ordering guarantees.
  - Mitigation: Preserve the existing component-stack iteration order and
    deliver exactly one handler per `Events` variant.

## Changelog

- 2026-01-14 (v0.1.0) — Initial implementation plan.
- 2026-01-14 (v0.1.1) — Align milestones with the current specification.
- 2026-01-14 (v0.1.3) — Mark runtime filtering increment complete.
- 2026-01-14 (v0.1.4) — Remove transitional milestones and re-number.
- 2026-01-16 (v0.1.5) — Mark docs updates increment complete.
