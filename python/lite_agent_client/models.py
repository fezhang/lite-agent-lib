"""
Data models for the lite-agent client.

These models match the Rust API models in lite-agent-server.
"""

from typing import Any, Dict, List, Optional
from datetime import datetime
from pathlib import Path

from pydantic import BaseModel, ConfigDict, Field


class AgentConfigOptions(BaseModel):
    """Optional configuration for agent execution."""

    work_dir: Optional[Path] = Field(None, description="Working directory (defaults to current directory)")
    env: Dict[str, str] = Field(default_factory=dict, description="Environment variables")
    timeout_secs: Optional[int] = Field(None, description="Timeout in seconds")
    custom: Optional[Dict[str, Any]] = Field(None, description="Custom agent-specific configuration")


class SpawnRequest(BaseModel):
    """Request to spawn an agent."""

    agent_type: str = Field(..., description="Type of agent to spawn (e.g., 'shell', 'echo')")
    input: str = Field(..., description="Input/prompt for the agent")
    session_id: Optional[str] = Field(None, description="Optional session ID to continue")
    config: AgentConfigOptions = Field(
        default_factory=AgentConfigOptions, description="Agent configuration"
    )


class SpawnResponse(BaseModel):
    """Response from spawning an agent."""

    session_id: str = Field(..., description="Session ID")
    execution_id: str = Field(..., description="Execution ID")
    agent_type: str = Field(..., description="Agent type")
    status: str = Field(..., description="Status of the spawn operation")


class SessionStatusResponse(BaseModel):
    """Response for getting session status."""

    session_id: str = Field(..., description="Session ID")
    agent_type: str = Field(..., description="Agent type")
    status: str = Field(..., description="Session status")
    execution_count: int = Field(..., description="Number of executions")
    created_at: str = Field(..., description="Creation timestamp (RFC3339)")
    updated_at: str = Field(..., description="Last update timestamp (RFC3339)")

    @property
    def created_datetime(self) -> datetime:
        """Parse created_at as datetime."""
        return datetime.fromisoformat(self.created_at.replace("Z", "+00:00"))

    @property
    def updated_datetime(self) -> datetime:
        """Parse updated_at as datetime."""
        return datetime.fromisoformat(self.updated_at.replace("Z", "+00:00"))


class ListSessionsResponse(BaseModel):
    """Response for listing sessions."""

    sessions: List[SessionStatusResponse] = Field(default_factory=list, description="List of sessions")
    total: int = Field(..., description="Total count")


class AgentInfo(BaseModel):
    """Agent information."""

    agent_type: str = Field(..., description="Agent type")
    description: Optional[str] = Field(None, description="Agent description")
    capabilities: List[str] = Field(default_factory=list, description="Agent capabilities")
    availability: str = Field(..., description="Availability status")


class ListAgentsResponse(BaseModel):
    """Response for listing available agents."""

    agents: List[AgentInfo] = Field(default_factory=list, description="List of available agents")
    total: int = Field(..., description="Total count")


class LogEntryType:
    """Log entry types matching the Rust implementation."""

    STDOUT = "stdout"
    STDERR = "stderr"
    INFO = "info"
    WARNING = "warning"
    ERROR = "error"
    DEBUG = "debug"


class LogEntry(BaseModel):
    """A single log entry from an agent."""

    model_config = ConfigDict(populate_by_name=True)

    timestamp: str = Field(..., description="Timestamp of the log entry")
    level: str = Field(..., alias="entry_type", description="Log level/type")
    content: str = Field(..., description="Log content")

    @property
    def datetime(self) -> datetime:
        """Parse timestamp as datetime."""
        return datetime.fromisoformat(self.timestamp.replace("Z", "+00:00"))


class ErrorResponse(BaseModel):
    """Error response from the API."""

    error: str = Field(..., description="Error message")
    details: Optional[str] = Field(None, description="Optional details")


class HealthResponse(BaseModel):
    """Health check response."""

    status: str = Field(..., description="Health status")
    timestamp: int = Field(..., description="Unix timestamp")
