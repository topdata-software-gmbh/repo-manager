use crate::git_runner::GitRunner;
use crate::models::{GitRepoState, RepoKind, Repository};
use ignore::WalkBuilder;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

const EXCLUDED_DIRS: &[&str] = &[
    "node_modules",
    "vendor",
    "target",
    "dist",
    "build",
    ".cache",
    ".venv",
    "venv",
    ".tox",
    "__pycache__",
    ".next",
    ".nuxt",
    ".idea",
    ".vscode",
];

pub struct RepoScanner;

impl RepoScanner {
    pub fn scan_path(root: &Path, max_depth: usize) -> Vec<PathBuf> {
        if !root.is_dir() {
            return Vec::new();
        }

        let found = Arc::new(Mutex::new(Vec::<PathBuf>::new()));
        let sink = Arc::clone(&found);

        let mut builder = WalkBuilder::new(root);
        builder
            .max_depth(Some(max_depth.max(1)))
            .hidden(false)
            .git_ignore(true)
            .git_global(true)
            .git_exclude(true);

        builder.filter_entry(move |entry| {
            let name = entry.file_name().to_string_lossy();

            // A `.git` directory (or gitfile used by worktrees/submodules) marks a
            // repository root. Record the parent and do not descend into metadata.
            if name == ".git" {
                if let Some(parent) = entry.path().parent() {
                    sink.lock().unwrap().push(parent.to_path_buf());
                }
                return false;
            }

            if entry.file_type().map_or(false, |ft| ft.is_dir())
                && EXCLUDED_DIRS
                    .iter()
                    .any(|excluded| *excluded == name.as_ref())
            {
                return false;
            }

            true
        });

        for _ in builder.build() {}

        let mut repos = found.lock().map(|repos| repos.clone()).unwrap_or_default();
        repos.sort();
        repos.dedup();
        repos
    }

    pub async fn scan_path_async(root: PathBuf, max_depth: usize) -> Vec<PathBuf> {
        tokio::task::spawn_blocking(move || Self::scan_path(&root, max_depth))
            .await
            .unwrap_or_default()
    }

    pub async fn inspect_repository(repo_path: &Path, group_ids: Vec<String>) -> Repository {
        let path_str = repo_path.to_string_lossy().to_string();
        let name = repo_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path_str.clone());
        let id = repository_id(&path_str);

        if !repo_path.is_dir() {
            return Repository {
                id,
                name,
                path: path_str,
                is_accessible: false,
                state: GitRepoState::default(),
                group_ids,
                kind: RepoKind::Other,
            };
        }

        let state = GitRunner::get_repo_state(repo_path).await;
        let kind = detect_repository_kind(repo_path);

        Repository {
            id,
            name,
            path: path_str,
            is_accessible: true,
            state,
            group_ids,
            kind,
        }
    }
}

fn detect_repository_kind(repo_path: &Path) -> RepoKind {
    if repo_path.join("src/Resources/theme.json").is_file() {
        return RepoKind::Theme;
    }

    let composer_path = repo_path.join("composer.json");
    if let Ok(raw) = std::fs::read_to_string(composer_path) {
        if let Ok(composer) = serde_json::from_str::<serde_json::Value>(&raw) {
            if composer.get("type").and_then(|value| value.as_str())
                == Some("shopware-platform-plugin")
            {
                return RepoKind::Plugin;
            }
        }
    }

    RepoKind::Other
}

fn repository_id(path: &str) -> String {
    // Deterministic IDs keep Vue rows (and selection state) stable across refreshes.
    uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_OID, path.as_bytes()).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn scans_repositories_and_skips_dependency_directories() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();

        let service_a = root.join("service-a");
        fs::create_dir_all(service_a.join(".git")).unwrap();

        let service_b = root.join("nested/deeper/service-b");
        fs::create_dir_all(service_b.join(".git")).unwrap();

        let worktree = root.join("worktree-repo");
        fs::create_dir_all(&worktree).unwrap();
        fs::write(worktree.join(".git"), "gitdir: /tmp/fake-git-dir\n").unwrap();

        fs::create_dir_all(root.join("node_modules/dependency/.git")).unwrap();
        fs::create_dir_all(root.join("vendor/library/.git")).unwrap();
        fs::create_dir_all(root.join("a/b/c/d/e/.git")).unwrap();

        let found = RepoScanner::scan_path(root, 4);

        assert!(found.contains(&service_a));
        assert!(found.contains(&service_b));
        assert!(found.contains(&worktree));
        assert!(!found
            .iter()
            .any(|p| p.starts_with(root.join("node_modules"))));
        assert!(!found.iter().any(|p| p.starts_with(root.join("vendor"))));
        assert!(!found.contains(&root.join("a/b/c/d/e")));
    }

    #[test]
    fn detects_theme_plugin_and_other_repositories() {
        let temp = tempfile::tempdir().unwrap();

        let theme = temp.path().join("theme");
        fs::create_dir_all(theme.join("src/Resources")).unwrap();
        fs::write(theme.join("src/Resources/theme.json"), "{}").unwrap();

        let plugin = temp.path().join("plugin");
        fs::create_dir_all(&plugin).unwrap();
        fs::write(
            plugin.join("composer.json"),
            r#"{"type":"shopware-platform-plugin"}"#,
        )
        .unwrap();

        let other = temp.path().join("other");
        fs::create_dir_all(&other).unwrap();

        assert_eq!(detect_repository_kind(&theme), RepoKind::Theme);
        assert_eq!(detect_repository_kind(&plugin), RepoKind::Plugin);
        assert_eq!(detect_repository_kind(&other), RepoKind::Other);
    }
}
