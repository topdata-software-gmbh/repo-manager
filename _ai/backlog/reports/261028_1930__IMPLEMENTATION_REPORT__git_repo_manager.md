---
filename: "_ai/backlog/reports/261028_1930__IMPLEMENTATION_REPORT__git_repo_manager.md"
title: "Report: git-repo-manager Multi-Git GUI Architecture and Implementation"
createdAt: 2026-10-28 19:30
updatedAt: 2026-10-28 20:05
planFile: "_ai/backlog/active/261028_1930__IMPLEMENTATION_PLAN__git_repo_manager.md"
specFile: ""
project: "git-repo-manager"
status: completed
filesCreated: 58
filesModified: 3
filesDeleted: 0
tags: [git, tauri, rust, vue3, multi-repo, web]
documentType: IMPLEMENTATION_REPORT
---

## 1. Summary

Implemented `git-repo-manager`, a multi-repository Git GUI built with Tauri v2,
Rust, and Vue 3. It discovers repositories below configured scan paths, groups
them, aggregates their Git state, watches `.git` metadata, and executes group-wide
branch switches and git actions with bounded concurrency.

A browser/web-server mode was added beyond the original plan so a colleague on
macOS can use the UI against repositories on a shared machine. The desktop and web
frontends share the same Rust service layer, the same Vue application, and the
same event bus; only the transport differs (Tauri IPC vs HTTP/SSE).

## 2. Files Changed

### Created

**Project and frontend build**

- `package.json`, `pnpm-lock.yaml`, `pnpm-workspace.yaml`
- `index.html`, `vite.config.ts`, `tsconfig.json`, `tailwind.config.js`,
  `postcss.config.js`, `src/vite-env.d.ts`
- `assets/icon-source.png` and generated `src-tauri/icons/*`

**Rust backend**

- `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`, `src-tauri/build.rs`
- `src-tauri/tauri.conf.json`, `src-tauri/capabilities/default.json`
- `src-tauri/src/main.rs` — CLI parsing, desktop setup, web/desktop dispatch
- `src-tauri/src/models.rs` — camelCase-serialized domain and IPC models
- `src-tauri/src/config.rs` — atomic JSON configuration and group/path lookup
- `src-tauri/src/scanner.rs` — `ignore`-based repository discovery
- `src-tauri/src/git_runner.rs` — Git CLI execution, state parsing, autostash
- `src-tauri/src/watcher.rs` — `notify` watcher, event bus, polling fallback
- `src-tauri/src/service.rs` — logic shared by Tauri commands and HTTP handlers
- `src-tauri/src/commands/config_commands.rs` — Tauri config/group commands
- `src-tauri/src/commands/git_commands.rs` — Tauri Git commands
- `src-tauri/src/web.rs` — axum API, SSE, token middleware, static file serving

**Vue frontend**

- `src/main.ts`, `src/style.css`, `src/App.vue`
- `src/types/domain.ts` — shared domain types, `OperationResult`, `BranchInfo`
- `src/lib/backend.ts` — Tauri/web transport switch, token capture, SSE client
- `src/stores/repoStore.ts` — Pinia state and group actions
- `src/components/Toolbar.vue` — group branch selector and action bar
- `src/components/Sidebar.vue` — groups and scan paths
- `src/components/RepoTable.vue` — repository status matrix
- `src/components/AddScanPathModal.vue`
- `src/components/GroupEditorModal.vue`
- `src/components/ToastStack.vue`

**Documentation and decisions**

- `README.md` (rewritten from placeholder)
- `_ai/technical_decisions/ADR__261028-1__shared-service-tauri-desktop-and-axum-web.md`
- `_ai/technical_decisions/README.md`
- This implementation report

### Modified

- `.gitignore` — added Tauri target/gen, frontend build, and local state artifacts.
- `README.md` — replaced the project placeholder with setup, desktop, web,
  configuration, API, and architecture documentation.
- `CHANGELOG.md` — replaced the placeholder with the v0.1.0 change set.

## 3. Key Changes

### Discovery and persistence

- The `ignore` crate scans configured paths, pruning `.git`, `node_modules`,
  `vendor`, `target`, build outputs, and caches. Worktrees and submodules whose
  `.git` is a `gitdir:` file are supported.
- Repositories receive stable UUID v5 IDs derived from their canonical path.
- Configuration is written through a temporary file and atomically renamed.
  JSON keys are camelCase on both sides (`scanPaths`, `repoPaths`, `maxDepth`).

### Git state and operations

- Status, branch, HEAD commit, remotes, conflicts, and operation state are read
  from porcelain/plumbing commands.
- Ahead/behind uses the configured upstream (`@{upstream}`) with an
  `origin/<branch>` fallback.
- Group actions run with a semaphore (default clients pass 5; the service clamps
  to 1–32).
