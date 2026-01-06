//! High-level agent runner API
//!
//! `AgentRunner` provides a simplified, high-level API for executing agents
//! with automatic log collection, session management, and error handling.

use std::path::PathBuf;

use futures_util::StreamExt;
use tokio::time::timeout as tokio_timeout;

use crate::agent::{AgentConfig, AgentError, AgentExecutor, AgentResult, ExitResult, SpawnedAgent};
use crate::logs::{EntryType, NormalizedEntry};
use crate::session::SessionManager;
use crate::workspace::{IsolationType, WorkspaceConfig, WorkspaceManager};

/// High-level agent runner with session management
///
/// `AgentRunner` wraps an `AgentExecutor` and provides a simplified API
/// for running agents with automatic resource management and log collection.
///
/// # Example
///
/// ```ignore
/// use lite_agent_core::{AgentRunner, AgentConfig};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let agent = MyAgent::new();
///     let runner = AgentRunner::new(agent);
///
///     let config = AgentConfig::default();
///     let result = runner.run("hello", config).await?;
///
///     println!("Output: {}", result.output);
///     Ok(())
/// }
/// ```
pub struct AgentRunner<T: AgentExecutor> {
    /// Underlying agent executor
    executor: T,

    /// Session manager for tracking executions
    session_manager: SessionManager,

    /// Workspace manager for isolation (optional)
    workspace_manager: Option<WorkspaceManager>,
}

impl<T: AgentExecutor> AgentRunner<T> {
    /// Create a new agent runner
    ///
    /// # Arguments
    ///
    /// * `executor` - Agent executor to wrap
    pub fn new(executor: T) -> Self {
        Self {
            executor,
            session_manager: SessionManager::new(),
            workspace_manager: None,
        }
    }

    /// Create a new agent runner with workspace support
    ///
    /// # Arguments
    ///
    /// * `executor` - Agent executor to wrap
    /// * `base_dir` - Base directory for workspace creation
    pub fn with_workspace(executor: T, base_dir: PathBuf) -> Self {
        Self {
            executor,
            session_manager: SessionManager::new(),
            workspace_manager: Some(WorkspaceManager::new(base_dir)),
        }
    }

    /// Run agent with input and collect output
    ///
    /// This is the simplest way to run an agent - it spawns the agent,
    /// waits for completion, and returns the collected output.
    ///
    /// # Arguments
    ///
    /// * `input` - Input/prompt for the agent
    /// * `config` - Agent configuration
    ///
    /// # Returns
    ///
    /// A `RunResult` containing exit status, normalized logs, and success status
    ///
    /// # Example
    ///
    /// ```ignore
    /// let result = runner.run("echo hello", config).await?;
    /// for entry in result.logs {
    ///     println!("{:?}", entry);
    /// }
    /// ```
    pub async fn run(&self, input: &str, mut config: AgentConfig) -> AgentResult<RunResult> {
        // Create session if workspace isolation is enabled
        let session_id = if let Some(_wsm) = &self.workspace_manager {
            if config.workspace.is_none() {
                // Auto-create workspace if not specified
                let workspace_config = WorkspaceConfig {
                    work_dir: config.work_dir.clone(),
                    isolation_type: IsolationType::TempDir,
                    base_branch: "main".to_string(),
                };
                config.workspace = Some(workspace_config);
            }

            // Create session - SessionManager generates its own ID
            let sid = self
                .session_manager
                .create_session(self.executor.agent_type().to_string(), input.to_string())
                .await?;

            Some(sid)
        } else {
            None
        };

        // Spawn agent
        let spawned = self.executor.spawn(&config, input).await?;

        // Clone log_store before wait() consumes spawned
        let log_store = spawned.log_store.clone();

        // Wait for completion with timeout if specified
        let exit_status = if let Some(duration) = config.timeout {
            tokio_timeout(duration, spawned.wait()).await
                .map_err(|_| AgentError::Timeout)?
        } else {
            spawned.wait().await
        }.map_err(AgentError::Io)?;

        // Convert ExitStatus to ExitResult
        let exit_result = if exit_status.success() {
            ExitResult::Success
        } else {
            exit_status
                .code()
                .map(|c| ExitResult::Failure(c))
                .unwrap_or(ExitResult::Interrupted)
        };

        // Normalize logs
        let logs: Vec<NormalizedEntry> = self
            .executor
            .normalize_logs(log_store)
            .collect()
            .await;

        // Build output from logs
        let output: String = logs
            .iter()
            .filter_map(|entry| {
                match &entry.entry_type {
                    EntryType::Output => Some(&entry.content),
                    _ => None,
                }
            })
            .cloned()
            .collect();

        let success = exit_result.exit_code() == Some(0);

        // Update session if created
        if let Some(sid) = session_id {
            if let Err(e) = self.session_manager.add_execution(&sid, input.to_string()).await {
                tracing::warn!("Failed to update session: {}", e);
            }
        }

        Ok(RunResult {
            exit_result,
            logs,
            output,
            success,
        })
    }

