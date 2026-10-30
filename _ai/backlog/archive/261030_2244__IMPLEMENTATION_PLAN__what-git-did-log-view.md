---
filename: "_ai/backlog/active/261030_2244__IMPLEMENTATION_PLAN__what-git-did-log-view.md"
title: "Implementation Plan: \"What Git did\" — template-driven Git actions with a learning log"
createdAt: 2026-10-30 22:44
updatedAt: 2026-10-30 22:44
status: completed
completedAt: 2026-10-30 23:25
priority: medium
tags: [git, observability, learning-ui, template, events, vue, rust]
estimatedComplexity: moderate
documentRevision: 1
sourceSpec: ""
sourceHandoff: ""
sourceAdrs: ["ADR__261030-3__git-actions-declared-in-embedded-yaml-template.md", "ADR__261030-4__typed-event-bus-for-desktop-and-web.md"]
documentType: IMPLEMENTATION_PLAN
---

# Implementation Plan: "What Git did" — template-driven Git actions with a learning log

Sources used: this conversation (design discussion), `_ai/technical_decisions/ADR__261030-3__git-actions-declared-in-embedded-yaml-template.md`,
`_ai/technical_decisions/ADR__261030-4__typed-event-bus-for-desktop-and-web.md`, and the
codebase (`src-tauri/src/{git_runner,service,watcher,web,main}.rs`, `src/{lib/backend.ts,stores/repoStore.ts,components/Toolbar.vue}`).
No feature spec and no design handoff exist, so `sourceSpec` and `sourceHandoff` are empty.

---

## 1. Problem

The application is a beginner-facing "Git for dummies" tool for a user who curates
several Shopware theme and plugin clones across multiple shops, finds Git
complicated, and is *learning-resistant*: screens full of Git jargon make her stop
reading.

Two problems follow:

1. **The Git work is invisible.** Every action shells out to the system `git`
   binary through one function, but the user only ever sees a toast. She therefore
   never builds a mental model of what "get latest" or "save to git" actually does.
2. **The behaviour and its explanations live in separate places.** Multi-step
   actions (`pull` = stash → pull → stash pop, conditional on the working tree) are
   hand-written Rust branches. Any log wording written next to them would drift
   from the behaviour it describes, and non-Rust reviewers could not improve it.

The requested outcome: a place in the app that shows, in the style of a commented
shell script, a gray `#` comment describing each step followed by the real command
that ran — including "you could have used this oh-my-zsh alias" hints — without
adding menus or surfacing Git syntax to someone who does not want to see it.

## 2. Executive summary

Git actions move out of hand-written Rust into a single embedded YAML document
(`src-tauri/macros/git-actions.yaml`) that is the **single source of truth for both
execution and explanation** (ADR 261030-3). A small engine evaluates per-step
guards (`dirty`, `clean`, `has_upstream`, `no_upstream`, `stashed`, `auto_stash`),
substitutes placeholders (`{message}`, `{branch}`, `{repo}`), executes steps through
the existing `run_git_cmd`, and emits one structured event per step through a new
`GitObserver` abstraction.

The existing broadcast bus is retyped from `String` to `enum AppEvent`
(ADR 261030-4), so repository-change notifications and Git-step events share one
transport: two Tauri event names on desktop, one SSE endpoint for the web build.

The frontend gains a **bottom drawer** reachable from one item in the existing "…"
toolbar menu. Each action renders as a friendly card (title, shop count, duration,
outcome). Expanding a card reveals the literal transcript: gray `#` comments, real
`git -C <repo> …` commands, grayed-out steps that were not needed, and an
oh-my-zsh alias hint where one exists. Identical commands across clones fold into a
single row with shop chips. Nothing is persisted; background inspection commands
are deliberately not streamed.

## 3. Design Decisions & Rejected Alternatives

| # | Decision | Rationale / alternative dropped |
|---|----------|--------------------------------|
| D1 | Actions declared in `src-tauri/macros/git-actions.yaml`, embedded with `include_str!` | One document drives execution **and** the log wording; wording stays reviewable outside Rust. Runtime loading rejected: desktop bundling and `--web` resolve paths differently (ADR 261030-3). |
| D2 | Steps carry `args` arrays, never shell strings | Existing code already passes argv arrays; no shell quoting bugs, no injection surface. A shell-transcript file format was rejected for the same reason. |
| D3 | Guards are a closed enum evaluated in order, `when` is a list (AND) | Lets `stash` steps exist without Rust `if` branches: `when: [dirty, auto_stash]`. Conditional logic as data is testable in one place. |
| D4 | `include_str!` + `OnceLock<Result<…>>` instead of `LazyLock` | `LazyLock` is Rust 1.80; the crate declares MSRV 1.78. `OnceLock` is stable since 1.70. |
| D5 | `serde_yaml_ng` for deserialization | `serde_yaml` is archived. `deny_unknown_fields` turns template typos into test failures (ADR 261030-3). |
| D6 | Alias hints come from a bundled, pinned oh-my-zsh extract, not from hand-written per-step text | Hints must be real; a test rejects unknown or denylisted aliases. Exact argument matching rejected because most aliases add flags (`gc` = `commit --verbose`), so hints rank exact → same subcommand → shortest. |
| D7 | `AppEvent` enum on the existing bus; one SSE endpoint, two event names | Avoids a second endpoint and a second frontend subscription. Untyped JSON envelopes rejected (ADR 261030-4). |
| D8 | Only user-initiated actions are streamed; inspection commands are not | 4 inspection commands × every repo × every refresh would evict the actions from a 200-entry buffer. The pane shows only what the user caused. |
| D9 | Session-only in-memory ring buffer | No config growth, no commit messages written to disk (ADR 261030-4). |
| D10 | Drawer opened from one item in the existing "…" menu, closed by default | The target user is overwhelmed by surfaces; no new navigation, no new screen, no new endpoint. |

---

## 4. Project environment and conventions

- **Repository**: `/topdata/repo-manager` (Tauri v2 + Vue 3 + Pinia, Rust core shared by desktop and axum web mode).
- **Git workflow**: no `AGENTS.md` exists. Do **not** create branches, do **not** open PRs; implement every phase on the current branch and commit only when the user asks.
- **Rust**: edition 2021, `rust-version = "1.78.0"`, no `unsafe`, `Result` over `unwrap` on recoverable paths, `cargo fmt` + `cargo clippy --all-targets -- -D warnings` clean. Unit tests live in `#[cfg(test)] mod tests` next to the code.
- **Frontend**: Vue 3 `<script setup lang="ts">`, Pinia store in `src/stores/repoStore.ts`, design system `@topdata-software-gmbh/ui-kit-web-vue3` + `--td-*` tokens. **No raw palette classes or hex colours** — `src/lib/__tests__/token-only.test.ts` enforces this.
- **Verification commands**:

```bash
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
pnpm typecheck
pnpm test
```

- **Known operational note**: the Vue packages are `link:` dependencies on `../topdata-design-system`; build them once with `(cd ../topdata-design-system && pnpm -r build)` before `pnpm typecheck` if types are missing.
- **New files introduced**: `src-tauri/macros/*.yaml`, `src-tauri/src/{macros,git_log,events}.rs`, `src/components/GitLogDrawer.vue`, `src/lib/gitLog.ts`, `src/lib/__tests__/gitLog.test.ts`. None require `.gitignore` changes.

---

## 5. Requirements traceability

Agreed requirements from the design discussion, used as acceptance checks:

| ID | Requirement | Satisfied in |
|----|-------------|--------------|
| R1 | All user-facing Git actions are declared in one YAML template that is executed, not merely described | Phase 1, Phase 2 |
| R2 | Log prints a gray `#` comment describing each step, then the real command | Phase 4 |
| R3 | Bundled multi-command actions appear as one card with ordered steps | Phase 2, Phase 4 |
| R4 | Conditional steps (autostash, upstream creation) stay conditional and are shown as "not needed this time" | Phase 1, Phase 2, Phase 4 |
| R5 | Alias hints come from oh-my-zsh aliases; destructive aliases never appear | Phase 1 |
| R6 | Desktop and web both receive events over their existing transport | Phase 3 |
| R7 | Identical commands across clones fold into one row with shop labels | Phase 4 |
| R8 | Failures show the existing plain-language message on the card (raw stderr disclosure deferred) | Phase 2, Phase 4 |
| R9 | No new navigation, no new HTTP endpoint, log is session-only | Phase 3, Phase 4 |
| R10 | Existing behaviour and tests keep passing (autostash, conflict refusal, upstream creation) | Phase 2 |

---

## 6. Phase 1 — Macro model, template documents, validation

**Goal**: introduce the YAML documents and the deserialization/validation layer
without changing any runtime behaviour. Everything is proven by tests.

### 6.1 [NEW FILE] `src-tauri/macros/git-actions.yaml`

```yaml
version: 1
macros:
  fetch:
    title: "Checked for updates"
    comment: "Look for new work on the server without changing your files"
    steps:
      - args: ["fetch", "--all", "--prune"]
        comment: "Ask the server whether anything new exists"
        alias: "gf"

  pull:
    title: "Got the latest"
    comment: "Bring in the changes the others made"
    steps:
      - when: [dirty]
        args: ["stash", "push", "-u", "-m", "git-repo-manager-autostash"]
        comment: "Set your unfinished work aside for a moment"
        sets: stashed
        alias: "gsta"
      - args: ["pull", "--ff-only"]
        comment: "Download the new commits"
        alias: "gl"
      - when: [stashed]
        args: ["stash", "pop"]
        comment: "Put your work back"
        alias: "gstp"

  push:
    title: "Sent changes"
    comment: "Upload your saved checkpoints to the server"
    require: on_branch
    steps:
      - when: [has_upstream]
        args: ["push"]
        comment: "Upload to the server version this project already follows"
        alias: "gp"
      - when: [no_upstream]
        args: ["push", "-u", "origin", "{branch}"]
        comment: "Upload and connect this project version to the server"

  stash:
    title: "Set changes aside"
    comment: "Temporarily store unfinished work"
    steps:
      - args: ["stash", "push", "-u"]
        comment: "Move your unsaved work to a safe place"
        alias: "gsta"

  stash_pop:
    title: "Brought back set-aside changes"
    comment: "Restore the last set of changes"
    steps:
      - args: ["stash", "pop"]
        comment: "Put the most recent set-aside work back"
        alias: "gstp"

  commit:
    title: "Saved changes"
    comment: "Keep a checkpoint of your work"
    steps:
      - args: ["add", "-A"]
        comment: "Mark every changed file for saving"
        alias: "gaa"
      - args: ["commit", "-m", "{message}"]
        comment: "Store them under one description"
        alias: "gcmsg"

  switch_branch:
    title: "Switched project version"
    comment: "Move every project to the version you chose"
    steps:
      - when: [dirty, auto_stash]
        args: ["stash", "push", "-u", "-m", "git-repo-manager-autostash"]
        comment: "Set your unfinished work aside for a moment"
        sets: stashed
        alias: "gsta"
      - args: ["checkout", "{branch}"]
        comment: "Switch to the version you chose"
        alias: "gco"
      - when: [stashed]
        args: ["stash", "pop"]
        comment: "Put your work back"
        alias: "gstp"
```

### 6.2 [NEW FILE] `src-tauri/macros/oh-my-zsh-git-aliases.yaml`

Generated from the upstream oh-my-zsh `git.plugin.zsh`. **Keep only aliases whose
target is a plain `git …` invocation** — drop every `!` shell function and every
target containing `$(`, `&&`, `|`, `;` or backticks (`ggpush`, `gbgD`, `glp`,
`gbgd`, `gpsup`, …). Committed and pinned; regenerating is a deliberate act.

```yaml
version: 1
aliases:
  ga: "git add"
  gaa: "git add --all"
  gapa: "git add --patch"
  gau: "git add --update"
  gst: "git status"
  gsb: "git status --short --branch"
  gss: "git status --short"
  gsn: "git status --untracked-files=no"
  gc: "git commit --verbose"
  gcmsg: "git commit --message"
  gca: "git commit --verbose --all"
  gds: "git diff --staged"
  gf: "git fetch"
  gfa: "git fetch --all --tags --prune --jobs=10"
  gl: "git pull"
  gpr: "git pull --rebase"
  gp: "git push"
  gpv: "git push --verbose"
  gco: "git checkout"
  gcb: "git checkout -b"
  gcor: "git checkout --recurse-submodules"
  gsta: "git stash push"
  gstp: "git stash pop"
  gstl: "git stash list"
  gstaa: "git stash apply"
  gstd: "git stash drop"
  gsh: "git show"
  grf: "git reflog"
  glo: "git log --oneline --decorate"
  glog: "git log --oneline --decorate --graph"
  # … copy the remaining plain targets from the alias dump, excluding the
  # shell-function and pipeline forms listed above.
```

### 6.3 [NEW FILE] `src-tauri/src/macros.rs`

