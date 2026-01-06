"""
Unit tests for data models.
"""

import pytest
from datetime import datetime
from pathlib import Path

from lite_agent_client.models import (
    AgentConfigOptions,
    SpawnRequest,
    SpawnResponse,
    SessionStatusResponse,
    ListSessionsResponse,
    AgentInfo,
    ListAgentsResponse,
    LogEntry,
    LogEntryType,
    ErrorResponse,
    HealthResponse,
)


class TestAgentConfigOptions:
    """Tests for AgentConfigOptions model."""

    def test_default_config(self):
        """Test default configuration."""
        config = AgentConfigOptions()
        assert config.work_dir is None
        assert config.env == {}
        assert config.timeout_secs is None
        assert config.custom is None

    def test_config_with_values(self):
        """Test configuration with values."""
        config = AgentConfigOptions(
            work_dir=Path("/tmp"),
            env={"KEY": "value"},
            timeout_secs=60,
            custom={"key": "value"},
        )
        assert config.work_dir == Path("/tmp")
        assert config.env == {"KEY": "value"}
        assert config.timeout_secs == 60
        assert config.custom == {"key": "value"}

    def test_config_serialization(self):
        """Test configuration serialization."""
        config = AgentConfigOptions(
            work_dir=Path("/tmp"),
            env={"KEY": "value"},
        )
        data = config.model_dump(exclude_none=True)
        assert "work_dir" in data
        assert "env" in data
        assert data["env"] == {"KEY": "value"}


class TestSpawnRequest:
    """Tests for SpawnRequest model."""

    def test_minimal_request(self):
        """Test minimal spawn request."""
        request = SpawnRequest(agent_type="echo", input="hello")
        assert request.agent_type == "echo"
        assert request.input == "hello"
        assert request.session_id is None
        assert isinstance(request.config, AgentConfigOptions)

    def test_full_request(self):
        """Test spawn request with all fields."""
        config = AgentConfigOptions(timeout_secs=30)
        request = SpawnRequest(
            agent_type="shell",
            input="ls -la",
            session_id="session-123",
            config=config,
        )
        assert request.agent_type == "shell"
        assert request.input == "ls -la"
        assert request.session_id == "session-123"
        assert request.config.timeout_secs == 30

    def test_request_serialization(self):
        """Test spawn request serialization."""
        request = SpawnRequest(agent_type="echo", input="hello")
        data = request.model_dump(exclude_none=True)
        assert data["agent_type"] == "echo"
        assert data["input"] == "hello"
        assert "session_id" not in data


class TestSpawnResponse:
    """Tests for SpawnResponse model."""

    def test_spawn_response(self):
        """Test spawn response parsing."""
        response = SpawnResponse(
            session_id="session-123",
            execution_id="exec-456",
            agent_type="echo",
            status="started",
        )
        assert response.session_id == "session-123"
        assert response.execution_id == "exec-456"
        assert response.agent_type == "echo"
        assert response.status == "started"

    def test_spawn_response_from_dict(self):
        """Test spawn response from dictionary."""
        data = {
            "session_id": "session-123",
            "execution_id": "exec-456",
            "agent_type": "echo",
            "status": "started",
        }
        response = SpawnResponse(**data)
        assert response.session_id == "session-123"


class TestSessionStatusResponse:
    """Tests for SessionStatusResponse model."""

    def test_session_status(self):
        """Test session status parsing."""
        response = SessionStatusResponse(
            session_id="session-123",
            agent_type="echo",
            status="Active",
            execution_count=2,
            created_at="2024-01-01T00:00:00Z",
            updated_at="2024-01-01T01:00:00Z",
        )
        assert response.session_id == "session-123"
        assert response.execution_count == 2

    def test_datetime_parsing(self):
        """Test datetime parsing properties."""
        response = SessionStatusResponse(
            session_id="session-123",
            agent_type="echo",
            status="Active",
            execution_count=1,
            created_at="2024-01-01T00:00:00Z",
            updated_at="2024-01-01T01:00:00Z",
        )
        created = response.created_datetime
        updated = response.updated_datetime
        assert isinstance(created, datetime)
        assert isinstance(updated, datetime)
        assert updated > created


