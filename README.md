# Multi-Agent Runtime Framework

A Rust-based framework for building and managing multi-agent systems with support for different communication protocols and scheduling modes.

## Features

- **Protocol Support**: OpenAI API and A2A (Agent-to-Agent) protocols
- **Agent Management**: Dynamic agent registration, health checks, and capability discovery
- **Team Composition**: Group agents into teams with different collaboration modes
- **Scheduling Modes**: 
  - Supervisor mode: One agent acts as a coordinator
  - Workflow mode: Sequential execution based on configuration
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
mode = "supervisor"

[[teams.agents]]
agent_id = "research-assistant"
role = "supervisor"
is_supervisor = true
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
- **Team**: Groups agents with specific collaboration patterns
- **Scheduler**: Implements different agent coordination strategies
- **TeamServer**: Provides HTTP API endpoints

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
let team_config = TeamConfig {
    id: "my-team".to_string(),
    name: "My Team".to_string(),
    description: "A sample team".to_string(),
    mode: TeamMode::Supervisor,
    agents: vec![TeamAgentConfig {
        agent_id: "my-agent".to_string(),
        role: "member".to_string(),
        capabilities: vec!["analysis".to_string()],
        is_supervisor: Some(true),
        order: None,
    }],
      context: HashMap::new(),
};

let team = Arc::new(Team::new(team_config, agent_manager));
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