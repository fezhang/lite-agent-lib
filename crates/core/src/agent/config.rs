use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

use crate::workspace::WorkspaceConfig;

/// Configuration for agent execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Working directory for agent execution
    pub work_dir: PathBuf,

    /// Environment variables for the agent process
    #[serde(default)]
    pub env: HashMap<String, String>,

    /// Workspace isolation configuration (optional)
    #[serde(default)]
    pub workspace: Option<WorkspaceConfig>,

    /// Timeout duration for agent execution
    #[serde(default)]
    pub timeout: Option<Duration>,

    /// Custom agent-specific configuration
    #[serde(default)]
    pub custom: serde_json::Value,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            work_dir: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            env: HashMap::new(),
            workspace: None,
            timeout: None,
            custom: serde_json::Value::Null,
        }
    }
}

impl AgentConfig {
    /// Create a new agent configuration with specified working directory
    pub fn new(work_dir: PathBuf) -> Self {
        Self {
            work_dir,
            ..Default::default()
        }
    }

    /// Set environment variables
    pub fn with_env(mut self, env: HashMap<String, String>) -> Self {
        self.env = env;
        self
    }

    /// Add a single environment variable
    pub fn add_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.insert(key.into(), value.into());
        self
    }

    /// Set workspace configuration
    pub fn with_workspace(mut self, workspace: WorkspaceConfig) -> Self {
        self.workspace = Some(workspace);
        self
    }

    /// Set timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Set custom configuration
    pub fn with_custom(mut self, custom: serde_json::Value) -> Self {
        self.custom = custom;
        self
    }
}
