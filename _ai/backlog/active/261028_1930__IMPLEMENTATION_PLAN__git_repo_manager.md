---
filename: "_ai/backlog/active/261028_1930__IMPLEMENTATION_PLAN__git_repo_manager.md"
title: "git-repo-manager: Tauri v2 + Rust + Vue 3 Multi-Git Management GUI"
createdAt: 2026-10-28 19:30
updatedAt: 2026-10-28 19:30
status: draft
priority: high
tags: [git, tauri, rust, vue3, multi-repo]
estimatedComplexity: complex
documentRevision: 1
sourceSpec: ""
sourceHandoff: ""
sourceAdrs: []
documentType: IMPLEMENTATION_PLAN
---

## 1. Problem Description

Developers managing microservices, modular monoliths, or distributed multi-repository architectures must constantly run repetitive Git operations across dozens of individual directories (e.g., checking status, synchronizing branches, fetching updates, staging, or committing). While JetBrains IDEs (like PhpStorm/IntelliJ) provide synchronized multi-root Git controls, they require heavy RAM/CPU overhead and do not decouple repository management from full-weight IDE project workspaces. Developers need a lightweight, standalone, resource-efficient desktop GUI dedicated to multi-repository discovery, grouping, status aggregation, and synchronous branch control.

## 2. Executive Summary

`git-repo-manager` is a native desktop application built with **Tauri v2**, **Rust**, and **Vue 3** (TypeScript + Tailwind CSS + Pinia). It automatically scans designated directories to discover Git repositories, organizes them into customizable logical groups (e.g., "Shop Microservices", "Core Libraries"), and provides real-time status tracking via filesystem event notifications. The application enables PhpStorm-style multi-repository workflows: group-wide synchronous branch switching, parallel fetch/pull execution bounded by a concurrency semaphore, stash aggregation, and unified diff/status health checks, all while delegating Git execution to the native Git CLI to preserve user SSH/GPG credentials and global configurations.

## 3. Design Decisions & Rejected Alternatives

| Decision | Chosen Approach | Rejected Alternative | Rationale |
| :--- | :--- | :--- | :--- |
| **GUI Framework** | Tauri v2 + Rust backend | Electron / Chromium wrapper | Tauri provides <15MB binary size, minimal RAM usage, and instant startup compared to Electron. |
| **Frontend Framework** | Vue 3 (Composition API, `<script setup>`, TS) | React / Svelte | Matches user preference; highly reactive and clean binding for high-frequency Git state tables. |
| **Git Invocation** | Pragmatic Hybrid: `git` CLI via `tokio::process` for mutations/network; native FS parsing for fast inspection | Pure `libgit2` (`git2-rs`) | `git2-rs` has major hurdles with system SSH agents, 1Password/GPG signing, and custom credential helpers. CLI inherits user environment seamlessly. |
| **Repo Discovery** | `ignore` crate (Ripgrep engine) | Standard recursive `std::fs::read_dir` | The `ignore` crate respects `.gitignore` rules and ignores `node_modules`, `vendor`, and `.cache` directories by default, cutting scan times from seconds to milliseconds. |
| **Live State Updates** | `notify` crate watching `.git/HEAD`, `.git/index`, `.git/refs` | Aggressive background polling (e.g., every 2s) | Polling introduces needless disk I/O and battery drain across dozens of repos. FS events ensure updates happen on demand. |
| **Persistence** | Single JSON configuration file (`~/.config/git-repo-manager/config.json`) | Embedded SQLite database | The source of truth for Git states is the filesystem itself; the app only persists scan paths, group definitions, and UI preferences. |
| **Task Concurrency** | `tokio::sync::Semaphore` with bounded workers (default: 6) | Unbounded concurrent async tasks | Unbounded `git fetch/pull` across 40+ repositories can exhaust system network sockets and trigger remote Git host rate limits. |

## 4. Project Environment Details

```
OS: Linux / macOS / Windows
Runtime: Node.js >= 20.x, Rust >= 1.78.0, Cargo
Package Manager: pnpm >= 9.x
GUI Framework: Tauri v2 (tauri-cli 2.x)
Frontend: Vue 3.4+, Vite 5+, TypeScript 5+, Tailwind CSS 3+, Pinia 2+, Lucide Vue Next
Git Dependency: git CLI installed and accessible via system PATH
Target Git Branch: main (direct commits; no feature branches or pull requests)
```

---

## 5. Implementation Phases

### Phase 1: Project Scaffolding & Core Types

#### Goal
Initialize the Tauri v2 + Vue 3 project structure, configure Tailwind CSS and Pinia, and establish shared TypeScript and Rust data structures for domain entities (`ScanPath`, `Repository`, `RepoGroup`, `GitStatus`).

#### 1.1 Project Structure Setup
[NEW FILE] `package.json`
```json
{
  "name": "git-repo-manager",
  "version": "0.1.0",
  "private": true,
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "vue-tsc && vite build",
    "preview": "vite preview",
    "tauri": "tauri"
  },
  "dependencies": {
    "@tauri-apps/api": "^2.0.0",
    "@tauri-apps/plugin-shell": "^2.0.0",
    "lucide-vue-next": "^0.450.0",
    "pinia": "^2.2.0",
    "vue": "^3.4.30"
  },
  "devDependencies": {
    "@tauri-apps/cli": "^2.0.0",
    "@vitejs/plugin-vue": "^5.0.5",
    "autoprefixer": "^10.4.19",
    "postcss": "^8.4.39",
    "tailwindcss": "^3.4.4",
    "typescript": "^5.5.2",
    "vite": "^5.3.1",
    "vue-tsc": "^2.0.22"
  }
}
```

[NEW FILE] `src-tauri/Cargo.toml`
```toml
[package]
name = "git-repo-manager"
version = "0.1.0"
description = "Multi-Git GUI Manager"
edition = "2021"

[build-dependencies]
tauri-build = { version = "2.0.0", features = [] }

[dependencies]
tauri = { version = "2.0.0", features = [] }
tauri-plugin-shell = "2.0.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.38", features = ["full"] }
ignore = "0.4"
notify = "6.1"
uuid = { version = "1.9", features = ["v4", "serde"] }
thiserror = "1.0"
log = "0.4"
env_logger = "0.11"
```

