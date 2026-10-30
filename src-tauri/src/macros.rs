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
        let mut best: Option<(
            bool,
            std::cmp::Reverse<usize>,
            std::cmp::Reverse<usize>,
            &str,
        )> = None;

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
        if self.version != 1 {
            return Err(format!(
                "git-actions.yaml: unsupported version {} (expected 1)",
                self.version
            ));
        }
        if aliases.version != 1 {
            return Err(format!(
                "oh-my-zsh-git-aliases.yaml: unsupported version {} (expected 1)",
                aliases.version
            ));
        }
        for (id, mac) in &self.macros {
            if mac.steps.is_empty() {
                return Err(format!("macro `{id}` has no steps"));
            }
            if mac.comment.trim().is_empty() {
                return Err(format!("macro `{id}` has no comment"));
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
        let args = render_args(&["commit".into(), "-m".into(), "{message}".into()], &ctx).unwrap();
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
            assert_eq!(
                aliases.hint_for(&args),
                None,
                "args {args:?} must not be hinted"
            );
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
