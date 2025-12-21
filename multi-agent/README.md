# Multi-Agent Runtime Framework

A Rust-based framework for building and managing multi-agent systems with support for different communication protocols and scheduling modes.

## Features

- **Protocol Support**: OpenAI API and A2A (Agent-to-Agent) protocols
- **Agent Management**: Dynamic agent registration, health checks, and capability discovery
- **Team Composition**: Group agents into teams with dynamic routing
- **Router-Based Coordination**: 
  - Dynamic message routing based on agent capabilities
  - Client Agent Extension for intelligent routing decisions
  - Fallback routing to default agent
  - Back-to-sender routing support
  - Max hop limits to prevent infinite loops
- **REST API**: OpenAI-compatible and A2A endpoints for team interactions
- **Configuration-driven**: TOML-based configuration for agents and teams

## Quick Start

1. **Set up your configuration** (config.toml):

```toml
[[agents]]
id = "research-assistant"
name = "Research Assistant"
endpoint = "https://api.openai.com/v1"
protocol = "openai"
capabilities = ["research", "analysis"]

[agents.metadata]
model = "gpt-4"
temperature = "0.7"

[[teams]]
id = "dev-team"
name = "Development Team"
description = "Team with dynamic router-based coordination"

[teams.router_config]
default_agent_id = "research-assistant"
max_routing_hops = 10

[[teams.agents]]
agent_id = "research-assistant"
role = "coordinator"
capabilities = ["research", "coordination"]
```

2. **Set environment variables**:

```bash
export OPENAI_API_KEY="your-api-key"
export CONFIG_PATH="config.toml"
export PORT=8080
```

3. **Run the server**:

```bash
cargo run
```

## API Endpoints

### OpenAI Compatible Endpoint
```
POST /v1/chat/completions
```

Example request:
```json
{
  "model": "gpt-4",
  "messages": [
    {
      "role": "user",
      "content": "Hello, team!"
    }
  ]
}
```

### A2A Endpoint
```
POST /v1/chat
```

Example request:
```json
{
  "id": "req-123",
  "messages": [
    {
      "role": "user",
      "content": "Hello, team!",
      "metadata": {}
    }
  ],
  "context": {}
}
```

### Health Check
```
GET /health
```

## Architecture

The framework consists of several key components:

- **Agent**: Represents a remote AI service with specific capabilities
- **Protocol**: Handles communication with agents using different protocols
- **AgentManager**: Manages agent lifecycle and discovery
- **Team**: Groups agents with dynamic router-based coordination
- **Router**: Implements intelligent message routing between agents
  - Supports Client Agent Extension for capable agents
  - Provides fallback routing for basic agents
  - Tracks sender history for back-to-sender routing
  - Enforces max hop limits to prevent infinite loops
- **TeamServer**: Provides HTTP API endpoints

## Client Agent Extension

The framework supports a Client Agent Extension (URI: `https://ranch.woi.dev/extensions/client-routing/v1`) that enables intelligent routing decisions:

### Extension Support

Agents declare extension support in their capabilities:

```rust
capabilities: vec![
    "https://ranch.woi.dev/extensions/client-routing/v1".to_string()
]
```

### Extension Data Flow

1. **Router → Agent**: Router injects peer agent list in message metadata:
```json
{
  "https://ranch.woi.dev/extensions/client-routing/v1": {
    "agentCards": [
      {
        "id": "researcher",
        "name": "Research Agent",
        "description": "Searches and summarizes",
        "capabilities": ["search", "summarize"],
        "supportsClientRouting": true
      }
    ],
    "sender": "user"
  }
}
```

2. **Agent → Router**: Agent returns routing decision in metadata:
```json
{
  "https://ranch.woi.dev/extensions/client-routing/v1": {
    "recipient": "researcher",
    "reason": "Query requires search capability"
  }
}
```

### Routing Options

- **Agent ID**: Route to specific agent (e.g., `"researcher"`)
- **"user"**: Return to user (end conversation)
- **"sender"**: Route back to previous sender
- **No decision**: Basic agents without extension return to user automatically

## Examples

### Creating an Agent Programmatically

```rust
use multi_agent::*;

let config = AgentConfig::new(
    "my-agent".to_string(),
    "https://api.example.com".to_string(),
    ProtocolType::OpenAI,
)
.with_capabilities(vec!["analysis".to_string()]);

let protocol = protocols::create_protocol_adapter(
    &ProtocolType::OpenAI,
    Some("api-key".to_string()),
);

let agent = Arc::new(RemoteAgent::new(config, protocol));
```

### Creating a Team

```rust
use multi_agent::team::{TeamConfig, TeamAgentConfig, RouterConfig};

let team_config = TeamConfig {
    id: "my-team".to_string(),
    name: "My Team".to_string(),
    description: "A sample team with dynamic routing".to_string(),
    agents: vec![TeamAgentConfig {
        agent_id: "my-agent".to_string(),
        role: "coordinator".to_string(),
        capabilities: vec!["analysis".to_string()],
    }],
    router_config: RouterConfig {
        default_agent_id: "my-agent".to_string(),
        max_routing_hops: 10,
    },
};

let team = Arc::new(Team::new(team_config, agent_manager));
```

### Using TeamServer (A2A JSON-RPC Server)

```rust
use multi_agent::{Team, TeamServer};
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create your team
    let team = Arc::new(Team::new(team_config, agent_manager));
    
    // Create and start TeamServer
    let server = TeamServer::new(team, 3000);
    server.start().await?;
    
    Ok(())
}
```

The TeamServer exposes your team as an A2A-compliant service via JSON-RPC 2.0:

**Supported JSON-RPC Methods:**
- `message/send` - Send a message to the team
- `task/get` - Get details about a task
- `task/status` - Get task status
- `task/cancel` - Cancel a running task
- `agent/card` - Get team's capabilities as an AgentCard

**Example JSON-RPC Request:**
```bash
curl -X POST http://localhost:3000/rpc \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "message/send",
    "params": {
      "message": {
        "role": "user",
        "parts": [{"type": "text", "text": "Hello team!"}]
      }
    },
    "id": 1
  }'
```

**Example Response:**
```json
{
  "jsonrpc": "2.0",
  "result": {
    "task_id": "task-abc123",
    "status": "queued"
  },
  "id": 1
}
```

## Configuration Reference

### Agent Configuration
- `id`: Unique identifier for the agent
- `name`: Human-readable name
- `endpoint`: API endpoint URL
- `protocol`: "openai" or "a2a"
- `capabilities`: List of agent capabilities
- `timeout_seconds`: Request timeout (default: 30)
- `max_retries`: Maximum retry attempts (default: 3)
- `metadata`: Protocol-specific configuration

### Team Configuration
- `id`: Unique identifier for the team
- `name`: Human-readable name
- `description`: Team description
- `mode`: "supervisor" or "workflow"
- `agents`: List of agent configurations

## Development

The framework is designed to be extensible:
- Add new protocols by implementing the `Protocol` trait
- Create custom schedulers by implementing the `Scheduler` trait
- Extend the HTTP API with new endpoints

## License

MIT License