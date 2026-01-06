use command_group::AsyncGroupChild;
use std::sync::Arc;
use tokio::process::{ChildStderr, ChildStdin, ChildStdout};
use tokio::sync::oneshot;

use crate::logs::LogStore;

/// Result of an agent exit
#[derive(Debug, Clone, Copy)]
pub enum ExitResult {
    /// Process completed successfully (exit code 0)
    Success,
    /// Process failed (non-zero exit code)
    Failure(i32),
    /// Process was interrupted
    Interrupted,
}

impl ExitResult {
    /// Get the exit code (if available)
    ///
    /// Returns `Some(0)` for success, `Some(code)` for failure,
    /// and `None` for interrupted processes.
    pub fn exit_code(&self) -> Option<i32> {
        match self {
            ExitResult::Success => Some(0),
            ExitResult::Failure(code) => Some(*code),
            ExitResult::Interrupted => None,
        }
    }
}

/// Result of spawning an agent
///
/// This struct contains the spawned process handle and communication channels.
pub struct SpawnedAgent {
    /// The spawned process (process group)
    pub child: AsyncGroupChild,

    /// Stdin handle for sending input to the agent
    pub stdin: Option<ChildStdin>,

    /// Stdout handle for reading output from the agent
    pub stdout: Option<ChildStdout>,

    /// Stderr handle for reading errors from the agent
    pub stderr: Option<ChildStderr>,

    /// Exit signal receiver (agent → manager)
    /// When this resolves, the agent has completed execution
    pub exit_signal: Option<oneshot::Receiver<ExitResult>>,

    /// Interrupt signal sender (manager → agent)
    /// Send a signal to request graceful interruption
    pub interrupt_signal: Option<oneshot::Sender<()>>,

    /// Log store for this execution
    pub log_store: Arc<LogStore>,
}

impl SpawnedAgent {
    /// Create a new SpawnedAgent from a process handle
    pub fn new(child: AsyncGroupChild, log_store: Arc<LogStore>) -> Self {
        Self {
            child,
            stdin: None,
            stdout: None,
            stderr: None,
            exit_signal: None,
            interrupt_signal: None,
            log_store,
        }
    }

    /// Set stdin handle
    pub fn with_stdin(mut self, stdin: ChildStdin) -> Self {
        self.stdin = Some(stdin);
        self
    }

    /// Set stdout handle
    pub fn with_stdout(mut self, stdout: ChildStdout) -> Self {
        self.stdout = Some(stdout);
        self
    }

    /// Set stderr handle
    pub fn with_stderr(mut self, stderr: ChildStderr) -> Self {
        self.stderr = Some(stderr);
        self
    }

    /// Set exit signal channel
    pub fn with_exit_signal(mut self, exit_signal: oneshot::Receiver<ExitResult>) -> Self {
        self.exit_signal = Some(exit_signal);
        self
    }

    /// Set interrupt signal channel
    pub fn with_interrupt_signal(mut self, interrupt_signal: oneshot::Sender<()>) -> Self {
        self.interrupt_signal = Some(interrupt_signal);
        self
    }

    /// Wait for the agent to complete
    pub async fn wait(mut self) -> std::io::Result<std::process::ExitStatus> {
        self.child.wait().await
    }

    /// Kill the agent process
    pub async fn kill(mut self) -> std::io::Result<()> {
        self.child.kill().await
    }
}

impl From<AsyncGroupChild> for SpawnedAgent {
    fn from(child: AsyncGroupChild) -> Self {
        Self::new(child, Arc::new(LogStore::new()))
    }
}