```rust
//! Declarative Git actions: the YAML template is the single source of truth for
//! both execution and the user-facing explanation of each step.

use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;
use std::sync::OnceLock;

const TEMPLATE_YAML: &str = include_str!("../macros/git-actions.yaml");
const ALIASES_YAML: &str = include_str!("../macros/oh-my-zsh-git-aliases.yaml");

const PLACEHOLDERS: &[&str] = &["message", "branch", "repo"];

/// Targets that must never be advertised to a beginner.
const ALIAS_DENYLIST: &[&str] = &[
    "--force",
    "reset --hard",
    "clean",
    "filter-repo",
    "bisect",
    "bind",
];

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GitMacros {
    pub version: u32,
    pub macros: HashMap<String, GitMacro>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GitMacro {
    pub title: String,
    pub comment: String,
    #[serde(default)]
    pub require: Require,
    pub steps: Vec<GitStep>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GitStep {
    pub args: Vec<String>,
    #[serde(default)]
    pub comment: String,
    #[serde(default)]
    pub when: Vec<Guard>,
    #[serde(default)]
    pub sets: Effect,
    #[serde(default)]
    pub alias: Option<String>,
}

#[derive(Debug, Clone, Copy, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Require {
    #[default]
    Nothing,
    OnBranch,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Guard {
    Dirty,
    Clean,
    HasUpstream,
    NoUpstream,
    Stashed,
    AutoStash,
}

#[derive(Debug, Clone, Copy, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Effect {
    #[default]
    Nothing,
    Stashed,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AliasTable {
    pub version: u32,
    pub aliases: HashMap<String, String>,
}

impl AliasTable {
    /// Alias a shell user could have typed instead of these arguments.
    ///
    /// Ranking: exact argument match first, then the target with the fewest
    /// extra tokens, then the shortest alias name, then alphabetical. Ranking
    /// is deterministic, so a hint never depends on hash-map iteration order.
    /// Destructive targets are never returned.
    pub fn hint_for(&self, args: &[String]) -> Option<String> {
        let subcommand = args.first()?;
        // Never teach a shortcut for a destructive invocation itself.
        if args.iter().any(|arg| is_denylisted(arg)) {
            return None;
        }
        let mut best: Option<(bool, std::cmp::Reverse<usize>, std::cmp::Reverse<usize>, &str)> =
            None;

        for (name, target) in &self.aliases {
            if is_denylisted(target) {
                continue;
            }
            let Some(rest) = target.strip_prefix("git ") else {
                continue;
            };
            let tokens: Vec<&str> = rest.split_whitespace().collect();
            if tokens.first() != Some(&subcommand.as_str()) {
                continue;
            }
            let exact = tokens.len() == args.len()
                && tokens.iter().zip(args).all(|(token, arg)| token == arg);
            let candidate = (
                exact,
                std::cmp::Reverse(tokens.len()),
                std::cmp::Reverse(name.len()),
                name.as_str(),
            );
            let better = best.map_or(true, |current| candidate > current);
            if better {
                best = Some(candidate);
            }
        }

        best.map(|(_, _, _, name)| name.to_string())
    }

    fn contains_plain(&self, alias: &str, args: &[String]) -> bool {
        let Some(target) = self.aliases.get(alias) else {
            return false;
        };
        if is_denylisted(target) {
            return false;
        }
        let Some(rest) = target.strip_prefix("git ") else {
            return false;
        };
        rest.split_whitespace().next() == args.first().map(String::as_str)
    }
}

fn is_denylisted(target: &str) -> bool {
    let lower = target.to_lowercase();
    ALIAS_DENYLIST.iter().any(|banned| lower.contains(banned))
}

impl GitMacros {
    fn validate(&self, aliases: &AliasTable) -> Result<(), String> {
        for (id, mac) in &self.macros {
            if mac.steps.is_empty() {
                return Err(format!("macro `{id}` has no steps"));
            }
            for (index, step) in mac.steps.iter().enumerate() {
                if step.args.is_empty() {
                    return Err(format!("macro `{id}` step {index} has no args"));
                }
                for arg in &step.args {
                    for placeholder in placeholders_of(arg) {
                        if !PLACEHOLDERS.contains(&placeholder.as_str()) {
                            return Err(format!(
                                "macro `{id}` step {index} uses unknown placeholder {{{placeholder}}}"
                            ));
                        }
                    }
                }
                if let Some(alias) = &step.alias {
                    if !aliases.contains_plain(alias, &step.args) {
                        return Err(format!(
                            "macro `{id}` step {index} references unknown or unusable alias `{alias}`"
                        ));
                    }
                }
            }
        }
        Ok(())
    }
}

fn placeholders_of(arg: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut rest = arg;
    while let Some(start) = rest.find('{') {
        let Some(end) = rest[start + 1..].find('}') else {
            break;
        };
        found.push(rest[start + 1..start + 1 + end].to_string());
        rest = &rest[start + 1 + end + 1..];
    }
    found
}

static TEMPLATE: OnceLock<Result<(GitMacros, AliasTable), String>> = OnceLock::new();

/// Parsed template and alias table, or a human-readable reason they are unusable.
pub fn template() -> Result<&'static (GitMacros, AliasTable), String> {
    TEMPLATE
        .get_or_init(|| {
            let macros: GitMacros = serde_yaml_ng::from_str(TEMPLATE_YAML)
                .map_err(|error| format!("git-actions.yaml: {error}"))?;
            let aliases: AliasTable = serde_yaml_ng::from_str(ALIASES_YAML)
                .map_err(|error| format!("oh-my-zsh-git-aliases.yaml: {error}"))?;
            macros.validate(&aliases)?;
            Ok((macros, aliases))
        })
        .as_ref()
        .map_err(Clone::clone)
}

/// Values a macro can interpolate into its arguments.
pub struct MacroContext<'a> {
    pub message: Option<&'a str>,
    pub branch: Option<&'a str>,
    pub repo_path: &'a Path,
    pub auto_stash: bool,
}

pub fn render_args(args: &[String], ctx: &MacroContext<'_>) -> Result<Vec<String>, String> {
    args.iter()
        .map(|arg| {
            let mut rendered = arg.clone();
            for placeholder in placeholders_of(arg) {
                let value = match placeholder.as_str() {
                    "message" => ctx.message.unwrap_or_default().to_string(),
                    "branch" => ctx.branch.unwrap_or_default().to_string(),
                    "repo" => ctx.repo_path.to_string_lossy().to_string(),
                    other => return Err(format!("unknown placeholder {{{other}}}")),
                };
                rendered = rendered.replace(&format!("{{{placeholder}}}"), &value);
            }
            Ok(rendered)
        })
        .collect()
}

/// Shell-style rendering for display only. Nothing is executed from this string.
pub fn render_command(repo_path: &Path, args: &[String]) -> String {
    let mut parts = vec![
        "git".to_string(),
        "-C".to_string(),
        shell_quote(&repo_path.to_string_lossy()),
    ];
    parts.extend(args.iter().map(|arg| shell_quote(arg)));
    parts.join(" ")
}

fn shell_quote(value: &str) -> String {
    let safe = !value.is_empty()
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "-_./:=@+".contains(c));
    if safe {
        value.to_string()
    } else {
        format!("'{}'", value.replace('\'', "'\\''"))
    }
}

/// Runtime facts a guard asks about, probed once per macro run.
#[derive(Debug, Clone, Copy, Default)]
pub struct GuardState {
    pub dirty: bool,
    pub has_upstream: bool,
    pub stashed: bool,
    pub auto_stash: bool,
}

impl GuardState {
    pub fn allows(&self, guards: &[Guard]) -> bool {
        guards.iter().all(|guard| match guard {
            Guard::Dirty => self.dirty,
            Guard::Clean => !self.dirty,
            Guard::HasUpstream => self.has_upstream,
            Guard::NoUpstream => !self.has_upstream,
            Guard::Stashed => self.stashed,
            Guard::AutoStash => self.auto_stash,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn load() -> &'static (GitMacros, AliasTable) {
        template().expect("template must be valid")
    }

    #[test]
    fn template_parses_and_validates() {
        let (macros, _) = load();
        assert!(macros.macros.contains_key("commit"));
        assert!(macros.macros.contains_key("switch_branch"));
    }

    #[test]
    fn every_step_arg_is_a_string() {
        let (macros, _) = load();
        for mac in macros.macros.values() {
            for step in &mac.steps {
                assert!(!step.args.is_empty());
            }
        }
    }

    #[test]
    fn renders_message_and_branch_placeholders() {
        let ctx = MacroContext {
            message: Some("Update hero slider"),
            branch: Some("main"),
            repo_path: Path::new("/tmp/shop-a"),
            auto_stash: true,
        };
        let args = render_args(
            &["commit".into(), "-m".into(), "{message}".into()],
            &ctx,
        )
        .unwrap();
        assert_eq!(args, vec!["commit", "-m", "Update hero slider"]);
    }

    #[test]
    fn renders_copy_pasteable_command() {
        let command = render_command(
            Path::new("/tmp/shop a"),
            &["commit".into(), "-m".into(), "Update hero slider".into()],
        );
        assert_eq!(
            command,
            "git -C '/tmp/shop a' commit -m 'Update hero slider'"
        );
    }

    #[test]
    fn prefers_exact_alias_then_shortest_subcommand_match() {
        let (_, aliases) = load();
        assert_eq!(
            aliases.hint_for(&["add".into(), "--all".into()]),
            Some("gaa".to_string())
        );
        assert_eq!(
            aliases.hint_for(&["status".into(), "--porcelain=v1".into()]),
            Some("gst".to_string())
        );
    }

    #[test]
    fn never_hints_destructive_commands() {
        let (_, aliases) = load();
        for args in [
            vec!["push".to_string(), "--force".to_string()],
            vec!["reset".to_string(), "--hard".to_string()],
            vec!["clean".to_string(), "-fd".to_string()],
        ] {
            assert_eq!(aliases.hint_for(&args), None, "args {args:?} must not be hinted");
        }

        // Every hint that is produced must point at a non-destructive target.
        for args in [
            vec!["status".to_string()],
            vec!["add".to_string(), "--all".to_string()],
            vec!["commit".to_string(), "-m".to_string(), "x".to_string()],
        ] {
            let alias = aliases.hint_for(&args).expect("alias expected");
            let target = aliases.aliases.get(&alias).expect("alias must exist");
            assert!(!is_denylisted(target), "{alias} -> {target} is destructive");
        }
    }

    #[test]
    fn guards_are_conjunctive() {
        let state = GuardState {
            dirty: true,
            auto_stash: false,
            ..GuardState::default()
        };
        assert!(!state.allows(&[Guard::Dirty, Guard::AutoStash]));
        assert!(state.allows(&[Guard::Dirty]));
        assert!(state.allows(&[]));
    }
}
```

