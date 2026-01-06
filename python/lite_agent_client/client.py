"""
Main client implementation for the lite-agent Python client.
"""

import asyncio
import json
from contextlib import asynccontextmanager
from typing import AsyncIterator, Optional, Dict, Any
from urllib.parse import urljoin

import aiohttp
from aiohttp import ClientSession, ClientTimeout, ContentTypeError
from yarl import URL

from lite_agent_client.models import (
    SpawnRequest,
    SpawnResponse,
    SessionStatusResponse,
    ListSessionsResponse,
    AgentInfo,
    ListAgentsResponse,
    LogEntry,
    ErrorResponse,
    HealthResponse,
)


class LiteAgentClientError(Exception):
    """Base exception for client errors."""

    def __init__(self, message: str, details: Optional[str] = None):
        self.message = message
        self.details = details
        super().__init__(self.message)


class LiteAgentClient:
    """
    Async client for interacting with the lite-agent REST API.

    Example:
        ```python
        async with LiteAgentClient("http://localhost:3000") as client:
            response = await client.spawn_agent(SpawnRequest(
                agent_type="shell",
                input="echo hello",
                config={}
            ))

            async for log in client.stream_logs(response.session_id):
                print(f"[{log.level}] {log.content}")
        ```
    """

    def __init__(
        self,
        base_url: str,
        timeout: float = 300.0,
        verify_ssl: bool = True,
    ):
        """
        Initialize the client.

        Args:
            base_url: Base URL of the lite-agent server (e.g., "http://localhost:3000")
            timeout: Request timeout in seconds (default: 300)
            verify_ssl: Whether to verify SSL certificates (default: True)
        """
        self.base_url = URL(base_url)
        self.timeout = ClientTimeout(total=timeout)
        self.verify_ssl = verify_ssl
        self._session: Optional[ClientSession] = None
        self._owner = False

    async def __aenter__(self) -> "LiteAgentClient":
        """Enter context manager and create session."""
        await self.start()
        return self

    async def __aexit__(self, exc_type, exc_val, exc_tb) -> None:
        """Exit context manager and close session."""
        await self.close()

    async def start(self) -> None:
        """Start the client session."""
        if self._session is None:
            self._session = ClientSession(
                base_url=self.base_url,
                timeout=self.timeout,
                connector=aiohttp.TCPConnector(ssl=self.verify_ssl),
            )
            self._owner = True

    async def close(self) -> None:
        """Close the client session."""
        if self._session is not None and self._owner:
            await self._session.close()
            self._session = None

    @property
    def session(self) -> ClientSession:
        """Get or create the HTTP session."""
        if self._session is None or self._session.closed:
            raise RuntimeError(
                "Client session not initialized. Use async context manager or call start() first."
            )
        return self._session

    async def _handle_response(self, response: aiohttp.ClientResponse) -> Any:
        """Handle HTTP response and raise errors if needed."""
        if response.status >= 400:
            try:
                error_data = await response.json()
                error_resp = ErrorResponse(**error_data)
                raise LiteAgentClientError(error_resp.error, error_resp.details)
            except (ContentTypeError, ValueError, json.JSONDecodeError):
                # Catch any exception during JSON parsing and fall back to text
                raise LiteAgentClientError(
                    f"HTTP {response.status}: {await response.text()}"
                )

        return await response.json()

    async def spawn_agent(self, request: SpawnRequest) -> SpawnResponse:
        """
        Spawn a new agent or continue an existing session.

        Args:
            request: Spawn request with agent_type, input, and optional session_id

        Returns:
            SpawnResponse with session_id, execution_id, and status

        Raises:
            LiteAgentClientError: If the request fails
        """
        async with self.session.post(
            "/api/agents/spawn",
            json=request.model_dump(exclude_none=True, by_alias=True),
        ) as response:
            data = await self._handle_response(response)
            return SpawnResponse(**data)

    async def get_session_status(self, session_id: str) -> SessionStatusResponse:
        """
        Get status of a specific session.

        Args:
            session_id: Session ID to query

        Returns:
            SessionStatusResponse with session details

        Raises:
            LiteAgentClientError: If the session is not found or request fails
        """
        async with self.session.get(f"/api/sessions/{session_id}") as response:
            data = await self._handle_response(response)
            return SessionStatusResponse(**data)

    async def list_sessions(self) -> ListSessionsResponse:
        """
        List all sessions.

        Returns:
            ListSessionsResponse with list of sessions

        Raises:
            LiteAgentClientError: If the request fails
        """
        async with self.session.get("/api/sessions") as response:
            data = await self._handle_response(response)
            return ListSessionsResponse(**data)

    async def delete_session(self, session_id: str) -> None:
        """
        Delete a session.

        Args:
            session_id: Session ID to delete

        Raises:
            LiteAgentClientError: If the session is not found or deletion fails
        """
        async with self.session.delete(f"/api/sessions/{session_id}") as response:
            if response.status >= 400:
                await self._handle_response(response)

    async def list_agents(self) -> ListAgentsResponse:
        """
        List available agent types.

        Returns:
            ListAgentsResponse with list of available agents

        Raises:
            LiteAgentClientError: If the request fails
        """
        async with self.session.get("/api/agents") as response:
            data = await self._handle_response(response)
            return ListAgentsResponse(**data)

    async def stream_logs(
        self,
        session_id: str,
    ) -> AsyncIterator[LogEntry]:
        """
        Stream logs for a session via Server-Sent Events (SSE).

        Args:
            session_id: Session ID to stream logs from

        Yields:
            LogEntry objects as they arrive

        Raises:
            LiteAgentClientError: If the session is not found or stream fails

        Example:
            ```python
            async for log in client.stream_logs(session_id):
                if log.level == LogEntryType.STDOUT:
                    print(log.content)
            ```
        """
        url = f"/api/logs/{session_id}/stream"
        async with self.session.get(url) as response:
            if response.status == 404:
                raise LiteAgentClientError(f"Session '{session_id}' not found")
            elif response.status >= 400:
                await self._handle_response(response)

            # Process SSE stream
            async for line in response.content:
                line_text = line.decode().strip()

                # Skip empty lines and comments
                if not line_text or line_text.startswith(":"):
                    continue

                # Parse SSE format: "data: {json}"
                if line_text.startswith("data: "):
                    data_str = line_text[6:]  # Remove "data: " prefix

                    try:
                        data = json.loads(data_str)

                        # Handle stream control events
                        if isinstance(data, dict):
                            event_type = data.get("type")
                            if event_type == "stream_started":
                                continue
                            elif event_type == "stream_ended":
                                break
                            elif event_type == "error":
                                raise LiteAgentClientError(data.get("message", "Unknown error"))

                        # Parse as log entry
                        yield LogEntry(**data)
                    except (json.JSONDecodeError, TypeError) as e:
                        # Skip malformed log entries
                        continue

    async def health_check(self) -> HealthResponse:
        """
        Check server health.

        Returns:
            HealthResponse with status and timestamp

        Raises:
            LiteAgentClientError: If the request fails
        """
        async with self.session.get("/api/health") as response:
            data = await self._handle_response(response)
            return HealthResponse(**data)

    async def spawn_and_stream(
        self,
        request: SpawnRequest,
    ) -> AsyncIterator[LogEntry]:
        """
        Convenience method to spawn an agent and stream its logs.

        This combines spawn_agent() and stream_logs() in a single operation.

        Args:
            request: Spawn request with agent_type, input, and optional session_id

        Yields:
            LogEntry objects as they arrive

        Raises:
            LiteAgentClientError: If spawn or streaming fails

        Example:
            ```python
            async for log in client.spawn_and_stream(SpawnRequest(
                agent_type="shell",
                input="ls -la"
            )):
                print(f"[{log.level}] {log.content}")
            ```
        """
        response = await self.spawn_agent(request)
        async for log in self.stream_logs(response.session_id):
            yield log
