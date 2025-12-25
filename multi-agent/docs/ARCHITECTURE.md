# Multi-Agent Architecture Documentation

## Core Principle

**The multi-agent crate is a CLIENT-SIDE collaboration framework, NOT an agent implementation framework.**

This document establishes the architectural principle that must be followed in all development, examples, and documentation.

## What multi-agent IS

multi-agent is a **collaboration ground** for coordinating remote agents:

✅ **Correct Responsibilities:**
- Connect to remote A2A protocol servers via `A2AAgent`
- Manage connections and discovery via `AgentManager`
- Route messages between remote agents dynamically via `Team` and `Router`
- Provide Client Agent Extension support for intelligent routing
- Expose team-level HTTP API for external access

## What multi-agent is NOT

multi-agent is NOT an agent implementation framework:

❌ **What NOT to Do:**
- Do NOT implement `Agent` trait for new agents in this crate
- Do NOT create local agent execution logic
- Do NOT include mock agents in examples
- Do NOT mix agent implementation with coordination logic

## Architecture Diagram

```text
┌─────────────────────────────────────────────────────────────┐
│ multi-agent (Client-Side Coordination Layer)               │
│                                                             │
│  ┌──────────────┐      ┌──────────────┐                   │
│  │ A2AAgent     │──────│ A2AAgent     │                   │
│  │ (Client)     │      │ (Client)     │                   │
│  └──────┬───────┘      └──────┬───────┘                   │
│         │                     │                            │
│         │  ┌──────────────────┴───────────┐               │
│         └──┤ Team (Router & Coordination) │               │
│            └──────────────────────────────┘               │
└─────────────────────────────────────────────────────────────┘
                        │                │
                        │ A2A Protocol   │
                        │ (JSON-RPC)     │
                        ▼                ▼
┌─────────────────────────────────────────────────────────────┐
│ a2a-protocol Servers (Agent Implementations)                │
│                                                             │
│  ┌──────────────┐      ┌──────────────┐                   │
│  │ Agent Server │      │ Agent Server │                   │
│  │ (Port 3000)  │      │ (Port 3001)  │                   │
│  │              │      │              │                   │
│  │ ProtocolAgent│      │ ProtocolAgent│                   │
│  └──────────────┘      └──────────────┘                   │
└─────────────────────────────────────────────────────────────┘
```

## Component Responsibilities

### a2a-protocol Crate (Agent Implementation)

**Purpose:** Implement agents as A2A protocol servers

**Key Traits:**
- `ProtocolAgent` - Server-side agent implementation
- `A2aClient` - Client for calling remote agents

**Components:**
- `server::ServerBuilder` - Build and run agent servers
- `server::TaskAwareHandler` - Handle A2A protocol requests
- Core types: `Message`, `Task`, `AgentProfile`, `AgentSkill`

### multi-agent Crate (Client-Side Coordination)

**Purpose:** Coordinate remote agents as teams

**Key Types:**
- `A2AAgent` - Client wrapper for remote A2A agents
- `AgentManager` - Registry for agent discovery
- `Team` - Group of agents with routing logic
- `Router` - Dynamic message routing engine

**Components:**
- `agent/` - Agent trait and remote client
- `manager/` - Agent registry and lifecycle
- `team/` - Team composition and routing
- `server/` - Team-level HTTP API

## Correct Usage Pattern

### Step 1: Implement Agents (a2a-protocol)