**Verification**: `cargo test --manifest-path src-tauri/Cargo.toml macros::` and
`cargo clippy …`. No behaviour changes yet — the existing `git_runner` tests must
still pass untouched.

---

## 7. Phase 2 — Step events, macro executor, migrated actions

**Goal**: execute macros instead of hand-written flows, emit structured step
events, and keep every existing behaviour test green.

### 7.1 [NEW FILE] `src-tauri/src/git_log.rs`

```rust
//! Structured Git step events and the observer seam used to publish them.

use serde::Serialize;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum GitStepStatus {
    Running,
    Ok,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitStepEvent {
    pub run_id: String,
    pub repo_path: String,
    pub macro_id: String,
    pub title: String,
    pub step_index: usize,
    pub comment: String,
    pub command: String,
    pub status: GitStepStatus,
    pub duration_ms: u64,
    pub message: Option<String>,
    pub alias: Option<String>,
}

/// Identity shared by every step of one macro invocation.
pub struct RunContext<'a> {
    pub run_id: &'a str,
    pub repo_path: &'a Path,
    pub macro_id: &'a str,
    pub title: &'a str,
}

impl GitStepEvent {
    pub fn new(
        run: &RunContext<'_>,
        step_index: usize,
        comment: &str,
        command: &str,
        status: GitStepStatus,
        alias: Option<String>,
    ) -> Self {
        Self {
            run_id: run.run_id.to_string(),
            repo_path: run.repo_path.to_string_lossy().to_string(),
            macro_id: run.macro_id.to_string(),
            title: run.title.to_string(),
            step_index,
            comment: comment.to_string(),
            command: command.to_string(),
            status,
            duration_ms: 0,
            message: None,
            alias,
        }
    }
}

/// Receives every step transition. Implemented by the event bus in production
/// and by a recording observer in tests.
pub trait GitObserver: Send + Sync {
    fn on_step(&self, event: &GitStepEvent);
}

/// Observer used by tests and by call sites that do not care about the log.
pub struct NullObserver;

impl GitObserver for NullObserver {
    fn on_step(&self, _event: &GitStepEvent) {}
}
```

### 7.2 [NEW FILE] `src-tauri/src/events.rs`

```rust
//! Typed payload of the application event bus.

use crate::git_log::GitStepEvent;

#[derive(Debug, Clone)]
pub enum AppEvent {
    FsChanged(String),
    GitStep(GitStepEvent),
}
```

### 7.3 [MODIFY] `src-tauri/src/git_runner.rs`

Keep every existing inspection helper. Replace `commit_changes`,
`checkout_branch`, `execute_action`, `stash_if_needed`, `restore_stash`,
`pull_with_autostash` and `push_branch` with one macro executor. Add the probes the
guards need.

