# Quickstart Guide: Complete A2A Integration

**Feature**: Complete A2A Integration with SDK Enhancements  
**Audience**: Developers new to the feature  
**Time to Complete**: 15 minutes

## Prerequisites

- Rust Edition 2021 installed
- Basic familiarity with async Rust and Tokio
- RANCH multi-agent framework cloned locally

## Step 1: Expose a Team as an A2A Service (5 min)

### Goal
Start a TeamServer that exposes your multi-agent team via HTTP/JSON-RPC 2.0.

### Code

```rust
use multi_agent::{Team, TeamServer, TeamConfig, AgentManager};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Create agent manager and register agents
    let manager = Arc::new(AgentManager::new());
    
    // Register your agents (A2A, OpenAI, or other teams)
    manager.register(Arc::new(my_agent)).await?;
    
    // 2. Load team configuration
    let team_config = TeamConfig {
        id: "my-team".to_string(),
        name: "My Team".to_string(),
        description: "A collaborative team".to_string(),
        mode: TeamMode::Supervisor,
        agents: vec![/* agent configs */],
        scheduler_config: SchedulerConfig::Supervisor(/* config */),
    };
    
    // 3. Create team
    let team = Arc::new(Team::new(team_config, manager));
    
    // 4. Create and start server
    let server = TeamServer::new(team, 3000);
    println!("Starting server on http://localhost:3000");
    server.start().await?;
    
    Ok(())
}
```

### Test It

```bash
# In terminal 1: Start the server
cargo run --example team_server

# In terminal 2: Send a request
curl -X POST http://localhost:3000/rpc \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "agent/card",
    "params": {},
    "id": 1
  }'
```

**Expected Output**: JSON response with team's capabilities

---

## Step 2: Use Ergonomic Config Conversions (3 min)

### Goal
Convert configuration files directly to agent instances without boilerplate.

### Before (Manual Conversion)
```rust
// 15+ lines of manual field mapping
let a2a_config = A2AAgentConfig {
    endpoint: config.endpoint.clone(),
    agent_id: Some(config.id.clone()),
    timeout: Duration::from_secs(config.timeout_seconds),
    max_retries: config.max_retries,
    auth: /* extract from metadata */,
    task_handling: /* parse from metadata */,
};
```

### After (Trait Conversion)
```rust
// 1 line - clean and idiomatic
let a2a_config: A2AAgentConfig = agent_config.into();
let agent = A2AAgent::with_config(client, a2a_config);
```

### Full Example

```rust
use multi_agent::{Config, A2AAgentConfig, A2AAgent};

// Load configuration from TOML
let config = Config::from_file("config.toml")?;

// Convert and create agents
for agent_config in config.agents {
    match agent_config.protocol {
        ProtocolType::A2A => {
            let a2a_config: A2AAgentConfig = agent_config.into();
            let agent = create_a2a_agent(a2a_config)?;
            manager.register(Arc::new(agent)).await?;
        }
        ProtocolType::OpenAI => {
            let openai_config: OpenAIAgentConfig = agent_config.into();
            let agent = OpenAIAgent::with_config(openai_config);
            manager.register(Arc::new(agent)).await?;
        }
    }
}
```

### Configuration File (config.toml)
```toml
[[agents]]
id = "research-agent"
name = "Research Agent"
endpoint = "https://research.example.com/rpc"
protocol = "a2a"
capabilities = ["research"]
timeout_seconds = 30
max_retries = 3

[agents.metadata]
api_key = "sk-abc123"
task_handling = "poll"
```

---

## Step 3: Create a Nested Team (3 min)

### Goal
Compose teams recursively - teams containing other teams.

### Code

```rust
// Create specialized sub-teams
let research_team = Arc::new(Team::new(research_config, manager.clone()));
let writing_team = Arc::new(Team::new(writing_config, manager.clone()));

// Register sub-teams as agents
manager.register(research_team.clone()).await?;
manager.register(writing_team.clone()).await?;

// Create parent team that coordinates sub-teams
let parent_config = TeamConfig {
    id: "parent-team".to_string(),
    mode: TeamMode::Supervisor,
    agents: vec![
        TeamAgentConfig {
            agent_id: "research-team".to_string(),
            role: "researcher",
            capabilities: vec!["research".to_string()],
        },
        TeamAgentConfig {
            agent_id: "writing-team".to_string(),
            role: "writer",
            capabilities: vec!["writing".to_string()],
        },
    ],
    // ... rest of config
};

let parent_team = Team::new(parent_config, manager);

// Use parent team - it will delegate to sub-teams
let message = user_message("Research and write about dragons");
let response = parent_team.process(message).await?;
```

### What Happens
1. Parent team receives message
2. Supervisor scheduler selects research-team (based on "research" capability)
3. Research team processes, returns research results
4. Supervisor selects writing-team (based on "writing" capability)
5. Writing team uses research to write content
6. Final response returned to caller

---

## Step 4: Run Complete Examples (4 min)

### Available Examples

