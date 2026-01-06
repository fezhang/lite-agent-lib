"""
lite-agent-client - Python client for lite-agent-lib

A lightweight, async-first Python client for managing different kinds of agents
through the lite-agent-lib REST API.
"""

from lite_agent_client.client import LiteAgentClient
from lite_agent_client.models import (
    SpawnRequest,
    SpawnResponse,
    SessionStatusResponse,
    ListSessionsResponse,
    AgentInfo,
    ListAgentsResponse,
    LogEntry,
)

__version__ = "0.1.0"
__all__ = [
    "LiteAgentClient",
    "SpawnRequest",
    "SpawnResponse",
    "SessionStatusResponse",
    "ListSessionsResponse",
    "AgentInfo",
    "ListAgentsResponse",
    "LogEntry",
]
