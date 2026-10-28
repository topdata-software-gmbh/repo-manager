use crate::models::{GitRepoState, OperationResult, RemoteInfo};
use std::path::{Path, PathBuf};
use std::process::Output;
use tokio::process::Command;

const AUTOSTASH_MESSAGE: &str = "git-repo-manager-autostash";

pub struct GitRunner;

impl GitRunner {
    async fn run_git_cmd(repo_path: &Path, args: &[&str]) -> Result<Output, String> {
        Command::new("git")
            .current_dir(repo_path)
            .args(args)
            .output()
            .await
            .map_err(|e| format!("Failed to execute git {:?}: {}", args, e))
    }

    async fn run_git_stdout(repo_path: &Path, args: &[&str]) -> Option<String> {
        match Self::run_git_cmd(repo_path, args).await {
            Ok(out) if out.status.success() => {
                Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
            }
            _ => None,
        }
    }

    /// Resolves the real git directory. Submodules and linked worktrees use a
    /// `.git` *file* pointing at a directory elsewhere.
    pub fn resolve_git_dir(repo_path: &Path) -> PathBuf {
        let dot_git = repo_path.join(".git");
        if dot_git.is_file() {
            if let Ok(contents) = std::fs::read_to_string(&dot_git) {
                if let Some(target) = contents.trim().strip_prefix("gitdir:") {
                    let target = PathBuf::from(target.trim());
                    return if target.is_absolute() {
                        target
                    } else {
                        repo_path.join(target)
                    };
                }
            }
        }
        dot_git
    }

    pub async fn get_repo_state(repo_path: &Path) -> GitRepoState {
        let (raw_branch, (modified, untracked, staged, conflicts), (hash, message), remotes) = tokio::join!(
            Self::get_current_branch(repo_path),
            Self::parse_status(repo_path),
            Self::get_head_commit(repo_path),
            Self::get_remotes(repo_path),
        );

        let is_detached_head = raw_branch == "HEAD" || raw_branch.is_empty();
        let current_branch = if is_detached_head {
            "HEAD (detached)".to_string()
        } else {
            raw_branch
        };
        let (ahead, behind) = if is_detached_head {
            (0, 0)
        } else {
            Self::get_ahead_behind(repo_path, &current_branch).await
        };

        GitRepoState {
            current_branch,
            is_detached_head,
            head_commit_hash: hash,
            head_commit_message: message,
            is_clean: modified == 0 && untracked == 0 && staged == 0 && !conflicts,
            ahead_count: ahead,
            behind_count: behind,
            modified_files: modified,
            untracked_files: untracked,
            staged_files: staged,
            has_conflicts: conflicts,
            operation_state: Self::detect_operation_state(repo_path),
            remotes,
        }
    }

    async fn get_current_branch(path: &Path) -> String {
        if let Some(name) = Self::run_git_stdout(path, &["rev-parse", "--abbrev-ref", "HEAD"]).await
        {
            if !name.is_empty() {
                return name;
            }
        }
        "HEAD".to_string()
    }

    async fn get_ahead_behind(path: &Path, branch: &str) -> (usize, usize) {
        let upstream = Self::run_git_stdout(
            path,
            &[
                "rev-parse",
                "--abbrev-ref",
                "--symbolic-full-name",
                "@{upstream}",
            ],
        )
        .await
        .filter(|upstream| !upstream.is_empty())
        .unwrap_or_else(|| format!("origin/{}", branch));

        let range = format!("{}...HEAD", upstream);
        if let Some(output) =
            Self::run_git_stdout(path, &["rev-list", "--left-right", "--count", &range]).await
        {
            let parts: Vec<&str> = output.split_whitespace().collect();
            if parts.len() == 2 {
                let behind = parts[0].parse::<usize>().unwrap_or(0);
                let ahead = parts[1].parse::<usize>().unwrap_or(0);
                return (ahead, behind);
            }
        }
        (0, 0)
    }

