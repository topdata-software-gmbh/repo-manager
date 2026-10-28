---
title: "Shared Service Layer for Tauri Desktop and Browser Access"
status: Accepted
date: 2026-10-28
deciders: [Marc]
tags: [tauri, axum, rust, vue, web, architecture]
adrId: 261028-1
---

# Shared Service Layer for Tauri Desktop and Browser Access

## Context

Git Repo Manager was planned as a Tauri v2 desktop application. A second use case
emerged: a colleague on macOS needs to use the same tool while the repositories
live on a shared Linux machine. Building a separate web application would
duplicate all Git, discovery, configuration, and state logic.

The business logic is transport-agnostic Rust: scan directories, inspect Git
state, execute Git CLI operations, persist JSON, and watch `.git` directories.
Only the transport for UI requests and live updates differs between a desktop
window and a browser.

## Decision

Introduce a shared service layer (`src-tauri/src/service.rs`) used by both
frontends. Thin Tauri command wrappers in `src-tauri/src/commands/` and axum
handlers in `src-tauri/src/web.rs` call the same functions.

- The same Rust binary supports desktop mode (default) and `--web` mode.
- Web mode serves the existing Vue `dist/` build, exposes the service over
  `/api/*`, and streams file-change events over server-sent events.
- Desktop mode emits the same events as Tauri `repo:fs-changed` events.
- A transport-neutral `EventBus` (tokio broadcast) fans events out to Tauri and
  SSE subscribers.
- Web mode accepts `--host`, `--port`, `--token`, `--config`, and `--dist` flags
  plus equivalent environment variables.
- Binding to a non-loopback address without an explicit token generates a random
  64-character token and prints a URL containing it. API routes require a bearer
  token or the `?token=` query parameter used by `EventSource`.
- The frontend selects the transport in `src/lib/backend.ts` by checking for
  `__TAURI_INTERNALS__`; the Vue components and Pinia store are unaware of it.

## Consequences

Positive:

- One implementation of discovery, Git operations, config, and watching.
- The existing Vue UI works in both desktop and browser without a second build.
- Web mode is deployable with the same binary; no Node runtime is needed in
  production.
- Token protection is on by default for remote binds.

Negative:

- The HTTP API is a second public surface that must stay in sync with Tauri
  commands; shared service signatures reduce but do not eliminate drift.
- Web mode has no TLS and a single shared token, so it is unsuitable for direct
  exposure to the internet without a reverse proxy or SSH tunnel.
- Tauri and axum code are compiled into every binary, slightly increasing build
  time and binary size.
- SSE reconnects are handled by the browser, but the web UI has no offline or
  multi-user conflict model.

## Alternatives Considered

- **Web-only application**: Rejected because the primary use case is a
  lightweight native desktop window, and Tauri provides lower resource usage.
- **Desktop-only application**: Rejected because it cannot serve the macOS
  colleague without building and distributing a separate macOS binary.
- **Separate web server project**: Rejected because it would duplicate the entire
  Rust service layer and drift over time.
- **Electron**: Rejected for the same reasons as the original plan: larger
  binaries, higher memory usage, and slower startup.
