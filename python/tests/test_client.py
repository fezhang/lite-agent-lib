"""
Unit tests for the LiteAgentClient.
"""

import asyncio
import json
from datetime import datetime
from unittest.mock import AsyncMock, Mock, MagicMock, patch

import pytest
from aiohttp import ClientSession, ClientError
from aiohttp.test_utils import TestClient, TestServer

from lite_agent_client import LiteAgentClient, SpawnRequest
from lite_agent_client.client import LiteAgentClientError
from lite_agent_client.models import (
    SpawnResponse,
    SessionStatusResponse,
    ListSessionsResponse,
    ListAgentsResponse,
    LogEntry,
    LogEntryType,
    HealthResponse,
)


class AsyncContextManagerMock:
    """Helper for mocking async context managers."""

    def __init__(self, return_value):
        self._return_value = return_value

    async def __aenter__(self):
        return self._return_value

    async def __aexit__(self, *args):
        pass

    def __call__(self, *args, **kwargs):
        """Make it callable to return itself."""
        return self


class MockAiohttpResponse:
    """Mock aiohttp response."""

    def __init__(self, status: int, json_data: dict = None, text_data: str = None):
        self.status = status
        self._json_data = json_data or {}
        self._text_data = text_data or ""
        self.closed = False

    async def json(self):
        return self._json_data

    async def text(self):
        return self._text_data

    async def __aenter__(self):
        return self

    async def __aexit__(self, *args):
        pass

    def close(self):
        self.closed = True


@pytest.fixture
async def client():
    """Create a client instance for testing."""
    async with LiteAgentClient("http://localhost:3000") as client:
        yield client


@pytest.fixture
def mock_session():
    """Create a mock session with properly configured methods."""

    # Create a mock session
    session = MagicMock(spec=ClientSession)
    session.closed = False

    # Make close a no-op async method
    async def mock_close():
        session.closed = True

    session.close = mock_close

    return session


class TestLiteAgentClient:
    """Tests for LiteAgentClient class."""

    def test_client_initialization(self):
        """Test client initialization."""
        client = LiteAgentClient("http://localhost:3000")
        assert client.base_url.host == "localhost"
        assert client.base_url.port == 3000
        assert client._session is None

    def test_client_initialization_with_timeout(self):
        """Test client initialization with custom timeout."""
        client = LiteAgentClient("http://localhost:3000", timeout=600.0)
        assert client.timeout.total == 600.0

    @pytest.mark.asyncio
    async def test_client_context_manager(self):
        """Test client as context manager."""
        async with LiteAgentClient("http://localhost:3000") as client:
            assert client._session is not None
            assert not client._session.closed

        # Session should be closed after exiting context
        assert client._session is None or client._session.closed

    @pytest.mark.asyncio
    async def test_client_start_close(self):
        """Test manual start and close."""
        client = LiteAgentClient("http://localhost:3000")
        assert client._session is None

        await client.start()
        assert client._session is not None

        await client.close()
        assert client._session is None or client._session.closed

    @pytest.mark.asyncio
    async def test_session_property_without_start(self):
        """Test session property raises error when not started."""
        client = LiteAgentClient("http://localhost:3000")
        with pytest.raises(RuntimeError, match="Client session not initialized"):
            _ = client.session


class TestHandleResponse:
    """Tests for _handle_response method."""

    @pytest.mark.asyncio
    async def test_handle_success_response(self, client):
        """Test handling successful response."""
        mock_response = Mock()
        mock_response.status = 200
        mock_response.json = AsyncMock(return_value={"key": "value"})

        result = await client._handle_response(mock_response)
        assert result == {"key": "value"}

    @pytest.mark.asyncio
    async def test_handle_error_response_with_json(self, client):
        """Test handling error response with JSON body."""
        mock_response = Mock()
        mock_response.status = 404
        mock_response.json = AsyncMock(
            return_value={"error": "Not found", "details": "Session does not exist"}
        )

        with pytest.raises(LiteAgentClientError) as exc_info:
            await client._handle_response(mock_response)

        assert exc_info.value.message == "Not found"
        assert exc_info.value.details == "Session does not exist"

    @pytest.mark.asyncio
    async def test_handle_error_response_without_json(self, client):
        """Test handling error response without JSON body."""
        mock_response = Mock()
        mock_response.status = 500
        # Use json.JSONDecodeError to simulate actual JSON parsing failure
        mock_response.json = AsyncMock(side_effect=json.JSONDecodeError("Invalid JSON", "", 0))
        mock_response.text = AsyncMock(return_value="Internal Server Error")

        with pytest.raises(LiteAgentClientError) as exc_info:
            await client._handle_response(mock_response)

        # The error should contain HTTP 500 information
        assert "500" in str(exc_info.value) or "HTTP" in str(exc_info.value)


