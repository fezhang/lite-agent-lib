//! Unit tests for agent module

use super::*;
use futures_util::StreamExt;
use std::path::PathBuf;

/// Mock agent executor for testing
struct MockAgent;

#[async_trait]
impl AgentExecutor for MockAgent {
    fn agent_type(&self) -> &str {
        "mock"
    }

    async fn spawn(
        &self,
        _config: &AgentConfig,
        _input: &str,
    ) -> Result<SpawnedAgent, AgentError> {
        Err(AgentError::Custom(
            "Mock agent does not actually spawn".to_string(),
        ))
    }

    fn normalize_logs(
        &self,
        _raw_logs: Arc<LogStore>,
    ) -> BoxStream<'static, NormalizedEntry> {
        futures_util::stream::empty().boxed()
    }

    async fn check_availability(&self) -> AvailabilityStatus {
        AvailabilityStatus::Available
    }

    fn capabilities(&self) -> Vec<AgentCapability> {
        vec![
            AgentCapability::SessionContinuation,
            AgentCapability::WorkspaceIsolation,
        ]
    }
}

#[test]
fn test_agent_type() {
    let agent = MockAgent;
    assert_eq!(agent.agent_type(), "mock");
}

#[tokio::test]
async fn test_agent_availability() {
    let agent = MockAgent;
    let status = agent.check_availability().await;
    assert!(status.is_available());
}

#[test]
fn test_agent_capabilities() {
    let agent = MockAgent;
    let caps = agent.capabilities();
    assert_eq!(caps.len(), 2);
    assert!(caps.contains(&AgentCapability::SessionContinuation));
    assert!(caps.contains(&AgentCapability::WorkspaceIsolation));
}

#[test]
fn test_agent_config_default() {
    let config = AgentConfig::default();
    assert!(config.work_dir.exists() || config.work_dir == PathBuf::from("."));
    assert!(config.env.is_empty());
    assert!(config.workspace.is_none());
    assert!(config.timeout.is_none());
}

#[test]
fn test_agent_config_builder() {
    let config = AgentConfig::new(PathBuf::from("/tmp"))
        .add_env("KEY", "VALUE")
        .with_timeout(std::time::Duration::from_secs(30));

    assert_eq!(config.work_dir, PathBuf::from("/tmp"));
    assert_eq!(config.env.get("KEY"), Some(&"VALUE".to_string()));
    assert_eq!(config.timeout, Some(std::time::Duration::from_secs(30)));
}

#[test]
fn test_agent_capability_serialization() {
    let cap = AgentCapability::SessionContinuation;
    let json = serde_json::to_string(&cap).unwrap();
    assert_eq!(json, "\"SESSION_CONTINUATION\"");

    let deserialized: AgentCapability = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized, cap);
}

#[test]
fn test_availability_status() {
    assert!(AvailabilityStatus::Available.is_available());
    assert!(!AvailabilityStatus::NotFound {
        reason: "test".to_string()
    }
    .is_available());
}

#[test]
fn test_exit_result() {
    let success = ExitResult::Success;
    let failure = ExitResult::Failure(1);
    let interrupted = ExitResult::Interrupted;

    // Just ensure they can be created
    assert!(matches!(success, ExitResult::Success));
    assert!(matches!(failure, ExitResult::Failure(1)));
    assert!(matches!(interrupted, ExitResult::Interrupted));
}

#[test]
fn test_agent_error_display() {
    let err = AgentError::Timeout;
    assert_eq!(err.to_string(), "Timeout error");

    let err = AgentError::SessionNotFound("test-session".to_string());
    assert_eq!(err.to_string(), "Session not found: test-session");
}
