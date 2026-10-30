mod commands {
    pub mod config_commands;
    pub mod git_commands;
}
mod config;
mod git_runner;
mod models;
mod scanner;
mod service;
mod watcher;
mod web;

use commands::config_commands::*;
use commands::git_commands::*;
use config::ConfigManager;
use std::path::PathBuf;
use tauri::{Emitter, Manager};
use watcher::{EventBus, WatcherState};
use web::WebOptions;

struct Cli {
    web: bool,
    host: String,
    port: u16,
    token: Option<String>,
    config: Option<PathBuf>,
    dist: Option<PathBuf>,
    help: bool,
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let cli = match parse_cli() {
        Ok(cli) => cli,
        Err(error) => {
            eprintln!("error: {error}\n\n{}", help_text());
            std::process::exit(2);
        }
    };

    if cli.help {
        print!("{}", help_text());
        return;
    }

    if cli.web {
        if let Err(error) = web::run(WebOptions {
            host: cli.host,
            port: cli.port,
            token: cli.token,
            config_path: cli.config,
            dist_dir: cli.dist,
        }) {
            eprintln!("web server failed: {error}");
            std::process::exit(1);
        }
        return;
    }

    run_desktop(cli);
}

fn run_desktop(cli: Cli) {
    let config_manager = match cli.config {
        Some(path) => ConfigManager::from_path(path),
        None => ConfigManager::init(),
    };
    let event_bus = EventBus::new();
    let watcher_state = WatcherState::new(event_bus.clone());

    tauri::Builder::default()
        .manage(config_manager)
        .manage(event_bus)
        .manage(watcher_state)
        .setup(|app| {
            let event_bus = app.state::<EventBus>().inner().clone();
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let mut receiver = event_bus.subscribe();
                while let Ok(repo_path) = receiver.recv().await {
                    let _ = handle.emit("repo:fs-changed", repo_path);
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_config,
            add_scan_path,
            remove_scan_path,
            save_repo_group,
            delete_repo_group,
            save_onboarding_state,
            discover_repositories,
            refresh_repository,
            list_repo_changes,
            commit_repo_changes,
            execute_group_action,
            switch_group_branch,
            list_group_branches
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn parse_cli() -> Result<Cli, String> {
    let mut cli = Cli {
        web: env_flag("GIT_REPO_MANAGER_WEB"),
        host: std::env::var("GIT_REPO_MANAGER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
        port: std::env::var("GIT_REPO_MANAGER_PORT")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(8787),
        token: std::env::var("GIT_REPO_MANAGER_TOKEN")
            .ok()
            .filter(|value| !value.is_empty()),
        config: std::env::var("GIT_REPO_MANAGER_CONFIG")
            .ok()
            .map(PathBuf::from),
        dist: std::env::var("GIT_REPO_MANAGER_DIST")
            .ok()
            .map(PathBuf::from),
        help: false,
    };

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--web" => cli.web = true,
            "--help" | "-h" => cli.help = true,
            "--host" => cli.host = args.next().ok_or("--host requires a value")?,
            "--port" => {
                cli.port = args
                    .next()
                    .ok_or("--port requires a value")?
                    .parse()
                    .map_err(|_| "invalid value for --port".to_string())?;
            }
            "--token" => {
                cli.token = args.next().filter(|value| !value.is_empty());
            }
            "--config" => {
                cli.config = Some(PathBuf::from(
                    args.next().ok_or("--config requires a value")?,
                ));
            }
            "--dist" => {
                cli.dist = Some(PathBuf::from(args.next().ok_or("--dist requires a value")?));
            }
            value if value.starts_with("--host=") => {
                cli.host = value["--host=".len()..].to_string();
            }
            value if value.starts_with("--port=") => {
                cli.port = value["--port=".len()..]
                    .parse()
                    .map_err(|_| "invalid value for --port".to_string())?;
            }
            value if value.starts_with("--token=") => {
                cli.token =
                    Some(value["--token=".len()..].to_string()).filter(|token| !token.is_empty());
            }
            value if value.starts_with("--config=") => {
                cli.config = Some(PathBuf::from(&value["--config=".len()..]));
            }
            value if value.starts_with("--dist=") => {
                cli.dist = Some(PathBuf::from(&value["--dist=".len()..]));
            }
            _ => {}
        }
    }

    Ok(cli)
}

fn env_flag(name: &str) -> bool {
    std::env::var(name)
        .map(|value| value == "1" || value.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

fn help_text() -> &'static str {
    "Git Repo Manager

Usage:
  git-repo-manager                 Launch the desktop application
  git-repo-manager --web           Serve the browser UI and HTTP API

Options:
  --web                 Run in web server mode (no desktop window)
  --host <HOST>         Bind address for web mode (default: 127.0.0.1)
  --port <PORT>         Port for web mode (default: 8787)
  --token <TOKEN>       Require this access token in web mode
  --config <PATH>       Use a custom config.json location
  --dist <PATH>         Frontend build directory for web mode (default: dist)
  -h, --help            Print this help

Environment variables:
  GIT_REPO_MANAGER_WEB=1
  GIT_REPO_MANAGER_HOST / GIT_REPO_MANAGER_PORT
  GIT_REPO_MANAGER_TOKEN / GIT_REPO_MANAGER_CONFIG / GIT_REPO_MANAGER_DIST

Web mode example:
  pnpm build
  git-repo-manager --web --host 0.0.0.0 --port 8787 --token \"$(openssl rand -hex 16)\"
"
}
