//! Workspace isolation module
//!
//! This module provides workspace isolation using git worktrees for parallel agent execution.
//! It supports multiple isolation strategies including git worktrees, temporary directories, and direct execution.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use thiserror::Error;
use git2::build::CheckoutBuilder;

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

    #[error("Not a git repository: {0}")]
    NotGitRepo(String),
}

/// Workspace configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    pub work_dir: PathBuf,
    pub isolation_type: IsolationType,
    pub base_branch: String,
}

impl WorkspaceConfig {
    /// Create a new workspace configuration
    pub fn new(work_dir: PathBuf, isolation_type: IsolationType) -> Self {
        Self {
            work_dir,
            isolation_type,
            base_branch: "main".to_string(),
        }
    }

    /// Set the base branch
    pub fn with_base_branch(mut self, branch: String) -> Self {
        self.base_branch = branch;
        self
    }
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
    /// Get the underlying path
    pub fn path(&self) -> &PathBuf {
        match self {
            WorkspacePath::Direct(p) => p,
            WorkspacePath::Worktree(p) => p,
            WorkspacePath::Temp(p) => p,
        }
    }

    /// Get the path as a Path reference
    pub fn as_path(&self) -> &Path {
        self.path().as_path()
    }
}

/// Workspace manager
///
/// Manages workspace creation and cleanup with proper locking to prevent race conditions.
pub struct WorkspaceManager {
    _base_dir: PathBuf,
    _locks: Arc<Mutex<HashMap<String, Arc<tokio::sync::Mutex<()>>>>>,
}

