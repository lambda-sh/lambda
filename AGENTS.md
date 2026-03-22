# Repository Guidelines

## Project Structure & Module Organization

Lambda is organized as a Cargo workspace with multiple crates.

- Core engine code lives in `crates/lambda-rs`.
- Platform and dependency abstractions live in
  `crates/lambda-rs-platform`.
- CLI parsing utilities live in `crates/lambda-rs-args`.
- Logging utilities live in `crates/lambda-rs-logging`.

Shared tooling lives under `lambda-sh/` and `scripts/`.

- Shader and logo assets are versioned in `crates/lambda-rs/assets/`.
- Integration tests live in `crates/lambda-rs/tests/`.

Run `scripts/setup.sh` once to install git hooks and `git-lfs`.
Use `cargo build --workspace` for a full Rust build and
`cargo test --workspace` to exercise unit and integration suites.
Example binaries can be launched with `cargo run --example minimal`
while iterating.

## Project Architecture & Structure

### General Architecture Rules

- `lambda-rs` is the primary API exposed to end users and MUST NOT leak
  internal platform or dependency details.
- `lambda-rs-platform` contains lower-level abstractions and dependency
  wrappers and SHOULD be designed to support the needs of the higher-level
  framework.
- Use the builder pattern where appropriate to expose resources provided by
  the APIs, such as windows, GPUs, shaders, and audio streams.
- In libraries exposed by this repository, avoid panics unless absolutely
  necessary and allow the caller to handle errors whenever possible.
- Errors SHOULD be actionable and descriptive, including context about the
  cause of the failure.

### `lambda-rs`

`lambda-rs`, located in `crates/lambda-rs`, is the primary API offered by
this repository. It is a high-level framework for building desktop
applications, games, and other GPU-backed software.

- Do not expose dependency code to end users unless absolutely necessary.
  Lower-level dependency or vendor code SHOULD be abstracted in
  `lambda-rs-platform` and consumed from there.

### `lambda-rs-platform`

`lambda-rs-platform`, located in `crates/lambda-rs-platform`, is the
platform and dependency abstraction layer. This library provides wrappers
around dependency code used by `lambda-rs`, allowing the primary API to stay
focused on higher-level abstractions.

- APIs in this crate SHOULD aim for stable interfaces for `lambda-rs`, but
  this crate is still treated as an internal support layer.
  Non-backward-compatible changes are acceptable when they are required to
  support the primary framework.

## Coding Practices, Style, & Naming Conventions

- Follow Rust 2021 idioms with 2-space indentation and `max_width = 80`,
  as enforced by `rustfmt.toml`.
- Always run `cargo +nightly fmt --all` and
  `cargo clippy --workspace --all-targets -- -D warnings` before sending
  changes.
- Module and file names MUST remain `snake_case`.
- Public types MUST use `UpperCamelCase`.
- Constants MUST use `SCREAMING_SNAKE_CASE`.
- Use explicit return statements.
- Do not use abbreviations or acronyms for variable names.
- Maintain readable spacing in new statements: keep spaces after keywords,
  around operators, and after commas and semicolons instead of tightly packed
  tokens.
- Add comprehensive documentation to all code and tests.
- Add documentation to code that is complex or not immediately clear.

### Rustdoc Requirements

- All public functions, methods, and types MUST have Rustdoc comments.
- Non-trivial private helper functions SHOULD have Rustdoc comments.
- Rustdoc for new or changed APIs MUST follow this structure:
  - One-line summary sentence describing behavior.
  - Optional paragraph describing nuances, constraints, or invariants.
  - `# Arguments` section documenting each parameter.
  - `# Returns` section describing the return value.
  - `# Errors` section for `Result`-returning APIs describing failure cases.
  - `# Panics` section only if the implementation can panic. Prefer avoiding
    panics in library code.
  - `# Safety` section for `unsafe` APIs describing required invariants.

- Do not add comments explaining why code was removed where that code used to
  be.

### Feature Flags & Documentation

- Non-essential code with production runtime cost, such as validation or
  extra logging, MUST be disabled by default in release builds and guarded
  behind explicit Cargo features. Debug builds MAY enable such checks via
  `debug_assertions`.
- Add features in the crate that owns the behavior. For example, rendering
  validation features belong in `lambda-rs`.
- Prefer narrowly scoped feature flags over broad umbrella flags. Umbrella
  bundles MAY exist for convenience but MUST NOT be enabled by default.
