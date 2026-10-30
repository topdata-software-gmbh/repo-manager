//! Typed payload of the application event bus.

use crate::git_log::GitStepEvent;

#[derive(Debug, Clone)]
pub enum AppEvent {
    FsChanged(String),
    GitStep(GitStepEvent),
}
