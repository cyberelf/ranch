# GitHub Copilot Instructions for RANCH

## Project Overview

**RANCH** (Rust Agent Networking & Coordination Hub) is a multi-agent system framework built in Rust for managing, coordinating, and facilitating communication between autonomous agents. The project implements the A2A (Agent-to-Agent) protocol and provides a robust runtime for multi-agent coordination.

## Repository Structure

This is a Rust workspace with two main crates:

- **`a2a-protocol/`**: Standalone implementation of the A2A (Agent-to-Agent) communication protocol v0.3.0, providing JSON-RPC 2.0 transport, task management, and agent discovery
- **`multi-agent/`**: Multi-agent runtime framework that orchestrates agents using different protocols (OpenAI API and A2A) with team composition and scheduling capabilities

## Development Guidelines

### Language and Tooling

- **Primary Language**: Rust (Edition 2021)
- **Build System**: Cargo (workspace-based)
- **Async Runtime**: Tokio with full features
- **Web Framework**: Axum 0.7 for HTTP servers
- **Testing**: Use `cargo test` for all tests
- **Linting**: Follow Clippy recommendations
- **Formatting**: Use `rustfmt` for consistent code style

### Building and Testing

```bash
# Build the entire workspace
cargo build

# Run all tests
cargo test

# Build specific crate
cargo build -p a2a-protocol
cargo build -p multi-agent

# Run examples
cargo run --example basic_agent
```

### Code Style and Conventions

1. **Error Handling**
   - Use `thiserror` for custom error types
   - Use `anyhow` for application-level error propagation
   - Always use `Result` types for fallible operations
   - Prefer specific error types over generic errors

2. **Async Code**
   - Use `#[async_trait]` for async trait methods
   - Leverage Tokio's async primitives (`spawn`, `select!`, etc.)
   - Use `Arc` for shared state across async tasks
   - Prefer `RwLock` from `tokio::sync` for async-compatible locking

3. **Type Safety**
   - Use strong typing with newtype patterns where appropriate
   - Leverage `serde` for serialization/deserialization
   - Use builder patterns for complex type construction
   - Prefer owned types over references in public APIs

4. **API Design**
   - Public APIs should be well-documented with rustdoc
   - Use `prelude` modules for commonly used imports
   - Feature-gate optional functionality
   - Maintain backward compatibility when possible

### A2A Protocol Crate (`a2a-protocol/`)

**Core Principles:**
- Strict compliance with A2A Protocol v0.3.0 specification
- JSON-RPC 2.0 transport over HTTP
- Production-ready error handling and task management

**Key Components:**
- `core/`: Core types (Message, Task, AgentCard, Error types)
- `transport/`: JSON-RPC 2.0 transport implementation
- `client/`: High-level A2A client with task management
- `server/`: Axum-based JSON-RPC 2.0 server (using `JsonRpcRouter`)
- `auth/`: Authentication strategies (API key, Bearer, OAuth2)

**Important Notes:**
- Use `JsonRpcTransport` for spec-compliant communication
- All RPC methods follow the pattern `category/action` (e.g., `message/send`, `task/get`)
- Tasks can be in states: `queued`, `working`, `completed`, `failed`, `cancelled`
- AgentCard is the discovery mechanism for agent capabilities

**Example Usage Patterns:**
```rust
// Client creation
let transport = Arc::new(JsonRpcTransport::new("https://agent.example.com/rpc")?);
let client = A2aClient::new(transport);

// Sending messages
let message = Message::user_text("Hello, agent!");
let response = client.send_message(message).await?;

// Server setup
let handler = TaskAwareHandler::new(agent_card);
let router = JsonRpcRouter::new(handler);
```

### Multi-Agent Runtime (`multi-agent/`)

**Core Principles:**
- Support multiple agent protocols (OpenAI API, A2A)
- Team-based agent coordination with supervisor and workflow modes
- Configuration-driven agent and team setup via TOML

**Key Components:**
- `agent/`: Agent abstraction and remote agent implementation
- `protocols/`: Protocol adapters (OpenAI, A2A)
- `team/`: Team composition and scheduling logic
- `server/`: HTTP API endpoints (OpenAI-compatible and A2A)

