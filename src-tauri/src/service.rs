use crate::config::ConfigManager;
use crate::git_log::{GitObserver, GitStepEvent};
use crate::git_runner::GitRunner;
use crate::models::{
    AppConfig, BranchInfo, FileChange, OnboardingState, OperationResult, RepoGroup, Repository,
    ScanPath,
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

pub fn save_onboarding_state(
    config: &ConfigManager,
    mut onboarding: OnboardingState,
) -> Result<AppConfig, String> {
    if onboarding.current_step > 3 {
        return Err("Onboarding step must be between 0 and 3.".to_string());
    }

    onboarding.owner_names = onboarding
        .owner_names
        .into_iter()
        .map(|owner| owner.trim().trim_start_matches('@').to_lowercase())
        .filter(|owner| !owner.is_empty())
        .collect();
    onboarding.owner_names.sort();
    onboarding.owner_names.dedup();

    if let Some(path) = onboarding.scan_path.as_mut() {
        *path = expand_home(path);
    }

    {
        let mut cfg = config.config.lock().map_err(|error| error.to_string())?;
        cfg.onboarding = onboarding;
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
    let watch_paths = repo_paths.clone();

    for repo_path in repo_paths {
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

    let mut discovered = Vec::with_capacity(tasks.len());
    for task in tasks {
        if let Ok(repo) = task.await {
            discovered.push(repo);
        }
    }
    discovered.sort_by_key(|a| a.path.to_lowercase());

    watcher.update(watch_paths);

    Ok(discovered)
}

pub async fn refresh_repository(path: String, group_ids: Vec<String>) -> Repository {
    RepoScanner::inspect_repository(&PathBuf::from(path), group_ids).await
}

pub async fn list_repo_changes(path: String) -> Vec<FileChange> {
    GitRunner::list_changes(&PathBuf::from(path)).await
}

pub async fn commit_repo_changes(
    path: String,
    message: String,
    observer: &dyn GitObserver,
) -> OperationResult {
    GitRunner::commit_changes(&PathBuf::from(path), &message, observer).await
}

/// Rewrites every step of one repository into the run shared by a group action,
/// so identical steps across clones fold into one card in the log.
struct RunScopedObserver {
    inner: Arc<dyn GitObserver>,
    run_id: String,
}

impl GitObserver for RunScopedObserver {
    fn on_step(&self, event: &GitStepEvent) {
        let mut scoped = event.clone();
        scoped.run_id = self.run_id.clone();
        self.inner.on_step(&scoped);
    }
}

pub async fn execute_group_action(
    repo_paths: Vec<String>,
    action: String,
    concurrency_limit: usize,
    observer: Arc<dyn GitObserver>,
) -> Result<Vec<OperationResult>, String> {
    if !matches!(
        action.as_str(),
        "fetch" | "pull" | "push" | "stash" | "stash_pop"
    ) {
        return Err(format!("Unsupported group action: {}", action));
    }

    let limit = concurrency_limit.clamp(1, 32);
    let semaphore = Arc::new(Semaphore::new(limit));
    let run_id = uuid::Uuid::new_v4().to_string();
    let mut tasks = Vec::with_capacity(repo_paths.len());

    for path in repo_paths {
        let semaphore = Arc::clone(&semaphore);
        let action = action.clone();
        let observer = Arc::clone(&observer);
        let run_id = run_id.clone();
        tasks.push(tokio::spawn(async move {
            let _permit = semaphore.acquire_owned().await.ok();
            let scoped = RunScopedObserver {
                inner: observer,
                run_id,
            };
            GitRunner::execute_action(&PathBuf::from(path), &action, &scoped).await
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
    observer: Arc<dyn GitObserver>,
) -> Result<Vec<OperationResult>, String> {
    let semaphore = Arc::new(Semaphore::new(6));
    let run_id = uuid::Uuid::new_v4().to_string();
    let mut tasks = Vec::with_capacity(repo_paths.len());

    for path in repo_paths {
        let semaphore = Arc::clone(&semaphore);
        let branch = branch.clone();
        let observer = Arc::clone(&observer);
        let run_id = run_id.clone();
        tasks.push(tokio::spawn(async move {
            let _permit = semaphore.acquire_owned().await.ok();
            let scoped = RunScopedObserver {
                inner: observer,
                run_id,
            };
            GitRunner::checkout_branch(&PathBuf::from(path), &branch, auto_stash, &scoped).await
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

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_manager() -> (tempfile::TempDir, ConfigManager) {
        let temp = tempfile::tempdir().unwrap();
        let manager = ConfigManager::from_path(temp.path().join("config.json"));
        (temp, manager)
    }

    #[test]
    fn normalizes_and_persists_onboarding_state() {
        let (_temp, manager) = temp_manager();

        let saved = save_onboarding_state(
            &manager,
            OnboardingState {
                started: true,
                current_step: 2,
                scan_path: Some("/tmp/projects".to_string()),
                owner_names: vec![" @Acme ".to_string(), "acme".to_string(), String::new()],
                selected_repo_paths: vec!["/tmp/projects/theme".to_string()],
                ..OnboardingState::default()
            },
        )
        .unwrap();

        assert_eq!(saved.onboarding.owner_names, vec!["acme".to_string()]);
        assert_eq!(saved.onboarding.current_step, 2);
        assert!(saved.onboarding.started);
        assert_eq!(manager.snapshot().unwrap().onboarding.current_step, 2);
    }

    #[test]
    fn rejects_out_of_range_onboarding_step() {
        let (_temp, manager) = temp_manager();
        let state = OnboardingState {
            current_step: 4,
            ..OnboardingState::default()
        };

        assert!(save_onboarding_state(&manager, state).is_err());
    }
}
