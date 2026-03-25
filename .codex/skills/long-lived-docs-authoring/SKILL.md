---
name: long-lived-docs-authoring
description: Use when drafting or revising long-lived repository documentation such as specs, roadmaps, guides, feature docs, or planning docs that need Lambda's required metadata, structure, and documentation style. Do not use for tutorials; use `tutorial-authoring` instead.
---

# Long Lived Docs Authoring

Use this skill for durable Markdown documents that are meant to stay in the
repository and be maintained over time. The goal is to apply Lambda's
documentation metadata, tone, and maintenance rules without keeping those
details in the main task prompt.

## When To Use

- New or updated specifications in `docs/specs/`.
- Roadmaps, plans, feature documents, and other durable docs in `docs/`.
- Documentation changes that need YAML front matter, changelog discipline,
  and repository-specific style constraints.
- Do not use this skill for tutorials in `docs/tutorials/`.
- Do not use this skill for Rustdoc-only work.

## Workflow

1. Identify the document type and location.
   Use `docs/specs/_spec-template.md` for new specifications instead of
   starting from another spec.
2. Add or preserve the metadata header.
   Long-lived docs require YAML front matter with these fields:
   `title`, `document_id`, `status`, `created`, `last_updated`, `version`,
   `engine_workspace_version`, `wgpu_version`, `shader_backend_default`,
   `winit_version`, `repo_commit`, `owners`, `reviewers`, and `tags`.
3. Structure the document for maintenance.
   Keep sections short, scannable, and diff-friendly. Specifications should
   include a table of contents. Use stable headings and avoid unnecessary
   renames.
4. Apply repository documentation style.
   Use clear, direct, neutral language in active voice. Prefer present tense.
   Avoid conversational phrasing, rhetorical questions, marketing language,
   speculation, emojis, exclamation-based emphasis, and AI authorship
   references.
5. Use consistent technical terminology.
   Define acronyms on first use in each document. Prefer American English.
   Refer to crates by name, such as `lambda-rs` and `lambda-rs-platform`.
   Wrap code identifiers in backticks.
6. Keep examples minimal and buildable.
   Use concise Rust sketches when proposing APIs. Mirror the repository's
   existing builder and command patterns. Use ASCII diagrams only and do not
   embed binaries.
7. Update maintenance metadata on substantive edits.
   Update `last_updated`, bump `version` semantically, set `repo_commit` to
   `HEAD`, and append or update the `Changelog` section.

## Required Style Rules

- Use RFC-2119 keywords when the content is normative.
- Prefer `Rationale` bullets instead of long "Why" paragraphs.
- Keep bullets short when possible.
- Avoid filler sentences and meta commentary such as "this aims to".
- Do not make unscoped promises such as "we will add" without a linked issue.

## Validation Checklist

- Metadata header is present and complete.
- The file uses stable headings and short sections.
- The tone is professional, direct, and repository-aligned.
- Acronyms are defined on first use.
- Code snippets are minimal and stylistically consistent with the repository.
- `last_updated`, `version`, `repo_commit`, and `Changelog` are updated for
  substantive edits.