```bash
# 1. Simple team with two agents
cargo run --example simple_team

# 2. Supervisor mode - dynamic delegation
cargo run --example supervisor_team

# 3. Workflow mode - sequential processing
cargo run --example workflow_team

# 4. Remote A2A agents - distributed system
cargo run --example remote_agents

# 5. Team as A2A service - server mode
cargo run --example team_server
```

### Example: Supervisor Team

```rust
// supervisor_team.rs demonstrates:
// - Creating a supervisor agent that makes decisions
// - Registering specialist agents (research, writing, editing)
// - Letting supervisor dynamically delegate based on task
// - Error handling and logging

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup tracing
    tracing_subscriber::fmt::init();
    
    // Create agents
    let supervisor = Arc::new(/* supervisor agent */);
    let research_agent = Arc::new(/* research agent */);
    let writing_agent = Arc::new(/* writing agent */);
    
    // Register agents
    let manager = Arc::new(AgentManager::new());
    manager.register(supervisor).await?;
    manager.register(research_agent).await?;
    manager.register(writing_agent).await?;
    
    // Create supervisor team
    let team_config = /* supervisor config */;
    let team = Team::new(team_config, manager);
    
    // Process messages
    let message = user_message("Write a story about dragons");
    println!("📨 Input: {}", extract_text(&message).unwrap());
    
    let response = team.process(message).await?;
    println!("📝 Output: {}", extract_text(&response).unwrap());
    
    Ok(())
}
```

---

## Step 5: Explore Documentation (Optional)

### AGENT.md Files

Read these in order:

1. **Root AGENT.md** - Architecture overview
   - Trait hierarchy (multi-agent Agent vs a2a-protocol Agent)
   - Component relationships
   - When to use which trait

2. **a2a-protocol/AGENT.md** - A2A protocol guide
   - Implementing a2a-protocol Agent trait
   - JSON-RPC server setup
   - Task lifecycle management

3. **multi-agent/AGENT.md** - Framework guide
   - Implementing multi-agent Agent trait
   - Scheduler patterns
   - Team composition

### API Documentation

```bash
# Generate and open documentation
cargo doc --open
```

---

## Common Patterns

### Pattern 1: Load Config and Create Team
```rust
let config = Config::from_file("team.toml")?;
let manager = Arc::new(AgentManager::new());

// Create all agents from config
for agent_config in config.agents {
    let agent = create_agent_from_config(agent_config)?;
    manager.register(agent).await?;
}

// Create team from config
let team = Team::new(config.teams[0].clone(), manager);
```

### Pattern 2: Health Check Before Processing
```rust
if !team.health_check().await {
    eprintln!("Team unhealthy - some agents unavailable");
    return Err("Health check failed".into());
}

let response = team.process(message).await?;
```

### Pattern 3: Error Handling
```rust
match team.process(message).await {
    Ok(response) => println!("Success: {}", extract_text(&response).unwrap()),
    Err(A2aError::Internal(msg)) => eprintln!("Internal error: {}", msg),
    Err(A2aError::TaskFailed { task_id, reason }) => {
        eprintln!("Task {} failed: {}", task_id, reason)
    }
    Err(e) => eprintln!("Error: {}", e),
}
```

---

## Troubleshooting

### Server Won't Start
**Problem**: "Address already in use"  
**Solution**: Change port number or kill process using the port
```bash
lsof -i :3000  # Find process
kill -9 <PID>  # Kill it
```

### Agent Not Found
**Problem**: "Agent agent-id not found"  
**Solution**: Ensure agent is registered before creating team
```rust
manager.register(Arc::new(agent)).await?;
// Then create team
```

### Config Conversion Panics
**Problem**: "Cannot convert AgentConfig with protocol OpenAI to A2AAgentConfig"  
**Solution**: Check protocol type matches conversion
```rust
match config.protocol {
    ProtocolType::A2A => {
        let a2a_config: A2AAgentConfig = config.into();
    }
    ProtocolType::OpenAI => {
        let openai_config: OpenAIAgentConfig = config.into();
    }
}
```

---

## Next Steps

1. **Read AGENT.md** - Understand architecture deeply
2. **Run Integration Tests** - See patterns in action
   ```bash
   cargo test --test integration
   ```
3. **Build Custom Scheduler** - Extend orchestration logic
4. **Deploy to Production** - Add authentication, monitoring, rate limiting

---

## Quick Reference

### Creating Teams
```rust
let team = Team::new(config, manager);
```

### Exposing as A2A Service
```rust
let server = TeamServer::new(Arc::new(team), 3000);
server.start().await?;
```

### Config Conversions
```rust
let a2a_config: A2AAgentConfig = config.into();
let openai_config: OpenAIAgentConfig = config.into();
```

### Processing Messages
```rust
let response = team.process(message).await?;
```

### Health Checks
```rust
if team.health_check().await { /* healthy */ }
```

---

**Need Help?** Check the examples directory or read the AGENT.md files for detailed guidance.
