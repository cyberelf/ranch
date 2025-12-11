# Agent Architecture Guide

This document provides a comprehensive overview of the agent architecture in the RANCH project, covering both the A2A protocol implementation and the multi-agent framework.

## Table of Contents

- [Overview](#overview)
- [Architecture Layers](#architecture-layers)
- [Trait Hierarchy](#trait-hierarchy)
- [Component Relationships](#component-relationships)
- [When to Use Which Trait](#when-to-use-which-trait)
- [Crate-Specific Guides](#crate-specific-guides)

## Overview

RANCH implements a layered agent architecture with two primary components:

1. **a2a-protocol**: Low-level A2A (Agent-to-Agent) protocol implementation
2. **multi-agent**: High-level orchestration framework for coordinating multiple agents

The design separates protocol compliance from orchestration concerns, enabling flexible agent composition and interoperability.

## Architecture Layers

```
┌─────────────────────────────────────────────────────────────┐
│                     Application Layer                        │
│  (Your custom agents, teams, and business logic)            │
└─────────────────────────────────────────────────────────────┘
                           │
                           ├──────────────────┬────────────────┐
                           ↓                  ↓                ↓
┌──────────────────────────────────┐  ┌──────────────────┐  ┌───────────────────┐
│   Multi-Agent Framework          │  │  A2A Protocol    │  │  External Agents  │
│                                  │  │  Implementation  │  │  (via A2A)        │
│  - Team Orchestration            │  │                  │  │                   │
│  - AgentManager (Registry)       │  │  - Protocol      │  │  - Remote A2A     │
│  - Schedulers (Routing Logic)    │  │    Agent Trait   │  │    Services       │
│  - TeamServer (A2A Service)      │  │  - Client/Server │  │  - Third-party    │
│  - Multi-Agent Agent Trait       │  │  - Task Mgmt     │  │    Agents         │
└──────────────────────────────────┘  └──────────────────┘  └───────────────────┘
                           │                  │
                           └──────────┬───────┘
                                      ↓
                        ┌────────────────────────────┐
                        │   Transport Layer          │
                        │  (JSON-RPC 2.0 over HTTP)  │
                        └────────────────────────────┘
```

## Trait Hierarchy

The project defines agents at two levels:

### 1. Protocol-Level Agent (`a2a-protocol::Agent`)

Represents an A2A protocol-compliant agent capable of:
- Exposing agent profile (capabilities, skills, version)
- Processing messages (text, data, files)
- Returning either immediate messages or task handles

**Location**: `a2a-protocol/src/server/agent.rs`

```rust
#[async_trait]
pub trait Agent: Send + Sync {
    async fn profile(&self) -> A2aResult<AgentProfile>;
    async fn process_message(&self, message: Message) -> A2aResult<Message>;
}
```

**Use Cases**:
- Implementing A2A-compliant agents
- Exposing services via JSON-RPC 2.0
- Task-aware processing with lifecycle management

### 2. Multi-Agent Framework Agent (`multi_agent::Agent`)

Represents a higher-level agent in the orchestration framework:
- Provides extended agent information
- Processes messages with enhanced context
- Integrates with team composition and scheduling

**Location**: `multi-agent/src/agent/traits.rs`

```rust
#[async_trait]
pub trait Agent: Send + Sync {
    async fn info(&self) -> MultiAgentResult<AgentInfo>;
    async fn process_message(&self, message: Message) -> MultiAgentResult<Message>;
}
```

**Use Cases**:
- Building coordinated multi-agent systems
- Team composition and nested teams
- Custom scheduling and routing logic

### Relationship Between Traits

```
┌─────────────────────────────────────────────────────────────┐
│  Application Agent                                          │
│  (Implements multi_agent::Agent OR a2a_protocol::Agent)    │
└─────────────────────────────────────────────────────────────┘
                           │
        ┌──────────────────┴──────────────────┐
        ↓                                     ↓
┌────────────────────────┐      ┌─────────────────────────┐
│  multi_agent::Agent    │      │  a2a_protocol::Agent    │
│                        │      │                         │
│  Used by:              │      │  Used by:               │
│  - Team                │      │  - TaskAwareHandler     │
│  - AgentManager        │      │  - JsonRpcRouter        │
│  - Custom Schedulers   │      │  - A2A Server           │
└────────────────────────┘      └─────────────────────────┘
```

**Key Distinction**: 
- `a2a_protocol::Agent` is for A2A protocol compliance
- `multi_agent::Agent` is for framework-level orchestration
- They are **different traits** despite similar signatures

## Component Relationships

### Core Components

#### 1. AgentManager
**Purpose**: Registry for discovering and managing agents

```rust
pub struct AgentManager {
    agents: RwLock<HashMap<String, Arc<dyn Agent>>>,
}
```

**Key Methods**:
- `register(agent)` - Add agent to registry
- `get(id)` - Retrieve agent by ID
- `find_by_capability(capability)` - Search by capability
- `health_check_all()` - Verify agent health

#### 2. Team
**Purpose**: Orchestrates multiple agents to accomplish complex tasks

```rust
pub struct Team {
    config: TeamConfig,
    agent_manager: Arc<AgentManager>,
    scheduler: Arc<dyn Scheduler>,
}
```

**Modes**:
- **Supervisor**: Leader agent delegates to specialists
- **Workflow**: Sequential processing through agent pipeline

**Key Feature**: `Team` implements `multi_agent::Agent`, enabling nested composition.

#### 3. Scheduler
**Purpose**: Determines message routing within a team

**Types**:
- `SupervisorScheduler`: Routes to supervisor who decides delegation
- `WorkflowScheduler`: Routes through predefined sequence

**Interface**:
```rust
#[async_trait]
pub trait Scheduler: Send + Sync {
    async fn determine_next_recipient(
        &self,
        message: &Message,
        context: &SchedulerContext,
    ) -> MultiAgentResult<String>;
}
```

#### 4. TeamServer
**Purpose**: Exposes a `Team` as an A2A-compliant service

```rust
pub struct TeamServer {
    team: Arc<Team>,
    port: u16,
}
```

**Features**:
- JSON-RPC 2.0 endpoint at `/rpc`
- Full A2A protocol compliance
- Task-aware message handling
- CORS support for web clients

### Integration Flow

```
User Request
     ↓
TeamServer (/rpc endpoint)
     ↓
TaskAwareHandler (wraps Team as a2a_protocol::Agent)
     ↓
JsonRpcRouter (handles JSON-RPC 2.0)
     ↓
Team (implements multi_agent::Agent)
     ↓
Scheduler (determines routing)
     ↓
AgentManager (retrieves target agent)
     ↓
Member Agent (processes message)
     ↓
Response (back through the stack)
```

## When to Use Which Trait

### Use `a2a_protocol::Agent` When:

✅ Building A2A protocol-compliant agents  
✅ Exposing services via JSON-RPC 2.0  
✅ Implementing standalone agents  
✅ Integrating with external A2A ecosystems  
✅ Managing task lifecycles explicitly  

**Example**:
```rust
use a2a_protocol::prelude::*;

struct MyAgent;

#[async_trait]
impl Agent for MyAgent {
    async fn profile(&self) -> A2aResult<AgentProfile> {
        Ok(AgentProfile {
            id: AgentId::new("my-agent"),
            name: "My Agent".to_string(),
            capabilities: vec![/* ... */],
            // ...
        })
    }

    async fn process_message(&self, message: Message) -> A2aResult<Message> {
        // Process and return response
        Ok(Message::agent_text("Response"))
    }
}
```

### Use `multi_agent::Agent` When:

✅ Building coordinated multi-agent systems  
✅ Composing teams of agents  
✅ Implementing custom orchestration logic  
✅ Creating nested team hierarchies  
✅ Using framework-level features (AgentManager, Schedulers)  

**Example**:
```rust
use multi_agent::*;

struct MyAgent;

#[async_trait]
impl Agent for MyAgent {
    async fn info(&self) -> MultiAgentResult<AgentInfo> {
        Ok(AgentInfo {
            id: "my-agent".to_string(),
            name: "My Agent".to_string(),
            capabilities: vec!["task1".to_string()],
            // ...
        })
    }

    async fn process_message(&self, message: Message) -> MultiAgentResult<Message> {
        // Process with framework context
        Ok(Message::agent_text("Response"))
    }
}
```

### Bridge Pattern: Using Both

The `A2AAgent` struct bridges the gap by:
1. Wrapping an `A2aClient` (protocol-level)
2. Implementing `multi_agent::Agent` (framework-level)
3. Enabling remote A2A agents to participate in teams

```rust
use multi_agent::*;
use a2a_protocol::prelude::*;

// Create A2A client for remote agent
let transport = JsonRpcTransport::new("https://remote-agent.com/rpc")?;
let client = A2aClient::new(Arc::new(transport));

// Wrap in multi-agent framework adapter
let agent = A2AAgent::new(client);

// Can now be used in teams!
team_manager.register(Arc::new(agent)).await?;
```

## Crate-Specific Guides

For detailed implementation guidance, see:

- **[a2a-protocol/AGENT.md](a2a-protocol/AGENT.md)** - A2A protocol agent implementation
  - Protocol compliance details
  - Task lifecycle management
  - JSON-RPC server setup
  - Authentication strategies

- **[multi-agent/AGENT.md](multi-agent/AGENT.md)** - Multi-agent framework guide
  - Team composition patterns
  - Scheduler implementation
  - Configuration via TOML
  - Nested team hierarchies

## Quick Start Examples

### 1. Simple A2A Agent

```rust
use a2a_protocol::prelude::*;

struct EchoAgent;

#[async_trait]
impl Agent for EchoAgent {
    async fn profile(&self) -> A2aResult<AgentProfile> {
        Ok(AgentProfile {
            id: AgentId::new("echo-agent"),
            name: "Echo Agent".to_string(),
            description: Some("Echoes back messages".to_string()),
            capabilities: vec![],
            skills: vec![],
            version: "1.0.0".to_string(),
            provider: None,
        })
    }

    async fn process_message(&self, message: Message) -> A2aResult<Message> {
        let text = message.text_content()
            .unwrap_or("(no text content)");
        Ok(Message::agent_text(format!("Echo: {}", text)))
    }
}
```

### 2. Team of Agents

```rust
use multi_agent::*;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create agent manager
    let manager = Arc::new(AgentManager::new());
    
    // Register agents
    let agent1 = Arc::new(MyAgent::new("agent-1"));
    let agent2 = Arc::new(MyAgent::new("agent-2"));
    
    manager.register(agent1).await?;
    manager.register(agent2).await?;
    
    // Create team
    let team_config = TeamConfig {
        id: "my-team".to_string(),
        name: "My Team".to_string(),
        mode: TeamMode::Supervisor,
        // ... config details
    };
    
    let team = Arc::new(Team::new(team_config, manager));
    
    // Process message through team
    let message = Message::user_text("Hello team!");
    let response = team.process_message(message).await?;
    
    println!("Response: {:?}", response);
    Ok(())
}
```

### 3. Expose Team as A2A Service

```rust
use multi_agent::*;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create team (as above)
    let team = create_my_team().await?;
    
    // Create and start server
    let server = TeamServer::new(team, 8080);
    println!("Starting team server on port 8080...");
    server.start().await?;
    
    Ok(())
}
```

## Best Practices

### 1. Agent Design

- **Single Responsibility**: Each agent should have a clear, focused purpose
- **Stateless When Possible**: Prefer stateless designs for better scalability
- **Graceful Degradation**: Handle errors without crashing the entire system
- **Capability Declaration**: Accurately declare capabilities in profile

### 2. Team Composition

- **Hierarchy Depth**: Keep team nesting shallow (2-3 levels max)
- **Cycle Prevention**: Built-in cycle detection prevents infinite loops
- **Capability Aggregation**: Team capabilities aggregate from members
- **Clear Roles**: Define distinct roles for team members

### 3. Error Handling

- **Use Proper Error Types**: `A2aError` for protocol, `MultiAgentError` for framework
- **Contextual Errors**: Provide context in error messages
- **Recovery Strategies**: Implement retry logic where appropriate
- **Logging**: Use `tracing` for observability

### 4. Performance

- **Connection Pooling**: Use for remote agent clients
- **Async All The Way**: Avoid blocking operations in agent logic
- **Resource Cleanup**: Properly drop unused agents/connections
- **Timeout Configuration**: Set appropriate timeouts for remote calls

## Additional Resources

- [A2A Protocol Specification](https://a2a-protocol.org/)
- [RANCH Repository](https://github.com/yourusername/ranch)
- [Multi-Agent Examples](multi-agent/examples/)
- [A2A Protocol Examples](a2a-protocol/examples/)

## Contributing

When implementing agents:

1. Follow trait contracts precisely
2. Add comprehensive tests
3. Document public APIs with rustdoc
4. Include usage examples
5. Update relevant AGENT.md files

---

**Last Updated**: 2025-12-11  
**Version**: 2.0.0