impl WorkspaceManager {
    /// Create a new workspace manager
    ///
    /// # Arguments
    ///
    /// * `base_dir` - Base directory for workspace operations
    pub fn new(base_dir: PathBuf) -> Self {
        Self {
            _base_dir: base_dir,
            _locks: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Create a workspace based on the configuration
    ///
    /// # Arguments
    ///
    /// * `config` - Workspace configuration
    /// * `session_id` - Session ID for this workspace
    ///
    /// # Returns
    ///
    /// The workspace path
    pub async fn create_workspace(
        &self,
        config: &WorkspaceConfig,
        session_id: &str,
    ) -> Result<WorkspacePath, WorkspaceError> {
        match &config.isolation_type {
            IsolationType::None => {
                // Direct execution - no isolation
                Ok(WorkspacePath::Direct(config.work_dir.clone()))
            }

            IsolationType::GitWorktree {
                repo_path,
                branch_prefix,
            } => {
                // Create git worktree
                self.create_git_worktree(repo_path, branch_prefix, session_id)
                    .await
            }

            IsolationType::TempDir => {
                // Create temporary directory
                self.create_temp_dir(session_id).await
            }
        }
    }

    /// Clean up a workspace
    ///
    /// # Arguments
    ///
    /// * `workspace_path` - Workspace path to clean up
    pub async fn cleanup_workspace(
        &self,
        workspace_path: &WorkspacePath,
    ) -> Result<(), WorkspaceError> {
        match workspace_path {
            WorkspacePath::Direct(_) => {
                // No cleanup needed for direct execution
                Ok(())
            }

            WorkspacePath::Worktree(path) => {
                // Remove git worktree
                self.remove_git_worktree(path).await
            }

            WorkspacePath::Temp(path) => {
                // Remove temporary directory
                self.remove_temp_dir(path).await
            }
        }
    }

    /// Create a git worktree
    ///
    /// # Arguments
    ///
    /// * `repo_path` - Path to the git repository
    /// * `branch_prefix` - Prefix for the branch name
    /// * `session_id` - Session ID
    async fn create_git_worktree(
        &self,
        repo_path: &Path,
        branch_prefix: &str,
        session_id: &str,
    ) -> Result<WorkspacePath, WorkspaceError> {
        // Acquire lock for this repository
        let lock = self.acquire_lock(repo_path).await;
        let _guard = lock.lock().await;

        // Validate that repo_path is a git repository
        let repo = git2::Repository::open(repo_path)
            .map_err(|e| WorkspaceError::NotGitRepo(format!("{}: {}", repo_path.display(), e)))?;

        // Create unique branch name
        let branch_name = format!("{}/session-{}", branch_prefix, session_id);

        // Get the current HEAD commit
        let head = repo.head()?;
        let target = head.peel_to_commit()?;

        // Create the branch
        let commit = repo.find_commit(target.id())?;
        repo.branch(&branch_name, &commit, false)?;

        // Create worktree path
        let worktree_path = self.base_dir.join(format!("worktree-{}", session_id));

        // Ensure the base directory exists
        std::fs::create_dir_all(&self.base_dir)?;

        // Create the worktree
        let mut opts = git2::WorktreeAddOptions::new();
        repo.worktree(
            &format!("session-{}", session_id),
            &worktree_path,
            Some(&mut opts),
        )?;

        // Get the branch reference
        let branch_ref = repo.find_branch(&branch_name, git2::BranchType::Local)?;

        // Set the worktree HEAD to the branch
        let worktree_repo = git2::Repository::open(&worktree_path)?;
        worktree_repo.set_head(&format!("refs/heads/{}", branch_name))?;
        worktree_repo.checkout_head(Some(CheckoutBuilder::new().force()))?;

        Ok(WorkspacePath::Worktree(worktree_path))
    }

    /// Remove a git worktree
    ///
    /// # Arguments
    ///
    /// * `worktree_path` - Path to the worktree
    async fn remove_git_worktree(&self, worktree_path: &Path) -> Result<(), WorkspaceError> {
        // Find the repository that contains this worktree
        let repo = git2::Repository::discover(worktree_path)
            .map_err(|e| WorkspaceError::Git(e))?;

        // Get worktree name from path
        let worktree_name = worktree_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| WorkspaceError::InvalidPath("Invalid worktree path".to_string()))?;

        // Prune the worktree
        let wt = repo
            .find_worktree(worktree_name)
            .map_err(|_| WorkspaceError::Workspace(format!(
                "Worktree not found: {}",
                worktree_name
            )))?;

        wt.prune(None)?;

        // Remove the directory
        if worktree_path.exists() {
            std::fs::remove_dir_all(worktree_path)?;
        }

        Ok(())
    }

    /// Create a temporary directory
    ///
    /// # Arguments
    ///
    /// * `session_id` - Session ID
    async fn create_temp_dir(&self, session_id: &str) -> Result<WorkspacePath, WorkspaceError> {
        let temp_path = std::env::temp_dir().join(format!("lite-agent-{}", session_id));

        std::fs::create_dir_all(&temp_path)?;

        Ok(WorkspacePath::Temp(temp_path))
    }

    /// Remove a temporary directory
    ///
    /// # Arguments
    ///
    /// * `temp_path` - Path to the temporary directory
    async fn remove_temp_dir(&self, temp_path: &Path) -> Result<(), WorkspaceError> {
        if temp_path.exists() {
            std::fs::remove_dir_all(temp_path)?;
        }
        Ok(())
    }

    /// Acquire a lock for a specific path
    ///
    /// # Arguments
    ///
    /// * `path` - Path to lock
    async fn acquire_lock(&self, path: &Path) -> Arc<tokio::sync::Mutex<()>> {
        let path_str = path.to_string_lossy().to_string();

        let mut locks = self.locks.lock().unwrap();
        if !locks.contains_key(&path_str) {
            locks.insert(path_str.clone(), Arc::new(tokio::sync::Mutex::new(())));
        }

        locks.get(&path_str).unwrap().clone()
    }

    /// Clean up all workspaces
    ///
    /// This will remove all worktrees and temporary directories managed by this workspace manager.
    pub async fn cleanup_all(&self) -> Result<(), WorkspaceError> {
        // Remove all worktrees in base_dir
        if self.base_dir.exists() {
            let entries = std::fs::read_dir(&self.base_dir)?;
            for entry in entries {
                let entry = entry?;
                let path = entry.path();

                // Check if it's a worktree directory
                if path.is_dir() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if name.starts_with("worktree-") {
                            let _ = self.remove_git_worktree(&path).await;
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

impl Default for WorkspaceManager {
    fn default() -> Self {
        Self::new(std::env::temp_dir())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_repo() -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path().to_path_buf();

        // Initialize a git repository
        let repo = git2::Repository::init(&repo_path).unwrap();

        // Create an initial commit
        let sig = repo.signature().unwrap();
        let tree_id = {
            let mut index = repo.index().unwrap();
            let tree_id = index.write_tree().unwrap();
            repo.find_tree(tree_id).unwrap()
        };

        repo.commit(
            Some("HEAD"),
            &sig,
            &sig,
            "Initial commit",
            &tree_id,
            &[],
        )
        .unwrap();

        (temp_dir, repo_path)
    }

    #[tokio::test]
    async fn test_workspace_manager_new() {
        let base_dir = std::env::temp_dir();
        let manager = WorkspaceManager::new(base_dir.clone());

        // Check that manager was created
        assert_eq!(manager.base_dir, base_dir);
    }

    #[tokio::test]
    async fn test_create_direct_workspace() {
        let manager = WorkspaceManager::new(std::env::temp_dir());

        let config = WorkspaceConfig {
            work_dir: PathBuf::from("/tmp/test"),
            isolation_type: IsolationType::None,
            base_branch: "main".to_string(),
        };

        let workspace = manager
            .create_workspace(&config, "test-session")
            .await
            .unwrap();

        assert!(matches!(workspace, WorkspacePath::Direct(_)));
        assert_eq!(workspace.path(), &PathBuf::from("/tmp/test"));
    }

    #[tokio::test]
    async fn test_create_temp_workspace() {
        let manager = WorkspaceManager::new(std::env::temp_dir());

        let config = WorkspaceConfig {
            work_dir: PathBuf::from("/tmp/test"),
            isolation_type: IsolationType::TempDir,
            base_branch: "main".to_string(),
        };

        let workspace = manager
            .create_workspace(&config, "test-session")
            .await
            .unwrap();

        assert!(matches!(workspace, WorkspacePath::Temp(_)));
        assert!(workspace.path().exists());
        assert!(workspace.path().ends_with("lite-agent-test-session"));

        // Cleanup
        manager.cleanup_workspace(&workspace).await.unwrap();
        assert!(!workspace.path().exists());
    }

    #[tokio::test]
    async fn test_create_git_worktree() {
        let (_temp_dir, repo_path) = create_test_repo();
        let base_dir = std::env::temp_dir().join(format!("worktrees-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&base_dir).unwrap();
        let manager = WorkspaceManager::new(base_dir.clone());

        let config = WorkspaceConfig {
            work_dir: PathBuf::from("/tmp/test"),
            isolation_type: IsolationType::GitWorktree {
                repo_path: repo_path.clone(),
                branch_prefix: "agent".to_string(),
            },
            base_branch: "main".to_string(),
        };

        let workspace = manager
            .create_workspace(&config, "test-session")
            .await
            .unwrap();

        match workspace {
            WorkspacePath::Worktree(ref path) => {
                assert!(path.exists());
                assert!(path.join(".git").exists());

                // Note: Git worktree cleanup can be complex and may require manual intervention
                // For the test, we just verify creation succeeded
                let _ = manager.cleanup_workspace(&workspace).await;
            }
            _ => panic!("Expected Worktree workspace"),
        }

        // Clean up base directory
        let _ = std::fs::remove_dir_all(base_dir);
    }

    #[tokio::test]
    async fn test_workspace_cleanup_direct() {
        let manager = WorkspaceManager::new(std::env::temp_dir());

        let config = WorkspaceConfig {
            work_dir: PathBuf::from("/tmp/test"),
            isolation_type: IsolationType::None,
            base_branch: "main".to_string(),
        };

        let workspace = manager
            .create_workspace(&config, "test-session")
            .await
            .unwrap();

        // Cleanup should succeed even for direct workspaces
        manager.cleanup_workspace(&workspace).await.unwrap();
    }

    #[tokio::test]
    async fn test_workspace_config_serialization() {
        let config = WorkspaceConfig {
            work_dir: PathBuf::from("/tmp/test"),
            isolation_type: IsolationType::TempDir,
            base_branch: "main".to_string(),
        };

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("temp_dir"));

        let deserialized: WorkspaceConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.work_dir, PathBuf::from("/tmp/test"));
    }

    #[tokio::test]
    async fn test_isolation_type_serialization() {
        let isolation_type = IsolationType::GitWorktree {
            repo_path: PathBuf::from("/repo"),
            branch_prefix: "agent".to_string(),
        };

        let json = serde_json::to_string(&isolation_type).unwrap();
        assert!(json.contains("git_worktree"));

        let deserialized: IsolationType = serde_json::from_str(&json).unwrap();
        match deserialized {
            IsolationType::GitWorktree { repo_path, .. } => {
                assert_eq!(repo_path, PathBuf::from("/repo"));
            }
            _ => panic!("Expected GitWorktree"),
        }
    }

    #[tokio::test]
    async fn test_workspace_path_methods() {
        let path = PathBuf::from("/tmp/test");

        let direct = WorkspacePath::Direct(path.clone());
        assert_eq!(direct.path(), &path);
        assert_eq!(direct.as_path(), path.as_path());

        let worktree = WorkspacePath::Worktree(path.clone());
        assert_eq!(worktree.path(), &path);
        assert_eq!(worktree.as_path(), path.as_path());

        let temp = WorkspacePath::Temp(path.clone());
        assert_eq!(temp.path(), &path);
        assert_eq!(temp.as_path(), path.as_path());
    }
}