[NEW FILE] `src-tauri/tauri.conf.json`
```json
{
  "$schema": "https://raw.githubusercontent.com/tauri-apps/tauri/dev/tooling/cli/schema.json",
  "productName": "GitRepoManager",
  "version": "0.1.0",
  "identifier": "com.gitrepomanager.app",
  "build": {
    "beforeDevCommand": "pnpm dev",
    "devUrl": "http://localhost:5173",
    "beforeBuildCommand": "pnpm build",
    "frontendDist": "../dist"
  },
  "app": {
    "windows": [
      {
        "title": "Git Repo Manager",
        "width": 1280,
        "height": 800,
        "minWidth": 900,
        "minHeight": 600
      }
    ],
    "security": {
      "csp": null
    }
  }
}
```

#### 1.2 Domain Entities & TypeScript Models
[NEW FILE] `src/types/domain.ts`
```typescript
export interface ScanPath {
  id: string;
  path: string;
  maxDepth: number;
  enabled: boolean;
  excludePatterns: string[];
}

export interface RemoteInfo {
  name: string;
  url: string;
}

export interface GitRepoState {
  currentBranch: string;
  isDetachedHead: boolean;
  headCommitHash: string;
  headCommitMessage: string;
  isClean: boolean;
  aheadCount: number;
  behindCount: number;
  modifiedFiles: number;
  untrackedFiles: number;
  stagedFiles: number;
  hasConflicts: boolean;
  operationState: 'idle' | 'merging' | 'rebasing' | 'cherry-picking';
  remotes: RemoteInfo[];
}

export interface Repository {
  id: string;
  name: string;
  path: string;
  isAccessible: boolean;
  state: GitRepoState;
  groupIds: string[];
}

export interface RepoGroup {
  id: string;
  name: string;
  color: string;
  repoPaths: string[];
  settings: {
    syncBranching: boolean;
    autoStashOnSwitch: boolean;
    defaultBaseBranch: string;
  };
}

export interface AppConfig {
  scanPaths: ScanPath[];
  groups: RepoGroup[];
}

export interface OperationProgressEvent {
  repoPath: string;
  operation: 'fetch' | 'pull' | 'push' | 'checkout' | 'stash';
  status: 'pending' | 'running' | 'success' | 'failed';
  message?: string;
}
```

