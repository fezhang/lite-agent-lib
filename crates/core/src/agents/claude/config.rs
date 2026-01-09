//! Claude Code CLI configuration

use crate::protocol::control::PermissionMode;
use serde::{Deserialize, Serialize};

/// Configuration for Claude Code CLI agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeConfig {
    /// Enable ANTHROPIC debug mode
    #[serde(default)]
    pub debug_mode: bool,

    /// Enable plan mode with auto-approve
    #[serde(default)]
    pub plan_mode: bool,

    /// Enable tool approval prompts
    #[serde(default)]
    pub approvals: bool,

    /// Model selection (e.g., "claude-sonnet-4")
    #[serde(default)]
    pub model: Option<String>,

    /// Use claude-code-router instead of official CLI
    #[serde(default)]
    pub use_router: bool,

    /// Permission mode to use
    #[serde(default)]
    pub permission_mode: PermissionMode,

    /// Dangerously skip all permissions
    #[serde(default, rename = "dangerouslySkipPermissions")]
    pub dangerously_skip_permissions: bool,

    /// Disable ANTHROPIC_API_KEY from environment
    #[serde(default, rename = "disableApiKey")]
    pub disable_api_key: bool,

    /// Custom path to claude executable
    #[serde(default)]
    pub custom_path: Option<std::path::PathBuf>,
}

impl Default for ClaudeConfig {
    fn default() -> Self {
        Self {
            debug_mode: false,
            plan_mode: false,
            approvals: false,
            model: None,
            use_router: false,
            permission_mode: PermissionMode::BypassPermissions,
            dangerously_skip_permissions: false,
            disable_api_key: false,
            custom_path: None,
        }
    }
}

impl ClaudeConfig {
    /// Create a new Claude configuration builder
    pub fn builder() -> ClaudeConfigBuilder {
        ClaudeConfigBuilder::default()
    }

    /// Get the effective permission mode based on config
    pub fn effective_permission_mode(&self) -> PermissionMode {
        if self.plan_mode {
            PermissionMode::Plan
        } else if self.approvals {
            PermissionMode::Default
        } else {
            self.permission_mode
        }
    }

    /// Get hooks configuration for the agent
    pub fn get_hooks(&self) -> Option<serde_json::Value> {
        if self.plan_mode {
            Some(serde_json::json!({
                "PreToolUse": [
                    {
                        "matcher": "^ExitPlanMode$",
                        "hookCallbackIds": ["tool_approval"],
                    },
                    {
                        "matcher": "^(?!ExitPlanMode$).*",
                        "hookCallbackIds": ["auto_approve"],
                    }
                ]
            }))
        } else if self.approvals {
            Some(serde_json::json!({
                "PreToolUse": [
                    {
                        "matcher": "^(?!(Glob|Grep|Read|Task|TodoWrite)$).*",
                        "hookCallbackIds": ["tool_approval"],
                    }
                ]
            }))
        } else {
            None
        }
    }

    /// Get the base command for Claude Code CLI
    pub fn base_command(&self) -> &'static str {
        if self.use_router {
            "npx -y @musistudio/claude-code-router@1.0.66 code"
        } else {
            "npx -y @anthropic-ai/claude-code@2.0.76"
        }
    }
}

/// Builder for Claude configuration
#[derive(Debug, Default)]
pub struct ClaudeConfigBuilder {
    config: ClaudeConfig,
}

impl ClaudeConfigBuilder {
    /// Enable ANTHROPIC debug mode
    pub fn with_debug_mode(mut self) -> Self {
        self.config.debug_mode = true;
        self
    }

    /// Enable plan mode with auto-approve
    pub fn with_plan_mode(mut self) -> Self {
        self.config.plan_mode = true;
        self
    }

    /// Enable tool approval prompts
    pub fn with_approvals(mut self) -> Self {
        self.config.approvals = true;
        self
    }

    /// Set the model to use
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.config.model = Some(model.into());
        self
    }

    /// Use claude-code-router
    pub fn with_router(mut self) -> Self {
        self.config.use_router = true;
        self
    }

    /// Set permission mode
    pub fn with_permission_mode(mut self, mode: PermissionMode) -> Self {
        self.config.permission_mode = mode;
        self
    }

    /// Dangerously skip all permissions
    pub fn with_dangerously_skip_permissions(mut self) -> Self {
        self.config.dangerously_skip_permissions = true;
        self
    }

    /// Disable ANTHROPIC_API_KEY
    pub fn with_disable_api_key(mut self) -> Self {
        self.config.disable_api_key = true;
        self
    }

    /// Set custom path to claude executable
    pub fn with_custom_path(mut self, path: std::path::PathBuf) -> Self {
        self.config.custom_path = Some(path);
        self
    }

    /// Build the configuration
    pub fn build(self) -> ClaudeConfig {
        self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ClaudeConfig::default();
        assert!(!config.debug_mode);
        assert!(!config.plan_mode);
        assert!(!config.approvals);
        assert_eq!(config.permission_mode, PermissionMode::BypassPermissions);
    }

    #[test]
    fn test_builder() {
        let config = ClaudeConfig::builder()
            .with_debug_mode()
            .with_plan_mode()
            .with_model("claude-sonnet-4")
            .build();

        assert!(config.debug_mode);
        assert!(config.plan_mode);
        assert_eq!(config.model.as_deref(), Some("claude-sonnet-4"));
    }

    #[test]
    fn test_effective_permission_mode() {
        let config = ClaudeConfig {
            plan_mode: true,
            permission_mode: PermissionMode::Default,
            ..Default::default()
        };
        assert_eq!(config.effective_permission_mode(), PermissionMode::Plan);

        let config = ClaudeConfig {
            approvals: true,
            permission_mode: PermissionMode::BypassPermissions,
            ..Default::default()
        };
        assert_eq!(config.effective_permission_mode(), PermissionMode::Default);
    }

    #[test]
    fn test_hooks_generation() {
        let config = ClaudeConfig {
            plan_mode: true,
            ..Default::default()
        };
        let hooks = config.get_hooks();
        assert!(hooks.is_some());

        let config = ClaudeConfig {
            approvals: true,
            ..Default::default()
        };
        let hooks = config.get_hooks();
        assert!(hooks.is_some());

        let config = ClaudeConfig::default();
        let hooks = config.get_hooks();
        assert!(hooks.is_none());
    }
}
