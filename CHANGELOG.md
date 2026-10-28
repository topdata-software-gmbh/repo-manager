# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-10-28

### Added

- Tauri v2 desktop application with a Vue 3, TypeScript, Tailwind CSS, and Pinia UI.
- Recursive multi-repository discovery with `.gitignore` awareness, dependency
  folder pruning, configurable depth, enable/disable, and exclusion patterns.
- Repository groups with colors, repository assignment, synchronous branch
  switching, auto-stash, and a default base branch setting.
- Live Git state updates from `.git` filesystem events, including worktree and
  submodule `gitdir` support and an automatic polling fallback when the OS
  inotify watch limit is exhausted.
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
  autostash checkout, and worktree path resolution.
- Project documentation covering desktop development, web mode, HTTP API,
  configuration, and architecture.
