---
filename: "_ai/backlog/reports/261030_2348__IMPLEMENTATION_REPORT__run-installed-app-launcher.md"
title: "Report: start.sh launcher for the installed app"
createdAt: 2026-10-30 23:48
updatedAt: 2026-10-30 23:48
planFile: "_ai/backlog/archive/261030_2348__IMPLEMENTATION_PLAN__run-installed-app-launcher.md"
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

## 1. Summary

`scripts/start.sh` is a thin Bash launcher for the already-installed
`git-repo-manager` binary. It closes the small recurring gap between
`pnpm install:bin` and actually starting the app, without building, installing,
or starting a development server. The same command is available as
`pnpm start` and documented in the README.

## 2. Files Changed

### Created

- `scripts/start.sh` — the launcher (executable).

### Modified

- `package.json` — added the `"start": "./scripts/start.sh"` script.
- `README.md` — added `pnpm start` to the command block and a paragraph
  explaining lookup, fallback, and argument forwarding.

### Deleted

- None.

## 3. Key Changes

- **Lookup order**: `command -v git-repo-manager` first, then
  `$HOME/.local/bin/git-repo-manager` when it exists and is executable.
- **Forwarding**: `exec "$BIN" "$@"` preserves quoting, replaces the shell, and
  returns the app's own exit code.
- **Missing binary**: prints the install (`./scripts/install.sh`), desktop dev
  (`pnpm tauri dev`), and browser dev (`./scripts/dev-up.sh`) hints to stderr
  and exits 1. No build, install, prompt, or file is produced.
- **Discoverability**: `pnpm start` delegates to the same script.

## 4. Deviations from Plan

- `pnpm start -- --web --port 8788` (pnpm 12.16.1) passes the literal `--`
  through to the launcher, so the binary sees `-- --web --port 8788` rather than
  `--web --port 8788`. The binary ignores unknown positional arguments, so the
  documented command still starts web mode correctly; verified with
  `pnpm start -- --help`, which prints the app's help. The script still forwards
  every argument verbatim (R3), and no argument stripping was added.
- Otherwise none.

## 5. Technical Decisions

- **D1** confirmed: PATH first, then `$HOME/.local/bin`.
- **D2** confirmed: `exec "$BIN" "$@"`, no flag translation.
- **D3** confirmed: missing-binary path only prints hints and exits 1.
- **D4** confirmed: the script defines no flags of its own; `--help` reaches the
  app.
- **D5** confirmed: `"start": "./scripts/start.sh"` added to `package.json`.
- **D6** confirmed: no CHANGELOG entry; this is a developer convenience.

## 6. Testing Notes

Static checks:

```text
$ bash -n scripts/start.sh
(no output)
$ test -x scripts/start.sh
exit 0
$ node -e "JSON.parse(require('fs').readFileSync('package.json','utf8'))"
(no output)
```

Argument forwarding and exit-code propagation with a fake binary:

```text
$ PATH="$TMP:/usr/bin:/bin" ./scripts/start.sh "one two" three
argc=2
arg1=one two
exit=42
```

Fallback to a temporary `$HOME/.local/bin`:

```text
$ HOME="$TMP_HOME" PATH=/usr/bin:/bin ./scripts/start.sh
fallback launched
exit=0
```

Missing-binary path:

```text
$ HOME="$TMP_HOME" PATH=/usr/bin:/bin ./scripts/start.sh
Git Repo Manager is not installed.

  Install it:    ./scripts/install.sh
  Desktop dev:   pnpm tauri dev
  Browser dev:   ./scripts/dev-up.sh
exit=1
```

Real binary (installed at `~/.local/bin/git-repo-manager`):

```text
$ ./scripts/start.sh --help
Git Repo Manager
Usage:
  git-repo-manager                 Launch the desktop application
  ...
exit=0
$ pnpm start -- --help
$ ./scripts/start.sh -- --help
Git Repo Manager
...
exit=0
```

`shellcheck` is not available in this environment, so it was not run. All Phase 3
cases were exercised; no case was skipped.

## 7. Usage Examples

```bash
pnpm start
./scripts/start.sh --config /tmp/c.json
pnpm start -- --web --port 8788
```

## 8. Documentation Updates

- `README.md` **Install and develop** command block now lists
  `pnpm start # launch the installed app (see below)`.
- A paragraph after the browser-development section explains that
  `pnpm start` / `./scripts/start.sh` launches the installed binary, falls back
  to `~/.local/bin`, does not build (`pnpm install:bin` first), and forwards
  extra arguments.

## 9. Next Steps

- Add a `shellcheck` CI target if the project later adopts a shell lint step.
- Optionally trim a leading `--` in future launchers if a stricter argument
  contract is desired; currently the app ignores the stray token.
