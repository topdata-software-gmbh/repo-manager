---
filename: "_ai/backlog/reports/261030_2244__IMPLEMENTATION_REPORT__what-git-did-log-view.md"
title: "Report: \"What Git did\" — template-driven Git actions with a learning log"
createdAt: 2026-10-30 23:35
updatedAt: 2026-10-30 23:35
planFile: "_ai/backlog/archive/261030_2244__IMPLEMENTATION_PLAN__what-git-did-log-view.md"
specFile: ""
project: "repo-manager"
status: partial
filesCreated: 8
filesModified: 16
filesDeleted: 0
tags: [git, observability, learning-ui, template, events, vue, rust]
documentType: IMPLEMENTATION_REPORT
---

# Report: "What Git did" — template-driven Git actions with a learning log

## 1. Summary

Git actions are now declared in `src-tauri/macros/git-actions.yaml` and executed
by a small macro engine, so the command that runs and the sentence that explains
it come from one document. Every step publishes a typed `GitStepEvent` through a
new `GitObserver` seam; the existing broadcast bus now carries `AppEvent`
(`FsChanged` / `GitStep`) to both transports. The frontend gained a session-only
**What Git did** drawer, opened from the existing "…" toolbar menu, which folds
identical steps across clones, shows the gray `#` comments, the rendered
`git -C …` commands, skipped-step markers, and oh-my-zsh alias hints.

All Rust tests, clippy, rustfmt, `pnpm typecheck`, and `pnpm test` pass.

## 2. Files Changed

### Created

- `src-tauri/macros/git-actions.yaml` — the seven user-facing actions.
- `src-tauri/macros/oh-my-zsh-git-aliases.yaml` — 44 pinned, non-destructive
  plain-command aliases.
- `src-tauri/src/macros.rs` — template model, validation, alias ranking,
  placeholder rendering, guard evaluation, and unit tests.
- `src-tauri/src/git_log.rs` — `GitStepEvent`, `GitStepStatus`, `RunContext`,
  `GitObserver`, test-only `NullObserver`.
- `src-tauri/src/events.rs` — `AppEvent`.
- `src/lib/gitLog.ts` — pure run/row folding and command normalisation.
- `src/lib/__tests__/gitLog.test.ts` — folding, ordering, status, and label tests.
- `src/components/GitLogDrawer.vue` — the bottom drawer.

### Modified

- `src-tauri/Cargo.toml` / `Cargo.lock` — added `serde_yaml_ng 0.10`.
- `src-tauri/src/git_runner.rs` — macro executor replaces the hand-written
  commit/checkout/pull/push helpers; new `is_dirty` / `has_upstream` probes;
  two integration-style macro tests.
- `src-tauri/src/watcher.rs` — bus retyped to `AppEvent`; `publish_fs_changed`;
  `EventBus` implements `GitObserver`.
- `src-tauri/src/service.rs` — observers threaded through commit/action/switch;
  group actions share one run id via `RunScopedObserver`.
- `src-tauri/src/commands/git_commands.rs` — commands take `State<EventBus>`.
- `src-tauri/src/main.rs` — new modules; desktop emits `repo:fs-changed` and
  `git:step`.
- `src-tauri/src/web.rs` — one SSE endpoint maps both variants.
- `src-tauri/src/scanner.rs` — small clippy fix (`is_some_and`).
- `src/types/domain.ts`, `src/lib/backend.ts`, `src/stores/repoStore.ts`,
  `src/App.vue`, `src/components/Toolbar.vue` — transport, store buffer, and
  drawer entry point.
- `README.md`, `CHANGELOG.md` — feature, layout, API, and architecture updates.

## 3. Key Changes

- **One source for behaviour and wording.** The executor walks the YAML, probes
  only the guards a macro actually uses (`dirty`/`clean`, `has_upstream`/
  `no_upstream`), renders `{message}` / `{branch}` / `{repo}`, executes argv
  arrays, and emits `running` then a terminal status per step. A template error
  disables actions with a readable message instead of panicking.
- **Conditional flows are data.** `pull` and `switch_branch` express autostash as
  `when: [dirty, auto_stash]` + `sets: stashed` + `when: [stashed]`; no Rust
  branching remains.
- **Typed transport.** `AppEvent` rides the existing 512-slot broadcast channel;
  desktop maps it to two Tauri event names, web maps it to two SSE event names on
  the unchanged `/api/events` route.
- **Folding that works in production.** Group actions share one run id, and the
  frontend normalises the `-C <path>` portion of a command before comparing, so
  the same step across six clones becomes one row with six shop chips.
- **Status aggregation by clone.** Each row keeps the latest status per clone
  before taking the worst, so a step's terminal `ok` correctly replaces the
  `running` event that preceded it.

## 4. Deviations from Plan

