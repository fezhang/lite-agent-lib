"""
Basic shell agent example.

This example demonstrates how to use the Python client to spawn a shell agent,
execute commands, and stream the output.
"""

import asyncio
import sys

# Add parent directory to path for imports when running directly
sys.path.insert(0, ".")

from lite_agent_client import LiteAgentClient, SpawnRequest, LogEntryType, AgentConfigOptions
from pathlib import Path


async def main():
    """Run the shell agent example."""
    # Create client and connect to server
    async with LiteAgentClient("http://localhost:3000") as client:
        print("✅ Connected to lite-agent server")

        # Check server health
        health = await client.health_check()
        print(f"✅ Server health: {health.status}")

        # List available agents
        agents = await client.list_agents()
        print(f"\n📋 Available agents ({agents.total}):")
        for agent in agents.agents:
            capabilities = ", ".join(agent.capabilities) if agent.capabilities else "None"
            print(f"  - {agent.agent_type}: {agent.description or 'No description'}")
            print(f"    Capabilities: {capabilities}")
            print(f"    Availability: {agent.availability}")

        # Spawn shell agent with a simple command
        print("\n🚀 Spawning shell agent to execute 'ls -la'...")
        request = SpawnRequest(
            agent_type="shell",
            input="ls -la",
            config=AgentConfigOptions(
                work_dir=Path.cwd(),
                env={"DEBUG": "1"},
            ),
        )
        response = await client.spawn_agent(request)
        print(f"✅ Agent spawned")
        print(f"   Session ID: {response.session_id}")
        print(f"   Execution ID: {response.execution_id}")

        # Stream logs
        print("\n📡 Streaming output:")
        async for log in client.stream_logs(response.session_id):
            if log.level == LogEntryType.STDOUT:
                print(f"  {log.content}", end="")
            elif log.level == LogEntryType.STDERR:
                print(f"  [stderr] {log.content}", file=sys.stderr, end="")
            elif log.level == LogEntryType.ERROR:
                print(f"  [error] {log.content}", file=sys.stderr)

        print("\n✅ Command complete")

        # Continue the session with another command
        print("\n🔄 Continuing session with 'echo $PATH'...")
        follow_up_request = SpawnRequest(
            agent_type="shell",
            input="echo $PATH",
            session_id=response.session_id,
            config=AgentConfigOptions(
                work_dir=Path.cwd(),
            ),
        )
        follow_up_response = await client.spawn_agent(follow_up_request)
        print(f"✅ Follow-up spawned")
        print(f"   Execution ID: {follow_up_response.execution_id}")

        print("\n📡 Streaming output:")
        async for log in client.stream_logs(response.session_id):
            if log.level == LogEntryType.STDOUT:
                print(f"  {log.content}", end="")
            elif log.level == LogEntryType.STDERR:
                print(f"  [stderr] {log.content}", file=sys.stderr, end="")

        print("\n✅ Follow-up complete")

        # Get final session status
        status = await client.get_session_status(response.session_id)
        print(f"\n📊 Session status:")
        print(f"   Status: {status.status}")
        print(f"   Total executions: {status.execution_count}")
        print(f"   Created: {status.created_at}")
        print(f"   Updated: {status.updated_at}")

        # List all sessions
        all_sessions = await client.list_sessions()
        print(f"\n📋 All sessions ({all_sessions.total}):")
        for session in all_sessions.sessions:
            print(f"  - {session.session_id}: {session.agent_type} ({session.status})")


if __name__ == "__main__":
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        print("\n⚠️  Interrupted by user")
        sys.exit(0)
    except Exception as e:
        print(f"\n❌ Error: {e}", file=sys.stderr)
        sys.exit(1)
