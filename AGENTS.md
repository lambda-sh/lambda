# Repository Guidelines

## Project Structure & Module Organization

Lambda is organised as a Cargo workspace with multiple crates.

* Core engine code lives in `crates/lambda-rs`
* Platform & dependency abstractions in `crates/lambda-rs-platform`
* CLI parsing utilities in `crates/lambda-rs-args`
* Logging in `crates/lambda-rs-logging`.

Shared tooling sits under `lambda-sh/` and `scripts/`,

* Shader and logo assets are versioned in `crates/lambda-rs/assets/`
* Integration tests run from `crates/lambda-rs/tests/`.

Run `scripts/setup.sh` once to install git hooks and git-lfs.
Use `cargo build --workspace` for a full Rust build and `cargo test --workspace`
to exercise unit and integration suites. Example binaries can be launched
with `cargo run --example minimal` while you iterate.

## Project Architecture & structure

### General architecture rules

* `lambda-rs` is the primary API exposed to end users and should not leak any
internal platform or dependency details.
* `lambda-rs-platform` is where lower level abstractions and dependency wrappers
should be designed to support the needs of the higher level framework.
* Use the builder pattern to expose resources provided by our APIs where
necessary (I.E. for creating a window, gpu, shaders, audio streams, etc).
* In libraries exposed by this repository, avoid using panic unless absolutely
necessary and allow for the user handle errors whenever possible.
* Errors should be as actionable & descriptive as possible, providing context
on what caused the error to occur.

### lambda-rs

This module located in `crates/lambda-rs` is the primary API offered by this
repository. It is a high level framework for building desktop applications,
games, and anything that utilizes GPU resources.

* Do not expose dependency code to the end user unless absolutely necessary. All
lower level dependency/vendor code should be abstracted in `lambda-rs-platform`
and then those abstractions should be imported into this library.

### lambda-rs-platform

This module located in `crates/lambda-rs-platform` is our platform and
dependency abstraction layer. This library provides wrappers for all of our
dependency code for our primary API, `lambda-rs`, allowing users to only focus
on the high level abstractions.

* While APIs here should aim to provide more stable interfaces for our
`lambda-rs`, this should be treated more as an internal API that is meant to
to support the needs of our primary framework, even if it means making non-backwards
compatible changes.

## Coding Practices, Styles, & Naming Conventions

* Follow Rust 2021 idioms with 2-space indentation and `max_width = 80` as
enforced by `rustfmt.toml`.
* Always run `cargo +nightly fmt --all` and
`cargo clippy --workspace --all-targets -- -D warnings` before sending changes.
* Module and file names stay snake_case.
* Public types use UpperCamelCase.
* constants use SCREAMING_SNAKE_CASE.
* Use explicit return statements.
* Do not use abbreviations or acronyms for variable names.
* Maintain readable spacing in new statements: keep spaces after keywords, around
  operators, and after commas/semicolons instead of tightly packed tokens.
* Add comprehensive documentation to all code & tests.
* Add documentation to any pieces of code that are not immediately clear/are
very complex.
* Rustdoc requirements for new/changed code:
  * All public functions, methods, and types MUST have Rustdoc comments.
  * Non-trivial private helper functions SHOULD have Rustdoc comments.
  * Rustdoc MUST follow this structure:
    * One-line summary sentence describing behavior.
    * Optional paragraph describing nuances, constraints, or invariants.
    * `# Arguments` section documenting each parameter.
    * `# Returns` section describing the return value.
    * `# Errors` section for `Result`-returning APIs describing failure cases.
    * `# Panics` section only if the implementation can panic (prefer avoiding
      panics in library code).
    * `# Safety` section for `unsafe` APIs describing required invariants.
* Do not add comments explaining why you removed code where the code used to be.