class TestSpawnAgent:
    """Tests for spawn_agent method."""

    @pytest.mark.asyncio
    async def test_spawn_agent_success(self, client, mock_session):
        """Test successful agent spawn."""
        client._session = mock_session

        mock_response = MockAiohttpResponse(
            status=201,
            json_data={
                "session_id": "session-123",
                "execution_id": "exec-456",
                "agent_type": "echo",
                "status": "started",
            }
        )

        # Directly assign the context manager - it's callable now
        mock_session.post = AsyncContextManagerMock(mock_response)

        request = SpawnRequest(agent_type="echo", input="hello")
        response = await client.spawn_agent(request)

        assert response.session_id == "session-123"
        assert response.execution_id == "exec-456"
        assert response.agent_type == "echo"
        assert response.status == "started"

    @pytest.mark.asyncio
    async def test_spawn_agent_not_found(self, client, mock_session):
        """Test spawn agent with unknown type."""
        client._session = mock_session

        mock_response = MockAiohttpResponse(
            status=404,
            json_data={"error": "Agent type 'unknown' not found"}
        )

        mock_session.post = AsyncContextManagerMock(mock_response)

        request = SpawnRequest(agent_type="unknown", input="test")

        with pytest.raises(LiteAgentClientError, match="not found"):
            await client.spawn_agent(request)


class TestGetSessionStatus:
    """Tests for get_session_status method."""

    @pytest.mark.asyncio
    async def test_get_session_status_success(self, client, mock_session):
        """Test successful session status retrieval."""
        client._session = mock_session

        mock_response = MockAiohttpResponse(
            status=200,
            json_data={
                "session_id": "session-123",
                "agent_type": "echo",
                "status": "Active",
                "execution_count": 2,
                "created_at": "2024-01-01T00:00:00Z",
                "updated_at": "2024-01-01T01:00:00Z",
            }
        )

        mock_session.get = AsyncContextManagerMock(mock_response)

        status = await client.get_session_status("session-123")

        assert status.session_id == "session-123"
        assert status.execution_count == 2
        assert status.status == "Active"


class TestListSessions:
    """Tests for list_sessions method."""

    @pytest.mark.asyncio
    async def test_list_sessions_success(self, client, mock_session):
        """Test successful sessions listing."""
        client._session = mock_session

        mock_response = MockAiohttpResponse(
            status=200,
            json_data={
                "sessions": [
                    {
                        "session_id": "session-1",
                        "agent_type": "echo",
                        "status": "Active",
                        "execution_count": 1,
                        "created_at": "2024-01-01T00:00:00Z",
                        "updated_at": "2024-01-01T01:00:00Z",
                    }
                ],
                "total": 1,
            }
        )

        mock_session.get = AsyncContextManagerMock(mock_response)

        result = await client.list_sessions()

        assert result.total == 1
        assert len(result.sessions) == 1
        assert result.sessions[0].session_id == "session-1"


class TestDeleteSession:
    """Tests for delete_session method."""

    @pytest.mark.asyncio
    async def test_delete_session_success(self, client, mock_session):
        """Test successful session deletion."""
        client._session = mock_session

        mock_response = MockAiohttpResponse(status=204)

        mock_session.delete = AsyncContextManagerMock(mock_response)

        # Should not raise any exception
        await client.delete_session("session-123")

    @pytest.mark.asyncio
    async def test_delete_session_not_found(self, client, mock_session):
        """Test deleting non-existent session."""
        client._session = mock_session

        mock_response = MockAiohttpResponse(
            status=404,
            json_data={"error": "Session not found"}
        )

        mock_session.delete = AsyncContextManagerMock(mock_response)

        with pytest.raises(LiteAgentClientError, match="not found"):
            await client.delete_session("unknown-session")


class TestListAgents:
    """Tests for list_agents method."""

    @pytest.mark.asyncio
    async def test_list_agents_success(self, client, mock_session):
        """Test successful agents listing."""
        client._session = mock_session

        mock_response = MockAiohttpResponse(
            status=200,
            json_data={
                "agents": [
                    {
                        "agent_type": "echo",
                        "description": "Echo agent",
                        "capabilities": ["execute"],
                        "availability": "Available",
                    },
                    {
                        "agent_type": "shell",
                        "description": "Shell command executor",
                        "capabilities": ["execute", "stream"],
                        "availability": "Available",
                    },
                ],
                "total": 2,
            }
        )

        mock_session.get = AsyncContextManagerMock(mock_response)

        result = await client.list_agents()

        assert result.total == 2
        assert len(result.agents) == 2
        assert result.agents[0].agent_type == "echo"
        assert result.agents[1].agent_type == "shell"


