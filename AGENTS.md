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

## Documentation Skills

Use the repo-local Codex skills for detailed documentation workflows.

- Use `.codex/skills/long-lived-docs-authoring/SKILL.md` for specs, roadmaps,
  guides, plans, and other long-lived Markdown documents.
- Use `.codex/skills/tutorial-authoring/SKILL.md` for step-by-step tutorials
  in `docs/tutorials/`.
- Keep repo-wide policy in this file and keep detailed documentation authoring
  workflow in those skills.

## Documentation Requirements

- Long-lived docs MUST include YAML front matter with `title`,
  `document_id`, `status`, `created`, `last_updated`, `version`,
  `engine_workspace_version`, `wgpu_version`, `shader_backend_default`,
  `winit_version`, `repo_commit`, `owners`, `reviewers`, and `tags`.
- New specifications MUST be created from `docs/specs/_spec-template.md`.
- Specifications and tutorials MUST include a table of contents.
- Substantive documentation edits MUST update `last_updated`, `version`,
  `repo_commit`, and the `Changelog`.
- Long-lived docs MUST use a professional, precise tone with active voice,
  consistent terminology, American English spelling, and RFC-2119 keywords
  where appropriate.
- Tutorials MUST live in `docs/tutorials/`, use kebab-case filenames, include
  the `tutorial` tag, and include `Goals`, `Overview`, `Prerequisites`,
  `Implementation Steps`, `Validation`, `Notes`, `Conclusion`, `Exercises`,
  and `Changelog`.