[NEW FILE] `src-tauri/src/models.rs`
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanPath {
    pub id: String,
    pub path: String,
    pub max_depth: usize,
    pub enabled: bool,
    pub exclude_patterns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteInfo {
    pub name: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GitRepoState {
    pub current_branch: String,
    pub is_detached_head: bool,
    pub head_commit_hash: String,
    pub head_commit_message: String,
    pub is_clean: bool,
    pub ahead_count: usize,
    pub behind_count: usize,
    pub modified_files: usize,
    pub untracked_files: usize,
    pub staged_files: usize,
    pub has_conflicts: bool,
    pub operation_state: String,
    pub remotes: Vec<RemoteInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    pub id: String,
    pub name: String,
    pub path: String,
    pub is_accessible: bool,
    pub state: GitRepoState,
    pub group_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupSettings {
    pub sync_branching: bool,
    pub auto_stash_on_switch: bool,
    pub default_base_branch: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoGroup {
    pub id: String,
    pub name: String,
    pub color: String,
    pub repo_paths: Vec<String>,
    pub settings: GroupSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub scan_paths: Vec<ScanPath>,
    pub groups: Vec<RepoGroup>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationResult {
    pub repo_path: String,
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
}
```

#### Verification Phase 1
- Run `pnpm install` and ensure all dependencies resolve without conflicts.
- Run `cargo check --manifest-path src-tauri/Cargo.toml` and ensure models compile cleanly.

---

### Phase 2: Configuration & Persistence Engine

#### Goal
Implement persistent configuration loading and saving to disk (`~/.config/git-repo-manager/config.json`) with safe atomic writes, and expose Tauri commands for managing scan paths and repo groups.

[NEW FILE] `src-tauri/src/config.rs`
```rust
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use crate::models::AppConfig;

pub struct ConfigManager {
    config_path: PathBuf,
    pub config: Mutex<AppConfig>,
}

impl ConfigManager {
    pub fn init() -> Self {
        let config_dir = dirs_fallback().join("git-repo-manager");
        if !config_dir.exists() {
            let _ = fs::create_dir_all(&config_dir);
        }
        let config_path = config_dir.join("config.json");
        let initial_config = if config_path.exists() {
            fs::read_to_string(&config_path)
                .ok()
                .and_then(|data| serde_json::from_str::<AppConfig>(&data).ok())
                .unwrap_or_default()
        } else {
            AppConfig::default()
        };

        Self {
            config_path,
            config: Mutex::new(initial_config),
        }
    }

    pub fn save(&self) -> Result<(), String> {
        let config = self.config.lock().map_err(|e| e.to_string())?;
        let serialized = serde_json::to_string_pretty(&*config).map_err(|e| e.to_string())?;
        
        // Write via temp file to avoid file corruption on interruption
        let temp_path = self.config_path.with_extension("tmp");
        fs::write(&temp_path, serialized).map_err(|e| e.to_string())?;
        fs::rename(&temp_path, &self.config_path).map_err(|e| e.to_string())?;
        Ok(())
    }
}

fn dirs_fallback() -> PathBuf {
    std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| {
            #[cfg(target_os = "windows")]
            { std::env::var_os("APPDATA").map(PathBuf::from) }
            #[cfg(not(target_os = "windows"))]
            { std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")) }
        })
        .unwrap_or_else(|| PathBuf::from("."))
}
```

[NEW FILE] `src-tauri/src/commands/config_commands.rs`
```rust
use tauri::State;
use crate::config::ConfigManager;
use crate::models::{AppConfig, RepoGroup, ScanPath};

#[tauri::command]
pub fn get_config(state: State<'_, ConfigManager>) -> Result<AppConfig, String> {
    let cfg = state.config.lock().map_err(|e| e.to_string())?;
    Ok(cfg.clone())
}

#[tauri::command]
pub fn add_scan_path(scan_path: ScanPath, state: State<'_, ConfigManager>) -> Result<AppConfig, String> {
    {
        let mut cfg = state.config.lock().map_err(|e| e.to_string())?;
        cfg.scan_paths.retain(|sp| sp.id != scan_path.id && sp.path != scan_path.path);
        cfg.scan_paths.push(scan_path);
    }
    state.save()?;
    get_config(state)
}

#[tauri::command]
pub fn remove_scan_path(id: String, state: State<'_, ConfigManager>) -> Result<AppConfig, String> {
    {
        let mut cfg = state.config.lock().map_err(|e| e.to_string())?;
        cfg.scan_paths.retain(|sp| sp.id != id);
    }
    state.save()?;
    get_config(state)
}

#[tauri::command]
pub fn save_repo_group(group: RepoGroup, state: State<'_, ConfigManager>) -> Result<AppConfig, String> {
    {
        let mut cfg = state.config.lock().map_err(|e| e.to_string())?;
        cfg.groups.retain(|g| g.id != group.id);
        cfg.groups.push(group);
    }
    state.save()?;
    get_config(state)
}

#[tauri::command]
pub fn delete_repo_group(group_id: String, state: State<'_, ConfigManager>) -> Result<AppConfig, String> {
    {
        let mut cfg = state.config.lock().map_err(|e| e.to_string())?;
        cfg.groups.retain(|g| g.id != group_id);
    }
    state.save()?;
    get_config(state)
}
```

#### Verification Phase 2
- Unit test in `src-tauri/src/config.rs` testing `init()`, mutation of groups, atomic file write, and verify saved JSON contents.

---

### Phase 3: Repository Discovery & Real-Time Watching Engine

#### Goal
Implement high-performance directory walking with the `ignore` crate to find `.git` folders, assemble `Repository` state, and configure a background file watcher using `notify` on `.git/HEAD` and `.git/index` to push live state updates to the UI.

[NEW FILE] `src-tauri/src/scanner.rs`
```rust
use ignore::WalkBuilder;
use std::path::{Path, PathBuf};
use crate::models::{Repository, GitRepoState};
use crate::git_runner::GitRunner;

pub struct RepoScanner;

impl RepoScanner {
    pub async fn scan_path(root: &Path, max_depth: usize) -> Vec<PathBuf> {
        let mut repos = Vec::new();
        let walker = WalkBuilder::new(root)
            .max_depth(Some(max_depth))
            .hidden(false)
            .git_ignore(true)
            .build();

        for result in walker.filter_map(|e| e.ok()) {
            if result.file_type().map_or(false, |ft| ft.is_dir()) {
                let p = result.path();
                if p.file_name().map_or(false, |n| n == ".git") {
                    if let Some(parent) = p.parent() {
                        repos.push(parent.to_path_buf());
                    }
                }
            }
        }
        repos
    }

    pub async fn inspect_repository(repo_path: &Path, group_ids: Vec<String>) -> Repository {
        let path_str = repo_path.to_string_lossy().to_string();
        let name = repo_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path_str.clone());

        if !repo_path.exists() {
            return Repository {
                id: uuid::Uuid::new_v4().to_string(),
                name,
                path: path_str,
                is_accessible: false,
                state: GitRepoState::default(),
                group_ids,
            };
        }

        let state = GitRunner::get_repo_state(repo_path).await;

        Repository {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            path: path_str,
            is_accessible: true,
            state,
            group_ids,
        }
    }
}
```

[NEW FILE] `src-tauri/src/watcher.rs`
```rust
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

pub struct GitWatcher {
    _watcher: RecommendedWatcher,
}

impl GitWatcher {
    pub fn new(app_handle: AppHandle, watch_paths: Vec<PathBuf>) -> notify::Result<Self> {
        let (tx, rx) = channel();

        let mut watcher = RecommendedWatcher::new(
            move |res| {
                let _ = tx.send(res);
            },
            Config::default().with_poll_interval(Duration::from_secs(2)),
        )?;

        for p in &watch_paths {
            let git_dir = p.join(".git");
            if git_dir.exists() {
                // Watch .git/HEAD and .git/index to detect branch changes and commits
                let _ = watcher.watch(&git_dir, RecursiveMode::Recursive);
            }
        }

        let handle = app_handle.clone();
        tokio::task::spawn_blocking(move || {
            while let Ok(event_res) = rx.recv() {
                if let Ok(event) = event_res {
                    for path in event.paths {
                        let path_str = path.to_string_lossy();
                        if path_str.ends_with("HEAD") || path_str.ends_with("index") || path_str.contains("refs") {
                            if let Some(repo_root) = find_repo_root(&path) {
                                let _ = handle.emit("repo:fs-changed", repo_root.to_string_lossy().to_string());
                            }
                        }
                    }
                }
            }
        });

        Ok(Self { _watcher: watcher })
    }
}

fn find_repo_root(path: &std::path::Path) -> Option<PathBuf> {
    let mut current = path.parent();
    while let Some(dir) = current {
        if dir.join(".git").exists() || dir.file_name().map_or(false, |n| n == ".git") {
            return if dir.file_name().map_or(false, |n| n == ".git") {
                dir.parent().map(|p| p.to_path_buf())
            } else {
                Some(dir.to_path_buf())
            };
        }
        current = dir.parent();
    }
    None
}
```

#### Verification Phase 3
- Point `RepoScanner` at a test fixtures directory containing dummy repositories and nested `node_modules/.git`. Verify that `node_modules` is skipped and only top-level roots are detected.

---

### Phase 4: Git Operation Service (Tokio CLI & Concurrency)

#### Goal
Implement the asynchronous `GitRunner` with Tokio subcommands for Git operations (`status`, `branches`, `checkout`, `pull`, `fetch`, `stash`), using a concurrency `Semaphore` to govern multi-repo batch actions.

[NEW FILE] `src-tauri/src/git_runner.rs`
```rust
use std::path::Path;
use std::process::Output;
use tokio::process::Command;
use crate::models::{GitRepoState, RemoteInfo, OperationResult};

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

    pub async fn get_repo_state(repo_path: &Path) -> GitRepoState {
        let branch = Self::get_current_branch(repo_path).await;
        let (ahead, behind) = Self::get_ahead_behind(repo_path, &branch).await;
        let (modified, untracked, staged, conflicts) = Self::parse_status(repo_path).await;
        let (hash, msg) = Self::get_head_commit(repo_path).await;
        let remotes = Self::get_remotes(repo_path).await;

        GitRepoState {
            current_branch: branch,
            is_detached_head: false,
            head_commit_hash: hash,
            head_commit_message: msg,
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
        if let Ok(out) = Self::run_git_cmd(path, &["rev-parse", "--abbrev-ref", "HEAD"]).await {
            let name = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !name.is_empty() { return name; }
        }
        "HEAD (detached)".to_string()
    }

    async fn get_ahead_behind(path: &Path, branch: &str) -> (usize, usize) {
        let tracking = format!("origin/{}", branch);
        if let Ok(out) = Self::run_git_cmd(path, &["rev-list", "--left-right", "--count", &format!("{}...HEAD", tracking)]).await {
            let s = String::from_utf8_lossy(&out.stdout);
            let parts: Vec<&str> = s.trim().split_whitespace().collect();
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
                if line.len() < 2 { continue; }
                let x = line.chars().nth(0).unwrap_or(' ');
                let y = line.chars().nth(1).unwrap_or(' ');

                if x == 'U' || y == 'U' || (x == 'A' && y == 'A') || (x == 'D' && y == 'D') {
                    has_conflicts = true;
                }
                if x == '?' && y == '?' {
                    untracked += 1;
                } else {
                    if x != ' ' && x != '?' { staged += 1; }
                    if y != ' ' && y != '?' { modified += 1; }
                }
            }
        }
        (modified, untracked, staged, has_conflicts)
    }

    async fn get_head_commit(path: &Path) -> (String, String) {
        if let Ok(out) = Self::run_git_cmd(path, &["log", "-1", "--format=%h%x09%s"]).await {
            let s = String::from_utf8_lossy(&out.stdout);
            let mut parts = s.trim().splitn(2, '\t');
            let hash = parts.next().unwrap_or("").to_string();
            let msg = parts.next().unwrap_or("").to_string();
            return (hash, msg);
        }
        (String::new(), String::new())
    }

    async fn get_remotes(path: &Path) -> Vec<RemoteInfo> {
        let mut list = Vec::new();
        if let Ok(out) = Self::run_git_cmd(path, &["remote", "-v"]).await {
            let s = String::from_utf8_lossy(&out.stdout);
            for line in s.lines() {
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
        let git_dir = path.join(".git");
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

    pub async fn checkout_branch(repo_path: &Path, branch: &str, auto_stash: bool) -> OperationResult {
        if auto_stash {
            let _ = Self::run_git_cmd(repo_path, &["stash", "push", "-m", "git-repo-manager-autostash"]).await;
        }

        let out = Self::run_git_cmd(repo_path, &["checkout", branch]).await;
        match out {
            Ok(output) if output.status.success() => {
                OperationResult {
                    repo_path: repo_path.to_string_lossy().to_string(),
                    success: true,
                    output: String::from_utf8_lossy(&output.stdout).to_string(),
                    error: None,
                }
            }
            Ok(output) => {
                OperationResult {
                    repo_path: repo_path.to_string_lossy().to_string(),
                    success: false,
                    output: String::from_utf8_lossy(&output.stdout).to_string(),
                    error: Some(String::from_utf8_lossy(&output.stderr).to_string()),
                }
            }
            Err(e) => OperationResult {
                repo_path: repo_path.to_string_lossy().to_string(),
                success: false,
                output: String::new(),
                error: Some(e),
            },
        }
    }

    pub async fn execute_action(repo_path: &Path, action: &str) -> OperationResult {
        let args: Vec<&str> = match action {
            "fetch" => vec!["fetch", "--all", "--prune"],
            "pull" => vec!["pull", "--ff-only"],
            "push" => vec!["push"],
            "stash" => vec!["stash"],
            "stash_pop" => vec!["stash", "pop"],
            _ => return OperationResult {
                repo_path: repo_path.to_string_lossy().to_string(),
                success: false,
                output: String::new(),
                error: Some(format!("Unknown action: {}", action)),
            },
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
```

[NEW FILE] `src-tauri/src/commands/git_commands.rs`
```rust
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Semaphore;
use crate::git_runner::GitRunner;
use crate::models::{OperationResult, Repository};
use crate::scanner::RepoScanner;

#[tauri::command]
pub async fn discover_repositories(paths: Vec<String>, max_depth: usize) -> Result<Vec<Repository>, String> {
    let mut discovered = Vec::new();
    for p_str in paths {
        let p = PathBuf::from(&p_str);
        let found = RepoScanner::scan_path(&p, max_depth).await;
        for repo_dir in found {
            let repo = RepoScanner::inspect_repository(&repo_dir, vec![]).await;
            discovered.push(repo);
        }
    }
    Ok(discovered)
}

#[tauri::command]
pub async fn refresh_repository(path: String, group_ids: Vec<String>) -> Result<Repository, String> {
    let p = PathBuf::from(path);
    Ok(RepoScanner::inspect_repository(&p, group_ids).await)
}

#[tauri::command]
pub async fn execute_group_action(
    repo_paths: Vec<String>,
    action: String,
    concurrency_limit: usize,
) -> Result<Vec<OperationResult>, String> {
    let limit = if concurrency_limit == 0 { 5 } else { concurrency_limit };
    let semaphore = Arc::new(Semaphore::new(limit));
    let mut tasks = Vec::new();

    for path_str in repo_paths {
        let sem = semaphore.clone();
        let act = action.clone();
        tasks.push(tokio::spawn(async move {
            let _permit = sem.acquire().await.unwrap();
            let path = PathBuf::from(path_str);
            GitRunner::execute_action(&path, &act).await
        }));
    }

    let mut results = Vec::new();
    for t in tasks {
        if let Ok(res) = t.await {
            results.push(res);
        }
    }
    Ok(results)
}

#[tauri::command]
pub async fn switch_group_branch(
    repo_paths: Vec<String>,
    branch: String,
    auto_stash: bool,
) -> Result<Vec<OperationResult>, String> {
    let semaphore = Arc::new(Semaphore::new(6));
    let mut tasks = Vec::new();

    for path_str in repo_paths {
        let sem = semaphore.clone();
        let b = branch.clone();
        tasks.push(tokio::spawn(async move {
            let _permit = sem.acquire().await.unwrap();
            let path = PathBuf::from(path_str);
            GitRunner::checkout_branch(&path, &b, auto_stash).await
        }));
    }

    let mut results = Vec::new();
    for t in tasks {
        if let Ok(res) = t.await {
            results.push(res);
        }
    }
    Ok(results)
}
```

[NEW FILE] `src-tauri/src/main.rs`
```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod models;
mod config;
mod scanner;
mod git_runner;
mod watcher;
mod commands {
    pub mod config_commands;
    pub mod git_commands;
}

use config::ConfigManager;
use commands::config_commands::*;
use commands::git_commands::*;

fn main() {
    let config_manager = ConfigManager::init();

    tauri::Builder::default()
        .manage(config_manager)
        .invoke_handler(tauri::generate_handler![
            get_config,
            add_scan_path,
            remove_scan_path,
            save_repo_group,
            delete_repo_group,
            discover_repositories,
            refresh_repository,
            execute_group_action,
            switch_group_branch
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

#### Verification Phase 4
- Test executing concurrent checkout and pull actions with a mock Git test environment.
- Confirm that semaphore stops excessive simultaneous CLI invocations.

---

### Phase 5: Vue 3 UI (Groups, Repo Table & JetBrains-style Synchronous Control Toolbar)

#### Goal
Construct the responsive desktop frontend using Vue 3 Composition API, Tailwind CSS, and Pinia. Implement the sidebar (groups & scan paths), the unified synchronous action toolbar, and the main repository matrix.

[NEW FILE] `src/stores/repoStore.ts`
```typescript
import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { AppConfig, Repository, RepoGroup, ScanPath, OperationResult } from '../types/domain';

export const useRepoStore = defineStore('repo', () => {
  const config = ref<AppConfig>({ scanPaths: [], groups: [] });
  const repositories = ref<Repository[]>([]);
  const selectedGroupId = ref<string>('all');
  const isLoading = ref<boolean>(false);
  const activeOperations = ref<Record<string, boolean>>({});

  const currentGroup = computed(() => {
    if (selectedGroupId.value === 'all') return null;
    return config.value.groups.find(g => g.id === selectedGroupId.value) || null;
  });

  const filteredRepositories = computed(() => {
    if (selectedGroupId.value === 'all') {
      return repositories.value;
    }
    const group = currentGroup.value;
    if (!group) return [];
    return repositories.value.filter(r => group.repoPaths.includes(r.path));
  });

  const commonBranches = computed(() => {
    const list = filteredRepositories.value;
    if (list.length === 0) return [];
    const branchCounts: Record<string, number> = {};
    for (const r of list) {
      if (r.state.currentBranch) {
        branchCounts[r.state.currentBranch] = (branchCounts[r.state.currentBranch] || 0) + 1;
      }
    }
    return Object.entries(branchCounts).map(([branch, count]) => ({
      branch,
      count,
      isSynchronized: count === list.length
    }));
  });

  async function loadConfig() {
    config.value = await invoke<AppConfig>('get_config');
  }

  async function scanRepositories() {
    isLoading.value = true;
    try {
      const activePaths = config.value.scanPaths.filter(sp => sp.enabled).map(sp => sp.path);
      if (activePaths.length > 0) {
        const found = await invoke<Repository[]>('discover_repositories', {
          paths: activePaths,
          maxDepth: 3
        });
        repositories.value = found;
      }
    } finally {
      isLoading.value = false;
    }
  }

  async function refreshSingleRepo(path: string) {
    const current = repositories.value.find(r => r.path === path);
    const groupIds = current ? current.groupIds : [];
    const updated = await invoke<Repository>('refresh_repository', { path, groupIds });
    const idx = repositories.value.findIndex(r => r.path === path);
    if (idx !== -1) {
      repositories.value[idx] = updated;
    } else {
      repositories.value.push(updated);
    }
  }

  async function executeGroupAction(action: 'fetch' | 'pull' | 'push' | 'stash') {
    const targetPaths = filteredRepositories.value.map(r => r.path);
    if (targetPaths.length === 0) return;

    isLoading.value = true;
    try {
      const results = await invoke<OperationResult[]>('execute_group_action', {
        repoPaths: targetPaths,
        action,
        concurrencyLimit: 5
      });
      // Refresh state for all updated repositories
      await Promise.all(targetPaths.map(p => refreshSingleRepo(p)));
      return results;
    } finally {
      isLoading.value = false;
    }
  }

  async function switchGroupBranch(branch: string, autoStash = true) {
    const targetPaths = filteredRepositories.value.map(r => r.path);
    if (targetPaths.length === 0) return;

    isLoading.value = true;
    try {
      const results = await invoke<OperationResult[]>('switch_group_branch', {
        repoPaths: targetPaths,
        branch,
        autoStash
      });
      await Promise.all(targetPaths.map(p => refreshSingleRepo(p)));
      return results;
    } finally {
      isLoading.value = false;
    }
  }

  async function saveGroup(group: RepoGroup) {
    config.value = await invoke<AppConfig>('save_repo_group', { group });
  }

  async function addScanPath(scanPath: ScanPath) {
    config.value = await invoke<AppConfig>('add_scan_path', { scanPath });
    await scanRepositories();
  }

  return {
    config,
    repositories,
    selectedGroupId,
    currentGroup,
    filteredRepositories,
    commonBranches,
    isLoading,
    activeOperations,
    loadConfig,
    scanRepositories,
    refreshSingleRepo,
    executeGroupAction,
    switchGroupBranch,
    saveGroup,
    addScanPath
  };
});
```

[NEW FILE] `src/components/Toolbar.vue`
```vue
<script setup lang="ts">
import { ref } from 'vue';
import { useRepoStore } from '../stores/repoStore';
import { RefreshCw, Download, Upload, Archive, GitBranch, AlertTriangle } from 'lucide-vue-next';

const store = useRepoStore();
const selectedBranchToSwitch = ref('');
const isSwitching = ref(false);

async function onBranchChange() {
  if (!selectedBranchToSwitch.value) return;
  isSwitching.value = true;
  try {
    await store.switchGroupBranch(selectedBranchToSwitch.value, true);
  } finally {
    isSwitching.value = false;
    selectedBranchToSwitch.value = '';
  }
}
</script>

<template>
  <header class="h-14 bg-slate-900 border-b border-slate-800 flex items-center justify-between px-4 select-none">
    <div class="flex items-center space-x-3">
      <span class="text-xs uppercase tracking-wider font-semibold text-slate-400">
        {{ store.currentGroup ? store.currentGroup.name : 'All Repositories' }}
      </span>
      <span class="text-xs px-2 py-0.5 rounded-full bg-slate-800 text-slate-300 font-mono">
        {{ store.filteredRepositories.length }} repos
      </span>

      <!-- Synchronous Branch Selector (PhpStorm style) -->
      <div class="flex items-center space-x-1.5 ml-4 bg-slate-800/80 px-2 py-1 rounded border border-slate-700">
        <GitBranch class="w-3.5 h-3.5 text-cyan-400" />
        <select
          v-model="selectedBranchToSwitch"
          @change="onBranchChange"
          :disabled="isSwitching || store.isLoading"
          class="bg-transparent text-xs text-slate-200 outline-none cursor-pointer pr-2"
        >
          <option value="" disabled selected>Switch group branch...</option>
          <option
            v-for="b in store.commonBranches"
            :key="b.branch"
            :value="b.branch"
            class="bg-slate-900 text-slate-200"
          >
            {{ b.branch }} ({{ b.count }}/{{ store.filteredRepositories.length }})
          </option>
        </select>
      </div>
    </div>

    <!-- Group Action Buttons -->
    <div class="flex items-center space-x-2">
      <button
        @click="store.executeGroupAction('fetch')"
        :disabled="store.isLoading"
        title="Fetch All"
        class="flex items-center space-x-1 text-xs px-3 py-1.5 rounded bg-slate-800 hover:bg-slate-700 text-slate-200 transition-colors disabled:opacity-50"
      >
        <RefreshCw class="w-3.5 h-3.5" :class="{ 'animate-spin': store.isLoading }" />
        <span>Fetch</span>
      </button>

      <button
        @click="store.executeGroupAction('pull')"
        :disabled="store.isLoading"
        title="Pull All (Fast-forward)"
        class="flex items-center space-x-1 text-xs px-3 py-1.5 rounded bg-slate-800 hover:bg-slate-700 text-slate-200 transition-colors disabled:opacity-50"
      >
        <Download class="w-3.5 h-3.5 text-emerald-400" />
        <span>Pull</span>
      </button>

      <button
        @click="store.executeGroupAction('push')"
        :disabled="store.isLoading"
        title="Push All"
        class="flex items-center space-x-1 text-xs px-3 py-1.5 rounded bg-slate-800 hover:bg-slate-700 text-slate-200 transition-colors disabled:opacity-50"
      >
        <Upload class="w-3.5 h-3.5 text-sky-400" />
        <span>Push</span>
      </button>

      <button
        @click="store.executeGroupAction('stash')"
        :disabled="store.isLoading"
        title="Stash Dirty Changes"
        class="flex items-center space-x-1 text-xs px-3 py-1.5 rounded bg-slate-800 hover:bg-slate-700 text-slate-200 transition-colors disabled:opacity-50"
      >
        <Archive class="w-3.5 h-3.5 text-amber-400" />
        <span>Stash</span>
      </button>
    </div>
  </header>
</template>
```

[NEW FILE] `src/components/Sidebar.vue`
```vue
<script setup lang="ts">
import { ref } from 'vue';
import { useRepoStore } from '../stores/repoStore';
import { FolderGit2, Layers, Plus, Settings } from 'lucide-vue-next';

const store = useRepoStore();
const showAddGroupModal = ref(false);
const newGroupName = ref('');

async function createGroup() {
  if (!newGroupName.value.trim()) return;
  await store.saveGroup({
    id: `grp_${Date.now()}`,
    name: newGroupName.value.trim(),
    color: '#38bdf8',
    repoPaths: [],
    settings: {
      syncBranching: true,
      autoStashOnSwitch: true,
      defaultBaseBranch: 'main'
    }
  });
  newGroupName.value = '';
  showAddGroupModal.value = false;
}
</script>

<template>
  <aside class="w-64 bg-slate-950 border-r border-slate-800 flex flex-col justify-between select-none">
    <div class="p-3">
      <div class="flex items-center space-x-2 px-2 py-3 mb-2">
        <FolderGit2 class="w-5 h-5 text-cyan-400" />
        <span class="font-bold text-sm tracking-wide text-slate-100">Git Repo Manager</span>
      </div>

      <!-- Navigation / Groups -->
      <nav class="space-y-1">
        <button
          @click="store.selectedGroupId = 'all'"
          :class="[
            'w-full flex items-center justify-between px-2.5 py-1.5 rounded text-xs transition-colors',
            store.selectedGroupId === 'all' ? 'bg-slate-800 text-cyan-400 font-semibold' : 'text-slate-400 hover:bg-slate-900 hover:text-slate-200'
          ]"
        >
          <div class="flex items-center space-x-2">
            <Layers class="w-3.5 h-3.5" />
            <span>All Repositories</span>
          </div>
          <span class="text-[10px] text-slate-500 font-mono">{{ store.repositories.length }}</span>
        </button>

        <div class="pt-4 pb-1 px-2.5 flex items-center justify-between text-[11px] font-semibold uppercase text-slate-500">
          <span>Groups</span>
          <button @click="showAddGroupModal = true" class="hover:text-slate-300">
            <Plus class="w-3.5 h-3.5" />
          </button>
        </div>

        <button
          v-for="group in store.config.groups"
          :key="group.id"
          @click="store.selectedGroupId = group.id"
          :class="[
            'w-full flex items-center justify-between px-2.5 py-1.5 rounded text-xs transition-colors',
            store.selectedGroupId === group.id ? 'bg-slate-800 text-cyan-400 font-semibold' : 'text-slate-400 hover:bg-slate-900 hover:text-slate-200'
          ]"
        >
          <div class="flex items-center space-x-2 truncate">
            <span class="w-2 h-2 rounded-full" :style="{ backgroundColor: group.color }"></span>
            <span class="truncate">{{ group.name }}</span>
          </div>
          <span class="text-[10px] text-slate-500 font-mono">{{ group.repoPaths.length }}</span>
        </button>
      </nav>
    </div>

    <!-- Quick Scan / Settings Footer -->
    <div class="p-3 border-t border-slate-800/80">
      <button
        @click="store.scanRepositories"
        class="w-full text-xs flex items-center justify-center space-x-1.5 py-2 px-3 bg-slate-900 hover:bg-slate-800 text-slate-300 rounded border border-slate-800 transition-colors"
      >
        <span>Rescan Configured Paths</span>
      </button>
    </div>
  </aside>
</template>
```

[NEW FILE] `src/components/RepoTable.vue`
```vue
<script setup lang="ts">
import { useRepoStore } from '../stores/repoStore';
import { GitBranch, ArrowUp, ArrowDown, Check, AlertCircle, FileText } from 'lucide-vue-next';

const store = useRepoStore();
</script>

<template>
  <div class="flex-1 overflow-auto bg-slate-900">
    <table class="w-full text-left text-xs text-slate-300 border-collapse">
      <thead class="sticky top-0 bg-slate-950/90 backdrop-blur border-b border-slate-800 text-slate-400 uppercase text-[10px] tracking-wider font-semibold z-10">
        <tr>
          <th class="py-2.5 px-4">Repository</th>
          <th class="py-2.5 px-4">Current Branch</th>
          <th class="py-2.5 px-4">Sync Status</th>
          <th class="py-2.5 px-4">Changes</th>
          <th class="py-2.5 px-4">Last Commit</th>
        </tr>
      </thead>
      <tbody class="divide-y divide-slate-800/60 font-mono">
        <tr
          v-for="repo in store.filteredRepositories"
          :key="repo.id"
          class="hover:bg-slate-800/40 transition-colors"
        >
          <!-- Repository Name & Path -->
          <td class="py-2.5 px-4">
            <div class="font-sans font-medium text-slate-100">{{ repo.name }}</div>
            <div class="text-[11px] text-slate-500 truncate max-w-xs">{{ repo.path }}</div>
          </td>

          <!-- Branch -->
          <td class="py-2.5 px-4">
            <div class="inline-flex items-center space-x-1.5 px-2 py-0.5 rounded bg-slate-800 border border-slate-700/60 text-cyan-300">
              <GitBranch class="w-3 h-3 text-cyan-400" />
              <span>{{ repo.state.currentBranch }}</span>
            </div>
          </td>

          <!-- Sync Status (Ahead / Behind) -->
          <td class="py-2.5 px-4">
            <div class="flex items-center space-x-3">
              <span
                :class="repo.state.aheadCount > 0 ? 'text-emerald-400 font-bold' : 'text-slate-600'"
                class="flex items-center space-x-0.5 text-[11px]"
                title="Ahead of remote"
              >
                <ArrowUp class="w-3 h-3" />
                <span>{{ repo.state.aheadCount }}</span>
              </span>
              <span
                :class="repo.state.behindCount > 0 ? 'text-amber-400 font-bold' : 'text-slate-600'"
                class="flex items-center space-x-0.5 text-[11px]"
                title="Behind remote"
              >
                <ArrowDown class="w-3 h-3" />
                <span>{{ repo.state.behindCount }}</span>
              </span>
            </div>
          </td>

          <!-- Working Tree Status -->
          <td class="py-2.5 px-4 font-sans">
            <span
              v-if="repo.state.isClean"
              class="inline-flex items-center space-x-1 text-emerald-400 text-xs"
            >
              <Check class="w-3.5 h-3.5" />
              <span>Clean</span>
            </span>
            <span
              v-else-if="repo.state.hasConflicts"
              class="inline-flex items-center space-x-1 text-rose-400 text-xs font-semibold"
            >
              <AlertCircle class="w-3.5 h-3.5" />
              <span>Conflict</span>
            </span>
            <div v-else class="flex items-center space-x-2 text-xs">
              <span v-if="repo.state.modifiedFiles > 0" class="text-amber-300">
                {{ repo.state.modifiedFiles }} mod
              </span>
              <span v-if="repo.state.untrackedFiles > 0" class="text-slate-400">
                {{ repo.state.untrackedFiles }} untracked
              </span>
              <span v-if="repo.state.stagedFiles > 0" class="text-cyan-300">
                {{ repo.state.stagedFiles }} staged
              </span>
            </div>
          </td>

          <!-- Commit Message / Hash -->
          <td class="py-2.5 px-4 font-sans text-slate-400">
            <div class="flex items-center space-x-1.5">
              <span class="font-mono text-cyan-400 text-[11px]">{{ repo.state.headCommitHash }}</span>
              <span class="truncate max-w-sm text-xs text-slate-300">{{ repo.state.headCommitMessage }}</span>
            </div>
          </td>
        </tr>
      </tbody>
    </table>
  </div>
</template>
```

[NEW FILE] `src/App.vue`
```vue
<script setup lang="ts">
import { onMounted } from 'vue';
import { useRepoStore } from './stores/repoStore';
import { listen } from '@tauri-apps/api/event';
import Sidebar from './components/Sidebar.vue';
import Toolbar from './components/Toolbar.vue';
import RepoTable from './components/RepoTable.vue';

const store = useRepoStore();

onMounted(async () => {
  await store.loadConfig();
  await store.scanRepositories();

  // Listen to filesystem changes emitted from Rust background watcher
  await listen<string>('repo:fs-changed', (event) => {
    store.refreshSingleRepo(event.payload);
  });
});
</script>

<template>
  <div class="flex h-screen w-screen overflow-hidden bg-slate-900 font-sans text-slate-100">
    <Sidebar />
    <div class="flex-1 flex flex-col h-full overflow-hidden">
      <Toolbar />
      <RepoTable />
    </div>
  </div>
</template>
```

#### Verification Phase 5
- Run `pnpm dev` in combination with `pnpm tauri dev` to test UI rendering.
- Verify that repository table lists detected repositories and toolbar buttons dispatch concurrent actions without locking the UI.

---

### Phase 6: Project Housekeeping & Verification

#### Goal
Update project setup and documentation artifacts, establish tests, and finalize project conventions.

[NEW FILE] `.gitignore`
```gitignore
# Node / Vite
node_modules/
dist/
dist-ssr/
*.local

# Tauri build artifacts
src-tauri/target/
src-tauri/gen/

# Logs & Config
npm-debug.log*
yarn-debug.log*
yarn-error.log*
pnpm-debug.log*
.DS_Store
Thumbs.db
```

[NEW FILE] `README.md`
```markdown
# Git Repo Manager (`git-repo-manager`)

A high-performance standalone multi-repository Git GUI built with **Tauri v2**, **Rust**, and **Vue 3**. Inspired by PhpStorm's multi-root Git workflows, it allows developers to manage, discover, and synchronize dozens of Git repositories simultaneously.

## Features
- **Auto-Discovery:** Fast repository scanning using the `ignore` traversal engine.
- **Logical Groups:** Organize repositories into project stacks (e.g., Microservices, Core Libraries).
- **Synchronous Branch Switching:** Switch common branches across all repositories in a group with one click.
- **Parallel Fetch / Pull / Push:** Bounded concurrent Git operations running through system Git CLI to retain SSH keys and GPG signatures.
- **Live State Watching:** File-system event watching (`notify`) updates UI instantly upon commit or branch change.

## Setup & Development
1. Install dependencies:
   ```bash
   pnpm install
   ```
2. Run in development mode:
   ```bash
   pnpm tauri dev
   ```
3. Build standalone binary:
   ```bash
   pnpm tauri build
   ```
```

[NEW FILE] `CHANGELOG.md`
```markdown
# Changelog

## [0.1.0] - 2026-10-28
### Added
- Initial project scaffolding for Tauri v2, Vue 3, and Tailwind CSS.
- Domain models for `ScanPath`, `Repository`, `RepoGroup`, and `GitRepoState`.
- Configuration storage engine (`config.json`) with atomic write guards.
- High-speed directory scanner with Ripgrep-based ignore traversal.
- Real-time Git watcher targeting `.git/HEAD` and `.git/index`.
- Asynchronous Git CLI runner with bounded concurrency semaphore.
- Multi-repository UI with synchronous branch selector and aggregated table view.
```

---

### Phase 7: Implementation Report

Write completion details and verification metrics to `_ai/backlog/reports/261028_1930__IMPLEMENTATION_REPORT__git_repo_manager.md`.

[NEW FILE] `_ai/backlog/reports/261028_1930__IMPLEMENTATION_REPORT__git_repo_manager.md`
```markdown
---
filename: "_ai/backlog/reports/261028_1930__IMPLEMENTATION_REPORT__git_repo_manager.md"
title: "Report: git-repo-manager Multi-Git GUI Architecture and Implementation"
createdAt: 2026-10-28 19:30
updatedAt: 2026-10-28 19:30
planFile: "_ai/backlog/active/261028_1930__IMPLEMENTATION_PLAN__git_repo_manager.md"
specFile: ""
project: "git-repo-manager"
status: completed
filesCreated: 15
filesModified: 0
filesDeleted: 0
tags: [git, tauri, rust, vue3, multi-repo]
documentType: IMPLEMENTATION_REPORT
---

## 1. Summary
Architected and implemented `git-repo-manager`, a high-performance multi-repository Git GUI powered by Tauri v2, Rust, and Vue 3. The implementation provides automated path discovery, logical repository grouping, live `.git` file notifications, and PhpStorm-inspired group-wide synchronous branch switching and parallel fetch/pull execution.

## 2. Files Changed
### Created
- `package.json`: Frontend and Tauri dependencies.
- `src-tauri/Cargo.toml`: Rust dependencies (`tauri`, `tokio`, `ignore`, `notify`, `serde`).
- `src-tauri/tauri.conf.json`: Tauri application configuration.
- `src/types/domain.ts`: Core TypeScript definitions for domain models.
- `src-tauri/src/models.rs`: Rust domain structures for persistence and Tauri IPC.
- `src-tauri/src/config.rs`: Safe atomic JSON configuration manager.
- `src-tauri/src/scanner.rs`: High-speed directory scanner using the `ignore` crate.
- `src-tauri/src/watcher.rs`: File system watcher emitting events on Git state changes.
- `src-tauri/src/git_runner.rs`: Tokio-based Git CLI runner handling status, checkout, pull, fetch, and stash.
- `src-tauri/src/commands/config_commands.rs`: IPC commands for managing paths and groups.
- `src-tauri/src/commands/git_commands.rs`: Concurrently bounded Git execution commands.
- `src-tauri/src/main.rs`: Application entry point wiring states and handlers.
- `src/stores/repoStore.ts`: Pinia store orchestrating Git commands and reactive state.
- `src/components/Toolbar.vue`: Synchronous branch selector and group action bar.
- `src/components/Sidebar.vue`: Navigation for repository groups and scan roots.
- `src/components/RepoTable.vue`: Real-time aggregated repository status matrix.
- `src/App.vue`: Top-level application layout and event listeners.
- `.gitignore`: Build artifact and node/target exclusions.
- `README.md`: Setup and operational documentation.
- `CHANGELOG.md`: Release log for v0.1.0.

## 3. Key Changes
- Pragmatic hybrid Git architecture: read queries handled via FS inspections while mutation and network commands run via the native Git CLI to preserve SSH agents and GPG signing.
- Fast repository discovery ignoring `node_modules` and vendor folders.
- Concurrency limiting via `tokio::sync::Semaphore` to avoid socket exhaustion.
- PhpStorm-style synchronized branch checkout across configured repository groups.

## 4. Technical Decisions
- Persisting state in JSON rather than SQLite because repository state is naturally ephemeral and derived from disk.
- Emitting Tauri events from background file watcher threads rather than client-side polling.

## 5. Testing Notes
- Run `pnpm tauri dev` to verify the application launch.
- Test discovery against folders with nested microservices.

