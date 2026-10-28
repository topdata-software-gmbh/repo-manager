use crate::config::ConfigManager;
use crate::models::{BranchInfo, OperationResult, Repository};
use crate::service;
use crate::watcher::WatcherState;
use tauri::State;

#[tauri::command]
pub async fn discover_repositories(
    paths: Vec<String>,
    max_depth: usize,
    config_state: State<'_, ConfigManager>,
    watcher_state: State<'_, WatcherState>,
) -> Result<Vec<Repository>, String> {
    service::discover_repositories(
        config_state.inner(),
        watcher_state.inner(),
        paths,
        max_depth,
    )
    .await
}

#[tauri::command]
pub async fn refresh_repository(
    path: String,
    group_ids: Vec<String>,
) -> Result<Repository, String> {
    Ok(service::refresh_repository(path, group_ids).await)
}

#[tauri::command]
pub async fn execute_group_action(
    repo_paths: Vec<String>,
    action: String,
    concurrency_limit: usize,
) -> Result<Vec<OperationResult>, String> {
    service::execute_group_action(repo_paths, action, concurrency_limit).await
}

#[tauri::command]
pub async fn switch_group_branch(
    repo_paths: Vec<String>,
    branch: String,
    auto_stash: bool,
) -> Result<Vec<OperationResult>, String> {
    service::switch_group_branch(repo_paths, branch, auto_stash).await
}

#[tauri::command]
pub async fn list_group_branches(repo_paths: Vec<String>) -> Result<Vec<BranchInfo>, String> {
    service::list_group_branches(repo_paths).await
}
