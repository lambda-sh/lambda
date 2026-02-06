---
title: "Release and Publishing Guide"
document_id: "release-guide-2025-09-28"
status: "living"
created: "2025-09-28T00:00:00Z"
last_updated: "2026-02-05T23:05:40Z"
version: "0.1.1"
engine_workspace_version: "2023.1.30"
wgpu_version: "26.0.1"
shader_backend_default: "naga"
winit_version: "0.29.10"
repo_commit: "544444652b4dc3639f8b3e297e56c302183a7a0b"
owners: ["lambda-sh"]
reviewers: ["engine", "rendering"]
tags: ["guide", "releases", "publishing", "ci"]
---

# Release and Publishing Guide

This guide explains how to cut a release, publish the Rust crates to
crates.io, and attach compiled artifacts to a GitHub Release using the
automated workflow in `.github/workflows/release.yml`.

## Versioning Scheme

- Format: `YYYY.MM.DD` for daily releases, with patch suffix `-NN` for
  same‑day follow‑ups (e.g., `2025.09.28`, `2025.09.28-01`).
- The workflow auto‑computes the next version by scanning existing tags.
- Tags are created as `v<version>` (e.g., `v2025.09.28-01`).

## One‑Time Setup

- Add repository secret `CARGO_REGISTRY_TOKEN` with a crates.io token that has
  publish rights for all workspace crates.
- Ensure branch protections allow the GitHub Actions bot to push to the main
  branch, or ask us to switch the workflow to open a PR instead of pushing.
- Run `scripts/setup.sh` once locally to install hooks and LFS, and prefer
  storing large assets through git‑lfs.

## How the Workflow Works

Jobs run in this order when manually triggered:

1) Validate
- Runs `cargo fmt --check`, `cargo clippy -D warnings`, and `cargo test` on
  Ubuntu.

2) Prepare version, tag, push
- Computes version from the provided date (or UTC today), auto‑increments
  `-NN` if a same‑day release already exists.
- Bumps all workspace crate versions and their internal dependency pins using
  `cargo workspaces version --exact`.
- Commits `[release] v<version>` and pushes to the current branch; pushes tag
  `v<version>`.

3) Publish to crates.io
- Uses `cargo workspaces publish --from-git --skip-published` to push all
  changed crates in dependency order. Already published versions are skipped.

4) Build tri‑platform artifacts
- Builds all binary targets in release mode for Linux, macOS, and Windows.
- Packages archives with:
  - `bin/` containing all workspace bin targets
  - `examples/minimal` if present
  - `crates/lambda-rs/assets/` if present
  - `README`, `LICENSE` when present
  - `VERSION` file and `.sha256` checksums

5) Create GitHub Release
- Creates a release for the tag and uploads the archives and checksums.
- Generates a markdown changelog with a compare link and a bullet list of
  commit links between the previous and current tag; used as the release body
  and attached as an asset.

## Running a Release

1) Pre‑flight
- Ensure CI is green on main. Locally, you can run:
  - `cargo fmt --all`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo test --workspace`

2) Dry run (safe test)
- Go to GitHub → Actions → `Release` → `Run workflow`.
- Set `dry_run: true` and leave the date blank to use today. This builds and
  packages artifacts but does not bump, tag, push, or publish.

3) Real release
- Run the workflow with `dry_run: false` (default). Optionally set
  `release_date` in `YYYY.MM.DD` if you need to back/forward‑date.
- The workflow updates versions on the branch you run it from (typically
  `main`), pushes the release commit and tag, publishes crates, builds, and
  creates a GitHub Release.

## Hotfix / Patch Releases

- To ship a same‑day fix, re‑run the workflow the same day. It will detect the
  previous tag and produce the next `-NN` suffix (e.g., `-01`, `-02`).
- Fixes should be ordinary commits on the branch; the release job will include
  them in the new tag.

## Changelog Details

- The workflow determines the previous `v*` tag and compares it with the new
  tag. If none exists, it marks the release as initial.
- The generated markdown contains:
  - A compare link (`/compare/prev...new`)
  - A list of commit subjects with links to each commit
- You can edit the release notes on GitHub after the run if you want to add
  highlights or screenshots.

## Troubleshooting

- Branch protections reject pushes
  - Symptom: The prepare step fails on `git push`.
  - Fix: Allow GitHub Actions to push to the branch, or switch the workflow to
    open a PR for the version bump.

- crates.io publish failures
  - Symptom: Network/registry hiccups yield partial publish.
  - Fix: Re‑run the `publish_crates` job or the entire workflow. Already
    published crates are skipped.

- Packaging misses a binary
  - Ensure the target is declared as a `[[bin]]` in its crate. The packager
    enumerates binary targets via `cargo metadata`.

- Assets not included
  - Only `crates/lambda-rs/assets` is packaged by default. If you need more
    assets included, expand the staging step in the workflow.

## Releasing New Crates in the Workspace

- Make sure new crates have `license`, `repository`, and `categories` fields
  set in `Cargo.toml`, and are members of the workspace.
- The workflow bumps versions for all workspace crates and publishes only those
  with changes present in the tag (`--from-git`).

## Manual Verification (optional)

- Before cutting a release, you can verify examples locally:
  - `cargo run -p lambda-demos-minimal --bin minimal`
- For native engine builds:
  - `scripts/compile_lambda.sh --build Debug`
  - `scripts/compile_and_test.sh --os MacOS`

## Changelog

- 0.1.1 – Update demo run commands for `demos/`.
- 0.1.0 – Initial authoring of the guide and workflow documentation.
