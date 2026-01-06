"""
Custom agent example with advanced features.

This example demonstrates advanced features of the Python client:
- Using spawn_and_stream convenience method
- Custom agent configuration
- Session management (create, continue, delete)
- Error handling
"""

import asyncio
import sys

# Add parent directory to path for imports when running directly
sys.path.insert(0, ".")

from lite_agent_client import LiteAgentClient, SpawnRequest, AgentConfigOptions, LiteAgentClientError
from pathlib import Path


async def example_spawn_and_stream():
    """Example using spawn_and_stream convenience method."""
    print("\n" + "=" * 60)
    print("Example: Using spawn_and_stream convenience method")
    print("=" * 60)

    async with LiteAgentClient("http://localhost:3000") as client:
        request = SpawnRequest(
            agent_type="echo",
            input="Testing spawn_and_stream",
        )

        print("🚀 Spawning agent and streaming logs in one call...")
        async for log in client.spawn_and_stream(request):
            print(f"  [{log.level}] {log.content}")

        print("✅ Complete")


async def example_session_management():
    """Example of session management operations."""
    print("\n" + "=" * 60)
    print("Example: Session Management")
    print("=" * 60)

    async with LiteAgentClient("http://localhost:3000") as client:
        # Create a new session
        print("🚀 Creating new session...")
        request = SpawnRequest(
            agent_type="shell",
            input="echo 'First command'",
            config=AgentConfigOptions(
                timeout_secs=10,
            ),
        )
        response = await client.spawn_agent(request)
        session_id = response.session_id
        print(f"✅ Session created: {session_id}")

        # Consume the logs
        async for _ in client.stream_logs(session_id):
            pass

        # Continue the session
        print("🔄 Continuing session...")
        follow_up = SpawnRequest(
            agent_type="shell",
            input="echo 'Second command'",
            session_id=session_id,
        )
        await client.spawn_agent(follow_up)

        async for _ in client.stream_logs(session_id):
            pass

        # Get session status
        status = await client.get_session_status(session_id)
        print(f"📊 Session status: {status.status}")
        print(f"   Executions: {status.execution_count}")

        # List all sessions
        all_sessions = await client.list_sessions()
        print(f"\n📋 Total sessions: {all_sessions.total}")

        # Clean up - delete the session
        print(f"\n🗑️  Deleting session: {session_id}")
        await client.delete_session(session_id)
        print("✅ Session deleted")


async def example_custom_configuration():
    """Example with custom agent configuration."""
    print("\n" + "=" * 60)
    print("Example: Custom Agent Configuration")
    print("=" * 60)

    async with LiteAgentClient("http://localhost:3000") as client:
        # Custom configuration with environment variables and timeout
        config = AgentConfigOptions(
            work_dir=Path("/tmp"),
            env={
                "CUSTOM_VAR": "custom_value",
                "ANOTHER_VAR": "another_value",
            },
            timeout_secs=30,
            custom={
                "retry_count": 3,
                "verbose": True,
            },
        )

        request = SpawnRequest(
            agent_type="shell",
            input="echo 'Custom config test'",
            config=config,
        )

        print("🚀 Spawning agent with custom configuration...")
        print(f"   Work dir: {config.work_dir}")
        print(f"   Environment: {config.env}")
        print(f"   Timeout: {config.timeout_secs}s")
        print(f"   Custom settings: {config.custom}")

        response = await client.spawn_agent(request)
        print(f"✅ Session ID: {response.session_id}")

        async for log in client.stream_logs(response.session_id):
            if log.level in ("stdout", "stderr"):
                print(f"  {log.content}", end="")

        print("\n✅ Complete")


async def example_error_handling():
    """Example demonstrating error handling."""
    print("\n" + "=" * 60)
    print("Example: Error Handling")
    print("=" * 60)

    async with LiteAgentClient("http://localhost:3000") as client:
        # Try to spawn non-existent agent
        print("🔍 Attempting to spawn non-existent agent...")
        try:
            request = SpawnRequest(
                agent_type="nonexistent_agent",
                input="test",
            )
            await client.spawn_agent(request)
        except LiteAgentClientError as e:
            print(f"⚠️  Expected error caught: {e.message}")
            if e.details:
                print(f"   Details: {e.details}")

        # Try to get non-existent session
        print("\n🔍 Attempting to get non-existent session...")
        try:
            await client.get_session_status("nonexistent-session")
        except LiteAgentClientError as e:
            print(f"⚠️  Expected error caught: {e.message}")

        print("✅ Error handling works correctly")


async def example_list_agents():
    """Example listing available agents."""
    print("\n" + "=" * 60)
    print("Example: Listing Available Agents")
    print("=" * 60)

    async with LiteAgentClient("http://localhost:3000") as client:
        agents_response = await client.list_agents()

        print(f"\n📋 Available agents ({agents_response.total}):")
        for agent in agents_response.agents:
            print(f"\n  🤖 {agent.agent_type}")
            if agent.description:
                print(f"     Description: {agent.description}")
            if agent.capabilities:
                caps = ", ".join(agent.capabilities)
                print(f"     Capabilities: {caps}")
            print(f"     Availability: {agent.availability}")


async def main():
    """Run all examples."""
    print("=" * 60)
    print("Lite Agent Python Client - Advanced Examples")
    print("=" * 60)

    examples = [
        ("Spawn and Stream", example_spawn_and_stream),
        ("List Agents", example_list_agents),
        ("Session Management", example_session_management),
        ("Custom Configuration", example_custom_configuration),
        ("Error Handling", example_error_handling),
    ]

    for name, example_func in examples:
        try:
            await example_func()
        except Exception as e:
            print(f"\n❌ Example '{name}' failed: {e}", file=sys.stderr)
            import traceback
            traceback.print_exc()

        # Small delay between examples
        await asyncio.sleep(0.5)

    print("\n" + "=" * 60)
    print("✅ All examples completed")
    print("=" * 60)


if __name__ == "__main__":
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        print("\n⚠️  Interrupted by user")
        sys.exit(0)
    except Exception as e:
        print(f"\n❌ Error: {e}", file=sys.stderr)
        import traceback
        traceback.print_exc()
        sys.exit(1)
