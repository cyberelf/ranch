# Multi-Agent Examples

This directory contains comprehensive examples demonstrating various patterns and capabilities of the RANCH multi-agent framework.

## Overview

These examples are designed to help you understand and implement different multi-agent coordination patterns. Each example is self-contained and can be run independently.

## Quick Start

All examples can be run using:

```bash
cargo run --example <example_name>
```

## Examples

### 1. Simple Team (`simple_team.rs`)

**What it demonstrates:**
- Creating mock agents for testing
- Registering agents with AgentManager
- Creating a basic two-agent team
- Processing messages through the team
- Team capability aggregation

**When to use this pattern:**
- Getting started with the framework
- Understanding basic team composition
- Testing team functionality

**Run it:**
```bash
cargo run --example simple_team
```

**Key concepts:**
- Agent trait implementation
- Team configuration
- Message processing flow

---

### 2. Supervisor Team (`supervisor_team.rs`)

**What it demonstrates:**
- Creating a supervisor agent that makes routing decisions
- Creating specialist agents with different capabilities
- Intelligent message routing based on content analysis
- Supervisor mode coordination pattern

**When to use this pattern:**
- When you need intelligent task routing
- Multiple specialists with different expertise
- Dynamic delegation based on request analysis
- Centralized coordination with distributed execution

**Run it:**
```bash
cargo run --example supervisor_team
```

**Key concepts:**
- Supervisor mode (`TeamMode::Supervisor`)
- Content-based routing logic
- Specialist agent patterns
- Decision-making in coordination

**Example output:**
```
Query: Can you help me implement a new API endpoint?
[Supervisor] Analyzing query: "Can you help me implement a new API endpoint?"
[Supervisor] Decision: Route to coding-specialist
[Coding Specialist] Processing: Can you help me implement a new API endpoint?
Response: I can help you implement that feature with clean, efficient code!
```

---

### 3. Workflow Team (`workflow_team.rs`)

**What it demonstrates:**
- Sequential message processing through multiple agents
- State transformation at each workflow step
- Multi-step processing pipeline
- Workflow mode coordination pattern

**When to use this pattern:**
- Multi-step processes (research → draft → review)
- Content transformation pipelines
- Sequential task execution
- When output of one agent becomes input for the next

**Run it:**
```bash
cargo run --example workflow_team
```

**Key concepts:**
- Workflow mode (`TeamMode::Workflow`)
- Sequential processing
- State passing between agents
- Pipeline pattern implementation

**Example flow:**
```
Input → [Research Agent] → [Draft Agent] → [Edit Agent] → Output
         Gathers info      Creates draft     Polishes content
```

---

### 4. Remote Agents (`remote_agents.rs`)

**What it demonstrates:**
- Coordinating remote A2A agents
- Distributed agent communication
- Network error handling
- A2A protocol usage
- Timeout and retry patterns

**When to use this pattern:**
- Agents distributed across different servers
- Integration with external A2A-compliant agents
- Microservices architecture
- Federated multi-agent systems

**Prerequisites:**
This example requires remote A2A agents to be running. For testing:

```bash
# Terminal 1: Start first agent
cargo run --example basic_echo_server -- --port 8081

# Terminal 2: Start second agent
cargo run --example basic_echo_server -- --port 8082

# Terminal 3: Run the example
cargo run --example remote_agents
```

**Run it:**
```bash
cargo run --example remote_agents
```

**Key concepts:**
- A2AAgent configuration
- JsonRpcTransport setup
- Remote agent registration
- Network timeout handling
- Distributed coordination

---

### 5. Team Server (`team_server.rs`)

**What it demonstrates:**
- Exposing a team as an HTTP service
- JSON-RPC 2.0 server implementation
- A2A protocol compliance
- Graceful shutdown handling
- API endpoint testing with curl

**When to use this pattern:**
- Exposing teams as services
- Building multi-agent APIs
- Enabling external systems to interact with teams
- Production deployment scenarios

**Run it:**
```bash
cargo run --example team_server
```

**Test the server:**

Get agent card (team capabilities):
```bash
curl -X POST http://localhost:8080/rpc \
  -H 'Content-Type: application/json' \
  -d '{
    "jsonrpc": "2.0",
    "method": "agent/card",
    "params": {},
    "id": 1
  }'
```

Send a message:
```bash
curl -X POST http://localhost:8080/rpc \
  -H 'Content-Type: application/json' \
  -d '{
    "jsonrpc": "2.0",
    "method": "message/send",
    "params": {
      "message": {
        "role": "user",
        "parts": [{"text": "Hello team!"}]
      }
    },
    "id": 2
  }'
```

**Key concepts:**
- TeamServer configuration
- JSON-RPC 2.0 endpoints
- A2A protocol methods:
  - `message/send` - Send messages to team
  - `task/get` - Get task details
  - `task/status` - Check task status
  - `task/cancel` - Cancel tasks
  - `agent/card` - Get team capabilities
