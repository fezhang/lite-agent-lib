//! Cursor agent configuration

use serde::{Deserialize, Serialize};

/// Configuration for Cursor agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorConfig {
    /// Auto-approve commands without prompting
    #[serde(default)]
    pub force: bool,

    /// Model selection (e.g., "sonnet-4.5", "gpt-5", "auto")
    #[serde(default)]
    pub model: Option<String>,

    /// Custom path to cursor-agent executable
    #[serde(default)]
    pub custom_path: Option<std::path::PathBuf>,
}

impl Default for CursorConfig {
    fn default() -> Self {
        Self {
            force: false,
            model: None,
            custom_path: None,
        }
    }
}

impl CursorConfig {
    /// Create a new Cursor configuration builder
    pub fn builder() -> CursorConfigBuilder {
        CursorConfigBuilder::default()
    }

    /// Get the base command for Cursor agent
    pub fn base_command(&self) -> &'static str {
        "cursor-agent"
    }
}

/// Builder for Cursor configuration
#[derive(Debug, Default)]
pub struct CursorConfigBuilder {
    config: CursorConfig,
}

impl CursorConfigBuilder {
    /// Enable auto-approve for commands
    pub fn with_force(mut self) -> Self {
        self.config.force = true;
        self
    }

    /// Set the model to use
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.config.model = Some(model.into());
        self
    }

    /// Set custom path to cursor-agent executable
    pub fn with_custom_path(mut self, path: std::path::PathBuf) -> Self {
        self.config.custom_path = Some(path);
        self
    }

    /// Build the configuration
    pub fn build(self) -> CursorConfig {
        self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = CursorConfig::default();
        assert!(!config.force);
        assert!(config.model.is_none());
    }

    #[test]
    fn test_builder() {
        let config = CursorConfig::builder()
            .with_force()
            .with_model("sonnet-4.5")
            .build();

        assert!(config.force);
        assert_eq!(config.model.as_deref(), Some("sonnet-4.5"));
    }

    #[test]
    fn test_base_command() {
        let config = CursorConfig::default();
        assert_eq!(config.base_command(), "cursor-agent");
    }
}
