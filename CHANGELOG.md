# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Shared `@topdata-software-gmbh/ui` and `@topdata-software-gmbh/design-tokens`
  dependencies, token/kit CSS imports, and a Vitest migration smoke test.

### Changed

- `AddScanPathModal` and `ToastStack` now use the shared design-system
  components and `--td-*` tokens instead of hand-rolled Tailwind markup.
- Success and info toasts expire after 5 seconds and at most 3 are visible;
  error toasts persist until dismissed. Previously errors auto-dismissed after
  9 seconds and the stack was unbounded.

## [0.1.0] - 2026-10-28

### Added

- Tauri v2 desktop application with a Vue 3, TypeScript, Tailwind CSS, and Pinia UI.
- `scripts/install.sh` / `pnpm install:bin` to build and install the release
  binary on `PATH`, with `--prefix`, `--symlink`, and `--no-build` options.
- Recursive multi-repository discovery with `.gitignore` awareness, dependency
  folder pruning, configurable depth, enable/disable, and exclusion patterns.
- Repository groups with colors, repository assignment, synchronous branch
  switching, auto-stash, and a default base branch setting.
- Live Git state updates from `.git` filesystem events, including worktree and
  submodule `gitdir` support and an automatic polling fallback when the OS
  inotify watch limit is exhausted.
- Guided **Save to Git** flow with changed-file preview and one commit message per
  selected project, available from the toolbar, guidance banner, and each row.
- Tauri commands and HTTP endpoints for listing changes and committing:
  `list_repo_changes`, `commit_repo_changes`,
  `POST /api/repositories/changes`, and `POST /api/repositories/commit`.
- Plain-language UI: next-step guidance banner, help/glossary modal, friendly
  status badges, `Get latest` / `Send changes` / `Working on` labels, and guided
  empty states.
- Friendly translations for common Git failures (missing upstream, rejected push,
  authentication, conflicts, locked index).
- Pull sets aside and restores unfinished changes; push creates the upstream branch
  when one is missing; stash includes untracked files.
- Scan paths expand `~` and are validated before being saved.
- Parallel fetch, fast-forward pull, push, stash, and stash pop across a selection
  or group, bounded by a concurrency semaphore.
- Repository status matrix with branch, ahead/behind counts, clean/dirty state,
  conflicts, remotes, and merge/rebase/cherry-pick detection.
- Atomic JSON configuration with a platform config directory and `--config`
  override.
- Browser/web-server mode using axum: same UI and API over HTTP, server-sent
  events for live updates, optional bearer-token authentication, static frontend
  serving, and CLI/environment configuration.
- `git-repo-manager --web` with `--host`, `--port`, `--token`, `--config`, and
  `--dist` options; non-loopback binds without a token generate one automatically.
- Rust unit tests for configuration persistence, repository scanning, Git state,
  autostash checkout, change listing/committing, friendly error mapping, and
  worktree path resolution.
- Project documentation covering desktop development, web mode, HTTP API,
  configuration, architecture, and the everyday save/get-latest/send workflow.

### Fixed

- Removed the unused `tauri-plugin-shell` dependency that blocked release builds
  with a Rust/npm version mismatch.
- Changed the bundle identifier to avoid the macOS `.app` suffix conflict warning.
- The first line of `git status --porcelain` lost its leading status column when
  the output was trimmed, which shifted the reported path by one character and
  could hide a modified file.