* Feature flags and documentation (brief)
  * Non‑essential code with production runtime cost (e.g., validation, extra logging) MUST be disabled by default in release builds and guarded behind explicit Cargo features. Debug builds MAY enable such checks via `debug_assertions`.
  * Add features in the crate that owns the behavior (e.g., rendering validation features live in `lambda-rs`). Prefer narrowly scoped flags over broad umbrellas; umbrella bundles MAY exist for convenience but MUST NOT be enabled by default.
  * Umbrella Cargo features (for example, `render-validation`, `render-validation-strict`, `render-validation-all`) MUST only compose granular feature flags. Code and documentation MUST gate behavior using granular feature names (for example, `render-validation-encoder`, `render-validation-instancing`) plus `debug_assertions`, never umbrella names.
  * Every granular feature that toggles behavior MUST be included in at least one umbrella feature for discoverability and consistency.
  * Do not leak platform/vendor details into public `lambda-rs` features; map high‑level features to `lambda-rs-platform` internals as needed.
  * Specifications that add or change behavior MUST list the exact Cargo features they introduce or rely on, and the same PR MUST update `docs/features.md` with: name, owning crate, default state (debug/release), summary, and expected runtime cost.
  * Do not include perf‑impacting features in a crate’s `default` feature set.

## Testing Guidelines

Unit tests reside alongside code; integration tests live in `crates/lambda-rs/tests/runnables.rs`. Add focused tests for new render paths or platform shims, grouping by feature. Run `cargo test -p lambda-rs -- --nocapture` when debugging rendering output, and keep coverage steady by updating or extending the runnable scenarios and examples. Document non-trivial manual verification steps in the PR body.

## Commit & Pull Request Guidelines

We follow the `[scope] message` style seen in `git log` (e.g. `[add] logging crate to packages.`). Each commit should remain narrowly scoped and buildable. Pull requests must describe intent, list test commands, and link any tracking issues. Include screenshots or clips if the change affects visuals, and mention required platform checks so reviewers can reproduce confidently.

## Setup & Tooling Tips

New contributors should enable the bundled pre-commit hooks via `pre-commit install` after running `scripts/setup.sh`. When working with the C++ engine archive, respect the `lambda_args_*` options exposed by `lambda-sh/lambda.sh` for consistent builds. Store large assets through git-lfs to keep history lean.

## Documentation Metadata & Authoring

To keep long‑lived docs consistent and traceable, include a metadata header (YAML front matter) at the top of all roadmap/spec/guide docs and follow the structure rules below.

Metadata schema (paste at the top of a doc):

---

title: "<short, descriptive title>"
document_id: "<stable-id, e.g., game-roadmap-YYYY-MM-DD>"
status: "draft"              # draft | living | frozen | deprecated
created: "<UTC ISO-8601>"   # e.g., 2025-09-24T00:00:00Z
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

Authoring guidance:

* Keep sections short and scannable; prefer bullets and code snippets.
* Include ASCII diagrams where helpful; avoid embedded binaries in repo.
* When proposing APIs, mirror existing builder/command patterns and show concise Rust sketches.
* Update `last_updated`, `version`, and `repo_commit` when making material changes; append a “Changelog” section.
* Prefer ISO‑8601 UTC timestamps. Use semantic versioning for the doc `version`.
* Create new specifications by copying `docs/specs/_spec-template.md` and
  completing the placeholders; do not start from an existing spec.
* Always create a table of contents for specs & tutorials.

### Documentation Tone & Style (Required)

All specs, and long‑lived docs must adopt a professional, precise tone:

* Voice and register
  * Use clear, direct, neutral language. Prefer active voice and present tense.
  * Avoid conversational phrasing, rhetorical questions, or tutorial chatter.
  * Do not use first‑person pronouns (“I”, “we”) or address the reader (“you”) unless the context requires an instruction; prefer “the API”, “the engine”, or “this document”.
  * Do not include meta commentary (e.g., “this aims to…”, “we want to…”). State requirements and behavior plainly.

* Normative language
  * Use RFC‑2119 keywords where appropriate: MUST, SHOULD, MAY, MUST NOT, SHOULD NOT.
  * When explaining decisions, use a short “Rationale” sub‑bullet rather than “Why” prose.

