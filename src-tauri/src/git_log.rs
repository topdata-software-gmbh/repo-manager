//! Structured Git step events and the observer seam used to publish them.

use serde::Serialize;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum GitStepStatus {
    Running,
    Ok,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitStepEvent {
    pub run_id: String,
    pub repo_path: String,
    pub macro_id: String,
    pub title: String,
    pub step_index: usize,
    pub comment: String,
    pub command: String,
    pub status: GitStepStatus,
    pub duration_ms: u64,
    pub message: Option<String>,
    pub alias: Option<String>,
}

/// Identity shared by every step of one macro invocation.
pub struct RunContext<'a> {
    pub run_id: &'a str,
    pub repo_path: &'a Path,
    pub macro_id: &'a str,
    pub title: &'a str,
}

impl GitStepEvent {
    pub fn new(
        run: &RunContext<'_>,
        step_index: usize,
        comment: &str,
        command: &str,
        status: GitStepStatus,
        alias: Option<String>,
    ) -> Self {
        Self {
            run_id: run.run_id.to_string(),
            repo_path: run.repo_path.to_string_lossy().to_string(),
            macro_id: run.macro_id.to_string(),
            title: run.title.to_string(),
            step_index,
            comment: comment.to_string(),
            command: command.to_string(),
            status,
            duration_ms: 0,
            message: None,
            alias,
        }
    }
}

/// Receives every step transition. Implemented by the event bus in production
/// and by a recording observer in tests.
pub trait GitObserver: Send + Sync {
    fn on_step(&self, event: &GitStepEvent);
}

/// Observer used by tests and by call sites that do not care about the log.
#[cfg(test)]
pub struct NullObserver;

#[cfg(test)]
impl GitObserver for NullObserver {
    fn on_step(&self, _event: &GitStepEvent) {}
}
