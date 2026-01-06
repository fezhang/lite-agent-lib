"""
Basic echo agent example.

This example demonstrates how to use the Python client to spawn an echo agent
and stream its logs.
"""

import asyncio
import sys

# Add parent directory to path for imports when running directly
sys.path.insert(0, ".")

from lite_agent_client import LiteAgentClient, SpawnRequest, LogEntryType


async def main():
    """Run the echo agent example."""
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
            print(f"  - {agent.agent_type}: {agent.description or 'No description'}")

        # Spawn echo agent
        print("\n🚀 Spawning echo agent...")
        request = SpawnRequest(
            agent_type="echo",
            input="Hello, echo agent!",
        )
        response = await client.spawn_agent(request)
        print(f"✅ Agent spawned")
        print(f"   Session ID: {response.session_id}")
        print(f"   Execution ID: {response.execution_id}")

        # Stream logs
        print("\n📡 Streaming logs:")
        async for log in client.stream_logs(response.session_id):
            if log.level == LogEntryType.STDOUT:
                print(f"  [stdout] {log.content}")
            elif log.level == LogEntryType.STDERR:
                print(f"  [stderr] {log.content}", file=sys.stderr)
            elif log.level == LogEntryType.INFO:
                print(f"  [info] {log.content}")
            elif log.level == LogEntryType.ERROR:
                print(f"  [error] {log.content}", file=sys.stderr)
            else:
                print(f"  [{log.level}] {log.content}")

        print("\n✅ Stream complete")

        # Get final session status
        status = await client.get_session_status(response.session_id)
        print(f"\n📊 Session status:")
        print(f"   Status: {status.status}")
        print(f"   Executions: {status.execution_count}")
        print(f"   Created: {status.created_at}")
        print(f"   Updated: {status.updated_at}")


if __name__ == "__main__":
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        print("\n⚠️  Interrupted by user")
        sys.exit(0)
    except Exception as e:
        print(f"\n❌ Error: {e}", file=sys.stderr)
        sys.exit(1)
