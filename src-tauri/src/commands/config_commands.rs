use crate::config::ConfigManager;
use crate::models::{AppConfig, RepoGroup, ScanPath};
use crate::service;
use tauri::State;

#[tauri::command]
pub fn get_config(state: State<'_, ConfigManager>) -> Result<AppConfig, String> {
    service::get_config(state.inner())
}

#[tauri::command]
pub fn add_scan_path(
    scan_path: ScanPath,
    state: State<'_, ConfigManager>,
) -> Result<AppConfig, String> {
    service::add_scan_path(state.inner(), scan_path)
}

#[tauri::command]
pub fn remove_scan_path(id: String, state: State<'_, ConfigManager>) -> Result<AppConfig, String> {
    service::remove_scan_path(state.inner(), id)
}

#[tauri::command]
pub fn save_repo_group(
    group: RepoGroup,
    state: State<'_, ConfigManager>,
) -> Result<AppConfig, String> {
    service::save_repo_group(state.inner(), group)
}

#[tauri::command]
pub fn delete_repo_group(
    group_id: String,
    state: State<'_, ConfigManager>,
) -> Result<AppConfig, String> {
    service::delete_repo_group(state.inner(), group_id)
}