- Umbrella Cargo features, such as `render-validation`,
  `render-validation-strict`, and `render-validation-all`, MUST only compose
  granular feature flags.
- Code and documentation MUST gate behavior using granular feature names, such
  as `render-validation-encoder` and `render-validation-instancing`, together
  with `debug_assertions`, never umbrella names.
- Every granular feature that toggles behavior MUST be included in at least
  one umbrella feature for discoverability and consistency.
- Public `lambda-rs` features MUST NOT leak platform or vendor details. Map
  high-level features to `lambda-rs-platform` internals as needed.
- Specifications that add or change behavior MUST list the exact Cargo
  features they introduce or rely on. The same change MUST also update
  `docs/features.md` with the feature name, owning crate, default state
  (debug or release), summary, and expected runtime cost.
- Do not include performance-impacting features in a crate's `default`
  feature set.

## Testing Guidelines

Unit tests live alongside the code. Integration test coverage lives under
`crates/lambda-rs/tests/`, with runnable scenarios defined in
`crates/lambda-rs/tests/runnables.rs`.

- Add focused tests for new render paths or platform shims, grouped by
  feature.
- Run `cargo test -p lambda-rs -- --nocapture` when debugging rendering
  output.
- Maintain coverage by updating or extending runnable scenarios and examples.
- Document non-trivial manual verification steps in the pull request body.

## Commit & Pull Request Guidelines

Commit messages follow the `[scope] message` style used in `git log`, such as
`[add] logging crate to packages.` Each commit SHOULD remain narrowly scoped
and buildable.

Pull requests MUST:

- Describe the intent of the change.
- List the test commands that were run.
- Link any tracking issues.
- Include screenshots or clips for visual changes.
- Note any required platform checks so reviewers can reproduce the change.

## Setup & Tooling Tips

- Run `scripts/setup.sh`, then enable the bundled hooks with
  `pre-commit install`.
- When working with the C++ engine archive, respect the `lambda_args_*`
  options exposed by `lambda-sh/lambda.sh` for consistent builds.
- Store large assets through `git-lfs` to keep history lean.

## Documentation Metadata & Authoring

Long-lived docs MUST include a metadata header with YAML front matter. Apply
this to roadmaps, specifications, guides, tutorials, and similar durable
documents.

Metadata schema:

```yaml
---
title: "<short, descriptive title>"
document_id: "<stable-id, e.g., game-roadmap-YYYY-MM-DD>"
status: "draft"              # draft | living | frozen | deprecated
created: "<UTC ISO-8601>"    # e.g., 2025-09-24T00:00:00Z
last_updated: "<UTC ISO-8601>"
version: "0.x.y"
engine_workspace_version: "2023.1.30"
wgpu_version: "26.0.1"
shader_backend_default: "naga"
winit_version: "0.29.10"
repo_commit: "<git sha at update>"
owners: ["lambda-sh"]
reviewers: ["engine", "rendering"]
tags: ["roadmap", "games", "2d", "3d", "desktop"]
---
```

### Authoring Guidance

- Keep sections short and scannable.
- Prefer bullets and concise code snippets.
- Include ASCII diagrams where helpful.
- Do not commit embedded binaries to the repository.
- When proposing APIs, mirror existing builder or command patterns and show
  concise Rust sketches.
- Update `last_updated`, `version`, and `repo_commit` when making material
  changes.
- Append a `Changelog` section for material updates.
- Prefer ISO-8601 UTC timestamps.
- Use semantic versioning for the document `version`.
- Create new specifications by copying `docs/specs/_spec-template.md` and
  completing the placeholders. Do not start from an existing specification.
- Always include a table of contents for specifications and tutorials.

### Documentation Tone & Style

All specifications and long-lived docs MUST adopt a professional and precise
tone.

#### Voice & Register

- Use clear, direct, neutral language.
- Prefer active voice and present tense.
- Avoid conversational phrasing, rhetorical questions, and tutorial chatter.
- Avoid first-person pronouns such as `I` and `we`.
- Do not address the reader directly unless the context requires an
  instruction. Prefer terms such as "the API", "the engine", or
  "this document".
- Do not include meta commentary, such as "this aims to" or "we want to".
  State requirements and behavior directly.

#### Normative Language

- Use RFC-2119 keywords where appropriate: MUST, SHOULD, MAY, MUST NOT, and
  SHOULD NOT.
