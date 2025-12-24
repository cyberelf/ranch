# Multi-Agent Client Collaboration Guide

**Important: This guide covers client-side coordination of remote agents, NOT agent implementation.**

## Table of Contents

- [Overview](#overview)
- [Core Principle](#core-principle)
- [A2AAgent - Remote Agent Client](#a2aagent---remote-agent-client)
- [Team Composition](#team-composition)
- [Router-Based Coordination](#router-based-coordination)
- [Client Agent Extension](#client-agent-extension)
- [Code Examples](#code-examples)
- [Best Practices](#best-practices)

## Overview

The `multi-agent` framework is a **client-side collaboration ground** for coordinating remote agents. It does NOT provide agent implementation capabilities - agents must be implemented as A2A protocol servers using the `a2a-protocol` crate.

**What multi-agent provides:**
- Client connections to remote A2A agents (`A2AAgent`)
- Team composition and routing logic
- Dynamic message routing between remote agents
- Extension support for intelligent routing decisions

**What multi-agent does NOT provide:**
- Agent implementation (use `a2a-protocol` for that)
- Local agent execution
- Mock agents for testing (use real A2A servers instead)

## Core Principle

### ✅ Correct Usage

```rust
// 1. Start A2A protocol servers (in separate processes/services)
// (See a2a-protocol crate for server implementation)

// 2. Connect to remote agents via A2AAgent
use multi_agent::{A2AAgent, AgentManager, Team};
use a2a_protocol::prelude::*;

let transport = Arc::new(JsonRpcTransport::new("http://localhost:3000/rpc")?);
let client = A2aClient::new(transport);
let remote_agent = Arc::new(A2AAgent::new(client));

// 3. Register and coordinate remote agents
let manager = Arc::new(AgentManager::new());
manager.register(remote_agent).await?;
```

### ❌ Incorrect Usage

```rust
// DON'T: Implement Agent trait locally for new agents
struct MyLocalAgent { /* ... */ }

#[async_trait]
impl Agent for MyLocalAgent {  // ❌ Wrong approach
    async fn info(&self) -> A2aResult<AgentInfo> { /* ... */ }
    async fn process(&self, msg: Message) -> A2aResult<Message> { /* ... */ }
}
```

**Instead:** Implement agents as A2A protocol servers using `a2a-protocol::server::ProtocolAgent`
```

### Implementing a Custom Agent

```rust
use multi_agent::*;
use async_trait::async_trait;

struct CalculatorAgent;

#[async_trait]
impl Agent for CalculatorAgent {
    async fn info(&self) -> MultiAgentResult<AgentInfo> {
        Ok(AgentInfo {
            id: "calculator".to_string(),
            name: "Calculator Agent".to_string(),
            description: "Performs mathematical calculations".to_string(),
            capabilities: vec![
                "addition".to_string(),
                "subtraction".to_string(),
                "multiplication".to_string(),
                "division".to_string(),
            ],
        })
    }

    async fn process_message(&self, message: Message) -> MultiAgentResult<Message> {
        let text = extract_text(&message)
            .ok_or_else(|| MultiAgentError::InvalidMessage {
                reason: "No text content".to_string()
            })?;
        
        // Parse and compute
        let result = self.calculate(&text)?;
        
        Ok(Message::agent_text(format!("Result: {}", result)))
    }
}
```

## Team Composition

A `Team` orchestrates multiple agents to accomplish complex tasks. Teams implement the `Agent` trait, enabling nested composition.

### Team Structure

```rust
pub struct Team {
    config: TeamConfig,
    agent_manager: Arc<AgentManager>,
    scheduler: Arc<dyn Scheduler>,
}
```

### Team Modes

#### 1. Supervisor Mode

A designated supervisor agent receives all messages and delegates to specialists:

```
User Message
     ↓
Supervisor Agent (analyzes request)
     ↓
Delegates to → Specialist Agent 1
            → Specialist Agent 2
            → Specialist Agent 3
```

**Use Cases**:
- Dynamic task delegation based on content
- Expert systems with a coordinator
- Adaptive routing based on agent availability

**Example**:
```rust
let team_config = TeamConfig {
    id: "research-team".to_string(),
    name: "Research Team".to_string(),
    description: "Collaborative research team".to_string(),
    mode: TeamMode::Supervisor,
    agents: vec![
        TeamAgentConfig {
            agent_id: "supervisor".to_string(),
            role: "coordinator".to_string(),
            capabilities: vec!["delegation".to_string()],
        },
        TeamAgentConfig {
            agent_id: "researcher-1".to_string(),
            role: "specialist".to_string(),
            capabilities: vec!["web-search".to_string()],
        },
        TeamAgentConfig {
            agent_id: "researcher-2".to_string(),
            role: "specialist".to_string(),
            capabilities: vec!["data-analysis".to_string()],
        },
    ],
    scheduler_config: SchedulerConfig::Supervisor(SupervisorSchedulerConfig {
        supervisor_id: "supervisor".to_string(),
        max_iterations: 10,
    }),
};
```

#### 2. Workflow Mode

Messages flow through a predefined sequence of agents:

```
User Message
     ↓
Agent 1 (research)
     ↓
Agent 2 (draft)
     ↓
Agent 3 (review)
     ↓
Final Response
```

**Use Cases**:
- Sequential processing pipelines
- Multi-stage workflows
- Assembly line patterns

**Example**:
```rust
let team_config = TeamConfig {
    id: "content-team".to_string(),
    name: "Content Creation Team".to_string(),
    description: "Creates and refines content".to_string(),
    mode: TeamMode::Workflow,
    agents: vec![
        TeamAgentConfig {
            agent_id: "researcher".to_string(),
            role: "research".to_string(),
            capabilities: vec!["research".to_string()],
        },
        TeamAgentConfig {
            agent_id: "writer".to_string(),
            role: "draft".to_string(),
            capabilities: vec!["writing".to_string()],
        },
        TeamAgentConfig {
            agent_id: "editor".to_string(),
            role: "review".to_string(),
            capabilities: vec!["editing".to_string()],
        },
    ],
    scheduler_config: SchedulerConfig::Workflow(WorkflowSchedulerConfig {
        steps: vec![
            WorkflowStepConfig {
                agent_id: "researcher".to_string(),
                condition: None,
            },
            WorkflowStepConfig {
                agent_id: "writer".to_string(),
                condition: None,
            },
            WorkflowStepConfig {
                agent_id: "editor".to_string(),
                condition: None,
            },
        ],
    }),
};
```

## Scheduler Patterns

Schedulers determine how messages are routed within a team.

### Scheduler Trait

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

### SupervisorScheduler

Routes all messages to a designated supervisor agent who decides delegation:

```rust
pub struct SupervisorScheduler {
    config: SupervisorSchedulerConfig,
}

pub struct SupervisorSchedulerConfig {
    pub supervisor_id: String,
    pub max_iterations: usize,
}
```

**Behavior**:
1. First message → Supervisor
2. Supervisor response analyzed for delegation signals
3. Continue until max iterations or task complete

### WorkflowScheduler

Routes messages through a predefined sequence:

```rust
pub struct WorkflowScheduler {
    config: WorkflowSchedulerConfig,
}

pub struct WorkflowSchedulerConfig {
    pub steps: Vec<WorkflowStepConfig>,
}

pub struct WorkflowStepConfig {
    pub agent_id: String,
    pub condition: Option<String>,  // Future: conditional routing
}
```

**Behavior**:
1. Route to first agent in sequence
2. Capture response, route to next agent
3. Each agent sees previous agent's output
4. Final agent's response returned to user

### Custom Scheduler

Implement your own routing logic:

```rust
use multi_agent::*;
use async_trait::async_trait;

struct CapabilityScheduler {
    capability_map: HashMap<String, Vec<String>>,
}

#[async_trait]
impl Scheduler for CapabilityScheduler {
    async fn determine_next_recipient(
        &self,
        message: &Message,
        context: &SchedulerContext,
    ) -> MultiAgentResult<String> {
        // Extract keywords from message
        let text = extract_text(message).unwrap_or("");
        let keywords = extract_keywords(text);
        
        // Match to agent capabilities
        for keyword in keywords {
            if let Some(agents) = self.capability_map.get(&keyword) {
                if let Some(agent_id) = agents.first() {
                    return Ok(agent_id.clone());
                }
            }
        }
        
        // Default to first available agent
        context.available_agents
            .first()
            .cloned()
            .ok_or_else(|| MultiAgentError::NoAgentsAvailable)
    }
}
```

## Configuration via TOML

Define agents and teams declaratively:

### Agent Configuration

```toml
[[agents]]
id = "researcher"
name = "Research Agent"
endpoint = "https://research-agent.example.com/rpc"
protocol = "a2a"
capabilities = ["web-search", "data-gathering"]
timeout_seconds = 60
max_retries = 3

[agents.metadata]
task_handling = "poll_until_complete"

[[agents]]
id = "writer"
name = "Writer Agent"
endpoint = "https://api.openai.com/v1"
protocol = "openai"
capabilities = ["writing", "editing"]
timeout_seconds = 30
max_retries = 3

[agents.metadata]
api_key = "sk-..."
model = "gpt-4"
temperature = "0.7"
max_tokens = "2000"
```

### Team Configuration

```toml
[[teams]]
id = "content-team"
name = "Content Creation Team"
description = "Creates high-quality content through research and writing"
mode = "workflow"

[[teams.agents]]
agent_id = "researcher"
role = "research"
capabilities = ["research"]

[[teams.agents]]
agent_id = "writer"
role = "writer"
capabilities = ["writing"]

[teams.scheduler_config.workflow]
steps = [
    { agent_id = "researcher" },
    { agent_id = "writer" }
]
```

### Loading Configuration

```rust
use multi_agent::*;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load config from file
    let config = Config::from_file("config.toml")?;
    
    // Create agent manager
    let manager = Arc::new(AgentManager::new());
    
    // Register agents
    for agent_config in config.to_agent_configs() {
        let agent = create_agent_from_config(agent_config)?;
        manager.register(agent).await?;
    }
    
    // Create teams
    let teams = config.to_team_configs();
    for team_config in teams {
        let team = Arc::new(Team::new(team_config, manager.clone()));
        // Use team...
    }
    
    Ok(())
}
```

### Config Conversion with TryFrom

Use type-safe config conversion:

```rust
use multi_agent::*;

// For A2A agents
let agent_config = /* ... */;
let a2a_config: A2AAgentConfig = agent_config.try_into()?;

let transport = JsonRpcTransport::new(&endpoint)?;
let client = A2aClient::new(Arc::new(transport));
let agent = A2AAgent::with_config(client, a2a_config);

// For OpenAI agents
let agent_config = /* ... */;
let openai_config: OpenAIAgentConfig = agent_config.try_into()?;

let agent = OpenAIAgent::new(endpoint, openai_config);
```

## Nested Teams

Teams can contain other teams, enabling hierarchical composition:

```
Organization Team (Supervisor)
    ↓
    ├── Research Department Team (Workflow)
    │   ├── Data Collector Agent
    │   ├── Analyst Agent
    │   └── Reporter Agent
    │
    ├── Development Department Team (Workflow)
    │   ├── Designer Agent
    │   ├── Coder Agent
    │   └── Tester Agent
    │
    └── Marketing Department Team (Supervisor)
        ├── Market Researcher Agent
        ├── Content Creator Agent
        └── Social Media Agent
```

### Cycle Detection

The framework automatically prevents circular dependencies:

```rust
// This is detected and prevented:
// Team A contains Team B
// Team B contains Team A
// → CycleError
```

### Implementing Nested Teams

```rust
use multi_agent::*;
use std::sync::Arc;

async fn create_nested_organization() -> Result<Arc<Team>, Box<dyn std::error::Error>> {
    let manager = Arc::new(AgentManager::new());
    
    // Register individual agents
    let researcher = Arc::new(ResearchAgent::new());
    let analyst = Arc::new(AnalystAgent::new());
    let writer = Arc::new(WriterAgent::new());
    
    manager.register(researcher).await?;
    manager.register(analyst).await?;
    manager.register(writer).await?;
    
    // Create research department team
    let research_team_config = TeamConfig {
        id: "research-dept".to_string(),
        name: "Research Department".to_string(),
        mode: TeamMode::Workflow,
        agents: vec![
            TeamAgentConfig {
                agent_id: "researcher".to_string(),
                role: "research".to_string(),
                capabilities: vec!["research".to_string()],
            },
            TeamAgentConfig {
                agent_id: "analyst".to_string(),
                role: "analysis".to_string(),
                capabilities: vec!["analysis".to_string()],
            },
        ],
        scheduler_config: SchedulerConfig::Workflow(WorkflowSchedulerConfig {
            steps: vec![
                WorkflowStepConfig { agent_id: "researcher".to_string(), condition: None },
                WorkflowStepConfig { agent_id: "analyst".to_string(), condition: None },
            ],
        }),
        // ... other config
    };
    
    let research_team = Arc::new(Team::new(research_team_config, manager.clone()));
    
    // Register team as an agent!
    manager.register(research_team.clone() as Arc<dyn Agent>).await?;
    
    // Create organization-level team that includes the research team
    let org_team_config = TeamConfig {
        id: "organization".to_string(),
        name: "Organization".to_string(),
        mode: TeamMode::Supervisor,
        agents: vec![
            TeamAgentConfig {
                agent_id: "writer".to_string(),
                role: "writer".to_string(),
                capabilities: vec!["writing".to_string()],
            },
            TeamAgentConfig {
                agent_id: "research-dept".to_string(),  // Team as agent!
                role: "research".to_string(),
                capabilities: vec!["research".to_string(), "analysis".to_string()],
            },
        ],
        scheduler_config: SchedulerConfig::Supervisor(SupervisorSchedulerConfig {
            supervisor_id: "writer".to_string(),
            max_iterations: 5,
        }),
        // ... other config
    };
    
    let org_team = Arc::new(Team::new(org_team_config, manager));
    
    Ok(org_team)
}
```

### Best Practices for Nested Teams

1. **Limit Depth**: Keep nesting to 2-3 levels maximum
2. **Clear Boundaries**: Each team should have a well-defined responsibility
3. **Capability Aggregation**: Parent teams inherit child capabilities
4. **Error Propagation**: Errors bubble up through the hierarchy
5. **Performance**: Nested teams add latency; use judiciously

## Code Examples

### Simple Two-Agent Team

```rust
use multi_agent::*;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create manager
    let manager = Arc::new(AgentManager::new());
    
    // Create and register agents
    let agent1 = Arc::new(MyAgent::new("agent-1", "First Agent"));
    let agent2 = Arc::new(MyAgent::new("agent-2", "Second Agent"));
    
    manager.register(agent1).await?;
    manager.register(agent2).await?;
    
    // Create team config
    let team_config = TeamConfig {
        id: "simple-team".to_string(),
        name: "Simple Team".to_string(),
        description: "A simple two-agent team".to_string(),
        mode: TeamMode::Supervisor,
        agents: vec![
            TeamAgentConfig {
                agent_id: "agent-1".to_string(),
                role: "supervisor".to_string(),
                capabilities: vec!["coordination".to_string()],
            },
            TeamAgentConfig {
                agent_id: "agent-2".to_string(),
                role: "worker".to_string(),
                capabilities: vec!["processing".to_string()],
            },
        ],
        scheduler_config: SchedulerConfig::Supervisor(SupervisorSchedulerConfig {
            supervisor_id: "agent-1".to_string(),
            max_iterations: 5,
        }),
    };
    
    // Create team
    let team = Arc::new(Team::new(team_config, manager));
    
    // Use team
    let message = Message::user_text("Hello, team!");
    let response = team.process_message(message).await?;
    
    println!("Response: {}", extract_text(&response).unwrap_or(""));
    
    Ok(())
}
```

### Workflow Team with Remote Agents

```rust
use multi_agent::*;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manager = Arc::new(AgentManager::new());
    
    // Create remote A2A agents
    let research_transport = JsonRpcTransport::new("https://research.example.com/rpc")?;
    let research_client = A2aClient::new(Arc::new(research_transport));
    let research_agent = Arc::new(A2AAgent::new(research_client));
    
    let writer_transport = JsonRpcTransport::new("https://writer.example.com/rpc")?;
    let writer_client = A2aClient::new(Arc::new(writer_transport));
    let writer_agent = Arc::new(A2AAgent::new(writer_client));
    
    // Register agents
    let research_id = manager.register(research_agent).await?;
    let writer_id = manager.register(writer_agent).await?;
    
    // Create workflow team
    let team_config = TeamConfig {
        id: "content-pipeline".to_string(),
        name: "Content Pipeline".to_string(),
        description: "Research then write".to_string(),
        mode: TeamMode::Workflow,
        agents: vec![
            TeamAgentConfig {
                agent_id: research_id,
                role: "research".to_string(),
                capabilities: vec!["research".to_string()],
            },
            TeamAgentConfig {
                agent_id: writer_id,
                role: "writing".to_string(),
                capabilities: vec!["writing".to_string()],
            },
        ],
        scheduler_config: SchedulerConfig::Workflow(WorkflowSchedulerConfig {
            steps: vec![
                WorkflowStepConfig { agent_id: research_id.clone(), condition: None },
                WorkflowStepConfig { agent_id: writer_id.clone(), condition: None },
            ],
        }),
    };
    
    let team = Arc::new(Team::new(team_config, manager));
    
    // Process through workflow
    let message = Message::user_text("Write an article about Rust async programming");
    let response = team.process_message(message).await?;
    
    println!("Article: {}", extract_text(&response).unwrap_or(""));
    
    Ok(())
}
```

### Exposing Team as A2A Service

```rust
use multi_agent::*;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create team (as above)
    let team = create_my_team().await?;
    
    // Create TeamServer
    let server = TeamServer::new(team, 8080);
    
    println!("🚀 Team server starting on port 8080");
    println!("📡 A2A endpoint: http://localhost:8080/rpc");
    println!();
    println!("Test with curl:");
    println!(r#"curl -X POST http://localhost:8080/rpc \
  -H "Content-Type: application/json" \
  -d '{{
    "jsonrpc": "2.0",
    "method": "message/send",
    "params": {{
      "message": {{
        "role": "user",
        "parts": [{{"type": "text", "text": "Hello team!"}}]
      }}
    }},
    "id": 1
  }}'"#);
    
    // Start server (blocks)
    server.start().await?;
    
    Ok(())
}
```

## Best Practices

### 1. Agent Design

- **Single Responsibility**: Each agent should do one thing well
- **Stateless Preferred**: Easier to scale and test
- **Capability-Driven**: Design around capabilities, not implementations
- **Error Handling**: Return meaningful errors, don't panic

### 2. Team Composition

- **Right Mode**: Choose supervisor for dynamic delegation, workflow for fixed pipelines
- **Agent Selection**: Include agents with complementary capabilities
- **Size Limits**: Teams of 3-7 agents are optimal
- **Clear Roles**: Each agent should have a distinct role

### 3. Scheduler Selection

| Use Case | Recommended Scheduler |
|----------|---------------------|
| Dynamic task routing | Supervisor |
| Fixed processing pipeline | Workflow |
| Content-based routing | Custom |
| Expert delegation | Supervisor |
| Sequential refinement | Workflow |

### 4. Configuration Management

- **Environment Variables**: Use for secrets (API keys)
- **Version Control**: Commit configs (except secrets)
- **Validation**: Use TryFrom for type-safe conversion
- **Documentation**: Comment complex configurations

### 5. Error Handling

```rust
// Use specific error types
return Err(MultiAgentError::AgentNotFound {
    agent_id: id.to_string()
});

// Provide context
return Err(MultiAgentError::ProcessingFailed {
    agent_id: agent_id.to_string(),
    reason: format!("Timeout after {}s", timeout),
});

// Log before returning errors
tracing::error!("Failed to process message: {}", e);
return Err(e);
```

### 6. Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_team_orchestration() {
        let manager = Arc::new(AgentManager::new());
        
        // Register mock agents
        let agent = Arc::new(MockAgent::new());
        manager.register(agent).await.unwrap();
        
        // Create team
        let team = Team::new(team_config, manager);
        
        // Test processing
        let message = Message::user_text("test");
        let response = team.process_message(message).await.unwrap();
        
        assert_eq!(response.role, MessageRole::Agent);
    }
}
```

### 7. Monitoring and Observability

```rust
use tracing::{info, warn, error, instrument};

#[instrument(skip(self, message))]
async fn process_message(&self, message: Message) -> MultiAgentResult<Message> {
    info!(
        team_id = %self.config.id,
        team_mode = ?self.config.mode,
        "Processing message"
    );
    
    let start = std::time::Instant::now();
    
    let result = self.orchestrate(message).await;
    
    let duration = start.elapsed();
    info!(
        team_id = %self.config.id,
        duration_ms = duration.as_millis(),
        success = result.is_ok(),
        "Completed processing"
    );
    
    result
}
```

### 8. Performance Optimization

- **Connection Pooling**: Reuse connections to remote agents
- **Caching**: Cache agent info to reduce lookups
- **Parallel Execution**: Run independent agents concurrently
- **Timeouts**: Set appropriate timeouts at all levels

```rust
// Parallel execution example
let futures: Vec<_> = agents.iter()
    .map(|agent| agent.process_message(message.clone()))
    .collect();

let results = futures::future::join_all(futures).await;
```

## Differences from a2a-protocol::Agent

| Aspect | multi_agent::Agent | a2a_protocol::Agent |
|--------|-------------------|-------------------|
| Purpose | Framework orchestration | Protocol compliance |
| Info Method | `info()` → `AgentInfo` | `profile()` → `AgentProfile` |
| Context | Team/scheduler aware | Protocol-only |
| Use With | AgentManager, Team | TaskAwareHandler, JsonRpcRouter |
| Nesting | Supports team nesting | Single agent focus |

Both traits are needed: use `a2a_protocol::Agent` for A2A compliance, and `multi_agent::Agent` for framework features. The `A2AAgent` adapter bridges them.

## Additional Resources

- [Root AGENT.md](../AGENT.md) - Architecture overview
- [a2a-protocol/AGENT.md](../a2a-protocol/AGENT.md) - A2A protocol details
- [Examples](../examples/) - Working examples
- [Configuration Guide](README.md#configuration) - TOML config reference

## Contributing

When extending the framework:

1. Follow existing patterns (Agent trait, Scheduler trait)
2. Add comprehensive tests
3. Document with rustdoc
4. Provide usage examples
5. Update this guide

---

**Last Updated**: 2025-12-11  
**Framework Version**: 2.0.0
