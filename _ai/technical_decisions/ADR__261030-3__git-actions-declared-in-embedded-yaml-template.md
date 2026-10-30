---
title: "Git Actions Declared in an Embedded YAML Template"
status: Accepted
date: 2026-10-30
deciders: [Marc]
tags: [git, template, architecture, observability, learning-ui]
adrId: 261030-3
---

# Git Actions Declared in an Embedded YAML Template

## Context

Every Git operation is executed through the system CLI, and all of them funnel
through a single function, `GitRunner::run_git_cmd`. The application is a
"Git for dummies" tool for a user who curates multiple theme and plugin clones
across several shops, dislikes Git because it is complicated, and learns best by
seeing what actually happened rather than by reading documentation.

That created a requirement for an in-app log view that prints, in the style of a
commented shell script, the `#` explanation of each step followed by the real
command. Several user-visible actions are not single commands but ordered
bundles: "get latest" is stash → pull → stash pop, with the stash steps
conditional on the working tree and on whether a stash was created.

Wording and behaviour therefore have to stay in step. Writing the explanations in
the Rust modules next to the argument arrays would put the narrative and the
logic in two places that inevitably drift, and would make the wording
unreviewable by anyone who does not read Rust.

## Decision

Declare every user-facing Git action as a **macro in an embedded YAML document**
at `src-tauri/macros/git.yaml`, compiled into the binary with `include_str!`.

- A macro has a `title` and a `comment` for the log, plus ordered `steps`. A step
  is an `args` array (never a shell string) with an optional `comment` and an
  optional `when` guard from a closed enum: `dirty`, `clean`, `has_upstream`,
  `no_upstream`, `stashed`.
- Execution walks the macro, evaluates guards, and runs each step through the
  existing `run_git_cmd`. Rendering walks the same macro, so the log lines and
  the executed commands cannot disagree.
- Placeholders are limited to `{message}`, `{branch}`, and `{repo}`.
- The template is parsed once into a `LazyLock<Result<Macros, String>>`.
  Validation failures disable Git actions with an explicit message instead of
  panicking during a click.
- `serde_yaml` is unmaintained, so `serde_yaml_ng` supplies deserialization.
  Quoting is mandatory for `args` entries, because YAML coerces `-1`, `on`,
  `no`, and `y` to non-strings.
- Each step may name an `alias`. A bundled, pinned extract of the oh-my-zsh Git
  aliases supplies the hints for the log, with shell-function and pipeline
  aliases excluded and destructive aliases denylisted.

## Consequences

Positive:

- One place defines what an action does and how it is explained, so the learning
  log cannot drift from the behaviour it describes.
- Conditional flows (auto-stash around pull and branch switch) become data rather
  than hand-written branches, which makes them readable and testable as a unit.
- Non-Rust contributors can review and improve the wording.
- A template validity test turns every typo, unknown guard, and unknown
  placeholder into a build failure.

Negative:

- Editing wording requires a rebuild, because the document is embedded. This is
  consistent with the rest of the application and avoids path resolution
  differences between the desktop bundle and `--web` mode.
- Guard evaluation and placeholder substitution are engine concerns that must be
  tested independently of any single macro.
- The alias hints are tied to oh-my-zsh, so they are only meaningful to users of
  that shell setup; the comments say so.

## Alternatives Considered

- **Author macros in Rust structs next to the argument arrays**: Rejected because
  the log wording and the execution logic live in two places and drift, and the
  wording becomes unreadable to non-Rust reviewers.
- **A shell-transcript template parsed line by line**: Rejected because it
  requires a custom parser plus quoting rules, and encourages passing shell
  strings, which the codebase deliberately avoids.
- **Exact-match alias lookup**: Rejected because most oh-my-zsh aliases differ
  from our invocations by extra flags (`gc` is `commit --verbose`), so exact
  matching would almost never fire and would mislead when it did.
- **Runtime-loaded template**: Rejected because desktop bundling and web mode
  resolve paths differently, and a missing file would turn into a runtime failure
  for a beginner-facing tool.
- **No alias hints**: Rejected per explicit requirement that the log bridge what
  the application did and what a human would type.
