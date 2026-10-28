use crate::models::{FileChange, GitRepoState, OperationResult, RemoteInfo};
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

    pub async fn list_changes(repo_path: &Path) -> Vec<FileChange> {
        match Self::run_git_cmd(
            repo_path,
            &["status", "--porcelain=v1", "--untracked-files=all"],
        )
        .await
        {
            Ok(output) if output.status.success() => String::from_utf8_lossy(&output.stdout)
                .lines()
                .filter_map(parse_file_change)
                .collect(),
            _ => Vec::new(),
        }
    }

    pub async fn commit_changes(repo_path: &Path, message: &str) -> OperationResult {
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
        if changes.iter().any(|change| change.status == "conflict") {
            return failure_result(
                repo_path,
                "Some files have conflicts. Open the project in your editor, fix the marked \
                 files, then save to Git again.",
                String::new(),
            );
        }

        match Self::run_git_cmd(repo_path, &["add", "-A"]).await {
            Ok(output) if output.status.success() => {}
            Ok(output) => {
                return failure_result(
                    repo_path,
                    friendly_error(&error_text(&output)),
                    output_text(&output),
                )
            }
            Err(error) => return failure_result(repo_path, error, String::new()),
        }

        Self::run_operation(repo_path, &["commit", "-m", message]).await
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
            match Self::stash_if_needed(repo_path).await {
                Ok(created) => stash_created = created,
                Err(message) => {
                    return failure_result(repo_path, message, String::new());
                }
            }
        }

        let mut result = Self::run_operation(repo_path, &["checkout", branch]).await;

        if stash_created {
            let restored = Self::restore_stash(repo_path).await;

            if result.success {
                if restored {
                    result
                        .output
                        .push_str("\nYour changes were put back in the project folder.");
                } else {
                    result.success = false;
                    result.error = Some(
                        "The branch was switched, but your changes could not be put back \
                         automatically. They are safe in the stash; use 'Bring back set-aside \
                         changes' to restore them."
                            .to_string(),
                    );
                }
            } else if let Some(error) = result.error.as_mut() {
                if restored {
                    error.push_str("\nYour changes were put back in the project folder.");
                } else {
                    error.push_str(
                        "\nYour changes are still saved in the stash. Use 'Bring back \
                         set-aside changes' to restore them.",
                    );
                }
            }
        }

        result
    }

    async fn stash_if_needed(repo_path: &Path) -> Result<bool, String> {
        let status = Self::run_git_cmd(repo_path, &["status", "--porcelain=v1"]).await?;
        if !status.status.success() {
            return Err(friendly_error(&error_text(&status)));
        }
        if String::from_utf8_lossy(&status.stdout).trim().is_empty() {
            return Ok(false);
        }

        let stash =
            Self::run_git_cmd(repo_path, &["stash", "push", "-u", "-m", AUTOSTASH_MESSAGE]).await?;
        if !stash.status.success() {
            return Err(friendly_error(&error_text(&stash)));
        }

        let text = output_text(&stash);
        Ok(text.contains("Saved working directory") || text.contains("WIP on"))
    }

    async fn restore_stash(repo_path: &Path) -> bool {
        matches!(
            Self::run_git_cmd(repo_path, &["stash", "pop"]).await,
            Ok(output) if output.status.success()
        )
    }

    async fn run_operation(repo_path: &Path, args: &[&str]) -> OperationResult {
        match Self::run_git_cmd(repo_path, args).await {
            Ok(output) if output.status.success() => OperationResult {
                repo_path: repo_path.to_string_lossy().to_string(),
                success: true,
                output: output_text(&output),
                error: None,
            },
            Ok(output) => failure_result(
                repo_path,
                friendly_error(&error_text(&output)),
                output_text(&output),
            ),
            Err(error) => failure_result(repo_path, error, String::new()),
        }
    }

    async fn pull_with_autostash(repo_path: &Path) -> OperationResult {
        let stash_created = match Self::stash_if_needed(repo_path).await {
            Ok(created) => created,
            Err(message) => return failure_result(repo_path, message, String::new()),
        };

        let mut result = Self::run_operation(repo_path, &["pull", "--ff-only"]).await;

        if stash_created {
            let restored = Self::restore_stash(repo_path).await;

            if result.success {
                if restored {
                    result
                        .output
                        .push_str("\nYour changes were put back in the project folder.");
                } else {
                    result.success = false;
                    result.error = Some(
                        "The latest changes arrived, but your local changes could not be put \
                         back automatically. They are safe in the stash; use 'Bring back \
                         set-aside changes' to restore them."
                            .to_string(),
                    );
                }
            } else if let Some(error) = result.error.as_mut() {
                if restored {
                    error.push_str("\nYour changes were put back in the project folder.");
                } else {
                    error.push_str(
                        "\nYour changes are still saved in the stash. Use 'Bring back \
                         set-aside changes' to restore them.",
                    );
                }
            }
        }

        result
    }

    async fn push_branch(repo_path: &Path) -> OperationResult {
        let branch = Self::get_current_branch(repo_path).await;
        if branch == "HEAD" || branch.is_empty() {
            return failure_result(
                repo_path,
                "You are not on a branch. Switch to a branch before sending changes.",
                String::new(),
            );
        }

        let has_upstream = Self::run_git_stdout(
            repo_path,
            &[
                "rev-parse",
                "--abbrev-ref",
                "--symbolic-full-name",
                "@{upstream}",
            ],
        )
        .await
        .is_some();

        if has_upstream {
            Self::run_operation(repo_path, &["push"]).await
        } else {
            Self::run_operation(repo_path, &["push", "-u", "origin", &branch]).await
        }
    }

    pub async fn execute_action(repo_path: &Path, action: &str) -> OperationResult {
        match action {
            "fetch" => Self::run_operation(repo_path, &["fetch", "--all", "--prune"]).await,
            "pull" => Self::pull_with_autostash(repo_path).await,
            "push" => Self::push_branch(repo_path).await,
            "stash" => Self::run_operation(repo_path, &["stash", "push", "-u"]).await,
            "stash_pop" => Self::run_operation(repo_path, &["stash", "pop"]).await,
            _ => failure_result(
                repo_path,
                format!("Unknown action: {action}"),
                String::new(),
            ),
        }
    }
}

