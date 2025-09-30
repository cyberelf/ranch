# Standalone A2A Protocol Crate Development Plan

## Overview

This document outlines the comprehensive plan for creating a standalone A2A (Agent-to-Agent) protocol crate in Rust. The goal is to develop a production-ready, reusable implementation that follows Rust conventions and can be published to crates.io.

## Current State Analysis

The existing implementation in `src/protocols/a2a.rs` provides a basic HTTP-based client with:
- Simple request/response model
- Basic authentication support
- Health check endpoint
- Integration with the existing framework's types

However, it lacks:
- Support for the full A2A protocol specification
- Multiple transport options (JSON-RPC, gRPC)
- Streaming capabilities
- Proper AgentCard handling
- Error handling alignment with A2A standards
- Standalone architecture

## Crate Structure

```
a2a-protocol/
├── Cargo.toml
├── LICENSE
├── README.md
├── CHANGELOG.md
├── src/
│   ├── lib.rs
│   ├── error.rs          # A2A-specific error types
│   ├── types.rs          # Core A2A types and structures
│   ├── agent_card.rs     # AgentCard implementation
│   ├── message.rs        # Message definitions and builders
│   ├── transport/        # Transport layer implementations
│   │   ├── mod.rs
│   │   ├── http.rs       # HTTP/JSON transport
│   │   ├── json_rpc.rs   # JSON-RPC 2.0 transport
│   │   ├── grpc.rs       # gRPC transport (optional feature)
│   │   └── traits.rs     # Transport traits
│   ├── client/           # Client implementations
│   │   ├── mod.rs
│   │   ├── traits.rs     # Client traits
│   │   ├── sync.rs       # Synchronous client
│   │   └── async.rs      # Asynchronous client
│   ├── server/           # Server framework (optional feature)
│   │   ├── mod.rs
│   │   ├── traits.rs
│   │   └── router.rs
│   ├── auth/             # Authentication mechanisms
│   │   ├── mod.rs
│   │   ├── traits.rs
│   │   ├── bearer.rs
│   │   └── api_key.rs
│   ├── streaming/        # Streaming support
│   │   ├── mod.rs
│   │   ├── sse.rs        # Server-Sent Events
│   │   └── traits.rs
│   └── utils.rs          # Utility functions
├── examples/
│   ├── basic_client.rs
│   ├── agent_card_demo.rs
│   ├── streaming_client.rs
│   └── server_demo.rs
├── tests/
│   ├── integration.rs
│   ├── conformance.rs
│   └── fixtures/
└── benches/
    └── performance.rs
```

## Core Interfaces and Traits

### 1. Transport Traits

```rust
// transport/traits.rs
#[async_trait]
pub trait A2ATransport: Send + Sync {
    type Error: Into<A2AError>;

    async fn send_request(&self, request: A2ARequest) -> Result<A2AResponse, Self::Error>;
    async fn health_check(&self, endpoint: &Url) -> Result<bool, Self::Error>;
    fn supports_streaming(&self) -> bool;
    fn max_message_size(&self) -> Option<usize>;
}

#[async_trait]
pub trait StreamingTransport: A2ATransport {
    type Stream: Stream<Item = Result<A2AStreamEvent, Self::Error>>;

    async fn send_streaming_request(
        &self,
        request: A2ARequest,
    ) -> Result<Self::Stream, Self::Error>;
}
```

### 2. Client Traits

```rust
// client/traits.rs
#[async_trait]
pub trait A2AClient: Send + Sync {
    async fn send_message(
        &self,
        request: SendMessageRequest,
    ) -> Result<SendMessageResponse, A2AError>;

    async fn create_task(
        &self,
        request: CreateTaskRequest,
    ) -> Result<Task, A2AError>;

    async fn get_task(
        &self,
        task_id: &str,
    ) -> Result<Task, A2AError>;

    async fn cancel_task(
        &self,
        task_id: &str,
    ) -> Result<bool, A2AError>;

    async fn get_agent_card(&self, endpoint: &Url) -> Result<AgentCard, A2AError>;

    fn supports_streaming(&self) -> bool;

    async fn stream_task_events(
        &self,
        task_id: &str,
    ) -> Result<BoxStream<A2AEvent>, A2AError>;
}
```