- Branch checkout runs `git stash push` when enabled, checks out, then restores
  the autostash on both success and failure. Switching to the current branch is a
  no-op. If restoration fails, the result is marked failed and the changes are
  reported as remaining in `git stash list`.
- All Rust models use `#[serde(rename_all = "camelCase")]`, fixing a plan-level
  mismatch between Rust snake_case fields and the TypeScript API.

### Live updates

- A `notify` watcher watches the resolved Git directory recursively. It supports
  linked worktrees, ignores irrelevant paths, and debounces events at 400 ms.
- Events flow through a `tokio::sync::broadcast::EventBus`. Desktop forwards them
  as Tauri `repo:fs-changed`; web streams them as SSE `repo-fs-changed`.
- If the operating system refuses a watch (for example exhausted inotify watches),
  the repository is polled every two seconds by comparing Git metadata
  fingerprints. This keeps auto-refresh functional without user intervention.

### Web mode

- The same binary runs the desktop app by default or an axum server with `--web`.
- `--host`, `--port`, `--token`, `--config`, and `--dist` are supported, along
  with environment variables and `--help`.
- Non-loopback binds without an explicit token generate a 64-character token and
  print a URL with the token. API routes accept `Authorization: Bearer` or the
  `?token=` query parameter required by `EventSource`.
- The frontend captures the token into `sessionStorage`, removes it from the
  address bar, and switches transports in `src/lib/backend.ts`.
- Vite proxies `/api` to `127.0.0.1:8787` during frontend-only development.

### UI

- The toolbar exposes a group branch selector, action buttons, group scope, repo
  count, clean count, and synchronized/mixed-branch indicators.
- The sidebar manages groups and scan paths; modals edit groups and add scan
  paths; toasts summarize multi-repository successes and failures.
- The repository table includes search, per-row refresh, status badges, branch,
  ahead/behind, remotes, and empty states.

## 4. Deviations and Plan Corrections

- The plan underestimated the project scaffolding (only 15 files were listed).
  Missing Tauri/Vite/TypeScript/Tailwind setup files, `index.html`, `main.ts`,
  icons, and capabilities were added.
- Added `list_group_branches`; the plan's branch selector only showed branches
  currently checked out, which made switching groups to another branch impossible.
- Added stable repository IDs, `.git` gitfile resolution, watcher debouncing,
  autostash restoration, and the polling fallback.
- Added a full web-server mode (axum, SSE, token handling, static serving) at the
  user's request; this was not in the original plan.
- Removed `windows_subsystem = "windows"` so web mode can print its URL and token
  in a terminal.

## 5. Verification

| Check | Result |
| :--- | :--- |
| `cargo check --manifest-path src-tauri/Cargo.toml` | Passed |
| `cargo fmt --manifest-path src-tauri/Cargo.toml` | Passed |
| `cargo test --manifest-path src-tauri/Cargo.toml` | 4/4 passed |
| `pnpm typecheck` (`vue-tsc --noEmit`) | Passed |
| `pnpm build` | Passed; production `dist/` generated |
| Web `GET /api/config` | 200 with empty config |
| Web `POST /api/scan-paths` | 200; config persisted |
| Web `POST /api/repositories/discover` | 200; returned repo with correct state |
| Web `POST /api/branches` | 200; `main` with repo count 1 |
| Web static `GET /` | 200 |
| Token-protected API without/with token | 401 / 200 |
| Token query auth with special characters | 200 |
| SSE `repo-fs-changed` after branch switch | Received repo path |
| `--web` startup watcher log | 1 repository watched (polling fallback path also implemented) |

The desktop window itself was not launched because the development session is
headless; Tauri compilation and IPC wiring were verified via `cargo check` and the
shared service is exercised by the tests and web smoke test.

## 6. Known Limitations

- Web mode has a single shared bearer token, no user accounts, and no TLS;
  expose it only on a trusted network or behind SSH/reverse-proxy TLS.
- Group branch switching lists local branches only; remote-only branches must be
  created or fetched first.
- Ahead/behind counts depend on a reachable upstream; repositories without one
  report zero.
- The polling fallback is a safety net; file-event watching remains the primary
  mechanism and is preferable for large repository sets.
- No macOS bundle was produced in this Linux environment. The browser mode works
  cross-platform, and macOS can build the app natively with the documented
  prerequisites.

## 7. Testing Notes

- Run `pnpm tauri dev` for the desktop app.
- Run the backend tests with `cargo test --manifest-path src-tauri/Cargo.toml`.
- For browser mode: `pnpm build && cargo run --manifest-path src-tauri/Cargo.toml -- --web`.
- Test discovery against nested microservice folders and confirm dependency
  directories are skipped.
- Test a group branch switch with local modifications to confirm autostash is
  restored; check `git stash list` is unchanged on success.
