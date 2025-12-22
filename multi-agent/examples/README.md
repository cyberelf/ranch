# Multi-Agent Examples

**Important: multi-agent is a CLIENT-SIDE framework for coordinating REMOTE agents.**

All examples follow this architecture:
- Agent implementations are A2A protocol servers (using `a2a-protocol` crate)
- This framework provides CLIENT connections to those servers
- Teams coordinate remote agents without local implementations

## Core Examples

### 1. Agent Servers + Team Client (Recommended Starting Point)

Demonstrates the proper architecture with remote agents.

**Terminal 1 - Start the agent servers:**
```bash
cargo run --example agent_servers
```

This starts three A2A protocol servers:
- Router Agent (port 3000) - Supports Client Agent Extension
- Worker Agent (port 3001) - Task execution
- Supervisor Agent (port 3002) - Quality control

**Terminal 2 - Run the team client:**
```bash
cargo run --example team_client
```

This connects to the remote agents and demonstrates:
- Creating `A2AAgent` clients for remote servers
- Registering agents in `AgentManager`
- Forming teams with dynamic routing
- Processing messages through the team

## Architecture Principles

### ✅ What multi-agent IS

- **Client-side coordination layer** for remote agents
- Connects to agents via A2A protocol (`A2AAgent`)
- Routes messages between remote agents
- Manages team composition and routing logic

### ❌ What multi-agent is NOT

- NOT an agent implementation framework
- Do NOT implement `Agent` trait for new agents locally
- Do NOT create mock agents in examples

### Correct Pattern

```rust
// ✅ CORRECT: Connect to remote A2A server
let transport = Arc::new(JsonRpcTransport::new("http://localhost:3000/rpc")?);
let client = A2aClient::new(transport);
let remote_agent = Arc::new(A2AAgent::new(client));

let manager = Arc::new(AgentManager::new());
manager.register(remote_agent).await?;
```

### Incorrect Pattern

```rust
// ❌ WRONG: Don't implement agents locally in multi-agent
struct MyAgent { /* ... */ }

#[async_trait]
impl Agent for MyAgent {  // Wrong!
    async fn info(&self) -> A2aResult<AgentInfo> { /* ... */ }
    async fn process(&self, msg: Message) -> A2aResult<Message> { /* ... */ }
}
```

**Instead:** Implement agents as A2A servers using `a2a-protocol::server::ProtocolAgent`

## How to Implement Agents

Agents should be implemented in the `a2a-protocol` crate:

```rust
use a2a_protocol::prelude::*;
use a2a_protocol::server::{ProtocolAgent, ServerBuilder, TaskAwareHandler};

struct MyAgent {
    profile: AgentProfile,
}

#[async_trait]
impl ProtocolAgent for MyAgent {
    async fn profile(&self) -> A2aResult<AgentProfile> {
        Ok(self.profile.clone())
    }
    
    async fn process_message(&self, message: Message) -> A2aResult<Message> {
        // Your agent logic here
        Ok(Message::agent_text("Response"))
    }
}

// Start the server
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let agent = Arc::new(MyAgent::new());
    let handler = TaskAwareHandler::new(agent);
    ServerBuilder::new(handler).with_port(3000).run().await?;
    Ok(())
}
```

Then connect to it from multi-agent using `A2AAgent`.

## Key Concepts

### A2AAgent - Remote Agent Client

The only agent type you should use in multi-agent:

```rust
use multi_agent::agent::A2AAgent;
use a2a_protocol::prelude::*;

// Connect to remote A2A server
let transport = Arc::new(JsonRpcTransport::new("http://agent-server:3000/rpc")?);
let client = A2aClient::new(transport);
let agent = Arc::new(A2AAgent::new(client));
```

### AgentManager - Registry

Central registry for discovering and accessing remote agents:

```rust
let manager = Arc::new(AgentManager::new());
manager.register(agent1).await?;
manager.register(agent2).await?;

// List all registered agents
for info in manager.list_info().await {
    println!("{}: {}", info.name, info.id);
}
```

### Team - Coordination

Forms groups of remote agents with routing:

```rust
let config = TeamConfig {
    id: "my-team".to_string(),
    name: "My Team".to_string(),
    description: "Team of remote agents".to_string(),
    agents: vec![
        TeamAgentConfig {
            agent_id: "agent1".to_string(),
            role: "coordinator".to_string(),
            capabilities: vec![],
        },
    ],
    router_config: RouterConfig {
        default_agent_id: "agent1".to_string(),
        max_routing_hops: 10,
    },
};

let team = Team::new(config, manager);
let response = team.process(message).await?;
```

## Client Agent Extension

The Router Agent in the examples demonstrates the Client Agent Extension:

1. Agent declares extension support in its AgentProfile
2. Team injects routing context (peer list) into messages
3. Agent makes routing decisions and returns recipient
4. Team routes message to the specified agent

See `agent_servers.rs` for the server-side implementation and `team_client.rs` for client usage.

## Common Mistakes to Avoid

1. ❌ Implementing `Agent` trait locally
2. ❌ Creating mock agents in examples
3. ❌ Mixing agent implementation with coordination logic
4. ❌ Not starting agent servers before running client

Always remember: **multi-agent is for CLIENT-SIDE coordination of REMOTE agents.**