- **Phase 2 and Phase 3 landed together.** The command layer could not compile
  until `EventBus` carried `AppEvent`; `service.rs`, `commands/`, `main.rs`, and
  `web.rs` were therefore changed in one pass.
- **Shared run id for group actions.** The plan generated one UUID per macro
  invocation, and each clone invoked the macro separately. That would have made
  R3/R7 impossible, because every clone produced a different `runId`. A
  `RunScopedObserver` rewrites the run id of every step in a group action to one
  value. Commit-from-the-save-modal still calls one command per repository, so
  multi-repo saves remain separate cards (see Next Steps).
- **Command normalisation for folding.** Rendered commands contain each clone's
  real `-C` path, so a literal `command + comment` key could never fold across
  clones. `commandKey()` replaces the `-C` argument with `{repo}` for the key
  only; the displayed command remains the real one.
- **Per-clone latest status instead of a plain maximum.** The plan's `worst()`
  over all events leaves a row stuck on `running` after its terminal `ok`. The
  reducer is unchanged, but it is applied to the latest status per clone.
- **Token names in the drawer.** `bg-danger` / `text-danger` do not exist in the
  token preset; the drawer uses `bg-error` / `text-error-fg`. The token-only test
  passes.
- **`NullObserver` is `#[cfg(test)]`.** No production call site ignores the log,
  so keeping it public tripped `-D dead-code` under clippy.
- **Small pre-existing clippy fixes.** `scanner.rs` and `service.rs` used
  `map_or(false, …)`, an unnecessary clone in the discovery loop, and
  `sort_by`; clippy 1.98 with `-D warnings` rejected them. The discovery loop now
  consumes the owned paths and clones the vector once for the watcher.

## 5. Technical Decisions

- `serde_yaml_ng`, `deny_unknown_fields`, and an explicit `version: 1` check turn
  template typos and unsupported versions into test failures.
- Alias hints rank exact match → fewest extra tokens → shortest name →
  alphabetical, so results never depend on `HashMap` iteration order.
- The alias extract contains only plain `git …` targets. Shell functions,
  pipelines, command substitution, and destructive forms (`--force`,
  `reset --hard`, `clean`, rebase family, branch delete, stash drop) were
  omitted; the denylist is a second guard.
- The 200-entry session buffer evicts by oldest run, so a partially received run
  is not split across a truncation boundary.

## 6. Testing Notes

Commands run from `/topdata/repo-manager`:

```text
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check     clean
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
                                                              clean
cargo test --manifest-path src-tauri/Cargo.toml                20 passed
pnpm typecheck                                                clean
pnpm test                                                     41 passed (6 files)
```

New tests cover template parsing/validation, placeholder rendering, shell-safe
command rendering, alias ranking and destructive filtering, conjunctive guards,
a commit run emitting two `ok` steps with comments and alias, a clean pull
emitting two skipped stash steps, cross-shop folding, step ordering, worst-status
promotion, and `running → ok` transitions.

**Step-event volume.** A clean six-shop pull emits four events per shop
(skipped stash, running + ok pull, skipped pop) = 24 events; a dirty autostash
pull emits six per shop = 36. The 200-entry buffer therefore holds roughly eight
clean or five dirty pull runs, which is adequate for the session log.

The desktop window and browser mode were not smoke-tested interactively in this
environment (no GUI session); the transport mapping and the SSE shape are
covered by compilation and the existing web tests. The manual checklist in the
plan (open the drawer, expand a card, break a YAML line, remove an upstream)
still needs a human pass.

## 7. Usage Examples

```bash
# Group toolbar → "…" → What Git did
# Expand a card to see:
# Set your unfinished work aside for a moment
# git -C '/home/me/shops/shop-a' stash push -u -m git-repo-manager-autostash  # oh-my-zsh: gsta
# Download the new commits
# git -C '/home/me/shops/shop-a' pull --ff-only  # oh-my-zsh: gl
# Put your work back
# git -C '/home/me/shops/shop-a' stash pop  # oh-my-zsh: gstp
```

To change the wording or the steps of an action, edit
`src-tauri/macros/git-actions.yaml` and rebuild; tests fail if a placeholder,
guard, or alias is invalid.

## 8. Documentation Updates

- `README.md`: feature bullet, a "What Git did" section, the SSE event names, the
  architecture diagram, and the project layout now mention the template and the
  new modules.
- `CHANGELOG.md`: two `Added` entries and one `Changed` entry under `Unreleased`.

## 9. Next Steps

`status: partial` because these v1-scoped items were intentionally left out:

- Raw-stderr disclosure in the drawer.
- The `finishedAt` timeline field (present in the model, not populated or shown).
- Alias hints outside the expanded transcript.
- Multi-repository **Save to Git** folding: add an optional run id to the commit
  command/HTTP body so the save modal can share one card across repositories.
- Interactive desktop and browser smoke test following the plan's rollout table.