fn parse_file_change(line: &str) -> Option<FileChange> {
    if line.len() < 4 {
        return None;
    }

    let code = &line[..2];
    let raw_path = line[3..].trim();
    let path = raw_path
        .rsplit(" -> ")
        .next()
        .unwrap_or(raw_path)
        .trim_matches('"')
        .to_string();

    if path.is_empty() {
        return None;
    }

    let status = if code.contains('U') || code == "AA" || code == "DD" {
        "conflict"
    } else if code == "??" || code.contains('A') {
        "new"
    } else if code.contains('R') {
        "renamed"
    } else if code.contains('D') {
        "deleted"
    } else {
        "modified"
    };

    Some(FileChange {
        path,
        status: status.to_string(),
    })
}

fn output_text(output: &Output) -> String {
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

    match (stdout.is_empty(), stderr.is_empty()) {
        (false, false) => format!("{stdout}\n{stderr}"),
        (false, true) => stdout,
        (true, false) => stderr,
        (true, true) => String::new(),
    }
}

fn error_text(output: &Output) -> String {
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if stderr.is_empty() {
        output_text(output)
    } else {
        stderr
    }
}

fn failure_result(repo_path: &Path, error: impl Into<String>, output: String) -> OperationResult {
    OperationResult {
        repo_path: repo_path.to_string_lossy().to_string(),
        success: false,
        output,
        error: Some(error.into()),
    }
}