```rust
use crate::git_log::{GitObserver, GitStepEvent, GitStepStatus, RunContext};
use crate::macros::{self, Guard, GuardState, MacroContext, Require};

const AUTOSTASH_MESSAGE: &str = "git-repo-manager-autostash";

impl GitRunner {
    /// Whether the working tree has anything not yet recorded in Git.
    pub(crate) async fn is_dirty(repo_path: &Path) -> bool {
        Self::run_git_stdout(repo_path, &["status", "--porcelain=v1"])
            .await
            .map_or(false, |output| !output.trim().is_empty())
    }

    pub(crate) async fn has_upstream(repo_path: &Path) -> bool {
        Self::run_git_stdout(
            repo_path,
            &[
                "rev-parse",
                "--abbrev-ref",
                "--symbolic-full-name",
                "@{upstream}",
            ],
        )
        .await
        .is_some()
    }

    /// Executes a declared macro and reports every step transition.
    pub async fn execute_macro(
        repo_path: &Path,
        macro_id: &str,
        ctx: &MacroContext<'_>,
        observer: &dyn GitObserver,
    ) -> OperationResult {
        let (macros, aliases) = match macros::template() {
            Ok(loaded) => loaded,
            Err(error) => {
                log::error!("git action template unusable: {error}");
                return failure_result(
                    repo_path,
                    "Git actions are not available in this build. \
                     Restart the application; if it persists, reinstall it.",
                    String::new(),
                );
            }
        };

        let Some(mac) = macros.macros.get(macro_id) else {
            return failure_result(
                repo_path,
                format!("This action is not available: {macro_id}"),
                String::new(),
            );
        };

        if mac.require == Require::OnBranch {
            let branch = Self::get_current_branch(repo_path).await;
            if branch == "HEAD" || branch.is_empty() {
                return failure_result(
                    repo_path,
                    "You are not on a branch. Switch to a branch before using this action.",
                    String::new(),
                );
            }
        }

        let mut guards = GuardState {
            auto_stash: ctx.auto_stash,
            ..GuardState::default()
        };
        if mac.steps.iter().any(|step| {
            step.when
                .iter()
                .any(|guard| matches!(guard, Guard::Dirty | Guard::Clean))
        }) {
            guards.dirty = Self::is_dirty(repo_path).await;
        }
        if mac.steps.iter().any(|step| {
            step.when
                .iter()
                .any(|guard| matches!(guard, Guard::HasUpstream | Guard::NoUpstream))
        }) {
            guards.has_upstream = Self::has_upstream(repo_path).await;
        }

        let run_id = uuid::Uuid::new_v4().to_string();
        let run = RunContext {
            run_id: &run_id,
            repo_path,
            macro_id,
            title: &mac.title,
        };
        let mut output = String::new();
        let mut failure: Option<String> = None;

        for (index, step) in mac.steps.iter().enumerate() {
            let args = match macros::render_args(&step.args, ctx) {
                Ok(args) => args,
                Err(error) => {
                    return failure_result(repo_path, error, String::new());
                }
            };
            let command = macros::render_command(repo_path, &args);
            let alias = step
                .alias
                .clone()
                .or_else(|| aliases.hint_for(&step.args));
            let base = GitStepEvent::new(
                &run,
                index,
                &step.comment,
                &command,
                GitStepStatus::Running,
                alias,
            );

            if !guards.allows(&step.when) {
                observer.on_step(&GitStepEvent {
                    status: GitStepStatus::Skipped,
                    ..base
                });
                continue;
            }

            observer.on_step(&base);
            let started = std::time::Instant::now();
            let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();

            match Self::run_git_cmd(repo_path, &arg_refs).await {
                Ok(result) if result.status.success() => {
                    let duration_ms = started.elapsed().as_millis() as u64;
                    let text = output_text(&result);
                    if !text.is_empty() {
                        if !output.is_empty() {
                            output.push('\n');
                        }
                        output.push_str(&text);
                    }
                    if step.sets == macros::Effect::Stashed {
                        guards.stashed = true;
                    }
                    observer.on_step(&GitStepEvent {
                        status: GitStepStatus::Ok,
                        duration_ms,
                        ..base
                    });
                }
                Ok(result) => {
                    let duration_ms = started.elapsed().as_millis() as u64;
                    let error = friendly_error(&error_text(&result));
                    observer.on_step(&GitStepEvent {
                        status: GitStepStatus::Failed,
                        duration_ms,
                        message: Some(error.clone()),
                        ..base
                    });
                    failure = Some(error);
                    Self::report_remaining_skipped(&run, &mac.steps[index + 1..], index + 1, observer);
                    break;
                }
                Err(error) => {
                    let duration_ms = started.elapsed().as_millis() as u64;
                    observer.on_step(&GitStepEvent {
                        status: GitStepStatus::Failed,
                        duration_ms,
                        message: Some(error.clone()),
                        ..base
                    });
                    failure = Some(error);
                    Self::report_remaining_skipped(&run, &mac.steps[index + 1..], index + 1, observer);
                    break;
                }
            }
        }

        match failure {
            Some(error) => failure_result(repo_path, error, output),
            None => OperationResult {
                repo_path: repo_path.to_string_lossy().to_string(),
                success: true,
                output,
                error: None,
            },
        }
    }

    fn report_remaining_skipped(
        run: &RunContext<'_>,
        remaining: &[macros::GitStep],
        offset: usize,
        observer: &dyn GitObserver,
    ) {
        for (index, step) in remaining.iter().enumerate() {
            observer.on_step(&GitStepEvent::new(
                run,
                offset + index,
                &step.comment,
                &macros::render_command(run.repo_path, &step.args),
                GitStepStatus::Skipped,
                step.alias.clone(),
            ));
        }
    }

    pub async fn commit_changes(
        repo_path: &Path,
        message: &str,
        observer: &dyn GitObserver,
    ) -> OperationResult {
        let message = message.trim();
        if message.is_empty() {
            return failure_result(
                repo_path,
                "Write a short message that describes what you changed.",
                String::new(),
            );
        }

        let changes = Self::list_changes(repo_path).await;
        if changes.is_empty() {
            return failure_result(repo_path, "There are no changes to save.", String::new());
        }

        let ctx = MacroContext {
            message: Some(message),
            branch: None,
            repo_path,
            auto_stash: true,
        };
        Self::execute_macro(repo_path, "commit", &ctx, observer).await
    }

    pub async fn checkout_branch(
        repo_path: &Path,
        branch: &str,
        auto_stash: bool,
        observer: &dyn GitObserver,
    ) -> OperationResult {
        let current_branch = Self::get_current_branch(repo_path).await;
        if current_branch == branch {
            return OperationResult {
                repo_path: repo_path.to_string_lossy().to_string(),
                success: true,
                output: format!("Already on branch '{}'.", branch),
                error: None,
            };
        }

        let ctx = MacroContext {
            message: None,
            branch: Some(branch),
            repo_path,
            auto_stash,
        };
        Self::execute_macro(repo_path, "switch_branch", &ctx, observer).await
    }

    pub async fn execute_action(
        repo_path: &Path,
        action: &str,
        observer: &dyn GitObserver,
    ) -> OperationResult {
        let macro_id = match action {
            "fetch" | "pull" | "push" | "stash" | "stash_pop" => action,
            other => {
                return failure_result(
                    repo_path,
                    format!("Unknown action: {other}"),
                    String::new(),
                )
            }
        };

        let branch = Self::get_current_branch(repo_path).await;
        let ctx = MacroContext {
            message: None,
            branch: Some(&branch),
            repo_path,
            auto_stash: true,
        };
        Self::execute_macro(repo_path, macro_id, &ctx, observer).await
    }
}
```

Delete `stash_if_needed`, `restore_stash`, `pull_with_autostash`, `push_branch`.
`AUTOSTASH_MESSAGE` moves into the YAML; remove the constant if it becomes unused
(the `stash list` assertion in tests uses `git` directly, so it can go).

Update remaining call sites inside the file and in tests to pass `&NullObserver`.
Add tests at the bottom of the module:

```rust
#[cfg(test)]
mod macro_tests {
    use super::*;
    use crate::git_log::{GitObserver, GitStepEvent};
    use std::sync::Mutex;

    #[derive(Default)]
    struct Recording(Mutex<Vec<GitStepEvent>>);

    impl GitObserver for Recording {
        fn on_step(&self, event: &GitStepEvent) {
            self.0.lock().unwrap().push(event.clone());
        }
    }

    #[tokio::test]
    async fn emits_comment_and_command_for_each_step() {
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        super::tests::init_repository(&repo);
        std::fs::write(repo.join("tracked.txt"), "changed\n").unwrap();

        let recorder = Recording::default();
        let result = GitRunner::commit_changes(&repo, "update", &recorder).await;
        assert!(result.success, "{:?}", result.error);

        let events = recorder.0.lock().unwrap().clone();
        let ok: Vec<_> = events
            .iter()
            .filter(|event| event.status == GitStepStatus::Ok)
            .collect();
        assert_eq!(ok.len(), 2);
        assert_eq!(ok[0].comment, "Mark every changed file for saving");
        assert!(ok[0].command.starts_with("git -C "));
        assert_eq!(ok[1].alias.as_deref(), Some("gcmsg"));
    }

    #[tokio::test]
    async fn pull_marks_stash_steps_as_skipped_when_clean() {
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        super::tests::init_repository(&repo);

        let recorder = Recording::default();
        let result = GitRunner::execute_action(&repo, "pull", &recorder).await;
        // No remote configured: the pull step fails, the stash steps never run.
        assert!(!result.success);

        let events = recorder.0.lock().unwrap().clone();
        let skipped: Vec<_> = events
            .iter()
            .filter(|event| event.status == GitStepStatus::Skipped)
            .collect();
        assert_eq!(skipped.len(), 2);
        assert!(skipped.iter().all(|event| event.command.contains("stash")));
    }
}
```

> Make `tests::init_repository` and `tests::run_git` `pub(super)` so the new
> module can reuse them instead of duplicating fixtures.

### 7.4 [MODIFY] `src-tauri/src/service.rs`

```rust
use crate::git_log::GitObserver;
use std::sync::Arc;

pub async fn commit_repo_changes(
    path: String,
    message: String,
    observer: &dyn GitObserver,
) -> OperationResult {
    GitRunner::commit_changes(&PathBuf::from(path), &message, observer).await
}

pub async fn execute_group_action(
    repo_paths: Vec<String>,
    action: String,
    concurrency_limit: usize,
    observer: Arc<dyn GitObserver>,
) -> Result<Vec<OperationResult>, String> {
    // …unchanged validation and semaphore setup…
    for path in repo_paths {
        let semaphore = Arc::clone(&semaphore);
        let action = action.clone();
        let observer = Arc::clone(&observer);
        tasks.push(tokio::spawn(async move {
            let _permit = semaphore.acquire_owned().await.ok();
            GitRunner::execute_action(&PathBuf::from(path), &action, observer.as_ref()).await
        }));
    }
    // …unchanged collection…
}

pub async fn switch_group_branch(
    repo_paths: Vec<String>,
    branch: String,
    auto_stash: bool,
    observer: Arc<dyn GitObserver>,
) -> Result<Vec<OperationResult>, String> {
    // …same pattern, calling
    // GitRunner::checkout_branch(&path, &branch, auto_stash, observer.as_ref())
}
```

