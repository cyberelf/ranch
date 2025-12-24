# GitHub Copilot Instructions for RANCH

## Governance

**All development MUST comply with the [RANCH Constitution](.specify/memory/constitution.md) v1.4.0.**

The Constitution defines mandatory principles for:
- Rust-First & Type Safety (including trait infallibility, separation of concerns)
- Protocol Compliance (A2A Protocol v0.3.0)
- Async-Native development patterns
- Testing & Quality (unit test co-location, integration tests)
- Documentation & Standards
- SDK Design & Developer Experience
- Feature Finalization & Status Tracking
- Security & Performance requirements

This document provides RANCH-specific technical guidance and conventions that complement the Constitution.

---

## Project Overview

**RANCH** (Rust Agent Networking & Coordination Hub) is a multi-agent system framework built in Rust for managing, coordinating, and facilitating communication between autonomous agents. The project implements the A2A (Agent-to-Agent) protocol and provides a robust runtime for multi-agent coordination.

## Repository Structure

This is a Rust workspace with two main crates:

- **`a2a-protocol/`**: Standalone implementation of the A2A (Agent-to-Agent) communication protocol v0.3.0, providing JSON-RPC 2.0 transport, task management, and agent discovery. **Use this for implementing agents.**
- **`multi-agent/`**: **Client-side collaboration framework** for coordinating remote A2A agents. This is NOT for implementing agents - it provides team composition, routing, and coordination of remote agents accessed via A2A protocol.

### Critical Architectural Principle

**The multi-agent crate is a CLIENT-SIDE coordination layer, NOT an agent implementation framework.**

✅ **Correct Usage:**
- Implement agents as A2A protocol servers in `a2a-protocol` crate using `ProtocolAgent` trait
- Use `multi-agent::A2AAgent` to connect to remote agent servers
- Form teams of remote agents for collaboration
- Route messages between remote agents dynamically

❌ **Incorrect Usage:**
- Do NOT implement agents locally using `multi-agent::Agent` trait
- Do NOT create mock agents in `multi-agent` examples
- Do NOT mix agent implementation with coordination logic

**All examples MUST follow this pattern:**
1. Agent servers implemented using `a2a-protocol::server::ProtocolAgent`
2. Client connects to servers using `multi-agent::A2AAgent`
3. Teams coordinate remote agents without local implementations

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

**Note**: Core principles (error handling, async patterns, type safety, API design) are defined in the Constitution. Below are RANCH-specific implementation details.

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
- **Client-side coordination layer** - NOT an agent implementation framework
- Connect to remote A2A agents via `A2AAgent`
- Route messages dynamically between remote agents
- Support Client Agent Extension for intelligent routing

**Key Components:**
- `agent/`: Agent trait and `A2AAgent` client for remote agents
- `manager/`: `AgentManager` for agent registry and discovery
- `team/`: Team composition and dynamic router
- `server/`: HTTP API endpoints for team-level operations

**Critical Usage Rules:**
- ✅ Use `A2AAgent` to connect to remote A2A protocol servers
- ✅ Form teams of remote agents via `Team`
- ✅ Use `AgentManager` for agent discovery
- ❌ Do NOT implement `Agent` trait for new agents locally
- ❌ Do NOT create mock agents in examples
- ❌ Agent implementation belongs in `a2a-protocol` crate

**Example Pattern:**
```rust
// Correct: Connect to remote agent
let transport = Arc::new(JsonRpcTransport::new("http://localhost:3000/rpc")?);
let client = A2aClient::new(transport);
let remote_agent = Arc::new(A2AAgent::new(client));

let manager = Arc::new(AgentManager::new());
manager.register(remote_agent).await?;

let team = Team::new(team_config, manager);
```

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

**Per Constitution Section IV**: Unit tests MUST be co-located with source code in `#[cfg(test)] mod tests` blocks. Integration tests MUST be in `tests/` directory. Detailed guidelines at `.github/TESTING_GUIDELINES.md`.


### Documentation Standards

**Per Constitution Section V**: All public items must have rustdoc comments. Complex APIs MUST include examples. `README.md` and `CHANGELOG.md` MUST be updated for major features. Semantic versioning MUST be followed.

### Dependencies Management

- Use workspace-level dependencies where possible
- Pin versions for critical dependencies
- Use feature flags to make dependencies optional
- Regular dependency updates via `cargo update`
- Audit dependencies with `cargo audit`

### Protocol Compliance

**Per Constitution Section II**: Strict adherence to A2A Protocol v0.3.0 specification. JSON-RPC 2.0 transport. Error codes -32001 through -32007 for A2A-specific errors.

When working with A2A protocol:
- Follow JSON-RPC 2.0 specification strictly
- Use proper error codes (-32001 through -32007 for A2A-specific errors)
- Validate message format according to spec
- Implement all required RPC methods: `message/send`, `task/get`, `task/status`, `task/cancel`, `agent/card`

### Performance Considerations

**Per Constitution Section "Security & Performance"**: Connection pooling and timeouts MUST be used for network operations.

- Use `Arc` for shared immutable data
- Prefer `tokio::spawn` for concurrent tasks
- Avoid blocking operations in async contexts
- Use connection pooling for HTTP clients
- Implement timeouts for all network operations

### Security Best Practices

**Per Constitution Section "Security & Performance"**: Input MUST be validated. Authentication (API Key, Bearer, OAuth2) is REQUIRED. Credentials MUST NEVER be logged.

- Validate all input from external sources
- Use authentication for all agent communications
- Support multiple auth strategies (API key, Bearer, OAuth2)
- Never log sensitive credentials
- Implement rate limiting in servers


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
cargo run --example agent_servers   # Start A2A agent servers
cargo run --example team_client     # Run team coordination client
```

### When to Use Which Crate

- **Use `a2a-protocol`** when you need:
  - Agent implementation (use `ProtocolAgent` trait)
  - Direct A2A protocol server or client
  - JSON-RPC 2.0 communication
  - Task lifecycle management
  - Standalone agent capabilities

- **Use `multi-agent`** when you need:
  - Client-side coordination of REMOTE agents
  - Team formation and routing
  - Dynamic message routing between agents
  - Client Agent Extension support
  - Multi-agent orchestration

**Remember: Always implement agents in `a2a-protocol`, then coordinate them via `multi-agent`.**

### Additional Resources
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