fn friendly_error(raw: &str) -> String {
    let lower = raw.to_lowercase();

    let hint = if lower.contains("permission denied (publickey)")
        || lower.contains("authentication failed")
        || lower.contains("could not read username")
        || lower.contains("terminal prompts disabled")
    {
        Some("Git could not sign in to the server. Check your SSH key or Git credentials.")
    } else if lower.contains("non-fast-forward")
        || lower.contains("fetch first")
        || lower.contains("updates were rejected")
    {
        Some("There are new changes on the server. Click 'Get latest' first, then send your changes again.")
    } else if lower.contains("would be overwritten")
        || lower.contains("please commit your changes or stash")
    {
        Some("You have changes that are not saved to Git. Save them first, then try again.")
    } else if lower.contains("automatic merge failed")
        || lower.contains("fix conflicts")
        || lower.contains("unmerged")
    {
        Some("Some files have conflicts. Open the project in your editor, fix the marked files, then save to Git.")
    } else if lower.contains("unable to auto-detect email")
        || lower.contains("please tell me who you are")
    {
        Some("Git does not know your name and email yet. Set them in your Git settings, then try again.")
    } else if lower.contains("nothing to commit") || lower.contains("no changes added to commit") {
        Some("There are no changes to save.")
    } else if lower.contains("not a git repository") {
        Some("This folder is no longer a Git project.")
    } else if lower.contains("does not appear to be a git repository")
        || lower.contains("no such remote")
        || lower.contains("no configured push destination")
    {
        Some("This project is not connected to a remote server yet.")
    } else {
        None
    };

    match hint {
        Some(hint) => format!("{hint}\n\nTechnical details: {}", raw.trim()),
        None => raw.trim().to_string(),
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

    #[tokio::test]
    async fn lists_and_commits_changes() {
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        init_repository(&repo);

        fs::write(repo.join("tracked.txt"), "changed\n").unwrap();
        fs::write(repo.join("new.txt"), "new\n").unwrap();

        let changes = GitRunner::list_changes(&repo).await;
        assert!(changes
            .iter()
            .any(|change| change.path == "tracked.txt" && change.status == "modified"));
        assert!(changes
            .iter()
            .any(|change| change.path == "new.txt" && change.status == "new"));

        let result = GitRunner::commit_changes(&repo, "update theme header").await;
        assert!(result.success, "{:?}", result.error);

        let state = GitRunner::get_repo_state(&repo).await;
        assert!(state.is_clean);
        assert_eq!(state.head_commit_message, "update theme header");

        let empty = GitRunner::commit_changes(&repo, "nothing").await;
        assert!(!empty.success);
        assert!(empty.error.unwrap().contains("no changes"));
    }

    #[tokio::test]
    async fn refuses_to_commit_unresolved_conflicts() {
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        init_repository(&repo);

        run_git(&repo, &["checkout", "-b", "feature"]);
        fs::write(repo.join("tracked.txt"), "feature\n").unwrap();
        run_git(&repo, &["add", "."]);
        run_git(
            &repo,
            &["-c", "commit.gpgsign=false", "commit", "-m", "feature"],
        );

        run_git(&repo, &["checkout", "main"]);
        fs::write(repo.join("tracked.txt"), "main\n").unwrap();
        run_git(&repo, &["add", "."]);
        run_git(
            &repo,
            &["-c", "commit.gpgsign=false", "commit", "-m", "main"],
        );

        let merge = StdCommand::new("git")
            .current_dir(&repo)
            .args(["merge", "feature"])
            .status()
            .unwrap();
        assert!(!merge.success());

        let changes = GitRunner::list_changes(&repo).await;
        assert!(changes.iter().any(|change| change.status == "conflict"));

        let result = GitRunner::commit_changes(&repo, "merge feature").await;
        assert!(!result.success);
        assert!(result.error.unwrap().contains("conflicts"));
    }

    #[test]
    fn explains_common_git_errors() {
        let rejected = friendly_error("! [rejected] main -> main (non-fast-forward)");
        assert!(rejected.contains("Get latest"));

        let auth = friendly_error("fatal: Authentication failed for 'https://example.com'");
        assert!(auth.contains("sign in"));
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