class TestStreamLogs:
    """Tests for stream_logs method."""

    @pytest.mark.asyncio
    async def test_stream_logs_success(self, client, mock_session):
        """Test successful log streaming."""
        client._session = mock_session

        # Create mock SSE stream
        async def mock_content_iterator():
            log_data = {
                "timestamp": "2024-01-01T00:00:00Z",
                "entry_type": "stdout",
                "content": "Hello, world!",
            }
            yield f"data: {json.dumps(log_data)}\n\n".encode()
            yield b": keepalive\n\n"
            yield b"data: {\"type\": \"stream_ended\"}\n\n"

        mock_response = Mock()
        mock_response.status = 200
        mock_response.content = mock_content_iterator()

        mock_session.get = AsyncContextManagerMock(mock_response)

        logs = []
        async for log in client.stream_logs("session-123"):
            logs.append(log)

        assert len(logs) == 1
        assert logs[0].content == "Hello, world!"
        assert logs[0].level == "stdout"

    @pytest.mark.asyncio
    async def test_stream_logs_session_not_found(self, client, mock_session):
        """Test streaming logs for non-existent session."""
        client._session = mock_session

        mock_response = Mock()
        mock_response.status = 404

        mock_session.get = AsyncContextManagerMock(mock_response)

        with pytest.raises(LiteAgentClientError, match="not found"):
            async for _ in client.stream_logs("unknown-session"):
                pass

    @pytest.mark.asyncio
    async def test_stream_logs_with_control_events(self, client, mock_session):
        """Test log streaming with control events."""
        client._session = mock_session

        async def mock_content_iterator():
            yield b"data: {\"type\": \"stream_started\", \"session_id\": \"session-123\"}\n\n"
            log_data = {
                "timestamp": "2024-01-01T00:00:00Z",
                "entry_type": "info",
                "content": "Processing",
            }
            yield f"data: {json.dumps(log_data)}\n\n".encode()
            yield b"data: {\"type\": \"stream_ended\"}\n\n"

        mock_response = Mock()
        mock_response.status = 200
        mock_response.content = mock_content_iterator()

        mock_session.get = AsyncContextManagerMock(mock_response)

        logs = []
        async for log in client.stream_logs("session-123"):
            logs.append(log)

        # Control events should be filtered
        assert len(logs) == 1
        assert logs[0].content == "Processing"


class TestHealthCheck:
    """Tests for health_check method."""

    @pytest.mark.asyncio
    async def test_health_check_success(self, client, mock_session):
        """Test successful health check."""
        client._session = mock_session

        mock_response = MockAiohttpResponse(
            status=200,
            json_data={
                "status": "healthy",
                "timestamp": 1704067200,
            }
        )

        mock_session.get = AsyncContextManagerMock(mock_response)

        health = await client.health_check()

        assert health.status == "healthy"
        assert health.timestamp == 1704067200


class TestSpawnAndStream:
    """Tests for spawn_and_stream convenience method."""

    @pytest.mark.asyncio
    async def test_spawn_and_stream(self, client, mock_session):
        """Test spawn and stream combined."""
        client._session = mock_session

        # Mock spawn response
        mock_spawn_response = MockAiohttpResponse(
            status=201,
            json_data={
                "session_id": "session-123",
                "execution_id": "exec-456",
                "agent_type": "echo",
                "status": "started",
            }
        )

        # Mock log stream
        async def mock_content_iterator():
            log_data = {
                "timestamp": "2024-01-01T00:00:00Z",
                "entry_type": "stdout",
                "content": "Echoed: hello",
            }
            yield f"data: {json.dumps(log_data)}\n\n".encode()
            yield b"data: {\"type\": \"stream_ended\"}\n\n"

        mock_stream_response = Mock()
        mock_stream_response.status = 200
        mock_stream_response.content = mock_content_iterator()

        # Set up both post and get mocks
        mock_session.post = AsyncContextManagerMock(mock_spawn_response)
        mock_session.get = AsyncContextManagerMock(mock_stream_response)

        request = SpawnRequest(agent_type="echo", input="hello")
        logs = []

        async for log in client.spawn_and_stream(request):
            logs.append(log)

        assert len(logs) == 1
        assert logs[0].content == "Echoed: hello"