### 7.5 [MODIFY] `src-tauri/src/commands/git_commands.rs`

```rust
use crate::watcher::EventBus;

#[tauri::command]
pub async fn commit_repo_changes(
    path: String,
    message: String,
    event_bus: State<'_, EventBus>,
) -> Result<OperationResult, String> {
    Ok(service::commit_repo_changes(path, message, event_bus.inner()).await)
}

#[tauri::command]
pub async fn execute_group_action(
    repo_paths: Vec<String>,
    action: String,
    concurrency_limit: usize,
    event_bus: State<'_, EventBus>,
) -> Result<Vec<OperationResult>, String> {
    service::execute_group_action(
        repo_paths,
        action,
        concurrency_limit,
        std::sync::Arc::new(event_bus.inner().clone()),
    )
    .await
}

#[tauri::command]
pub async fn switch_group_branch(
    repo_paths: Vec<String>,
    branch: String,
    auto_stash: bool,
    event_bus: State<'_, EventBus>,
) -> Result<Vec<OperationResult>, String> {
    service::switch_group_branch(
        repo_paths,
        branch,
        auto_stash,
        std::sync::Arc::new(event_bus.inner().clone()),
    )
    .await
}
```

> Step 7.4/7.5 compile only after Phase 3 retypes the bus; if the phases are
> implemented in one pass, land 7.4 and 7.5 together with 8.1–8.3.

**Verification** (after Phase 3 for the command layer):

```bash
cargo test --manifest-path src-tauri/Cargo.toml
```

Existing parity tests that must pass unchanged in intent:
`inspects_working_tree_branches_and_operations`, `lists_and_commits_changes`,
`refuses_to_commit_unresolved_conflicts`, `explains_common_git_errors`,
`resolves_gitfile_for_worktrees`. Adjust only the added `&NullObserver` argument.

---

## 8. Phase 3 — Retype the bus and wire both transports

**Goal**: `AppEvent` flows through the existing broadcast channel; desktop and web
deliver both event kinds; one frontend subscription helper replaces the old one.

### 8.1 [MODIFY] `src-tauri/src/watcher.rs`

```rust
use crate::events::AppEvent;
use crate::git_log::{GitObserver, GitStepEvent};
use tokio::sync::broadcast;

/// Fan-out channel shared by desktop (Tauri events) and web (SSE) frontends.
#[derive(Clone)]
pub struct EventBus {
    sender: broadcast::Sender<AppEvent>,
}

impl EventBus {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(512);
        Self { sender }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<AppEvent> {
        self.sender.subscribe()
    }

    pub fn publish_fs_changed(&self, repo_path: String) {
        let _ = self.sender.send(AppEvent::FsChanged(repo_path));
    }
}

impl GitObserver for EventBus {
    fn on_step(&self, event: &GitStepEvent) {
        let _ = self.sender.send(AppEvent::GitStep(event.clone()));
    }
}
```

Replace the single `event_bus.publish(key)` call in the watcher thread with
`event_bus.publish_fs_changed(key)`.

### 8.2 [MODIFY] `src-tauri/src/main.rs`

Register the new modules and map variants to Tauri event names:

```rust
mod events;
mod git_log;
mod macros;

// inside .setup(…)
tauri::async_runtime::spawn(async move {
    let mut receiver = event_bus.subscribe();
    while let Ok(event) = receiver.recv().await {
        match event {
            crate::events::AppEvent::FsChanged(repo_path) => {
                let _ = handle.emit("repo:fs-changed", repo_path);
            }
            crate::events::AppEvent::GitStep(step) => {
                let _ = handle.emit("git:step", step);
            }
        }
    }
});
```

### 8.3 [MODIFY] `src-tauri/src/web.rs`

`repo_events` keeps its route and auth layer but maps both variants:

```rust
async fn repo_events(
    State(state): State<Arc<WebState>>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    let receiver = state.event_bus.subscribe();
    let stream = BroadcastStream::new(receiver)
        .filter_map(|message| message.ok())
        .filter_map(|event| match event {
            crate::events::AppEvent::FsChanged(path) => {
                Some(Event::default().event("repo-fs-changed").data(path))
            }
            crate::events::AppEvent::GitStep(step) => serde_json::to_string(&step)
                .ok()
                .map(|json| Event::default().event("git-step").data(json)),
        });

    Sse::new(stream).keep_alive(KeepAlive::default())
}
```

Handlers for commit/actions/switch pass the bus down. The spawned action handlers
need an owned `Arc`; the single-commit handler borrows the bus:

```rust
async fn commit_repo_changes(
    State(state): State<Arc<WebState>>,
    Json(request): Json<CommitRequest>,
) -> ApiResult<OperationResult> {
    Ok(Json(
        service::commit_repo_changes(request.path, request.message, &state.event_bus).await,
    ))
}

async fn execute_group_action(
    State(state): State<Arc<WebState>>,
    Json(request): Json<GroupActionRequest>,
) -> ApiResult<Vec<OperationResult>> {
    service::execute_group_action(
        request.repo_paths,
        request.action,
        request.concurrency_limit,
        Arc::new(state.event_bus.clone()),
    )
    .await
    .map(Json)
    .map_err(api_error)
}
```

`switch_group_branch` follows the `execute_group_action` shape. Because the
handlers now take `State`, the corresponding `.route(...)` entries are unchanged —
axum extracts `State` from the router's `with_state`, not from the route table.

**Verification**: `cargo clippy … && cargo test …`; then run the web build and
confirm `curl -N http://127.0.0.1:8787/api/events` still emits
`event: repo-fs-changed` when a `.git` file changes.

---

## 9. Phase 4 — Frontend: transport, store, folding, drawer

### 9.1 [MODIFY] `src/types/domain.ts`

```ts
export type GitStepStatus = 'running' | 'ok' | 'failed' | 'skipped';

export interface GitStepEvent {
  runId: string;
  repoPath: string;
  macroId: string;
  title: string;
  stepIndex: number;
  comment: string;
  command: string;
  status: GitStepStatus;
  durationMs: number;
  message: string | null;
  alias: string | null;
}

export interface GitLogRow {
  key: string;
  comment: string;
  command: string;
  alias: string | null;
  status: GitStepStatus;
  repoPaths: string[];
  message: string | null;
}

export interface GitRun {
  runId: string;
  title: string;
  macroId: string;
  status: GitStepStatus;
  durationMs: number;
  rows: GitLogRow[];
  finishedAt: string;
}
```

### 9.2 [MODIFY] `src/lib/backend.ts`

Replace `subscribeToRepoChanges` with one helper; both event kinds share the
connection.

```ts
export interface AppEventHandlers {
  onRepoChanged?: (path: string) => void;
  onGitStep?: (step: GitStepEvent) => void;
}

export async function subscribeToEvents(handlers: AppEventHandlers): Promise<() => void> {
  if (isTauri) {
    const { listen } = await import('@tauri-apps/api/event');
    const unlisten = await Promise.all([
      listen<string>('repo:fs-changed', (event) => handlers.onRepoChanged?.(event.payload)),
      listen<GitStepEvent>('git:step', (event) => handlers.onGitStep?.(event.payload)),
    ]);
    return () => unlisten.forEach((stop) => stop());
  }

  const token = getWebToken();
  const url = new URL('/api/events', window.location.href);
  if (token) url.searchParams.set('token', token);

  const source = new EventSource(url);
  const repoHandler = (event: Event) => {
    handlers.onRepoChanged?.((event as MessageEvent<string>).data);
  };
  const stepHandler = (event: Event) => {
    try {
      handlers.onGitStep?.(JSON.parse((event as MessageEvent<string>).data) as GitStepEvent);
    } catch {
      // A malformed frame must never break the live view.
    }
  };
  source.addEventListener('repo-fs-changed', repoHandler as EventListener);
  source.addEventListener('git-step', stepHandler as EventListener);

  return () => source.close();
}
```