    async fn parse_status(path: &Path) -> (usize, usize, usize, bool) {
        let mut modified = 0;
        let mut untracked = 0;
        let mut staged = 0;
        let mut has_conflicts = false;

        if let Ok(out) = Self::run_git_cmd(path, &["status", "--porcelain=v1"]).await {
            let stdout = String::from_utf8_lossy(&out.stdout);
            for line in stdout.lines() {
                if line.len() < 2 {
                    continue;
                }
                let bytes = line.as_bytes();
                let x = bytes[0] as char;
                let y = bytes[1] as char;

                if x == 'U' || y == 'U' || (x == 'A' && y == 'A') || (x == 'D' && y == 'D') {
                    has_conflicts = true;
                }
                if x == '?' && y == '?' {
                    untracked += 1;
                } else {
                    if x != ' ' && x != '?' {
                        staged += 1;
                    }
                    if y != ' ' && y != '?' {
                        modified += 1;
                    }
                }
            }
        }
        (modified, untracked, staged, has_conflicts)
    }

    async fn get_head_commit(path: &Path) -> (String, String) {
        if let Some(output) = Self::run_git_stdout(path, &["log", "-1", "--format=%h%x09%s"]).await
        {
            let mut parts = output.splitn(2, '\t');
            let hash = parts.next().unwrap_or("").to_string();
            let message = parts.next().unwrap_or("").to_string();
            return (hash, message);
        }
        (String::new(), String::new())
    }