### 3. Server Traits (Optional Feature)

```rust
// server/traits.rs
#[async_trait]
pub trait A2AServer: Send + Sync {
    type Context;

    async fn handle_message(
        &self,
        message: A2AMessage,
        context: Self::Context,
    ) -> Result<A2AResponse, A2AError>;

    async fn handle_streaming_message(
        &self,
        message: A2AMessage,
        context: Self::Context,
        sender: MessageSender,
    ) -> Result<(), A2AError>;

    fn agent_card(&self) -> AgentCard;
}
```

## Module Breakdown

### 1. Types Module (`types.rs`)

Core A2A types:
- `A2ARequest`: Request structure
- `A2AResponse`: Response structure
- `A2AMessage`: Message format
- `Task`: Task representation
- `TaskStatus`: Task lifecycle states
- `A2AError`: Comprehensive error types
- `A2AEvent`: Event types for streaming

### 2. AgentCard Module (`agent_card.rs`)

AgentCard implementation with:
- Card structure and validation
- Capability discovery
- Endpoint configuration
- Authentication schemes
- Serialization/deserialization

### 3. Message Module (`message.rs`)

Message handling:
- Message builders
- Validation
- Message types (chat, task, etc.)
- Message history

### 4. Transport Implementations

- **HTTP Transport**: RESTful HTTP/JSON
- **JSON-RPC Transport**: JSON-RPC 2.0 compliance
- **gRPC Transport** (optional): High-performance gRPC

### 5. Authentication Module

Multiple auth strategies:
- Bearer token
- API key
- Custom headers
- OAuth2 (future extension)

### 6. Streaming Module

Streaming support:
- Server-Sent Events (SSE)
- WebSockets (future)
- Stream processing utilities

## Implementation Phases

### Phase 1: Core Foundation (Week 1-2)
- [ ] Set up crate structure
- [ ] Implement core types and error handling
- [ ] Create AgentCard implementation
- [ ] Define transport traits
- [ ] Basic HTTP transport implementation

### Phase 2: Client Implementation (Week 3-4)
- [ ] Sync client implementation
- [ ] Async client implementation
- [ ] Message builders and validation
- [ ] Basic authentication support
- [ ] Unit tests for client functionality

### Phase 3: Advanced Features (Week 5-6)
- [ ] JSON-RPC transport implementation
- [ ] Streaming support with SSE
- [ ] Task management APIs
- [ ] Enhanced authentication
- [ ] Configuration management

### Phase 4: Server Framework (Week 7-8) [Optional]
- [ ] Server traits and router
- [ ] Request handling middleware
- [ ] Streaming server support
- [ ] Integration examples

### Phase 5: Testing and Documentation (Week 9-10)
- [ ] Comprehensive test suite
- [ ] Integration tests
- [ ] Performance benchmarks
- [ ] Documentation and examples
- [ ] Conformance tests

### Phase 6: Polish and Release (Week 11-12)
- [ ] Code review and optimization
- [ ] Security audit
- [ ] API stability check
- [ ] Publish to crates.io
- [ ] Integration guide for existing framework

## Testing Strategy

### 1. Unit Tests
- 90%+ code coverage
- Property-based testing for edge cases
- Mock implementations for transport testing

### 2. Integration Tests
- Real HTTP server tests
- WebSocket/SSE streaming tests
- Authentication flow tests
- Error scenario testing

### 3. Conformance Tests
- A2A protocol compliance
- JSON-RPC 2.0 specification tests
- Transport layer validation

### 4. Performance Tests
- Request throughput
- Memory usage
- Streaming performance
- Concurrent request handling

## Documentation Plan

### 1. API Documentation
- Complete rustdoc coverage
- Examples for every public API
- Trait documentation with usage patterns

### 2. User Guide
- Getting started guide
- Authentication setup
- Transport selection guide
- Streaming usage examples
- Error handling patterns

