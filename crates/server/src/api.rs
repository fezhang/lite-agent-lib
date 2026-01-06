//! API models and server state

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

use lite_agent_core::{
    AgentConfig, AgentError, AgentExecutor, SessionManager, WorkspaceManager,
};

use serde::{Deserialize, Serialize};

/// Request to spawn an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnRequest {
    /// Type of agent to spawn (e.g., "shell", "echo")
    pub agent_type: String,

    /// Input/prompt for the agent
    pub input: String,

    /// Optional session ID to continue
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,

    /// Agent configuration
    #[serde(default)]
    pub config: AgentConfigOptions,
}

/// Optional configuration for agent execution
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentConfigOptions {
    /// Working directory (defaults to current directory)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub work_dir: Option<PathBuf>,

    /// Environment variables
    #[serde(default)]
    pub env: HashMap<String, String>,

    /// Timeout in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_secs: Option<u64>,

    /// Custom agent-specific configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom: Option<serde_json::Value>,
}

impl From<AgentConfigOptions> for AgentConfig {
    fn from(options: AgentConfigOptions) -> Self {
        AgentConfig {
            work_dir: options
                .work_dir
                .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))),
            env: options.env,
            workspace: None,
            timeout: options.timeout_secs.map(std::time::Duration::from_secs),
            custom: options.custom.unwrap_or(serde_json::Value::Null),
        }
    }
}

/// Response from spawning an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnResponse {
    /// Session ID
    pub session_id: String,

    /// Execution ID
    pub execution_id: String,

    /// Agent type
    pub agent_type: String,

    /// Status of the spawn operation
    pub status: String,
}

/// Response for getting session status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStatusResponse {
    /// Session ID
    pub session_id: String,

    /// Agent type
    pub agent_type: String,

    /// Session status
    pub status: String,

    /// Number of executions
    pub execution_count: usize,

    /// Creation timestamp
    pub created_at: String,

    /// Last update timestamp
    pub updated_at: String,
}

/// Response for listing sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListSessionsResponse {
    /// List of sessions
    pub sessions: Vec<SessionStatusResponse>,

    /// Total count
    pub total: usize,
}

/// Agent information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    /// Agent type
    pub agent_type: String,

    /// Agent description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Agent capabilities
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub capabilities: Vec<String>,

    /// Availability status
    pub availability: String,
}

/// Response for listing available agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListAgentsResponse {
    /// List of available agents
    pub agents: Vec<AgentInfo>,

    /// Total count
    pub total: usize,
}

/// Error response
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// Error message
    pub error: String,

    /// Optional details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

impl From<AgentError> for ErrorResponse {
    fn from(err: AgentError) -> Self {
        ErrorResponse {
            error: format!("{:?}", err),
            details: None,
        }
    }
}

/// Registry of available agents
#[derive(Clone)]
pub struct AgentRegistry {
    agents: Arc<RwLock<HashMap<String, Arc<dyn AgentExecutor>>>>,
}

impl AgentRegistry {
    /// Create a new agent registry
    pub fn new() -> Self {
        Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register an agent
    pub async fn register(&self, agent: Arc<dyn AgentExecutor>) {
        let agent_type = agent.agent_type().to_string();
        let mut agents = self.agents.write().await;
        agents.insert(agent_type, agent);
    }

    /// Get an agent by type
    pub async fn get(&self, agent_type: &str) -> Option<Arc<dyn AgentExecutor>> {
        let agents = self.agents.read().await;
        agents.get(agent_type).cloned()
    }

    /// List all registered agent types
    pub async fn list_types(&self) -> Vec<String> {
        let agents = self.agents.read().await;
        agents.keys().cloned().collect()
    }

    /// Get info for all agents
    pub async fn get_all_info(&self) -> Vec<AgentInfo> {
        let agents = self.agents.read().await;
        let mut infos = Vec::new();

        for (agent_type, agent) in agents.iter() {
            let description = agent.description();
            let capabilities = agent
                .capabilities()
                .into_iter()
                .map(|c| format!("{:?}", c))
                .collect();

            // Check availability (this is async, so we need to spawn a task)
            let availability =
                tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::try_current()
                        .unwrap()
                        .block_on(agent.check_availability())
                });

            infos.push(AgentInfo {
                agent_type: agent_type.clone(),
                description,
                capabilities,
                availability: format!("{:?}", availability),
            });
        }

        infos
    }
}

impl Default for AgentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Server state
///
/// Shared state across all HTTP handlers.
#[derive(Clone)]
pub struct ServerState {
    /// Agent registry
    pub agent_registry: AgentRegistry,

    /// Session manager
    pub session_manager: Arc<SessionManager>,

    /// Workspace manager
    pub workspace_manager: Arc<WorkspaceManager>,
}

impl ServerState {
    /// Create a new server state
    pub fn new(session_manager: Arc<SessionManager>, workspace_manager: Arc<WorkspaceManager>) -> Self {
        Self {
            agent_registry: AgentRegistry::new(),
            session_manager,
            workspace_manager,
        }
    }

    /// Register an agent
    pub async fn register_agent(&self, agent: Arc<dyn AgentExecutor>) {
        self.agent_registry.register(agent).await;
    }

    /// Get agent registry
    pub fn agent_registry_ref(&self) -> &AgentRegistry {
        &self.agent_registry
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lite_agent_examples::EchoAgent;

    #[test]
    fn test_spawn_request_serialization() {
        let request = SpawnRequest {
            agent_type: "echo".to_string(),
            input: "test input".to_string(),
            session_id: None,
            config: AgentConfigOptions::default(),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("echo"));
        assert!(json.contains("test input"));

        let parsed: SpawnRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.agent_type, "echo");
        assert_eq!(parsed.input, "test input");
    }

    #[test]
    fn test_spawn_response_serialization() {
        let response = SpawnResponse {
            session_id: "session-123".to_string(),
            execution_id: "exec-456".to_string(),
            agent_type: "echo".to_string(),
            status: "started".to_string(),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("session-123"));

        let parsed: SpawnResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.session_id, "session-123");
    }

    #[tokio::test]
    async fn test_agent_registry() {
        let registry = AgentRegistry::new();
        let agent: Arc<dyn AgentExecutor> = Arc::new(EchoAgent::new());

        registry.register(agent.clone()).await;

        let retrieved = registry.get("echo").await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().agent_type(), "echo");

        let types = registry.list_types().await;
        assert_eq!(types, vec!["echo".to_string()]);
    }

    #[tokio::test]
    async fn test_server_state() {
        let session_manager = Arc::new(SessionManager::new());
        let workspace_manager = Arc::new(WorkspaceManager::new(".".into()));

        let state = ServerState::new(session_manager, workspace_manager);
        let agent: Arc<dyn AgentExecutor> = Arc::new(EchoAgent::new());

        state.register_agent(agent).await;

        let retrieved = state.agent_registry_ref().get("echo").await;
        assert!(retrieved.is_some());
    }
}
