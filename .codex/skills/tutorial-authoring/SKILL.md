---
name: tutorial-authoring
description: Use when creating or revising step-by-step tutorials in `docs/tutorials/`. This skill applies Lambda's required metadata, instructional structure, tone, code-snippet rules, and validation expectations for tutorial documents.
---

# Tutorial Authoring

Use this skill for tutorial documents that teach a discrete task in the
repository. It includes the long-lived documentation rules that tutorials
inherit, plus the additional section and narrative requirements specific to
`docs/tutorials/`.

## When To Use

- New tutorials in `docs/tutorials/`.
- Revisions to existing tutorials that change structure, examples, or prose.
- Documentation tasks that teach a step-by-step workflow with validation and
  exercises.
- Do not use this skill for generic guides, specs, or roadmaps outside
  `docs/tutorials/`.

## Workflow

1. Place the file correctly.
   Tutorials belong in `docs/tutorials/` and should use kebab-case file names.
   Include the `tutorial` tag in the metadata.
2. Add the required metadata header.
   Tutorials use the same YAML front matter fields as other long-lived docs:
   `title`, `document_id`, `status`, `created`, `last_updated`, `version`,
   `engine_workspace_version`, `wgpu_version`, `shader_backend_default`,
   `winit_version`, `repo_commit`, `owners`, `reviewers`, and `tags`.
3. Build the required section layout.
   Tutorials should include a table of contents and these sections:
   `Goals`, `Overview`, `Prerequisites`, `Implementation Steps`,
   `Validation`, `Notes`, `Conclusion`, `Exercises`, and `Changelog`.
4. Write each implementation step as instruction plus outcome.
   Explain intent and rationale before each code block. After each code block,
   include a short paragraph describing the resulting state and why it matters.
5. Keep the instructional tone precise.
   Use a book-like instructional narrative while staying professional and
   direct. Imperative phrasing is preferred. Limited second-person phrasing is
   acceptable when it clearly improves the instruction.
6. Keep terminology and examples consistent.
   Define acronyms on first use, use terms such as `GPU`, `CPU`, `wgpu`,
   `bind group`, and `pipeline layout` consistently, and refer to identifiers
   with backticks.
7. Keep code snippets minimal and repository-aligned.
   Match repository style: 2-space indentation, line width near 80 columns,
   explicit return statements, and no abbreviations or acronyms in code
   identifiers. Prefer workspace-relative file paths in prose and examples.
8. Update maintenance metadata on substantive edits.
   Update `last_updated`, bump `version` semantically, set `repo_commit` to
   `HEAD`, and maintain the `Changelog`.

## Required Style Rules

- Use clear, direct, neutral language.
- Prefer active voice and present tense.
- Avoid conversational filler, rhetorical questions, emojis, marketing claims,
  speculation, and AI authorship references.
- Use RFC-2119 keywords in `Notes` when the content is normative.
- Do not include commentary about internal process documents or guideline
  updates.

## Validation Checklist

- The tutorial lives under `docs/tutorials/` with a kebab-case file name.
- Metadata header and `tutorial` tag are present.
- A table of contents is present.
- All required sections are present and ordered clearly.
- Each implementation step includes intent before the code and outcome after it.
- Code snippets are minimal, buildable, and repository-aligned.
- `Validation` contains exact commands and expected behavior.
- `Exercises` contains 5 to 7 focused follow-up tasks.