    async fn get_remotes(path: &Path) -> Vec<RemoteInfo> {
        let mut list = Vec::new();
        if let Some(output) = Self::run_git_stdout(path, &["remote", "-v"]).await {
            for line in output.lines() {
                if line.ends_with("(fetch)") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        list.push(RemoteInfo {
                            name: parts[0].to_string(),
                            url: parts[1].to_string(),
                        });
                    }
                }
            }
        }
        list
    }

    fn detect_operation_state(path: &Path) -> String {
        let git_dir = Self::resolve_git_dir(path);
        if git_dir.join("MERGE_HEAD").exists() {
            "merging".to_string()
        } else if git_dir.join("rebase-apply").exists() || git_dir.join("rebase-merge").exists() {
            "rebasing".to_string()
        } else if git_dir.join("CHERRY_PICK_HEAD").exists() {
            "cherry-picking".to_string()
        } else {
            "idle".to_string()
        }
    }

    pub async fn list_local_branches(repo_path: &Path) -> Vec<String> {
        Self::run_git_stdout(
            repo_path,
            &["for-each-ref", "--format=%(refname:short)", "refs/heads"],
        )
        .await
        .map(|output| {
            output
                .lines()
                .map(|line| line.trim().to_string())
                .filter(|line| !line.is_empty())
                .collect()
        })
        .unwrap_or_default()
    }

    pub async fn checkout_branch(
        repo_path: &Path,
        branch: &str,
        auto_stash: bool,
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

        let mut stash_created = false;

        if auto_stash {
            if let Ok(out) =
                Self::run_git_cmd(repo_path, &["stash", "push", "-m", AUTOSTASH_MESSAGE]).await
            {
                let mut text = String::from_utf8_lossy(&out.stdout).to_string();
                text.push_str(&String::from_utf8_lossy(&out.stderr));
                stash_created = text.contains("Saved working directory") || text.contains("WIP on");
            }
        }

        let mut result = match Self::run_git_cmd(repo_path, &["checkout", branch]).await {
            Ok(output) if output.status.success() => OperationResult {
                repo_path: repo_path.to_string_lossy().to_string(),
                success: true,
                output: String::from_utf8_lossy(&output.stdout).to_string(),
                error: None,
            },
            Ok(output) => OperationResult {
                repo_path: repo_path.to_string_lossy().to_string(),
                success: false,
                output: String::from_utf8_lossy(&output.stdout).to_string(),
                error: Some(String::from_utf8_lossy(&output.stderr).to_string()),
            },
            Err(e) => OperationResult {
                repo_path: repo_path.to_string_lossy().to_string(),
                success: false,
                output: String::new(),
                error: Some(e),
            },
        };

        if stash_created {
            let restore = Self::run_git_cmd(repo_path, &["stash", "pop"]).await;
            let restored = matches!(&restore, Ok(out) if out.status.success());

            if result.success {
                if restored {
                    result
                        .output
                        .push_str("\nLocal changes were restored from the autostash.");
                } else {
                    result.success = false;
                    result.error = Some(
                        "Branch switched, but restoring the autostash failed. \
                         The changes remain in `git stash list`."
                            .to_string(),
                    );
                }
            } else if let Some(error) = result.error.as_mut() {
                if restored {
                    error.push_str("\nLocal changes were restored from the autostash.");
                } else {
                    error.push_str(
                        "\nLocal changes are still saved in the autostash (`git stash list`).",
                    );
                }
            }
        }

        result
    }

    pub async fn execute_action(repo_path: &Path, action: &str) -> OperationResult {
        let args: Vec<&str> = match action {
            "fetch" => vec!["fetch", "--all", "--prune"],
            "pull" => vec!["pull", "--ff-only"],
            "push" => vec!["push"],
            "stash" => vec!["stash", "push"],
            "stash_pop" => vec!["stash", "pop"],
            _ => {
                return OperationResult {
                    repo_path: repo_path.to_string_lossy().to_string(),
                    success: false,
                    output: String::new(),
                    error: Some(format!("Unknown action: {}", action)),
                }
            }
        };

        match Self::run_git_cmd(repo_path, &args).await {
            Ok(out) if out.status.success() => OperationResult {
                repo_path: repo_path.to_string_lossy().to_string(),
                success: true,
                output: String::from_utf8_lossy(&out.stdout).to_string(),
                error: None,
            },
            Ok(out) => OperationResult {
                repo_path: repo_path.to_string_lossy().to_string(),
                success: false,
                output: String::from_utf8_lossy(&out.stdout).to_string(),
                error: Some(String::from_utf8_lossy(&out.stderr).to_string()),
            },
            Err(e) => OperationResult {
                repo_path: repo_path.to_string_lossy().to_string(),
                success: false,
                output: String::new(),
                error: Some(e),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::process::Command as StdCommand;

    fn run_git(dir: &Path, args: &[&str]) {
        let status = StdCommand::new("git")
            .current_dir(dir)
            .args(args)
            .status()
            .expect("git must be installed for tests");
        assert!(status.success(), "git {:?} failed", args);
    }

    fn init_repository(dir: &Path) {
        fs::create_dir_all(dir).unwrap();
        run_git(dir, &["init", "-b", "main"]);
        run_git(dir, &["config", "user.name", "Test User"]);
        run_git(dir, &["config", "user.email", "test@example.com"]);
        fs::write(dir.join("tracked.txt"), "initial\n").unwrap();
        run_git(dir, &["add", "."]);
        run_git(
            dir,
            &["-c", "commit.gpgsign=false", "commit", "-m", "initial"],
        );
    }

    #[tokio::test]
    async fn inspects_working_tree_branches_and_operations() {
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        init_repository(&repo);

        fs::write(repo.join("tracked.txt"), "changed\n").unwrap();
        fs::write(repo.join("new.txt"), "new\n").unwrap();

        let state = GitRunner::get_repo_state(&repo).await;
        assert_eq!(state.current_branch, "main");
        assert!(!state.is_detached_head);
        assert!(!state.is_clean);
        assert_eq!(state.modified_files, 1);
        assert_eq!(state.untracked_files, 1);
        assert_eq!(state.staged_files, 0);
        assert!(!state.head_commit_hash.is_empty());
        assert_eq!(state.head_commit_message, "initial");
        assert_eq!(state.operation_state, "idle");

        let branches = GitRunner::list_local_branches(&repo).await;
        assert_eq!(branches, vec!["main".to_string()]);

        assert_eq!(GitRunner::resolve_git_dir(&repo), repo.join(".git"));

        run_git(&repo, &["branch", "feature"]);
        let result = GitRunner::checkout_branch(&repo, "feature", true).await;
        assert!(result.success, "{:?}", result.error);

        let state = GitRunner::get_repo_state(&repo).await;
        assert_eq!(state.current_branch, "feature");
        assert_eq!(state.modified_files, 1);
        assert_eq!(state.untracked_files, 1);

        let stash_list = StdCommand::new("git")
            .current_dir(&repo)
            .args(["stash", "list"])
            .output()
            .unwrap();
        assert!(String::from_utf8_lossy(&stash_list.stdout)
            .trim()
            .is_empty());
    }

    #[test]
    fn resolves_gitfile_for_worktrees() {
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("worktree");
        fs::create_dir_all(&repo).unwrap();
        fs::write(repo.join(".git"), "gitdir: /tmp/example-git-dir\n").unwrap();
        assert_eq!(
            GitRunner::resolve_git_dir(&repo),
            PathBuf::from("/tmp/example-git-dir")
        );
    }
}
