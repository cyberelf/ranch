# Multi-Agent Client Collaboration Framework

**A client-side framework for coordinating remote agents, not for implementing agents.**

This crate provides a **collaboration ground** where remote agents (accessed via A2A protocol) can work together as teams. It is NOT a framework for implementing new agents - agents should be implemented as A2A protocol servers using the `a2a-protocol` crate.

## Core Principle

**Multi-agent is a client-side coordination layer:**
- ✅ Use `A2AAgent` to connect to remote A2A protocol servers
- ✅ Form teams of remote agents for collaboration
- ✅ Route messages between remote agents dynamically
- ❌ Do NOT implement local agents using the `Agent` trait
- ❌ Do NOT create mock agents in examples - use real A2A servers

## Features

- **Remote Agent Coordination**: Connect to remote A2A protocol agents via `A2AAgent`
- **Team Composition**: Form teams of remote agents with dynamic routing
- **Router-Based Coordination**: 
  - Dynamic message routing based on agent skills
  - Client Agent Extension for intelligent routing decisions
  - Handoffs feature for filtered peer suggestions
  - Max hop limits to prevent infinite loops
- **Protocol Adapters**: OpenAI API adapter for legacy systems (transitional)
- **REST API**: Team-level endpoints for external interaction

## Quick Start

### 1. Start Remote Agent Servers

First, implement and start your agents as A2A protocol servers:

```rust
// In your agent server crate
use a2a_protocol::prelude::*;
use a2a_protocol::server::{ProtocolAgent, ServerBuilder, TaskAwareHandler};

#[async_trait]
impl ProtocolAgent for MyAgent {
    async fn profile(&self) -> A2aResult<AgentProfile> {
        // Return agent profile with skills
    }
    
    async fn process_message(&self, msg: Message) -> A2aResult<Message> {
        // Handle messages
    }
}

// Start server on port 3000
let handler = TaskAwareHandler::new(Arc::new(my_agent));
ServerBuilder::new(handler).with_port(3000).run().await?;
```

### 2. Create Client-Side Team

Then use multi-agent to coordinate remote agents:

```rust
use multi_agent::{A2AAgent, AgentManager, Team, TeamConfig};
use a2a_protocol::prelude::*;

// Connect to remote agents via A2A protocol
let client1 = A2aClient::new(Arc::new(
    JsonRpcTransport::new("http://localhost:3000/rpc")?
));
let agent1 = Arc::new(A2AAgent::new(client1));

let client2 = A2aClient::new(Arc::new(
    JsonRpcTransport::new("http://localhost:3001/rpc")?
));
let agent2 = Arc::new(A2AAgent::new(client2));

// Register agents
let manager = Arc::new(AgentManager::new());
manager.register(agent1).await?;
manager.register(agent2).await?;

// Form team
let team = Team::new(team_config, manager);
let response = team.process(message).await?;
```

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