### 9.3 [NEW FILE] `src/lib/gitLog.ts`

Pure, unit-testable folding logic — no Vue imports.

```ts
import type { GitLogRow, GitRun, GitStepEvent, GitStepStatus } from '../types/domain';

const STATUS_PRIORITY: Record<GitStepStatus, number> = {
  failed: 4,
  running: 3,
  ok: 2,
  skipped: 1,
};

function worst(statuses: GitStepStatus[]): GitStepStatus {
  return statuses.reduce<GitStepStatus>(
    (worstSoFar, status) =>
      STATUS_PRIORITY[status] > STATUS_PRIORITY[worstSoFar] ? status : worstSoFar,
    'skipped',
  );
}

export function shopName(path: string): string {
  const parts = path.split(/[\\/]/).filter(Boolean);
  return parts[parts.length - 1] ?? path;
}

export function groupGitRuns(steps: GitStepEvent[]): GitRun[] {
  const runs = new Map<string, GitRun>();

  for (const step of steps) {
    const run = runs.get(step.runId) ?? {
      runId: step.runId,
      title: step.title,
      macroId: step.macroId,
      status: 'skipped' as GitStepStatus,
      durationMs: 0,
      rows: [],
      finishedAt: '',
    };

    const key = `${step.command}\u0000${step.comment}`;
    const row = run.rows.find((candidate) => candidate.key === key);
    if (row) {
      if (!row.repoPaths.includes(step.repoPath)) row.repoPaths.push(step.repoPath);
      row.status = worst([row.status, step.status]);
      row.message = row.message ?? step.message;
    } else {
      run.rows.push({
        key,
        comment: step.comment,
        command: step.command,
        alias: step.alias,
        status: step.status,
        repoPaths: [step.repoPath],
        message: step.message,
      });
    }

    run.status = worst([run.status, step.status]);
    run.durationMs = Math.max(run.durationMs, step.durationMs);
    runs.set(step.runId, run);
  }

  return [...runs.values()].reverse();
}
```

> `finishedAt` is intentionally omitted from rendering in v1; keep the field for a
> future timeline but do not populate it from a wall clock inside a pure function.

### 9.4 [MODIFY] `src/stores/repoStore.ts`

```ts
import type { GitStepEvent } from '../types/domain';

const GIT_STEP_LIMIT = 200;

const gitSteps = ref<GitStepEvent[]>([]);
const gitLogOpen = ref(false);

function recordGitStep(step: GitStepEvent): void {
  gitSteps.value = [...gitSteps.value, step];
  while (gitSteps.value.length > GIT_STEP_LIMIT) {
    const oldestRun = gitSteps.value[0].runId;
    const next = gitSteps.value.filter((entry) => entry.runId !== oldestRun);
    if (next.length === gitSteps.value.length) {
      gitSteps.value = gitSteps.value.slice(-50);
      break;
    }
    gitSteps.value = next;
  }
}

function openGitLog(): void {
  gitLogOpen.value = true;
}

function closeGitLog(): void {
  gitLogOpen.value = false;
}
```

Export `gitSteps`, `gitLogOpen`, `recordGitStep`, `openGitLog`, `closeGitLog` from
the store's return object.

### 9.5 [MODIFY] `src/App.vue`

```ts
onMounted(async () => {
  await store.initialize();

  try {
    unsubscribe = await subscribeToEvents({
      onRepoChanged: (path) => void store.refreshSingleRepo(path),
      onGitStep: (step) => store.recordGitStep(step),
    });
  } catch (error) {
    store.showToast('error', 'Live updates unavailable', String(error));
  }
});
```

Mount `<GitLogDrawer />` next to the other modals.

### 9.6 [MODIFY] `src/components/Toolbar.vue`

