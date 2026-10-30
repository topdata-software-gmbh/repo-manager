---
filename: "_ai/backlog/active/261030_2348__IMPLEMENTATION_PLAN__run-installed-app-launcher.md"
title: "Implementation Plan: start.sh launcher for the installed app"
createdAt: 2026-10-30 23:48
updatedAt: 2026-10-30 23:48
status: completed
completedAt: 2026-10-30 23:50
priority: low
tags: [scripts, developer-experience, launcher, bash]
estimatedComplexity: simple
documentRevision: 1
sourceSpec: ""
sourceHandoff: ""
sourceAdrs: []
documentType: IMPLEMENTATION_PLAN
---

# Implementation Plan: start.sh launcher for the installed app

## 1. Problem

The repository has two scripts with clear jobs: `scripts/dev-up.sh` starts the
browser development stack, and `scripts/install.sh` builds and installs the
release binary. There is no short, obvious way to simply **run the installed
app**. Users either remember the binary name or fall back to `pnpm tauri dev`,
which starts a development build and Vite instead of the app they installed.

The gap is small but recurring: after `pnpm install:bin`, the next question is
"how do I start it?" A launcher named `scripts/start.sh` answers that without
adding a build step, a package manager dependency, or another process manager.

## 2. Executive summary

Add `scripts/start.sh`, a thin Bash launcher that resolves the
`git-repo-manager` binary from `PATH`, then from the install script's default
prefix `~/.local/bin`, and `exec`s it with every argument forwarded. If no
binary is found it prints the three existing entry points (install, desktop dev,
browser dev) and exits 1. It never builds, installs, or prompts.

Expose the same command as `pnpm start`, and document it in one README
paragraph. No Rust, Vue, dependency, configuration, or API changes are needed.

## 3. Design Decisions & Rejected Alternatives

| # | Decision | Rationale / alternative dropped |
|---|----------|--------------------------------|
| D1 | Resolve `PATH` first, then `$HOME/.local/bin` | Respects a deliberate install location, but also finds `install.sh`'s default prefix, which is often absent from a fresh shell's `PATH`. PATH-only was rejected as too brittle. |
| D2 | `exec "$BIN" "$@"` | Forwards all flags unchanged, preserves quoting, replaces the shell, and propagates the app's exit code. Restricting or translating flags was rejected as needless policy. |
| D3 | No build, install, or prompt on the missing-binary path | A launcher that silently builds is slow and surprising. The error names `install.sh`, `pnpm tauri dev`, and `dev-up.sh` so the user chooses. |
| D4 | The script has no flags of its own | `./scripts/start.sh --help` should show the app's real help. A second help surface would duplicate and drift. |
| D5 | Add `"start": "./scripts/start.sh"` to `package.json` | Makes the launcher discoverable without inventing a second command path; it delegates to the same script. |
| D6 | No CHANGELOG entry | This is a developer convenience with no user-facing application behaviour. README is the right place. |

## 4. Project environment and conventions

- **Repository**: `/topdata/repo-manager` (Tauri v2 + Vue 3, Rust backend, Bash scripts).
- **Git workflow**: no `AGENTS.md` exists. Do **not** create branches or pull requests; implement on the current branch (`main`). Commit only when the user asks.
- **Shell**: Bash (`#!/usr/bin/env bash`), `set -euo pipefail`. The script must stay portable to the bash 3.2 shipped with macOS; avoid associative arrays and `${var,,}`.
- **Existing scripts**: `scripts/dev-up.sh` (browser dev via mprocs), `scripts/install.sh` (release build + install). Do not modify them.
- **No new dependencies**. `shellcheck` is not installed in this environment; `bash -n` is the mandatory syntax check, shellcheck is optional when available.
- **Verification commands**:

```bash
bash -n scripts/start.sh
pnpm start -- --help      # only meaningful once the app is installed
```

## 5. Requirements traceability