```rust
// In your agent implementation crate
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
        // Your agent logic
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

### Step 2: Coordinate Agents (multi-agent)

```rust
// In your coordination application
use multi_agent::{A2AAgent, AgentManager, Team, TeamConfig};
use a2a_protocol::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to remote agents
    let transport1 = Arc::new(JsonRpcTransport::new("http://localhost:3000/rpc")?);
    let client1 = A2aClient::new(transport1);
    let agent1 = Arc::new(A2AAgent::new(client1));

    let transport2 = Arc::new(JsonRpcTransport::new("http://localhost:3001/rpc")?);
    let client2 = A2aClient::new(transport2);
    let agent2 = Arc::new(A2AAgent::new(client2));

    // Register agents
    let manager = Arc::new(AgentManager::new());
    manager.register(agent1).await?;
    manager.register(agent2).await?;

    // Form team
    let team = Team::new(team_config, manager);
    
    // Process messages
    let response = team.process(Message::user_text("Hello")).await?;
    
    Ok(())
}
```

## Example Structure

All examples MUST follow this pattern:

### Example: agent_servers.rs

Located in `multi-agent/examples/agent_servers.rs`

**Purpose:** Start multiple A2A protocol servers

**Structure:**
1. Implement agents using `ProtocolAgent`
2. Configure agent profiles with skills
3. Start servers on different ports
4. Wait for connections

### Example: team_client.rs

Located in `multi-agent/examples/team_client.rs`

**Purpose:** Connect to remote agents and form team

**Structure:**
1. Create `A2AAgent` clients for each remote server
2. Register clients in `AgentManager`
3. Configure `Team` with routing rules
4. Process messages through the team

**Running:**
```bash
# Terminal 1: Start agent servers
cargo run --example agent_servers

# Terminal 2: Run team client
cargo run --example team_client
```

## Documentation Requirements

All documentation MUST emphasize the client-side nature of multi-agent:

### README.md

- Lead with "Client-Side Collaboration Framework"
- Clearly state it's NOT for agent implementation
- Show architecture diagram
- Provide correct usage examples

### AGENT.md

- Explain the client-server architecture
- Show correct vs incorrect patterns
- Document A2AAgent usage
- Link to a2a-protocol for agent implementation

### lib.rs

- Module-level docs explain the architecture
- Show ASCII diagram of client-server relationship
- Provide complete usage example
- Link to relevant examples

### Examples README.md

- Explain the two-phase architecture (servers + client)
- Document how to run examples
- Show correct patterns prominently
- Call out common mistakes

## Testing Guidelines

### Unit Tests

- Use real A2A protocol communication (via HTTP)
- Do NOT mock the `Agent` trait locally
- Test client-side coordination logic only

### Integration Tests

- Start actual A2A servers in background
- Connect via `A2AAgent`
- Test team coordination and routing
- Clean up servers after tests

### Example Tests

Examples should be runnable and demonstrate real usage:
- Agent servers that can be started
- Team clients that connect to those servers
- Clear instructions for running

## Common Mistakes to Avoid

### ❌ Mistake 1: Implementing Agent Locally

```rust
// WRONG!
struct MyLocalAgent { }

#[async_trait]
impl Agent for MyLocalAgent {
    // Don't do this in multi-agent
}
```

**Fix:** Implement in a2a-protocol using `ProtocolAgent`

### ❌ Mistake 2: Mock Agents in Examples

```rust
// WRONG!
struct MockAgent {
    responses: Vec<String>,
}
```

**Fix:** Use real A2A servers via ServerBuilder

### ❌ Mistake 3: Mixed Responsibilities

```rust
// WRONG!
impl Agent for MyAgent {
    async fn process(&self, msg: Message) -> A2aResult<Message> {
        // Mixing agent logic with coordination
        let result = self.do_work(); // Agent logic
        self.route_to_next(); // Coordination logic
    }
}
```

**Fix:** Separate agent logic (a2a-protocol) from coordination (multi-agent)

## Compliance Checklist

Before submitting code, verify:

- [ ] No local `Agent` trait implementations in multi-agent
- [ ] All examples use real A2A servers
- [ ] Documentation emphasizes client-side nature
- [ ] Architecture diagrams show client-server split
- [ ] Code comments reference the correct crate for agent implementation
- [ ] Tests use real HTTP communication, not mocks
- [ ] README and guides are updated
- [ ] No mock agents in examples

## References

- A2A Protocol Specification: https://a2a-protocol.org/
- Agent Implementation: See `a2a-protocol` crate
- Example Usage: `multi-agent/examples/agent_servers.rs` and `team_client.rs`
- Architecture Guide: `multi-agent/AGENT.md`
