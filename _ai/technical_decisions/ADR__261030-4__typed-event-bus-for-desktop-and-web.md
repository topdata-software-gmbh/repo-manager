---
title: "Typed Application Event Bus for Desktop and Web"
status: Accepted
date: 2026-10-30
deciders: [Marc]
tags: [events, sse, tauri, architecture, observability]
adrId: 261030-4
---

# Typed Application Event Bus for Desktop and Web

## Context

The core is shared by a Tauri desktop window and an axum web server. Both already
need to tell the frontend when a repository changes on disk: the desktop build
emits a Tauri event, and the web build streams the same information over
server-sent events from `/api/events`.

That plumbing currently carries a bare `String`, the repository path, through a
`broadcast` channel in `EventBus`. The new "What Git did" log needs a second,
structurally different stream: one event per executed Git step, carrying the
macro, step index, rendered comment, rendered command, status, duration, and an
optional message.

The application must not grow two parallel notification mechanisms, and it must
not grow a second HTTP endpoint, because the number of user-visible surfaces is
itself a design constraint for this target user.

## Decision

Make `AppEvent` the payload of the existing bus and keep one transport per
platform.

- `AppEvent` is an enum with `FsChanged(String)` and `GitStep(GitStepEvent)`.
- The watcher emits `FsChanged` exactly as before. The Git runner emits one
  `GitStep` with status `running` before a step executes and a terminal status
  after, so the UI can show a step light up rather than appear late.
- The desktop build forwards the variants as the Tauri events `repo:fs-changed`
  and `git:step`.
- The web build keeps the single `/api/events` SSE endpoint and distinguishes
  variants with the SSE event name (`repo-fs-changed`, `git-step`).
- The frontend exposes one `subscribeToEvents({ onRepoChanged, onGitStep })`
  helper: a single `EventSource` in web mode, two `listen` calls in desktop mode,
  and one teardown function.
- The Git step log lives in a bounded in-memory ring buffer for the session only.
  Nothing is persisted.
- Background inspection commands (`status --porcelain`, `rev-list`, `log -1`,
  `remote -v`) are **not** streamed. They run four times per repository per
  refresh; streaming them would evict the user-initiated actions from the buffer
  and drown the pane.

## Consequences

Positive:

- Existing repository-change notifications keep working unchanged on the wire;
  only the Rust payload type and the frontend helper are refactored.
- A single subscription and a single endpoint serve both event kinds, so the UI
  keeps one connection and one teardown path.
- Typed events make it impossible to confuse a repository path with a step
  payload, and they let the frontend narrow exhaustively.
- Session-only retention avoids configuration growth and avoids writing commit
  messages to disk.

Negative:

- The bus change touches the watcher, the desktop entry point, the web server,
  and the frontend helper in one step; they must ship together.
- Adding a future event kind now requires a variant plus a mapping in two
  transports, rather than a new ad-hoc path.
- Because inspection commands are excluded, the pane cannot explain the
  background polling that dominates command volume. This is accepted: it shows
  only what the user caused.

## Alternatives Considered

- **A second dedicated bus and a second SSE endpoint**: Rejected because it
  doubles the wiring in both transports and adds a user-visible endpoint for no
  benefit.
- **Untyped JSON envelopes on the existing bus**: Rejected because it moves
  variant discrimination into stringly-typed frontend code and loses exhaustiveness.
- **Streaming every Git invocation, filtering in the frontend**: Rejected because
  inspection traffic is orders of magnitude larger than user actions and would
  crowd the ring buffer before the frontend could filter.
- **Persisting the log to the config file**: Rejected because it grows a config
  file that is otherwise small and stable, and stores commit messages on disk for
  no stated need.
