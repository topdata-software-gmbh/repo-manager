use crate::config::ConfigManager;
use crate::git_runner::GitRunner;
use crate::models::{
    AppConfig, BranchInfo, FileChange, OperationResult, RepoGroup, Repository, ScanPath,
};
use crate::scanner::RepoScanner;
use crate::watcher::WatcherState;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Semaphore;

const INSPECTION_CONCURRENCY: usize = 8;

pub fn get_config(config: &ConfigManager) -> Result<AppConfig, String> {
    config.snapshot()
}

pub fn add_scan_path(config: &ConfigManager, mut scan_path: ScanPath) -> Result<AppConfig, String> {
    let path = expand_home(&scan_path.path);
    if !PathBuf::from(&path).is_dir() {
        return Err(format!(
            "That folder does not exist or cannot be opened: {path}. Check the path and try again."
        ));
    }
    scan_path.path = path;

    {
        let mut cfg = config.config.lock().map_err(|e| e.to_string())?;
        cfg.scan_paths
            .retain(|sp| sp.id != scan_path.id && sp.path != scan_path.path);
        cfg.scan_paths.push(scan_path);
    }
    config.save()?;
    config.snapshot()
}

fn expand_home(raw: &str) -> String {
    let trimmed = raw.trim();
    let home = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE"));

    if trimmed == "~" {
        return home
            .map(|home| PathBuf::from(home).to_string_lossy().to_string())
            .unwrap_or_else(|| trimmed.to_string());
    }

    if let (Some(rest), Some(home)) = (trimmed.strip_prefix("~/"), home) {
        return PathBuf::from(home).join(rest).to_string_lossy().to_string();
    }

    trimmed.to_string()
}

pub fn remove_scan_path(config: &ConfigManager, id: String) -> Result<AppConfig, String> {
    {
        let mut cfg = config.config.lock().map_err(|e| e.to_string())?;
        cfg.scan_paths.retain(|sp| sp.id != id);
    }
    config.save()?;
    config.snapshot()
}

pub fn save_repo_group(config: &ConfigManager, group: RepoGroup) -> Result<AppConfig, String> {
    {
        let mut cfg = config.config.lock().map_err(|e| e.to_string())?;
        cfg.groups.retain(|g| g.id != group.id);
        cfg.groups.push(group);
    }
    config.save()?;
    config.snapshot()
}

pub fn delete_repo_group(config: &ConfigManager, group_id: String) -> Result<AppConfig, String> {
    {
        let mut cfg = config.config.lock().map_err(|e| e.to_string())?;
        cfg.groups.retain(|g| g.id != group_id);
    }
    config.save()?;
    config.snapshot()
}

pub async fn discover_repositories(
    config: &ConfigManager,
    watcher: &WatcherState,
    paths: Vec<String>,
    max_depth: usize,
) -> Result<Vec<Repository>, String> {
    let group_map = config.group_ids_by_path()?;
    let limit = max_depth.max(1);
    let mut repo_paths = Vec::new();

    for path in paths.into_iter().filter(|path| !path.trim().is_empty()) {
        let found = RepoScanner::scan_path_async(PathBuf::from(&path), limit).await;
        repo_paths.extend(found);
    }

    repo_paths.sort();
    repo_paths.dedup();

    let semaphore = Arc::new(Semaphore::new(INSPECTION_CONCURRENCY));
    let mut tasks = Vec::with_capacity(repo_paths.len());

    for repo_path in repo_paths.iter().cloned() {
        let semaphore = Arc::clone(&semaphore);
        let group_ids = group_map
            .get(&repo_path.to_string_lossy().to_string())
            .cloned()
            .unwrap_or_default();

        tasks.push(tokio::spawn(async move {
            let _permit = semaphore.acquire_owned().await.ok();
            RepoScanner::inspect_repository(&repo_path, group_ids).await
        }));
    }

    let mut discovered = Vec::with_capacity(repo_paths.len());
    for task in tasks {
        if let Ok(repo) = task.await {
            discovered.push(repo);
        }
    }
    discovered.sort_by(|a, b| a.path.to_lowercase().cmp(&b.path.to_lowercase()));

    watcher.update(repo_paths);

    Ok(discovered)
}

pub async fn refresh_repository(path: String, group_ids: Vec<String>) -> Repository {
    RepoScanner::inspect_repository(&PathBuf::from(path), group_ids).await
}

pub async fn list_repo_changes(path: String) -> Vec<FileChange> {
    GitRunner::list_changes(&PathBuf::from(path)).await
}

pub async fn commit_repo_changes(path: String, message: String) -> OperationResult {
    GitRunner::commit_changes(&PathBuf::from(path), &message).await
}

pub async fn execute_group_action(
    repo_paths: Vec<String>,
    action: String,
    concurrency_limit: usize,
) -> Result<Vec<OperationResult>, String> {
    if !matches!(
        action.as_str(),
        "fetch" | "pull" | "push" | "stash" | "stash_pop"
    ) {
        return Err(format!("Unsupported group action: {}", action));
    }

    let limit = concurrency_limit.clamp(1, 32);
    let semaphore = Arc::new(Semaphore::new(limit));
    let mut tasks = Vec::with_capacity(repo_paths.len());

    for path in repo_paths {
        let semaphore = Arc::clone(&semaphore);
        let action = action.clone();
        tasks.push(tokio::spawn(async move {
            let _permit = semaphore.acquire_owned().await.ok();
            GitRunner::execute_action(&PathBuf::from(path), &action).await
        }));
    }

    let mut results = Vec::with_capacity(tasks.len());
    for task in tasks {
        if let Ok(result) = task.await {
            results.push(result);
        }
    }
    Ok(results)
}

pub async fn switch_group_branch(
    repo_paths: Vec<String>,
    branch: String,
    auto_stash: bool,
) -> Result<Vec<OperationResult>, String> {
    let semaphore = Arc::new(Semaphore::new(6));
    let mut tasks = Vec::with_capacity(repo_paths.len());

    for path in repo_paths {
        let semaphore = Arc::clone(&semaphore);
        let branch = branch.clone();
        tasks.push(tokio::spawn(async move {
            let _permit = semaphore.acquire_owned().await.ok();
            GitRunner::checkout_branch(&PathBuf::from(path), &branch, auto_stash).await
        }));
    }

    let mut results = Vec::with_capacity(tasks.len());
    for task in tasks {
        if let Ok(result) = task.await {
            results.push(result);
        }
    }
    Ok(results)
}

pub async fn list_group_branches(repo_paths: Vec<String>) -> Result<Vec<BranchInfo>, String> {
    let semaphore = Arc::new(Semaphore::new(INSPECTION_CONCURRENCY));
    let mut tasks = Vec::with_capacity(repo_paths.len());

    for path in repo_paths {
        let semaphore = Arc::clone(&semaphore);
        tasks.push(tokio::spawn(async move {
            let _permit = semaphore.acquire_owned().await.ok();
            GitRunner::list_local_branches(&PathBuf::from(path)).await
        }));
    }

    let mut counts: HashMap<String, usize> = HashMap::new();
    for task in tasks {
        if let Ok(branches) = task.await {
            for branch in branches {
                *counts.entry(branch).or_insert(0) += 1;
            }
        }
    }

    let mut branches: Vec<BranchInfo> = counts
        .into_iter()
        .map(|(name, repo_count)| BranchInfo { name, repo_count })
        .collect();
    branches.sort_by(|a, b| {
        b.repo_count
            .cmp(&a.repo_count)
            .then_with(|| a.name.cmp(&b.name))
    });

    Ok(branches)
}
