# Git Repo Manager

A lightweight, beginner-friendly multi-repository Git GUI. Discover repositories
below configured folders, group them (for example "Shop themes"), watch their Git
state, and save, fetch, pull, push, or stash across a whole group at once — with
plain-language guidance instead of Git jargon.

The core is written in Rust and shared by two frontends:

- **Desktop** — Tauri v2 window with a Vue 3 UI.
- **Web** — an axum HTTP/SSE server that serves the same Vue UI to any browser,
  so a colleague can use it from macOS while the repositories live on another
  machine.

Git itself is always executed through the system `git` CLI, preserving the
user's SSH agent, credential helpers, and GPG signing configuration.

## Features

- Recursive repository discovery with `.gitignore` awareness and dependency
  folder pruning (`node_modules`, `vendor`, `target`, caches).
- Multiple scan paths with per-path depth, enable/disable, and exclusions.
- Logical groups with repository assignment, colors, synchronous branch
  switching, auto-stash on switch, and a configurable base branch.
- Live repository state via `.git` filesystem events (with automatic polling
  fallback when the operating system's watch limit is exhausted).
- Guided **Save to Git** flow: preview changed files, write one plain-language
  message, and commit each dirty project in the selection.
- Pull automatically sets aside unfinished changes and restores them afterwards;
  push creates the upstream branch when needed.
- Friendly error messages translate common Git failures into the next action.
- Parallel group actions bounded by a concurrency semaphore (fetch, fast-forward
  pull, push, stash, stash pop).
- Status matrix with plain-language badges, branch, ahead/behind, conflicts, and
  in-progress operation detection (merge, rebase, cherry-pick).
- Built-in help modal, next-step guidance banner, and friendly empty states.
- JSON configuration persisted atomically; no database.
- Browser mode with optional bearer-token protection and server-sent events.

## Everyday workflow

1. **Add project folder** once. The app finds the Git projects inside it.
2. Edit your files normally in your editor.
3. Click **Save to Git** at a checkpoint and describe what changed.
4. Click **Get latest** before starting work and **Send changes** when finished.
5. Use **How this works** in the app for a plain-English glossary and recovery
   guidance.

The toolbar groups the common actions; fetch, stash, and stash pop live under the
**More** menu.

## Requirements

- Rust 1.78+ and Cargo
- Node.js 20+ and pnpm 9+
- Git on `PATH`
- Linux desktop builds additionally need the Tauri system dependencies
  (`webkit2gtk-4.1`, GTK 3, and related libraries)
- macOS desktop builds need Xcode Command Line Tools

## Install and develop

```bash
pnpm install
pnpm tauri dev          # desktop window + Vite dev server
pnpm typecheck          # vue-tsc
pnpm build              # type-check and build dist/
```

Run the Rust tests:

```bash
cargo test --manifest-path src-tauri/Cargo.toml
```

Build a desktop installer/bundle:

```bash
pnpm tauri build
```

### Install a command on your PATH

Build a release binary and install it as `git-repo-manager`:

```bash
pnpm install:bin
```

The default target is `~/.local/bin`. To install elsewhere, or to symlink the
release binary for live rebuilds:

```bash
./scripts/install.sh --prefix /usr/local/bin
./scripts/install.sh --symlink
```

Then start the desktop app with:

```bash
git-repo-manager
```

For browser mode from this checkout, point the server at the built assets:

```bash
git-repo-manager --web --port 8787 --dist ./dist
```

On first launch the main panel shows **Let's find your projects**: add the folder
where your repositories live, then click **Find my projects**.

## Web server mode

Build the frontend once, then start the binary in web mode:

```bash
pnpm build
cargo run --manifest-path src-tauri/Cargo.toml -- \
  --web --host 0.0.0.0 --port 8787 --token "$(openssl rand -hex 16)"
```

Or use the already-built binary:

```bash
./src-tauri/target/release/git-repo-manager --web --host 0.0.0.0 --port 8787
```

Open the printed URL, for example:

```text
http://127.0.0.1:8787/?token=<token>
```

The token is copied into `sessionStorage` and removed from the address bar. API
requests use `Authorization: Bearer <token>`; the SSE endpoint accepts the token
as a query parameter because `EventSource` cannot set headers.

If the bind address is not loopback and no `--token` is supplied, a random token
is generated and printed. Binding to `0.0.0.0` without a token is therefore safe
by default. Static UI files are public, but every `/api/*` request is protected.

### Options and environment variables

| Flag | Environment variable | Default |
| :--- | :--- | :--- |
| `--web` | `GIT_REPO_MANAGER_WEB=1` | desktop mode |
| `--host <HOST>` | `GIT_REPO_MANAGER_HOST` | `127.0.0.1` |
| `--port <PORT>` | `GIT_REPO_MANAGER_PORT` | `8787` |
| `--token <TOKEN>` | `GIT_REPO_MANAGER_TOKEN` | auto-generated for non-loopback |
| `--config <PATH>` | `GIT_REPO_MANAGER_CONFIG` | platform config directory |
| `--dist <PATH>` | `GIT_REPO_MANAGER_DIST` | `dist` |

Use `--help` for the full usage text.

### Accessing from another computer (macOS example)

Run the server on the machine that owns the repositories:

```bash
pnpm build
./src-tauri/target/release/git-repo-manager \
  --web --host 0.0.0.0 --port 8787 --token "long-random-token"
```

Then open `http://<server-ip>:8787/?token=long-random-token` in Safari or Chrome
on the Mac. Allow port 8787 through the server firewall and keep the token
private. For a trusted local-only setup, bind to `127.0.0.1` and forward the
port instead:

```bash
ssh -L 8787:127.0.0.1:8787 user@repo-host
# then open http://127.0.0.1:8787/
```

To run natively on the Mac, build the desktop or web binary there with the same
Rust/Node prerequisites:

```bash
pnpm install
pnpm tauri build
# or: pnpm build && cargo build --release --manifest-path src-tauri/Cargo.toml
```

## Configuration

Configuration is stored as JSON:

- Linux/macOS: `~/.config/git-repo-manager/config.json`
- Windows: `%APPDATA%\git-repo-manager\config.json`
- Override with `--config <path>` or `GIT_REPO_MANAGER_CONFIG`.

Example:

```json
{
  "scanPaths": [
    {
      "id": "scan-1",
      "path": "/srv/projects",
      "maxDepth": 3,
      "enabled": true,
      "excludePatterns": ["legacy"]
    }
  ],
  "groups": [
    {
      "id": "group-1",
      "name": "Shop Microservices",
      "color": "#38bdf8",
      "repoPaths": ["/srv/projects/api", "/srv/projects/worker"],
      "settings": {
        "syncBranching": true,
        "autoStashOnSwitch": true,
        "defaultBaseBranch": "main"
      }
    }
  ]
}
```

Repository state itself is never persisted; it is read from Git on demand.

## HTTP API

All routes are under `/api` and require the token when one is configured.

| Method | Path | Purpose |
| :--- | :--- | :--- |
| GET | `/api/config` | Read scan paths and groups |
| POST | `/api/scan-paths` | Add or update a scan path |
| DELETE | `/api/scan-paths/{id}` | Remove a scan path |
| POST | `/api/groups` | Add or update a group |
| DELETE | `/api/groups/{id}` | Remove a group |
| POST | `/api/repositories/discover` | Scan paths and return repository states |
| POST | `/api/repositories/refresh` | Refresh one repository |
| POST | `/api/repositories/changes` | List changed files for one repository |
| POST | `/api/repositories/commit` | Commit all changes in one repository |
| POST | `/api/repositories/actions` | Fetch/pull/push/stash across repositories |
| POST | `/api/repositories/switch-branch` | Switch a set of repositories to a branch |
| POST | `/api/branches` | List local branches with repository counts |
| GET | `/api/events` | Server-sent events (`repo-fs-changed`) |

## Architecture

```text
Vue 3 UI (Pinia store, Tailwind)
        │
        ├── Tauri IPC ──────────────┐
        └── fetch + EventSource ────┤
                                    ▼
                         service.rs (shared logic)
                         ├── config.rs      JSON persistence
                         ├── scanner.rs     discovery via ignore
                         ├── git_runner.rs  git CLI + state parsing
                         └── watcher.rs     notify + polling fallback
                                    │
                         EventBus (tokio broadcast)
                         ├── Tauri event: repo:fs-changed
                         └── SSE event:   repo-fs-changed
```

Thin command wrappers in `src-tauri/src/commands/` expose the service to Tauri;
`src-tauri/src/web.rs` exposes the same service over HTTP. The frontend chooses
the transport in `src/lib/backend.ts`.

## Project layout

```text
src/                     Vue 3 application
  components/            Toolbar, sidebar, repository table, modals, toasts
  lib/backend.ts         Tauri/web transport abstraction
  stores/repoStore.ts    Application state and actions
  types/domain.ts        Shared TypeScript models
src-tauri/src/           Rust backend
  commands/              Tauri command wrappers
  config.rs              Atomic JSON configuration
  git_runner.rs          Git CLI execution and state parsing
  scanner.rs             Repository discovery
  service.rs             Logic shared by desktop and web
  watcher.rs             Live updates and polling fallback
  web.rs                 axum server, SSE, token middleware
  main.rs                CLI and application entry point
assets/                  Icon source
scripts/install.sh       Build and install the release binary
```

## Security notes

- The web server has no TLS. Use SSH port forwarding or a reverse proxy with TLS
  when exposing it beyond a trusted network.
- Tokens are compared verbatim and are not stored server-side beyond process
  memory. Prefer long random values (the auto-generated token is 64 hex chars).
- The API can run Git operations with the server user's credentials; treat the
  token like an SSH key.
