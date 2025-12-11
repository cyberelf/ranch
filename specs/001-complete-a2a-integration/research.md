# Research & Architecture Decisions

**Feature**: Complete A2A Integration with SDK Enhancements  
**Phase**: 0 - Research & Outline  
**Date**: 2025-12-11

## Research Questions & Decisions

### 1. How should Team implement the Agent trait given it's a multi-agent coordination abstraction?

**Research Task**: Determine the best approach for Team to expose itself as an Agent while maintaining its orchestration logic.

**Decision**: Team implements the multi-agent Agent trait (not a2a-protocol's Agent trait directly)
- The multi-agent Agent trait has two methods: `async fn info() -> AgentInfo` and `async fn process(Message) -> Message`
- Team.info() generates an AgentInfo by aggregating capabilities from all member agents
- Team.process() delegates to the scheduler which orchestrates member agents
- This creates a compositional pattern where teams can be nested as agents within other teams

**Rationale**:
- Keeps separation of concerns: multi-agent Agent trait is for coordination, a2a-protocol Agent trait is for external RPC
- TeamServer bridges the gap by wrapping Team (as multi-agent Agent) with TaskAwareHandler (which implements a2a-protocol Agent)
- Enables recursive composition: teams containing teams, all implementing the same interface
- Follows existing pattern where A2AAgent and OpenAIAgent implement multi-agent Agent trait

**Alternatives Considered**:
- ❌ Team implements a2a-protocol Agent directly: Would tightly couple orchestration logic with JSON-RPC concerns
- ❌ Create separate TeamAgent wrapper: Adds unnecessary indirection when Team can implement the trait directly
- ❌ Don't implement Agent trait at all: Would prevent recursive composition and require special-case handling

### 2. How should TeamServer expose teams via JSON-RPC 2.0 while handling async task management?

**Research Task**: Determine architecture for HTTP server that wraps Team and provides A2A protocol compliance.

**Decision**: Use a2a-protocol's TaskAwareHandler + JsonRpcRouter architecture
- TeamServer owns an Arc<Team>
- Wraps Team with TaskAwareHandler::new(team_as_agent) 
- TaskAwareHandler implements a2a-protocol Agent trait, provides async task management
- JsonRpcRouter routes all five RPC methods to the handler
- Axum Router serves JSON-RPC endpoint at `/rpc`

**Rationale**:
- Leverages existing a2a-protocol infrastructure designed for exactly this use case
- TaskAwareHandler automatically handles task lifecycle (queued -> working -> completed/failed)
- JsonRpcRouter validates JSON-RPC 2.0 format and routes to correct handler methods
- Clean separation: Team focuses on orchestration, TaskAwareHandler focuses on protocol, JsonRpcRouter focuses on transport
- Consistent with how other A2A agents expose themselves

**Alternatives Considered**:
- ❌ Custom JSON-RPC implementation: Reinvents wheel, error-prone, violates DRY
- ❌ Team implements a2a-protocol Agent directly: Mixes concerns, harder to test
- ❌ HTTP wrapper without TaskAwareHandler: Would need to reimplement task management

### 3. What's the best pattern for AgentConfig conversion trait implementations?

**Research Task**: Research Rust conversion trait best practices for config -> protocol-specific config conversion.

**Decision**: Implement `TryFrom<AgentConfig>` for both A2AAgentConfig and OpenAIAgentConfig as the primary API
- Pattern: Extract common fields (endpoint, timeout, max_retries) from AgentConfig
- Validate protocol type matches (return WrongProtocol error if mismatch)
- Validate required fields exist (return MissingField error)
- Validate value ranges (return InvalidValue error)
- Return Result<Self, ConfigConversionError> for all conversions

**Rationale**:
- TryFrom is idiomatic Rust for fallible conversions - config conversion CAN fail (wrong protocol, missing fields, invalid values)
- From trait should NEVER panic - violates Rust's semantic contract
- Enables `.try_into()?` syntax: `let config: A2AAgentConfig = agent_config.try_into()?;`
- Provides clear error messages instead of panics at runtime
- Errors can be propagated with `?` operator or handled explicitly
- Standard library pattern used throughout Rust ecosystem (TryFrom<String> for IpAddr, etc.)

**Alternatives Considered**:
- ❌ From trait with panics: Violates Rust idioms, poor user experience, not testable
- ❌ From trait with asserts: Same problems as panics, not composable
- ❌ Builder pattern: More code, less ergonomic for simple conversions
- ❌ Manual constructor functions: Doesn't leverage Rust's trait system, less discoverable

**Best Practices**:
- Use `#[derive(Clone)]` on config structs for easy copying
- Define comprehensive ConfigConversionError with thiserror
- Document conversion behavior in rustdoc with examples showing error handling
- Add unit tests verifying successful conversions AND error cases
- Never use panic/assert/expect in conversion implementations

### 4. What should AGENT.md files contain and how should they be structured?

**Research Task**: Research technical documentation patterns for multi-trait agent architectures.

**Decision**: Three-tier documentation structure
1. **Root AGENT.md**: High-level architecture overview
   - Trait hierarchy diagram (multi-agent Agent vs a2a-protocol Agent)
   - Component relationships (Team, AgentManager, Schedulers)
   - When to use which trait
   - Links to crate-specific docs

2. **a2a-protocol/AGENT.md**: A2A protocol agent implementation guide
   - How to implement a2a-protocol Agent trait
   - Task lifecycle management
   - JSON-RPC server setup with TaskAwareHandler + JsonRpcRouter
   - Authentication strategies
   - Code examples: minimal agent, stateful agent, streaming agent

3. **multi-agent/AGENT.md**: Multi-agent framework guide
   - How to implement multi-agent Agent trait
   - Scheduler patterns (Supervisor vs Workflow)
   - Team composition and configuration
   - Differences between multi-agent Agent and a2a-protocol Agent
   - Code examples: custom agent, custom scheduler, nested teams

**Rationale**:
- Layered documentation matches layered architecture
- Developers can start at root and drill down to what they need
- Each AGENT.md is focused on one responsibility
- Code examples provide concrete guidance
- Diagrams make abstract concepts concrete

**Alternatives Considered**:
- ❌ Single AGENT.md at root: Too long, mixes concerns, hard to maintain
- ❌ Per-module rustdoc only: Not discoverable, lacks narrative flow
- ❌ Wiki or external docs: Drifts out of sync, requires separate deploy

### 5. How should integration tests be structured for Team orchestration and TeamServer?

**Research Task**: Research Rust integration testing best practices for async multi-agent systems.

**Decision**: Two integration test files in multi-agent/tests/
1. **integration.rs**: Team orchestration tests
   - Mock agents implementing Agent trait
   - Test supervisor delegation
   - Test workflow sequencing
   - Test nested team composition
   - Test error propagation

2. **server.rs**: TeamServer integration tests
   - Start TeamServer on random port
   - Use a2a-protocol client to send requests
   - Verify JSON-RPC 2.0 compliance
   - Test all five RPC methods
   - Test task lifecycle
   - Test concurrent requests
   - Test error responses

**Rationale**:
- Integration tests in `tests/` directory run in separate process, test public API
- Separation allows parallel test execution
- Mock agents keep tests fast and deterministic
- Real A2A client verifies protocol compliance
- Random ports avoid test conflicts

**Best Practices**:
- Use `#[tokio::test]` for async tests
- Create test utilities module for common setup (mock agents, test configs)
- Use `tracing_subscriber::fmt::try_init()` for test logging
- Clean up resources (shutdown servers) in test teardown
- Aim for >80% code coverage as per constitution

### 6. What examples should be created and what should each demonstrate?

**Research Task**: Identify minimal but comprehensive example set.

**Decision**: Five examples covering all major use cases
1. **simple_team.rs**: Basic two-agent team
   - Demonstrates Team creation and registration
   - Shows basic message flow
   - Minimal code for "hello world" experience

2. **supervisor_team.rs**: Supervisor mode
   - One supervisor agent delegates to specialists
   - Shows dynamic agent selection
   - Demonstrates error handling

3. **workflow_team.rs**: Sequential workflow
   - Multi-step processing pipeline
   - Shows conditional execution
   - Demonstrates state passing between agents

4. **remote_agents.rs**: Connecting to A2A agents
   - A2AAgent configuration from endpoints
   - Team coordinating remote agents
   - Shows distributed system pattern

5. **team_server.rs**: Exposing team as service
   - TeamServer setup
   - Accepting A2A requests
   - Shows how to expose any team

**Rationale**:
- Covers both coordination modes (supervisor, workflow)
- Shows both local and remote agents
- Demonstrates server-side and client-side
- Each example is independently runnable
- Progressive complexity: simple -> supervisor -> workflow -> remote -> server

**Each Example Includes**:
- Clear README comment block explaining purpose
- Inline comments explaining key concepts
- Console output showing execution flow
- Error handling patterns
- Clean shutdown

## Technology Choices

### Core Technologies (Already in Use)
- **Rust Edition 2021**: Language foundation, strong type safety
- **Tokio**: Async runtime with full features for HTTP server and concurrent tasks
- **Axum 0.7**: HTTP framework for TeamServer (already used by a2a-protocol)
- **serde/serde_json**: Serialization for configs and JSON-RPC
- **async-trait**: Enables async trait methods
- **thiserror**: Structured error types
- **anyhow**: Application-level error propagation

### Testing & Quality
- **cargo test**: Native Rust test runner
- **cargo clippy**: Linter (zero warnings required)
- **cargo fmt**: Code formatter
- **tracing/tracing-subscriber**: Structured logging for debugging
- **tokio::test**: Async test support

### New Dependencies (None Required)
- All required functionality exists in current dependencies
- No new crates needed for this feature

## Architecture Patterns

### 1. Trait-Based Agent Abstraction
```rust
// Multi-agent coordination trait
#[async_trait]
pub trait Agent: Send + Sync {
    async fn info(&self) -> A2aResult<AgentInfo>;
    async fn process(&self, message: Message) -> A2aResult<Message>;
}

// Team implements this trait
impl Agent for Team {
    async fn info(&self) -> A2aResult<AgentInfo> {
        // Aggregate member capabilities
    }
    async fn process(&self, message: Message) -> A2aResult<Message> {
        // Delegate via scheduler
    }
}
```

### 2. Server Composition Pattern
```rust
pub struct TeamServer {
    team: Arc<Team>,
    port: u16,
}

impl TeamServer {
    pub async fn start(self) -> Result<()> {
        // Wrap Team with protocol handler
        let handler = TaskAwareHandler::new(self.team as Arc<dyn Agent>);
        let router = JsonRpcRouter::new(Arc::new(handler));
        
        // Serve via Axum
        let app = Router::new()
            .route("/rpc", post(handle_rpc))
            .layer(cors());
        
        axum::serve(listener, app).await
    }
}
```

### 3. Config Conversion Pattern
```rust
impl TryFrom<AgentConfig> for A2AAgentConfig {
    type Error = ConfigConversionError;
    
    fn try_from(config: AgentConfig) -> Result<Self, Self::Error> {
        if config.protocol != ProtocolType::A2A {
            return Err(ConfigConversionError::WrongProtocol {
                expected: ProtocolType::A2A,
                found: config.protocol,
            });
        }
        
        if config.endpoint.is_empty() {
            return Err(ConfigConversionError::MissingField("endpoint"));
        }
        
        Ok(Self {
            endpoint: config.endpoint,
            timeout: Duration::from_secs(config.timeout_seconds),
            max_retries: config.max_retries,
            // ... map other fields
        })
    }
}

// Usage - explicit error handling
let a2a_config = A2AAgentConfig::try_from(agent_config)
    .map_err(|e| format!("Invalid config: {}", e))?;

// Or with ? operator
let a2a_config: A2AAgentConfig = agent_config.try_into()?;
```

### 4. Nested Team Pattern
```rust
// Team as agent within another team
let research_team = Team::new(research_config, manager.clone());
let writing_team = Team::new(writing_config, manager.clone());

manager.register(Arc::new(research_team)).await?;
manager.register(Arc::new(writing_team)).await?;

// Parent team coordinates child teams
let parent_team = Team::new(parent_config, manager.clone());
parent_team.process(message).await?; // Delegates to child teams
```

## Implementation Phases

### Phase 3: Team as Agent (Priority P1)
- Implement Agent trait for Team
- Generate AgentInfo from team config + member capabilities
- Route process() calls through scheduler
- Add cycle detection for nested teams
- Unit tests for Team agent behavior

### Phase 4: TeamServer (Priority P1)
- Create server.rs module with TeamServer struct
- Wrap Team with TaskAwareHandler
- Setup Axum router with JSON-RPC endpoint
- Add CORS support
- Integration tests for all five RPC methods

### Phase 5: SDK Improvements (Priority P2)
- Define ConfigConversionError with thiserror
- Implement TryFrom<AgentConfig> for A2AAgentConfig
- Implement TryFrom<AgentConfig> for OpenAIAgentConfig
- Add comprehensive validation (protocol, required fields, value ranges)
- Update examples to use .try_into()? conversions
- Unit tests for successful conversions AND error cases

### Phase 6: Documentation & Examples (Priority P2-P3)
- Write root AGENT.md (architecture overview)
- Write a2a-protocol/AGENT.md (protocol guide)
- Write multi-agent/AGENT.md (framework guide)
- Create five examples (simple -> server)
- Integration tests with >80% coverage
- Update IMPLEMENTATION_PLAN.md marking phases complete

## Open Questions (Resolved)
All research questions have been resolved with documented decisions above. No blockers remain for proceeding to Phase 1 (Design & Contracts).