| ID | Requirement | Satisfied in |
|----|-------------|--------------|
| R1 | `scripts/start.sh` launches the installed `git-repo-manager` binary | Phase 1 |
| R2 | Falls back to `$HOME/.local/bin/git-repo-manager` when not on `PATH` | Phase 1 |
| R3 | Every argument is forwarded; the app's exit code is preserved | Phase 1 |
| R4 | Missing binary prints setup hints and exits 1, with no side effects | Phase 1 |
| R5 | `pnpm start` runs the same launcher | Phase 2 |
| R6 | README explains when to use it | Phase 2 |
| R7 | Existing scripts and app code are untouched | All phases |

---

## 6. Phase 1 — Add the launcher

### 6.1 [NEW FILE] `scripts/start.sh`

```bash
#!/usr/bin/env bash
#
# Launch the installed Git Repo Manager.
# 10/2026 created
#

set -euo pipefail

if BIN="$(command -v git-repo-manager 2>/dev/null)"; then
  :
elif [[ -n "${HOME:-}" && -x "${HOME}/.local/bin/git-repo-manager" ]]; then
  BIN="${HOME}/.local/bin/git-repo-manager"
else
  cat >&2 <<'EOF'
Git Repo Manager is not installed.

  Install it:    ./scripts/install.sh
  Desktop dev:   pnpm tauri dev
  Browser dev:   ./scripts/dev-up.sh
EOF
  exit 1
fi

exec "$BIN" "$@"
```

### 6.2 Mark the script executable

```bash
chmod +x scripts/start.sh
```

**Notes**

- The `HOME` guard prevents an unset variable from resolving to `/.local/bin`.
- `-x` follows the symlink created by `install.sh --symlink`, so that mode works too.
- A non-executable file in `~/.local/bin` is treated as missing, matching the
  "installed" contract.
- `exec` means the app owns the terminal and signals; the script adds no cleanup.

**Verification**

```bash
bash -n scripts/start.sh
test -x scripts/start.sh
```

Then exercise the two lookup paths and the failure path with temporary values
(Phase 3 gives the exact commands). No app process should be launched during
automated checks.

---

## 7. Phase 2 — Discoverability and documentation

### 7.1 [MODIFY] `package.json`

Add one script next to the existing installer alias:

```json
  "scripts": {
    "dev": "vite",
    "build": "vue-tsc --noEmit && vite build",
    "typecheck": "vue-tsc --noEmit",
    "preview": "vite preview",
    "tauri": "tauri",
    "install:bin": "./scripts/install.sh",
    "start": "./scripts/start.sh",
    "test": "vitest run"
  },
```

`pnpm start -- --web --port 8788` must reach the binary as
`--web --port 8788`; pnpm passes arguments after `--` through to the script,
and the script forwards them verbatim.

### 7.2 [MODIFY] `README.md`

In **Install and develop**, add `pnpm start` to the command block:

```markdown
pnpm install
pnpm tauri dev          # desktop window + Vite dev server
pnpm start              # launch the installed app (see below)
pnpm typecheck          # vue-tsc
pnpm build              # type-check and build dist/
```

After the browser-development paragraph, add:

```markdown
`pnpm start` (or `./scripts/start.sh`) launches the installed
`git-repo-manager` command, falling back to `~/.local/bin`. It does not build;
run `pnpm install:bin` first. Any extra arguments are forwarded to the app, so
`pnpm start -- --web --port 8788` starts web mode.
```

**Verification**

```bash
node -e "JSON.parse(require('fs').readFileSync('package.json','utf8'))"
grep -n "pnpm start" README.md
```

---

## 8. Phase 3 — Validation

Run the non-interactive checks first:

```bash
bash -n scripts/start.sh
test -x scripts/start.sh
node -e "JSON.parse(require('fs').readFileSync('package.json','utf8'))"
```

Exercise argument forwarding and exit-code propagation with a fake binary:

```bash
TMP=$(mktemp -d)
cat > "$TMP/git-repo-manager" <<'EOF'
#!/usr/bin/env bash
printf 'argc=%s\n' "$#"
printf 'arg1=%s\n' "${1:-}"
exit 42
EOF
chmod +x "$TMP/git-repo-manager"
PATH="$TMP:/usr/bin:/bin" ./scripts/start.sh "one two" three; echo "exit=$?"
rm -rf "$TMP"
```

Expected: `argc=2`, `arg1=one two`, `exit=42`.

Exercise the `~/.local/bin` fallback:

```bash
TMP_HOME=$(mktemp -d)
mkdir -p "$TMP_HOME/.local/bin"
cat > "$TMP_HOME/.local/bin/git-repo-manager" <<'EOF'
#!/usr/bin/env bash
echo "fallback launched"
EOF
chmod +x "$TMP_HOME/.local/bin/git-repo-manager"
HOME="$TMP_HOME" PATH=/usr/bin:/bin ./scripts/start.sh; echo "exit=$?"
rm -rf "$TMP_HOME"
```

Expected: `fallback launched`, `exit=0`.

Exercise the missing-binary path:

```bash
TMP_HOME=$(mktemp -d)
HOME="$TMP_HOME" PATH=/usr/bin:/bin ./scripts/start.sh; echo "exit=$?"
rm -rf "$TMP_HOME"
```

Expected: the three-line hint on stderr, `exit=1`, and no files created.

If `shellcheck` is available, also run `shellcheck scripts/start.sh` and fix any
findings. Do not add shellcheck as a project dependency.

---

## 9. Phase 4 — Housekeeping and implementation report

### 9.1 Housekeeping

- **`.gitignore`**: no change. `scripts/start.sh` is committed source and the
  script creates no artifacts.
- **`CHANGELOG.md`**: no change. This is a developer convenience and does not
  change user-facing behaviour (decision D6).
- **`README.md`**: already updated in Phase 2.
- **Archive the plan**: after implementation and commit, run the `finish-plan`
  workflow so this file moves to `_ai/backlog/archive/`.

### 9.2 [NEW FILE] Implementation report

Write
`_ai/backlog/reports/261030_2348__IMPLEMENTATION_REPORT__run-installed-app-launcher.md`:

```markdown
---
filename: "_ai/backlog/reports/261030_2348__IMPLEMENTATION_REPORT__run-installed-app-launcher.md"
title: "Report: start.sh launcher for the installed app"
createdAt: 2026-10-30 23:48
updatedAt: 2026-10-30 23:48
planFile: "_ai/backlog/active/261030_2348__IMPLEMENTATION_PLAN__run-installed-app-launcher.md"
specFile: ""
project: "repo-manager"
status: completed
filesCreated: 1
filesModified: 2
filesDeleted: 0
tags: [scripts, developer-experience, launcher, bash]
documentType: IMPLEMENTATION_REPORT
---

# Report: start.sh launcher for the installed app
```

Report sections, in order:

1. **Summary** — what the launcher does and the gap it closes (2–3 sentences).
2. **Files Changed** — `scripts/start.sh` (new); `package.json` and `README.md`
   (modified); none deleted.
3. **Key Changes** — lookup order, `exec` forwarding, missing-binary hint,
   `pnpm start` alias.
4. **Deviations from Plan** — any difference in lookup behaviour, wording, or
   documentation, with the reason. "None" is a valid entry.
5. **Technical Decisions** — confirm D1–D6 or record a change.
6. **Testing Notes** — paste the Phase 3 commands and their observed results,
   including whether `shellcheck` was available.
7. **Usage Examples** — `pnpm start`, `./scripts/start.sh --config /tmp/c.json`,
   and `pnpm start -- --web --port 8788`.
8. **Documentation Updates** — the README paragraph and command-block line.
9. **Next Steps** — only real follow-ups (for example, a shellcheck CI target if
   the project later adds one).

Report `status: partial` if any Phase 3 case could not be exercised, and name the
skipped case under Next Steps.

---

## 10. Rollout and verification checklist

| Step | Command / action | Expected |
|------|------------------|----------|
| 1 | `bash -n scripts/start.sh` | no output |
| 2 | `test -x scripts/start.sh` | exit 0 |
| 3 | Fake binary on `PATH`, run with two args | args forwarded, exit 42 |
| 4 | Fake binary only in a temp `~/.local/bin` | fallback launched, exit 0 |
| 5 | Empty `PATH` and temporary `HOME` | install hint on stderr, exit 1 |
| 6 | `node -e "JSON.parse(...package.json)"` | parses cleanly |
| 7 | `pnpm start` on a machine with the app installed | desktop window opens |
| 8 | `pnpm start -- --help` | the app's help, not a script help |