**Important Notes:**
- Agents are configured via TOML (`config.toml`)
- Teams can operate in `supervisor` mode (one coordinator) or `workflow` mode (sequential)
- Use `AgentManager` for dynamic agent registration and discovery
- The server provides both `/v1/chat/completions` (OpenAI) and `/v1/chat` (A2A) endpoints

**Configuration Format:**
```toml
[[agents]]
id = "agent-id"
name = "Agent Name"
endpoint = "https://api.example.com"
protocol = "openai" # or "a2a"
capabilities = ["research", "analysis"]

[[teams]]
id = "team-id"
name = "Team Name"
mode = "supervisor" # or "workflow"
```

### Common Patterns

1. **Creating Messages**
   ```rust
   // Simple text message
   let msg = Message::user_text("Hello!");
   
   // Message with multiple parts
   let msg = Message::new(
       Role::User,
       vec![TextPart::new("Hello!").into()]
   );
   ```

2. **Task Management**
   ```rust
   // Create and track task
   let task = client.get_task(&task_id).await?;
   match task.status.state {
       TaskState::Completed => { /* handle result */ }
       TaskState::Failed => { /* handle error */ }
       _ => { /* still processing */ }
   }
   ```

3. **Error Handling**
   ```rust
   match client.send_message(msg).await {
       Ok(response) => { /* success */ }
       Err(A2aError::TaskNotFound { task_id }) => { /* specific error */ }
       Err(e) => { /* general error */ }
   }
   ```

### Testing Guidelines

- Write unit tests for all public APIs
- Use integration tests for cross-module functionality
- Mock external dependencies using traits
- Test error conditions and edge cases
- Aim for high code coverage on critical paths

### Documentation Standards

- All public items must have rustdoc comments
- Include examples in rustdoc for complex APIs
- Update README.md when adding major features
- Maintain CHANGELOG.md for version tracking
- Document breaking changes clearly

### Dependencies Management

- Use workspace-level dependencies where possible
- Pin versions for critical dependencies
- Use feature flags to make dependencies optional
- Regular dependency updates via `cargo update`
- Audit dependencies with `cargo audit`

### Protocol Compliance

When working with A2A protocol:
- Follow JSON-RPC 2.0 specification strictly
- Use proper error codes (-32001 through -32007 for A2A-specific errors)
- Validate message format according to spec
- Implement all required RPC methods: `message/send`, `task/get`, `task/status`, `task/cancel`, `agent/card`

### Performance Considerations

- Use `Arc` for shared immutable data
- Prefer `tokio::spawn` for concurrent tasks
- Avoid blocking operations in async contexts
- Use connection pooling for HTTP clients
- Implement timeouts for all network operations

### Security Best Practices

- Validate all input from external sources
- Use authentication for all agent communications
- Support multiple auth strategies (API key, Bearer, OAuth2)
- Never log sensitive credentials
- Implement rate limiting in servers

### Future Development

Refer to these planning documents for roadmap and features:
- `A2A_STANDALONE_PLAN.md`: Comprehensive A2A protocol development plan
- `a2a-protocol/IMPLEMENTATION_ROADMAP.md`: A2A implementation status
- `a2a-protocol/UNIMPLEMENTED_FEATURES.md`: Planned features
- `a2a-protocol/TODO_v0.6.0.md`: Version-specific tasks

### Common Commands

```bash
# Development workflow
cargo build                          # Build all crates
cargo test                           # Run all tests
cargo clippy -- -D warnings         # Lint with warnings as errors
cargo fmt                           # Format code

# Specific crate operations
cargo test -p a2a-protocol          # Test A2A protocol only
cargo doc --open                    # Generate and open documentation
cargo run -p multi-agent            # Run multi-agent server

# Examples
cargo run --example basic_agent     # Run basic agent example
```

### When to Use Which Crate

- **Use `a2a-protocol`** when you need:
  - Direct A2A protocol implementation
  - JSON-RPC 2.0 client or server
  - Standalone agent communication
  - Task lifecycle management

- **Use `multi-agent`** when you need:
  - Multiple agent orchestration
  - Team-based coordination
  - Mixed protocol support (OpenAI + A2A)
  - Configuration-driven agent setup

### Additional Resources

- [A2A Protocol Specification](https://a2a-protocol.org/)
- [Tokio Documentation](https://tokio.rs/)
- [Axum Documentation](https://docs.rs/axum/)
- [Rust Async Book](https://rust-lang.github.io/async-book/)