* Terminology and consistency
  * Define acronyms on first use in each document (e.g., “uniform buffer object (UBO)”), then use the acronym. Acronyms are permitted in docs (not in code).
  * Use consistent technical terms: `GPU`, `CPU`, `wgpu`, “bind group”, “pipeline layout”. Refer to code identifiers with backticks (e.g., `BindGroupLayout`).
  * Prefer American English spelling (e.g., “behavior”, “color”).
  * When describing implementation areas, refer to crates by name (for example, `lambda-rs`, `lambda-rs-platform`) instead of generic labels like “engine layer” or “platform layer”.

* Structure and formatting
  * Keep headings stable and diff‑friendly; avoid frequent renames.
  * Keep bullets to one line when possible; avoid filler sentences.
  * Include only minimal, buildable code snippets; match repository style.
  * Avoid marketing claims, subjective adjectives, and speculation.

* Metadata and changelog discipline
  * Update `last_updated`, bump `version` semantically, and set `repo_commit` to the current `HEAD` when making substantive edits.
  * Changelog entries use ISO‑8601 date, version, and a concise imperative summary of the changes (content, not process).

* Prohibitions
  * No emojis, exclamation marks for emphasis, or informal asides.
  * No references to AI authorship or generation.
  * No unscoped promises like “we will add…” without a linked tracking issue.
  * Do not add commentary about what you MUST or SHOULD do in the guidelines
  within specs or tutorials based on the AGENTS.md file.

### Tutorials Authoring (Required)

Tutorials are step‑by‑step instructional documents that teach a discrete task using the engine. They MUST follow the same metadata schema and tone rules as other docs, with the additions below.

* Location and naming
  * Place tutorials under `docs/tutorials/`.
  * Use kebab‑case filenames (e.g., `uniform-buffers.md`).
  * Include the `tutorial` tag in the metadata `tags` array.

* Tone and voice
  * Follow “Documentation Tone & Style (Required)” while adopting a book‑like instructional narrative.
  * Explain intent before each code block: what is about to be done and why it is necessary.
  * After each code block, include a short narrative paragraph that describes the outcome and what has been achieved.
  * Include a final Conclusion section that summarizes what was built and what outcomes were achieved across the tutorial.
  * Imperative instructions are preferred; limited second‑person (“you”) MAY be used to guide the reader when clarity improves.
  * Define acronyms on first use (e.g., “uniform buffer object (UBO)”) and then use the acronym consistently.

* Standard section structure
  * Goals: clearly state what will be built and what will be learned.
  * Overview: one short paragraph defining the task and constraints.
  * Prerequisites: version assumptions, build commands, and paths to examples.
  * Implementation Steps: numbered, with an explanation of intent and rationale preceding each code block, followed by a narrative paragraph after each code block that summarizes the resulting state and why it matters.
  * Validation: exact commands to build/run and expected visible behavior.
  * Notes: normative requirements and pitfalls using RFC‑2119 terms (MUST/SHOULD/MAY).
  * Conclusion: concise summary of accomplishments and the final state of the system built in the tutorial.
  * Exercises: 5–7 focused extensions that reuse concepts from the tutorial.
  * Changelog: ISO‑8601 date, version, and concise changes.

* Code and snippets
  * Match repository style: 2‑space indentation, line width ≈ 80, explicit `return` statements, and no abbreviations or acronyms in code identifiers.
  * Prefer minimal, buildable snippets; avoid large, redundant code blocks.
  * Reference files via workspace‑relative paths (e.g., `crates/lambda-rs/examples/uniform_buffer_triangle.rs`).
  * Use ASCII diagrams only; do not embed binaries.

* Metadata discipline
  * Update `last_updated`, bump `version` semantically, and set `repo_commit` to the current `HEAD` when making substantive edits.
  * Keep `document_id` stable across revisions; use semantic versioning in `version`.

* Consistency
  * Keep headings stable and diff‑friendly across tutorial revisions.
  * Use consistent terminology: `bind group`, `pipeline layout`, `GPU`, `CPU`, `wgpu`.

* Scope and process isolation
  * Do not reference internal process documents (e.g., this file) or include commentary about guideline updates within tutorials or specs.
