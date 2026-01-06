//! Log collection from process stdout/stderr
//!
//! Provides utilities for collecting and streaming logs from spawned agent processes.

use crate::logs::LogStore;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::ChildStdout;
use tokio::task::JoinHandle;

/// Log collector for capturing stdout/stderr from processes
///
/// Spawns background tasks that read from process stdout/stderr and
/// store the output in a LogStore.
#[derive(Debug)]
pub struct LogCollector {
    agent_type: String,
    log_store: Arc<LogStore>,
    _stdout_handle: Option<JoinHandle<()>>,
    _stderr_handle: Option<JoinHandle<()>>,
}

impl LogCollector {
    /// Create a new log collector
    pub fn new(agent_type: String, log_store: Arc<LogStore>) -> Self {
        Self {
            agent_type,
            log_store,
            _stdout_handle: None,
            _stderr_handle: None,
        }
    }

    /// Start collecting stdout from a process
    ///
    /// Spawns a background task that reads line-by-line from stdout
    /// and stores each line in the log store.
    pub fn collect_stdout(&mut self, stdout: ChildStdout) {
        let agent_type = self.agent_type.clone();
        let log_store = self.log_store.clone();

        let handle = tokio::spawn(async move {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();

            while let Ok(Some(line)) = lines.next_line().await {
                log_store.add_stdout(line, agent_type.clone()).await;
            }
        });

        self._stdout_handle = Some(handle);
    }

    /// Start collecting stderr from a process
    ///
    /// Spawns a background task that reads line-by-line from stderr
    /// and stores each line in the log store.
    pub fn collect_stderr(&mut self, stderr: tokio::process::ChildStderr) {
        let agent_type = self.agent_type.clone();
        let log_store = self.log_store.clone();

        let handle = tokio::spawn(async move {
            let reader = BufReader::new(stderr);
            let mut lines = reader.lines();

            while let Ok(Some(line)) = lines.next_line().await {
                log_store.add_stderr(line, agent_type.clone()).await;
            }
        });

        self._stderr_handle = Some(handle);
    }

    /// Start collecting both stdout and stderr
    ///
    /// Convenience method that starts collection on both streams.
    pub fn collect_both(
        &mut self,
        stdout: ChildStdout,
        stderr: tokio::process::ChildStderr,
    ) {
        self.collect_stdout(stdout);
        self.collect_stderr(stderr);
    }

    /// Get a reference to the log store
    pub fn log_store(&self) -> &Arc<LogStore> {
        &self.log_store
    }
}

/// Builder for creating log collectors
pub struct LogCollectorBuilder {
    agent_type: String,
    capacity: usize,
}

impl LogCollectorBuilder {
    /// Create a new log collector builder
    pub fn new(agent_type: String) -> Self {
        Self {
            agent_type,
            capacity: 1000,
        }
    }

    /// Set the capacity for the log store
    pub fn with_capacity(mut self, capacity: usize) -> Self {
        self.capacity = capacity;
        self
    }

    /// Build the log collector
    pub fn build(self) -> LogCollector {
        let log_store = Arc::new(LogStore::with_capacity(self.capacity));
        LogCollector::new(self.agent_type, log_store)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_log_collector_builder() {
        let collector = LogCollectorBuilder::new("test-agent".to_string())
            .with_capacity(500)
            .build();

        assert_eq!(collector.log_store().len().await, 0);
    }
}