    /// Run agent and stream logs in real-time
    ///
    /// This spawns the agent and returns a stream of normalized logs
    /// as they are produced, allowing for real-time monitoring.
    ///
    /// # Arguments
    ///
    /// * `input` - Input/prompt for the agent
    /// * `config` - Agent configuration
    ///
    /// # Returns
    ///
    /// A tuple of (spawned agent, log stream)
    ///
    /// # Example
    ///
    /// ```ignore
    /// let (spawned, log_stream) = runner.run_streamed("echo hello", config).await?;
    /// tokio::spawn(async move {
    ///     futures_util::pin_mut!(log_stream);
    ///     while let Some(entry) = log_stream.next().await {
    ///         println!("{:?}", entry);
    ///     }
    /// });
    /// spawned.wait().await?;
    /// ```
    pub async fn run_streamed(
        &self,
        input: &str,
        config: &AgentConfig,
    ) -> AgentResult<(SpawnedAgent, futures_util::stream::BoxStream<'static, NormalizedEntry>)> {
        let spawned = self.executor.spawn(config, input).await?;
        let log_stream = self.executor.normalize_logs(spawned.log_store.clone());

        Ok((spawned, log_stream))
    }

    /// Continue an existing session
    ///
    /// This allows conversation continuity by continuing from a previous session.
    /// Only works if the agent supports session continuation.
    ///
    /// # Arguments
    ///
    /// * `session_id` - ID of the session to continue
    /// * `input` - Follow-up input/prompt
    /// * `config` - Agent configuration
    ///
    /// # Returns
    ///
    /// A `RunResult` containing exit status, logs, and output
    pub async fn continue_session(
        &self,
        session_id: &str,
        input: &str,
        config: &AgentConfig,
    ) -> AgentResult<RunResult> {
        // Verify session exists
        let session = self
            .session_manager
            .get_session(session_id)
            .await
            .ok_or_else(|| AgentError::SessionNotFound(session_id.to_string()))?;

        if session.agent_type != self.executor.agent_type() {
            return Err(AgentError::Custom(format!(
                "Session agent type mismatch: expected {}, got {}",
                self.executor.agent_type(),
                session.agent_type
            )));
        }

        // Spawn follow-up
        let spawned = self
            .executor
            .spawn_follow_up(config, input, session_id)
            .await?;

        // Clone log_store before wait() consumes spawned
        let log_store = spawned.log_store.clone();

        // Wait for completion
        let exit_status = spawned.wait().await.map_err(AgentError::Io)?;

        // Convert ExitStatus to ExitResult
        let exit_result = if exit_status.success() {
            ExitResult::Success
        } else {
            exit_status
                .code()
                .map(|c| ExitResult::Failure(c))
                .unwrap_or(ExitResult::Interrupted)
        };

        // Normalize logs
        let logs: Vec<NormalizedEntry> = self
            .executor
            .normalize_logs(log_store)
            .collect()
            .await;

        // Build output
        let output: String = logs
            .iter()
            .filter_map(|entry| {
                match &entry.entry_type {
                    EntryType::Output => Some(&entry.content),
                    _ => None,
                }
            })
            .cloned()
            .collect();

        let success = exit_result.exit_code() == Some(0);

        // Update session
        if let Err(e) = self.session_manager.add_execution(session_id, input.to_string()).await {
            tracing::warn!("Failed to update session: {}", e);
        }

        Ok(RunResult {
            exit_result,
            logs,
            output,
            success,
        })
    }

    /// Get reference to underlying executor
    pub fn executor(&self) -> &T {
        &self.executor
    }

    /// Get reference to session manager
    pub fn session_manager(&self) -> &SessionManager {
        &self.session_manager
    }

    /// Get reference to workspace manager (if available)
    pub fn workspace_manager(&self) -> Option<&WorkspaceManager> {
        self.workspace_manager.as_ref()
    }

    /// Consume and return the inner executor
    pub fn into_inner(self) -> T {
        self.executor
    }
}

/// Result of an agent execution
///
/// Contains the exit status, normalized logs, and output text.
#[derive(Debug, Clone)]
pub struct RunResult {
    /// Exit result of the agent process
    pub exit_result: ExitResult,

    /// Normalized log entries
    pub logs: Vec<NormalizedEntry>,

    /// Concatenated output from Output-type log entries
    pub output: String,

    /// Whether execution was successful
    pub success: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logs::LogStore;
    use async_trait::async_trait;
    use futures_util::stream::BoxStream;
    use std::sync::Arc;

    // Simple mock agent for testing
    struct TestAgent;

    #[async_trait]
    impl AgentExecutor for TestAgent {
        fn agent_type(&self) -> &str {
            "test"
        }

        async fn spawn(
            &self,
            _config: &AgentConfig,
            _input: &str,
        ) -> Result<SpawnedAgent, AgentError> {
            Err(AgentError::Custom("Test agent".to_string()))
        }

        fn normalize_logs(
            &self,
            _raw_logs: Arc<LogStore>,
        ) -> BoxStream<'static, NormalizedEntry> {
            futures_util::stream::empty().boxed()
        }
    }

    #[tokio::test]
    async fn test_runner_creation() {
        let agent = TestAgent;
        let runner = AgentRunner::new(agent);
        assert_eq!(runner.executor().agent_type(), "test");
    }

    #[tokio::test]
    async fn test_runner_with_workspace() {
        let agent = TestAgent;
        let temp_dir = tempfile::tempdir().unwrap();
        let runner = AgentRunner::with_workspace(agent, temp_dir.path().to_path_buf());
        assert!(runner.workspace_manager().is_some());
    }
}