### 3. Integration Guide
- Framework integration patterns
- Migration guide from current implementation
- Best practices and patterns

### 4. Examples
- Basic client usage
- Server implementation
- Streaming examples
- Authentication examples
- Custom transport implementation

## Dependencies

```toml
[dependencies]
# Core
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
async-trait = "0.1"
thiserror = "1.0"
uuid = { version = "1.0", features = ["v4", "serde"] }
url = "2.4"

# HTTP/JSON
reqwest = { version = "0.11", features = ["json", "stream"], optional = true }
http = "0.2"

# Async runtime
tokio = { version = "1.0", features = ["full"], optional = true }
tokio-stream = { version = "0.1", optional = true }
futures = "0.3", optional = true

# gRPC (optional)
tonic = { version = "0.10", optional = true }
prost = { version = "0.12", optional = true }

# Streaming
eventsource-stream = { version = "0.2", optional = true }

# Tracing
tracing = { version = "0.1", optional = true }

# Dev dependencies
mockall = "0.11"
proptest = "1.0"
criterion = "0.5"

[features]
default = ["http", "async"]
http = ["reqwest", "tokio"]
json-rpc = ["http"]
grpc = ["tonic", "prost", "tokio"]
streaming = ["eventsource-stream", "tokio-stream", "futures"]
server = ["axum", "tower", "tower-http"]
async = ["tokio", "futures"]
sync = []
full = ["http", "json-rpc", "grpc", "streaming", "server", "async"]
```

## Integration Strategy for Existing Framework

### 1. Adapter Pattern
Create an adapter that implements the existing `Protocol` trait using the new A2A client:

```rust
pub struct A2AProtocolAdapter {
    client: Box<dyn A2AClient>,
    config: A2AConfig,
}

#[async_trait]
impl Protocol for A2AProtocolAdapter {
    async fn send_message(
        &self,
        config: &AgentConfig,
        messages: Vec<AgentMessage>,
    ) -> Result<AgentResponse, ProtocolError> {
        // Convert between old and new types
        // Use the standalone A2A client
    }
}
```

### 2. Migration Steps
1. Add the new crate as a dependency
2. Create the adapter implementation
3. Gradually migrate existing code
4. Remove old implementation
5. Update examples and documentation

### 3. Benefits of Integration
- Cleaner separation of concerns
- Reusable A2A implementation
- Better testability
- Access to advanced features (streaming, multiple transports)
- Easier maintenance and updates

## Quality Assurance

### 1. Code Quality
- Rustfmt and Clippy enforcement
- Continuous integration with multiple Rust versions
- Security scanning with cargo-audit
- Performance regression testing

### 2. API Design
- Semantic versioning
- Deprecation policy
- Breaking change management
- Feature flags for optional components

### 3. Community Ready
- Clear contribution guidelines
- Code of conduct
- Issue templates
- PR review process

## Success Criteria

1. **Protocol Compliance**: Full adherence to A2A specification
2. **Performance**: Match or exceed current implementation
3. **Usability**: Intuitive API with clear documentation
4. **Reliability**: Comprehensive error handling and recovery
5. **Extensibility**: Easy to add new transports and features
6. **Integration**: Seamless integration with existing framework
7. **Publication**: Ready for crates.io publication

## Timeline Estimate

- **Phase 1-2**: 4 weeks (Core + Client)
- **Phase 3-4**: 4 weeks (Advanced + Server)
- **Phase 5-6**: 4 weeks (Testing + Release)
- **Total**: 12 weeks for full implementation
- **Minimum Viable Product**: 6 weeks (Phases 1-2)

## Risk Mitigation

1. **Specification Changes**: Design for extensibility
2. **Performance Issues**: Early performance testing
3. **Compatibility**: Maintain adapter for existing code
4. **Adoption**: Clear migration path and documentation

This plan provides a comprehensive roadmap for creating a production-ready A2A protocol crate that serves both the immediate needs of the existing framework and provides a solid foundation for broader adoption in the Rust ecosystem.