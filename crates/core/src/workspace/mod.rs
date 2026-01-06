//! Workspace isolation module (stub - to be implemented in Phase 3)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use thiserror::Error;

/// Workspace errors
#[derive(Debug, Error)]
pub enum WorkspaceError {
    #[error("Git error: {0}")]
    Git(#[from] git2::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Workspace error: {0}")]
    Workspace(String),

    #[error("Invalid path: {0}")]
    InvalidPath(String),
}

/// Workspace configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    pub work_dir: PathBuf,
    pub isolation_type: IsolationType,
    pub base_branch: String,
}

/// Isolation type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum IsolationType {
    /// No isolation - use work_dir directly
    None,

    /// Git worktree isolation
    GitWorktree {
        repo_path: PathBuf,
        branch_prefix: String,
    },

    /// Temporary directory isolation
    TempDir,
}

/// Workspace path
#[derive(Debug, Clone)]
pub enum WorkspacePath {
    Direct(PathBuf),
    Worktree(PathBuf),
    Temp(PathBuf),
}

impl WorkspacePath {
    pub fn path(&self) -> &PathBuf {
        match self {
            WorkspacePath::Direct(p) => p,
            WorkspacePath::Worktree(p) => p,
            WorkspacePath::Temp(p) => p,
        }
    }
}

/// Workspace manager (stub)
pub struct WorkspaceManager {
    base_dir: PathBuf,
    locks: Arc<Mutex<HashMap<String, Arc<tokio::sync::Mutex<()>>>>>,
}

impl WorkspaceManager {
    pub fn new(base_dir: PathBuf) -> Self {
        Self {
            base_dir,
            locks: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}