class TestListSessionsResponse:
    """Tests for ListSessionsResponse model."""

    def test_empty_sessions(self):
        """Test empty sessions list."""
        response = ListSessionsResponse(sessions=[], total=0)
        assert response.total == 0
        assert len(response.sessions) == 0

    def test_multiple_sessions(self):
        """Test multiple sessions."""
        sessions = [
            SessionStatusResponse(
                session_id=f"session-{i}",
                agent_type="echo",
                status="Active",
                execution_count=1,
                created_at="2024-01-01T00:00:00Z",
                updated_at="2024-01-01T01:00:00Z",
            )
            for i in range(3)
        ]
        response = ListSessionsResponse(sessions=sessions, total=3)
        assert response.total == 3
        assert len(response.sessions) == 3


class TestAgentInfo:
    """Tests for AgentInfo model."""

    def test_minimal_agent_info(self):
        """Test minimal agent info."""
        info = AgentInfo(agent_type="echo", availability="Available")
        assert info.agent_type == "echo"
        assert info.availability == "Available"
        assert info.description is None
        assert info.capabilities == []

    def test_full_agent_info(self):
        """Test full agent info."""
        info = AgentInfo(
            agent_type="shell",
            description="Shell command executor",
            capabilities=["execute", "stream"],
            availability="Available",
        )
        assert info.agent_type == "shell"
        assert info.description == "Shell command executor"
        assert len(info.capabilities) == 2


class TestListAgentsResponse:
    """Tests for ListAgentsResponse model."""

    def test_empty_agents(self):
        """Test empty agents list."""
        response = ListAgentsResponse(agents=[], total=0)
        assert response.total == 0
        assert len(response.agents) == 0

    def test_multiple_agents(self):
        """Test multiple agents."""
        agents = [
            AgentInfo(
                agent_type=f"agent-{i}",
                availability="Available",
            )
            for i in range(2)
        ]
        response = ListAgentsResponse(agents=agents, total=2)
        assert response.total == 2
        assert len(response.agents) == 2


class TestLogEntry:
    """Tests for LogEntry model."""

    def test_log_entry(self):
        """Test log entry parsing."""
        entry = LogEntry(
            timestamp="2024-01-01T00:00:00Z",
            level=LogEntryType.STDOUT,
            content="Hello, world!",
        )
        assert entry.level == LogEntryType.STDOUT
        assert entry.content == "Hello, world!"

    def test_datetime_parsing(self):
        """Test datetime parsing property."""
        entry = LogEntry(
            timestamp="2024-01-01T00:00:00Z",
            level=LogEntryType.INFO,
            content="Test",
        )
        dt = entry.datetime
        assert isinstance(dt, datetime)

    def test_log_entry_from_dict(self):
        """Test log entry from dictionary with alias."""
        data = {
            "timestamp": "2024-01-01T00:00:00Z",
            "entry_type": "stdout",
            "content": "Output",
        }
        entry = LogEntry(**data)
        assert entry.level == "stdout"
        assert entry.content == "Output"


class TestErrorResponse:
    """Tests for ErrorResponse model."""

    def test_error_response(self):
        """Test error response."""
        response = ErrorResponse(error="Not found", details="Session does not exist")
        assert response.error == "Not found"
        assert response.details == "Session does not exist"

    def test_error_response_without_details(self):
        """Test error response without details."""
        response = ErrorResponse(error="Internal error")
        assert response.error == "Internal error"
        assert response.details is None


class TestHealthResponse:
    """Tests for HealthResponse model."""

    def test_health_response(self):
        """Test health response."""
        response = HealthResponse(status="healthy", timestamp=1704067200)
        assert response.status == "healthy"
        assert response.timestamp == 1704067200