- When explaining decisions, use a short `Rationale` sub-bullet instead of a
  "Why" paragraph.

#### Terminology & Consistency

- Define acronyms on first use in each document, such as
  "uniform buffer object (UBO)", then use the acronym consistently.
- Acronyms are permitted in documentation but not in code identifiers.
- Use consistent technical terms such as `GPU`, `CPU`, `wgpu`, "bind group",
  and "pipeline layout".
- Refer to code identifiers with backticks, such as `BindGroupLayout`.
- Prefer American English spelling, such as "behavior" and "color".
- When describing implementation areas, refer to crates by name, such as
  `lambda-rs` and `lambda-rs-platform`, rather than generic labels like
  "engine layer" or "platform layer".

#### Structure & Formatting

- Keep headings stable and diff-friendly.
- Avoid frequent heading renames.
- Keep bullets to one line when practical.
- Avoid filler sentences.
- Include only minimal, buildable code snippets.
- Match repository style in all examples.
- Avoid marketing claims, subjective adjectives, and speculation.

#### Metadata & Changelog Discipline

- Update `last_updated`, bump `version` semantically, and set `repo_commit` to
  the current `HEAD` when making substantive edits.
- Changelog entries MUST use an ISO-8601 date, the document version, and a
  concise imperative summary of the content change.

#### Prohibitions

- Do not use emojis, exclamation marks for emphasis, or informal asides.
- Do not reference AI authorship or generation.
- Do not make unscoped promises such as "we will add" without a linked
  tracking issue.
- Do not add commentary in specifications or tutorials about what must or
  should be done because of this `AGENTS.md` file.

## Tutorials Authoring

Tutorials are step-by-step instructional documents that teach a discrete task
using the engine. They MUST follow the same metadata schema and tone rules as
other docs, with the additions below.

### Location & Naming

- Place tutorials under `docs/tutorials/`.
- Use kebab-case filenames, such as `uniform-buffers.md`.
- Include the `tutorial` tag in the metadata `tags` array.

### Tone & Voice

- Follow the Documentation Tone & Style rules while using a book-like
  instructional narrative.
- Explain intent before each code block, including what is about to be done
  and why it is necessary.
- After each code block, include a short narrative paragraph describing the
  outcome and what was achieved.
- Include a final `Conclusion` section summarizing what was built and what the
  tutorial accomplished.
- Imperative instructions are preferred. Limited second-person language MAY be
  used when it materially improves clarity.
- Define acronyms on first use, such as "uniform buffer object (UBO)", and
  use them consistently.

### Standard Section Structure

- `Goals`: clearly state what will be built and what will be learned.
- `Overview`: provide a short paragraph defining the task and constraints.
- `Prerequisites`: list version assumptions, build commands, and paths to
  examples.
- `Implementation Steps`: use numbered steps. Each step MUST explain intent
  and rationale before the code block and include a short narrative paragraph
  afterward describing the resulting state and why it matters.
- `Validation`: provide exact commands to build and run, plus the expected
  visible behavior.
- `Notes`: capture normative requirements and pitfalls using MUST, SHOULD, and
  MAY.
- `Conclusion`: summarize the final system and what was accomplished.
- `Exercises`: include 5 to 7 focused extensions that reuse concepts from the
  tutorial.
- `Changelog`: record the ISO-8601 date, version, and concise change summary.

### Code & Snippets

- Match repository style: 2-space indentation, line width near 80 columns,
  explicit return statements, and no abbreviations or acronyms in code
  identifiers.
- Prefer minimal, buildable snippets.
- Avoid large, redundant code blocks.
- Reference files with workspace-relative paths, such as
  `crates/lambda-rs/examples/uniform_buffer_triangle.rs`.
- Use ASCII diagrams only.
- Do not embed binaries.

### Metadata Discipline

- Update `last_updated`, bump `version` semantically, and set `repo_commit` to
  the current `HEAD` when making substantive edits.
- Keep `document_id` stable across revisions.
- Use semantic versioning for the document `version`.

### Consistency

- Keep headings stable and diff-friendly across tutorial revisions.
- Use consistent terminology such as `bind group`, `pipeline layout`, `GPU`,
  `CPU`, and `wgpu`.

### Scope & Process Isolation

- Do not reference internal process documents, such as this file.
- Do not include commentary about guideline updates within tutorials or
  specifications.