- CORS configuration
- Graceful shutdown (Ctrl+C)

---

### 6. Fantasy Story Writer (`fantasy_story_writer.rs`)

**What it demonstrates:**
- Real-world multi-agent application
- Complex team composition
- Story generation workflow
- OpenAI integration example

**Run it:**
```bash
cargo run --example fantasy_story_writer
```

See [README_FANTASY_STORY.md](README_FANTASY_STORY.md) for detailed documentation.

---

## Pattern Selection Guide

### Choose **Simple Team** when:
- ✓ Learning the framework basics
- ✓ Building proof-of-concepts
- ✓ Testing individual agents

### Choose **Supervisor Team** when:
- ✓ Intelligent routing is needed
- ✓ Multiple specialists with different skills
- ✓ Dynamic task delegation
- ✓ Centralized decision-making

### Choose **Workflow Team** when:
- ✓ Sequential processing is required
- ✓ Multi-stage pipelines
- ✓ Content transformation flows
- ✓ Each step builds on the previous

### Choose **Remote Agents** when:
- ✓ Agents are distributed across servers
- ✓ Integrating with external services
- ✓ Microservices architecture
- ✓ Scalability is important

### Choose **Team Server** when:
- ✓ Exposing teams as HTTP services
- ✓ Building APIs for external clients
- ✓ Production deployments
- ✓ Integration with non-Rust systems

---

## Common Patterns

### Creating a Mock Agent

```rust
use a2a_protocol::prelude::Message;
use async_trait::async_trait;
use multi_agent::{Agent, AgentInfo, A2aResult, extract_text};

struct MockAgent {
    id: String,
    name: String,
}

#[async_trait]
impl Agent for MockAgent {
    async fn info(&self) -> A2aResult<AgentInfo> {
        Ok(AgentInfo {
            id: self.id.clone(),
            name: self.name.clone(),
            description: "Mock agent for testing".to_string(),
            capabilities: vec!["testing".to_string()],
            metadata: std::collections::HashMap::new(),
        })
    }

    async fn process(&self, message: Message) -> A2aResult<Message> {
        let input = extract_text(&message).unwrap_or_default();
        Ok(Message::agent_text(&format!("Echo: {}", input)))
    }
}
```

### Setting Up a Team

```rust
use multi_agent::team::{TeamConfig, TeamMode, TeamAgentConfig, SchedulerConfig, SupervisorSchedulerConfig};
use std::sync::Arc;

let team_config = TeamConfig {
    id: "my-team".to_string(),
    name: "My Team".to_string(),
    description: "A sample team".to_string(),
    mode: TeamMode::Supervisor,
    agents: vec![
        TeamAgentConfig {
            agent_id: "agent-1".to_string(),
            role: "coordinator".to_string(),
            capabilities: vec!["coordination".to_string()],
        },
    ],
    scheduler_config: SchedulerConfig::Supervisor(SupervisorSchedulerConfig {
        supervisor_agent_id: "agent-1".to_string(),
    }),
};

let team = Arc::new(Team::new(team_config, manager));
```

### Processing Messages

```rust
use a2a_protocol::prelude::Message;

// Create a message
let message = Message::user_text("Hello team!");

// Process through team
let response = team.process(message).await?;

// Extract text from response
let text = extract_text(&response).unwrap_or_default();
println!("Response: {}", text);
```

---

## Architecture Documentation

For detailed architecture information, see:
- [Root AGENT.md](../../AGENT.md) - Overall architecture overview
- [multi-agent AGENT.md](../AGENT.md) - Multi-agent framework guide
- [a2a-protocol AGENT.md](../../a2a-protocol/AGENT.md) - A2A protocol guide

---

## Troubleshooting

### Common Issues

**"No such example" error:**
- Verify the example name matches the file name (without `.rs`)
- Run from the repository root directory

**"Connection refused" for remote agents:**
- Ensure remote agents are running before starting the example
- Check port numbers match the configuration
- Verify firewall settings allow connections

**Compilation errors:**
- Run `cargo build` to see detailed error messages
- Ensure all dependencies are up to date: `cargo update`

**Port already in use:**
- Choose a different port number
- Kill existing process: `lsof -ti:8080 | xargs kill` (macOS/Linux)

---

## Next Steps

1. **Run all examples** to understand different patterns
2. **Modify examples** to experiment with configurations
3. **Create custom agents** based on your use case
4. **Build your own team** combining multiple patterns
5. **Deploy with TeamServer** for production use

---

## Contributing

When adding new examples:
1. Follow the existing pattern structure
2. Include comprehensive documentation in the file header
3. Add clear console output showing what's happening
4. Update this README with the new example
5. Test thoroughly before committing

---

## Additional Resources

- [Multi-Agent Framework Documentation](../README.md)
- [A2A Protocol Specification](https://a2a-protocol.org/)
- [Rust Async Programming Book](https://rust-lang.github.io/async-book/)
- [Tokio Documentation](https://tokio.rs/)