Add the item at the end of the existing `TdMenu` (after "Bring back set-aside
changes") and import `Terminal` from `lucide-vue-next`:

```vue
<TdMenuItem @select="store.openGitLog()">
  <template #icon><Terminal /></template>
  What Git did
  <template #description>See the exact commands behind the last actions.</template>
</TdMenuItem>
```

The item is never disabled: the drawer also explains why it is empty.

### 9.7 [NEW FILE] `src/components/GitLogDrawer.vue`

```vue
<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { X } from 'lucide-vue-next';
import { TdBadge, TdButton } from '@topdata-software-gmbh/ui-kit-web-vue3';
import { groupGitRuns, shopName } from '../lib/gitLog';
import { useRepoStore } from '../stores/repoStore';

const store = useRepoStore();
const expanded = ref<Set<string>>(new Set());

const runs = computed(() => groupGitRuns(store.gitSteps));

function toggle(runId: string): void {
  const next = new Set(expanded.value);
  if (next.has(runId)) next.delete(runId);
  else next.add(runId);
  expanded.value = next;
}

function onKeydown(event: KeyboardEvent): void {
  if (event.key === 'Escape' && store.gitLogOpen) store.closeGitLog();
}

watch(
  () => store.gitLogOpen,
  (open) => {
    if (!open) return;
    expanded.value = new Set();
  },
);
</script>

<template>
  <aside
    v-if="store.gitLogOpen"
    class="absolute inset-x-0 bottom-0 z-30 max-h-[45%] overflow-hidden border-t border-border bg-surface shadow-lg"
    role="region"
    aria-label="What Git did"
    @keydown="onKeydown"
  >
    <header class="flex items-center justify-between border-b border-border px-4 py-2">
      <div class="flex items-center gap-2">
        <span class="text-sm font-semibold text-fg">What Git did</span>
        <TdBadge variant="neutral">{{ runs.length }}</TdBadge>
      </div>
      <TdButton variant="ghost" size="sm" aria-label="Close" @click="store.closeGitLog()">
        <template #icon-left><X /></template>
      </TdButton>
    </header>

    <div v-if="runs.length === 0" class="px-4 py-6 text-sm text-fg-muted">
      Nothing yet. Save, get the latest, or send changes and it will show up here.
    </div>

    <ol v-else class="max-h-[38vh] overflow-y-auto px-4 py-2">
      <li v-for="run in runs" :key="run.runId" class="border-b border-border py-2 last:border-b-0">
        <button
          type="button"
          class="flex w-full items-start gap-2 text-left"
          :aria-expanded="expanded.has(run.runId)"
          @click="toggle(run.runId)"
        >
          <span
            class="mt-1.5 h-2 w-2 shrink-0 rounded-full"
            :class="{
              'bg-success': run.status === 'ok',
              'bg-warning': run.status === 'running',
              'bg-danger': run.status === 'failed',
              'bg-fg-muted': run.status === 'skipped',
            }"
          ></span>
          <span class="min-w-0 flex-1">
            <span class="block text-sm font-medium text-fg">{{ run.title }}</span>
            <span class="block text-[11px] text-fg-muted">
              {{ run.rows[0]?.repoPaths.length ?? 0 }}
              {{ (run.rows[0]?.repoPaths.length ?? 0) === 1 ? 'shop' : 'shops' }}
              · {{ run.durationMs }} ms
            </span>
          </span>
          <span class="text-[11px] text-fg-muted">
            {{ expanded.has(run.runId) ? 'Hide commands' : 'Git commands' }}
          </span>
        </button>

        <div v-if="expanded.has(run.runId)" class="mt-2 space-y-1 pl-4">
          <div v-for="row in run.rows" :key="row.key">
            <p class="font-mono text-[11px] text-fg-muted"># {{ row.comment }}</p>
            <p class="font-mono text-[12px] text-fg">
              {{ row.command }}
              <span v-if="row.alias" class="text-fg-muted">  # oh-my-zsh: {{ row.alias }}</span>
            </p>
            <p class="text-[11px] text-fg-muted">
              {{ row.repoPaths.map(shopName).join(' · ') }}
            </p>
            <p v-if="row.status === 'skipped'" class="text-[11px] text-fg-muted">
              not needed this time
            </p>
          </div>
        </div>

        <p
          v-for="row in run.rows.filter((entry) => entry.message)"
          :key="`${row.key}-error`"
          class="mt-2 rounded border border-border bg-surface-raised px-2 py-1 text-[11px] text-danger"
        >
          {{ row.message }}
        </p>
      </li>
    </ol>
  </aside>
</template>
```

The plain-language failure text therefore appears on the card without a click, and
raw stderr is not rendered in v1 (listed under Next Steps).

### 9.8 [NEW FILE] `src/lib/__tests__/gitLog.test.ts`

```ts
import { describe, expect, it } from 'vitest';
import { groupGitRuns, shopName } from '../gitLog';
import type { GitStepEvent } from '../../types/domain';

function step(overrides: Partial<GitStepEvent>): GitStepEvent {
  return {
    runId: 'run-1',
    repoPath: '/dev/shop-a',
    macroId: 'pull',
    title: 'Got the latest',
    stepIndex: 0,
    comment: 'Download the new commits',
    command: 'git -C /dev/shop-a pull --ff-only',
    status: 'ok',
    durationMs: 10,
    message: null,
    alias: 'gl',
    ...overrides,
  };
}

describe('groupGitRuns', () => {
  it('folds identical commands across shops into one row', () => {
    const runs = groupGitRuns([
      step({}),
      step({ repoPath: '/dev/shop-b' }),
      step({ repoPath: '/dev/shop-c' }),
    ]);
    expect(runs).toHaveLength(1);
    expect(runs[0].rows).toHaveLength(1);
    expect(runs[0].rows[0].repoPaths).toEqual([
      '/dev/shop-a',
      '/dev/shop-b',
      '/dev/shop-c',
    ]);
  });

  it('keeps distinct steps of one run in order', () => {
    const runs = groupGitRuns([
      step({ stepIndex: 0, comment: 'Set your unfinished work aside' }),
      step({ stepIndex: 1, comment: 'Download the new commits' }),
    ]);
    expect(runs[0].rows.map((row) => row.comment)).toEqual([
      'Set your unfinished work aside',
      'Download the new commits',
    ]);
  });

  it('promotes the worst status to the run', () => {
    const runs = groupGitRuns([
      step({}),
      step({ repoPath: '/dev/shop-b', status: 'failed' }),
    ]);
    expect(runs[0].status).toBe('failed');
    expect(runs[0].rows[0].status).toBe('failed');
  });

  it('lists the newest run first', () => {
    const runs = groupGitRuns([
      step({ runId: 'older' }),
      step({ runId: 'newer' }),
    ]);
    expect(runs.map((run) => run.runId)).toEqual(['newer', 'older']);
  });

  it('derives a shop label from a path', () => {
    expect(shopName('/dev/shops/shop-a')).toBe('shop-a');
  });
});
```

**Verification**: `pnpm typecheck && pnpm test`. Manual check: run the app, click
**Save to Git**, open "…" → **What Git did**, expand the card, confirm gray `#`
comments, `git -C …` commands, alias hints, and the "not needed this time" marker
after a clean "Get latest".

---

## 10. Phase 5 — Documentation and housekeeping

### 10.1 [MODIFY] `README.md`

- Features: add "**What Git did** drawer that explains each multi-step action as a
  commented shell transcript, with oh-my-zsh alias hints", and note that actions
  are declared in `src-tauri/macros/git-actions.yaml`.
- Structure tree: add `macros.rs` / `git_log.rs` / `events.rs` and the two YAML
  documents.
- Add a short section after "Everyday workflow":

```markdown
### What Git did

Open the "…" menu in a group toolbar and choose **What Git did** to see the exact
commands behind the last actions. Each action is one card; expand it to read the
plain-language `#` comments and the real `git -C <project> …` commands, including
the oh-my-zsh alias you could have typed instead. Nothing is stored: the list is
kept for the current session only.

The commands themselves are declared in `src-tauri/macros/git-actions.yaml`, so
the explanation and the behaviour cannot drift apart.
```

### 10.2 [MODIFY] `CHANGELOG.md`

Under `## [Unreleased] → ### Added`:

```markdown
- "What Git did" drawer: a session-only, plain-language transcript of the commands
  behind each Git action, with gray `#` step comments, `git -C` rendering,
  skipped-step markers, and oh-my-zsh alias hints.
- Git actions are declared once in `src-tauri/macros/git-actions.yaml` and executed
  from that template, so the log wording and the behaviour share one source.
```

Under `### Changed`:

```markdown
- The application event bus now carries typed `AppEvent` values; repository-change
  notifications and Git step events share one SSE endpoint (`/api/events`) and one
  frontend subscription.
```

### 10.3 `.gitignore`

No change required: `src-tauri/macros/` is committed source, and no new build
artifacts are produced.

---

## 11. Phase 6 — Implementation report

Write `_ai/backlog/reports/261030_2244__IMPLEMENTATION_REPORT__what-git-did-log-view.md`
with the required frontmatter (`planFile`, `specFile: ""`, `project: "repo-manager"`,
`status`, file counts, tags) and the sections: Summary, Files Changed, Key Changes,
Deviations from Plan, Technical Decisions, Testing Notes, Usage Examples,
Documentation Updates, Next Steps.

Record there, at minimum:

- Whether Phase 2 had to be combined with Phase 3 for the command layer to compile.
- The exact alias table size and which aliases were dropped and why.
- Any behaviour difference found while deleting `pull_with_autostash` /
  `push_branch` (for example stash-pop failure wording).
- The measured step-event volume for a 6-shop group action, to confirm the
  200-entry buffer is adequate.

Report `status: partial` if any of the following is skipped, and name it in
Next Steps: raw-stderr disclosure in the drawer, `finishedAt` timeline, alias hints
outside the expanded transcript.

---

## 12. Rollout and verification checklist

| Step | Command / action | Expected |
|------|------------------|----------|
| 1 | `cargo fmt --check` | clean |
| 2 | `cargo clippy --all-targets -- -D warnings` | clean |
| 3 | `cargo test` | all tests pass, including the five pre-existing `git_runner` tests |
| 4 | `pnpm typecheck` | clean |
| 5 | `pnpm test` | includes new `gitLog` tests |
| 6 | Desktop run: save / get latest / send / set aside / bring back / switch branch | drawer opens, one card per action, statuses correct |
| 7 | `pnpm build && git-repo-manager --web` | same cards over SSE; `repo-fs-changed` still updates the table |
| 8 | Break a YAML line locally, run `cargo test` | template validation fails with a readable message |
| 9 | Delete a local branch's upstream, run **Send changes** | second `push -u` step runs, first is "not needed this time" |

## 13. Explicitly out of scope (v1)

- Streaming background inspection commands; the drawer shows only user actions.
- Persisting the log across restarts.
- A raw-stderr disclosure and a copy-to-clipboard button on commands.
- Read-only "learning" macros (log, diff, show, reflog) — deferred until the log
  itself proves useful to the user.
- Alias hints outside the expanded transcript.
