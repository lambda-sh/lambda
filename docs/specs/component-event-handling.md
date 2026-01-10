---
title: "Component Event Handling"
document_id: "component-event-handling-2026-01-10"
status: "draft"
created: "2026-01-10T00:00:00Z"
last_updated: "2026-01-10T00:00:00Z"
version: "0.1.0"
engine_workspace_version: "2023.1.30"
wgpu_version: "26.0.1"
shader_backend_default: "naga"
winit_version: "0.29.10"
repo_commit: "45ae58a8f66208f4dcfeb0a08e2963f5248f9016"
owners: ["lambda-sh"]
reviewers: ["engine", "rendering"]
tags: ["spec", "events", "components", "runtime"]
---

# Component Event Handling

## Table of Contents
- [Summary](#summary)
- [Scope](#scope)
- [Terminology](#terminology)
- [Architecture Overview](#architecture-overview)
- [Design](#design)
  - [API Surface](#api-surface)
  - [Behavior](#behavior)
  - [Validation and Errors](#validation-and-errors)
  - [Cargo Features](#cargo-features)
- [Constraints and Rules](#constraints-and-rules)
- [Performance Considerations](#performance-considerations)
- [Requirements Checklist](#requirements-checklist)
- [Verification and Testing](#verification-and-testing)
- [Compatibility and Migration](#compatibility-and-migration)
- [Changelog](#changelog)

## Summary

- Replace the monolithic `Component::on_event` entry point with granular,
  opt-in `on_*_event` handlers that default to a no-op.
- Introduce `EventMask`, a compact bitmask used by runtimes to filter
  event dispatch to only components that declare interest in an event category.
- Reduce boilerplate pattern matching in components and reduce runtime work
  by skipping dispatch for uninterested components.

## Scope

### Goals

- Introduce `EventMask` as an O(1) event category filter in `lambda-rs`.
- Add granular `Component` event handler methods with default no-op behavior.
- Update `ApplicationRuntime` to skip dispatch for uninterested components.
- Remove `Component::on_event` from the public API.
- Update all examples to use the new event handling API.
- Add unit tests for `EventMask` behavior.
- Update user-facing documentation for component event handling.

### Non-Goals

- Generic `Handles<E>` trait systems for static event typing.
- Dynamic event subscription or unsubscription at runtime.
- Event prioritization, re-ordering, or new ordering guarantees.
- Changes to component lifecycle callbacks (`on_attach`, `on_detach`,
  `on_update`, `on_render`) beyond event system refactoring.

## Terminology

- Event category: a coarse grouping of events mapped 1:1 to `Events` variants
  (for example, window events and keyboard events).
- Event mask: a bitmask (`EventMask`) describing a set of event categories.
- Dispatch: invoking a component callback in response to an incoming event.
- Component stack: the ordered list of components stored by a runtime.

## Architecture Overview

- Crate `lambda-rs`
  - `events.rs` defines `Events` and its sub-event types (`WindowEvent`, `Key`,
    `Mouse`, `RuntimeEvent`, `ComponentEvent`).
  - `component.rs` defines `Component`, the trait implemented by user
    components.
  - `runtimes/application.rs` defines `ApplicationRuntime`, which maps platform
    events into `Events` and dispatches them to components.

Data flow

```
winit event loop
  └── ApplicationRuntime event mapping
        └── Events (category + payload)
              ├── event.mask() -> EventMask::<CATEGORY>
              └── for component in component_stack:
                    if component.event_mask().contains(event.mask()):
                      component.on_<category>_event(&payload)
```

## Design

### API Surface

- `EventMask` (`crates/lambda-rs/src/events.rs`)
  - `EventMask` MUST be a `Copy` newtype around an integer bitmask.
  - `EventMask` MUST define category constants:
    - `NONE`
    - `WINDOW`
    - `KEYBOARD`
    - `MOUSE`
    - `RUNTIME`
    - `COMPONENT`
  - `EventMask` MUST provide `contains(self, other) -> bool`.
  - `EventMask` MUST provide a union operation via `union(self, other) -> Self`.
  - `EventMask` MAY additionally implement operator traits (for example,
    `BitOr`) as sugar for `union`.

- `Events::mask` (`crates/lambda-rs/src/events.rs`)
  - `Events` MUST expose `mask(&self) -> EventMask`.
  - `mask` MUST return the category mask for the event variant:
    - `Events::Window { .. }` -> `EventMask::WINDOW`
    - `Events::Keyboard { .. }` -> `EventMask::KEYBOARD`
    - `Events::Mouse { .. }` -> `EventMask::MOUSE`
    - `Events::Runtime { .. }` -> `EventMask::RUNTIME`
    - `Events::Component { .. }` -> `EventMask::COMPONENT`

- `Component` (`crates/lambda-rs/src/component.rs`)
  - `Component` MUST remove the `on_event` method.
  - `Component` MUST add `event_mask(&self) -> EventMask` with a default
    implementation returning `EventMask::NONE`.
  - `Component` MUST add granular event handlers with default no-op behavior:
    - `on_window_event(&mut self, _event: &WindowEvent) -> Result<(), E>`
    - `on_keyboard_event(&mut self, _event: &Key) -> Result<(), E>`
    - `on_mouse_event(&mut self, _event: &Mouse) -> Result<(), E>`
    - `on_runtime_event(&mut self, _event: &RuntimeEvent) -> Result<(), E>`
    - `on_component_event(&mut self, _event: &ComponentEvent) -> Result<(), E>`
  - The signatures above MUST be used so default no-op implementations can
    return `Ok(())` without constraining `R`.

### Behavior

- Dispatch filtering
  - Runtimes that dispatch `Events` to components (for example,
    `ApplicationRuntime`) MUST compute `event_mask = event.mask()` once per
    incoming event.
  - Runtimes MUST skip dispatch to a component when
    `!component.event_mask().contains(event_mask)`.
  - A component that overrides an `on_*_event` handler MUST include the
    corresponding category bit in `event_mask`. Otherwise, the handler is not
    invoked.

- Dispatch mapping
  - When dispatch occurs, the runtime MUST call exactly one handler based on
    the event variant:
    - `Events::Window { event, .. }` -> `Component::on_window_event(&event)`
    - `Events::Keyboard { event, .. }` -> `Component::on_keyboard_event(&event)`
    - `Events::Mouse { event, .. }` -> `Component::on_mouse_event(&event)`
    - `Events::Runtime { event, .. }` -> `Component::on_runtime_event(&event)`
    - `Events::Component { event, .. }` ->
      `Component::on_component_event(&event)`

- Ordering
  - The runtime MUST preserve the existing component stack iteration order.
  - The runtime MUST preserve the existing event ordering from the platform
    event loop.

### Validation and Errors

- `Events::mask` MUST NOT return unknown or reserved category bits.
- `EventMask` bits beyond the defined categories are reserved for future
  expansion and MUST NOT be relied upon by components.
- Runtime handling of event handler failures
  - When a handler returns `Err(e)`, the runtime MUST treat it as a fatal
    component failure for the current run and MUST publish a
    `RuntimeEvent::ComponentPanic` (or the runtime-equivalent error signal)
    with a descriptive message that includes the formatted error value.

### Cargo Features

- This change introduces no new Cargo features.

## Constraints and Rules

- `EventMask` MUST remain small and cheap to copy.
  - `u8` is sufficient for the initial set of categories.
  - If more than 8 categories are required, the mask storage type MUST be
    expanded (for example, `u16` or `u32`) without changing the semantics of
    `contains` and `union`.
- Each `Events` variant MUST map to exactly one category bit.
- Category bits MUST remain stable within a major version of `lambda-rs`.

## Performance Considerations

- Recommendations
  - Runtimes SHOULD avoid cloning `Events` per component when dispatching.
  - Runtimes SHOULD compute the event category mask once per event.
  - Components SHOULD keep `event_mask` side-effect free and allocation free.
- Rationale
  - Skipping uninterested components reduces work on hot event dispatch paths.
  - Avoiding per-component `Events` clones reduces allocation and copy work.

## Requirements Checklist

- Functionality
  - [ ] `EventMask` defined in `crates/lambda-rs/src/events.rs`
  - [ ] `Events::mask()` implemented for all `Events` variants
  - [ ] `Component` updated with `event_mask` and `on_*_event` handlers
  - [ ] `Component::on_event` removed from the public API
  - [ ] `ApplicationRuntime` dispatch filters components by `EventMask`
- API Surface
  - [ ] Public `EventMask` constants documented
  - [ ] `Component` trait documentation updated
  - [ ] Backwards compatibility assessed and migration documented
- Validation and Errors
  - [ ] Error behavior specified for handler failures
  - [ ] Runtime publishes `RuntimeEvent::ComponentPanic` on handler errors
- Performance
  - [ ] Per-event dispatch avoids per-component `Events` clones
  - [ ] Filtering implemented before dispatch match
- Documentation and Examples
  - [ ] Examples updated to implement `event_mask` and granular handlers
  - [ ] `docs/features.md` checked for relevance (no new features expected)
  - [ ] Migration notes added to component documentation

For each checked item, include a reference to a commit, pull request, or file
path that demonstrates the implementation.

## Verification and Testing

- Unit Tests
  - `EventMask::contains` and `EventMask::union` cover:
    - `EventMask::NONE` behavior
    - union associativity for multiple categories
    - contains behavior for present and absent categories
  - `Events::mask` covers one representative value per variant.
  - Commands: `cargo test -p lambda-rs -- --nocapture`

- Integration Tests
  - A runnable scenario SHOULD validate that an event triggers a handler only
    on components whose `event_mask` includes the event category.
  - Commands: `cargo test --workspace`

## Compatibility and Migration

- Breaking changes
  - `Component::on_event` is removed.
  - Components MUST migrate to `event_mask` and granular handlers.

- Migration steps
  - Replace `fn on_event(&mut self, event: Events) -> Result<R, E>` with:
    - `fn event_mask(&self) -> EventMask` returning the relevant categories.
    - One or more `on_*_event` handlers for the event payload types previously
      matched inside `on_event`.
  - Pattern matching moves from `Events` to payload types:
    - `match event { Events::Window { event, .. } => ... }` becomes
      `fn on_window_event(&mut self, event: &WindowEvent) -> Result<(), E> { .. }`.

## Changelog

- 2026-01-10 (v0.1.0) — Initial draft.
