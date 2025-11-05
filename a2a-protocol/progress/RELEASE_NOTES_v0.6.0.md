# Release Notes: v0.6.0

**Released:** November 5, 2025  
**Theme:** Complete SSE Streaming + Developer Experience

## 🎉 Overview

Version 0.6.0 is a major milestone release that completes the SSE (Server-Sent Events) streaming implementation and introduces significant developer experience improvements inspired by the a2a-go implementation.

This release increases spec compliance from ~75% to ~80% and adds 51 new tests (110 → 161).

## ✨ Highlights

### Complete SSE Streaming
- **Server-side streaming** with W3C-compliant SSE infrastructure
- **Client-side streaming API** with automatic reconnection and Last-Event-ID support
- Real-time task updates via `message/stream` and `task/resubscribe` endpoints
- Type-safe streaming client with `A2aStreamingClient<T>`

### Developer Experience Improvements
- **ServerBuilder** - One-line server setup with fluent API
- **AgentLogic trait** - Simplified agent implementation (single `process_message()` method)
- **8 comprehensive examples** covering basic to advanced use cases
- **Complete documentation** - README, GETTING_STARTED, and examples guide

### Production Ready
- **161 tests passing** (+51 from v0.5.0)
- Full integration test coverage
- Documentation tests embedded in code
- Feature-gated streaming support

## 📦 What's New

### SSE Streaming Infrastructure

#### Server Components
```rust
// New SSE types
- SseEvent       // W3C event formatting and parsing
- SseWriter      // Broadcast-based event publisher
- EventBuffer    // Replay buffer with Last-Event-ID support

// New handler methods
trait A2aHandler {
    async fn rpc_message_stream(...) -> Result<SseWriter, A2aError>;
    async fn rpc_task_resubscribe(...) -> Result<SseWriter, A2aError>;
}
```

#### Client Components
```rust
// New streaming client
let client = ClientBuilder::new()
    .with_json_rpc("http://localhost:3000/rpc")
    .build_streaming()?;

// Stream messages in real-time
let mut stream = client.stream_message(message).await?;
while let Some(event) = stream.next().await {
    match event {
        SseEvent::Message(msg) => println!("Got: {}", msg.text()),
        SseEvent::TaskUpdate(task) => println!("Status: {:?}", task.status),
        _ => {}
    }
}
```

### Simplified APIs

#### ServerBuilder
```rust
// Before (v0.5.0) - Required manual Axum setup
let router = JsonRpcRouter::new(handler);
let app = Router::new().nest("/", router.into_router());
Server::bind(&addr).serve(app.into_make_service()).await?;

// After (v0.6.0) - One line!
ServerBuilder::new(handler).with_port(3000).run().await?;
```

#### AgentLogic Trait
```rust
// Simple trait for basic agents
#[async_trait]
impl AgentLogic for MyAgent {
    async fn process_message(&self, msg: Message) -> A2aResult<Message> {
        // Your business logic here
        Ok(Message::agent_text("Response"))
    }
}

// Wrap and run
let handler = TaskAwareHandler::with_logic(agent_card, MyAgent);
ServerBuilder::new(handler).with_port(3000).run().await?;
```

### Examples (8 Total)

1. **basic_echo_server.rs** - Minimal server using AgentLogic trait
2. **echo_client.rs** - Simple client with message handling
3. **simple_server.rs** - ServerBuilder one-line setup
4. **streaming_server.rs** - SSE streaming server demonstration
5. **streaming_client.rs** - SSE client with reconnection
6. **streaming_type_safety.rs** - Type-safe streaming patterns
7. **task_server.rs** - Long-running task management
8. **multi_agent.rs** - Agent-to-agent communication

Run any example with:
```bash
cargo run --example basic_echo_server --features streaming
```

### Documentation

#### New Documentation Files
- **GETTING_STARTED.md** - Step-by-step tutorial for new users
- **examples/README.md** - Comprehensive guide to all examples
- **CHANGELOG.md** - Complete version history
- **DOCS_INDEX.md** - Documentation navigation

#### Updated Documentation
- **README.md** - 5-minute quick start guide
- Complete API documentation with examples
- Trait selection guide (AgentLogic vs A2aHandler)

## 🔧 Technical Details

### Streaming Architecture
- **Transport:** `axum::response::sse` for W3C compliance
- **Event Format:** JSON-RPC 2.0 messages in SSE data field
- **Buffering:** Last 100 events per task with Last-Event-ID
- **Cleanup:** Automatic on task completion or timeout
- **Type Safety:** Generic `A2aStreamingClient<T>` with Deref pattern

### Feature Flags
```toml
[features]
default = ["http-client", "json-rpc", "streaming"]
streaming = ["json-rpc", "futures-util", "async-stream", "bytes"]
```

## 📊 Test Coverage

**Total Tests: 161** (+51 from v0.5.0)

Breakdown:
- 110 library tests (+26)
- 8 streaming integration tests (new)
- 17 compliance tests
- 8 RPC tests
- 18 documentation tests (+17)

All tests passing with 100% success rate.

## 🚀 Migration from v0.5.0

### No Breaking Changes!

Version 0.6.0 is **fully backward compatible** with v0.5.0. All existing code continues to work.

### Optional Upgrades

#### Use ServerBuilder (recommended)
```rust
// Old way still works
let router = JsonRpcRouter::new(handler);
// ... manual axum setup

// New way (easier!)
ServerBuilder::new(handler).with_port(3000).run().await?;
```

#### Use AgentLogic for simple agents
```rust
// If you only need to process messages, use AgentLogic
impl AgentLogic for MyAgent {
    async fn process_message(&self, msg: Message) -> A2aResult<Message> {
        // Simple business logic
    }
}

// If you need full control, keep using A2aHandler
impl A2aHandler for AdvancedAgent {
    // Full control over all RPC methods
}
```

#### Enable Streaming
```rust
// Add streaming feature to enable SSE support
let client = ClientBuilder::new()
    .with_json_rpc(url)
    .build_streaming()?;  // New method
```

## 📝 API Changes

### New APIs
- `ServerBuilder` - Fluent API for server configuration
- `AgentLogic` trait - Simplified agent interface
- `A2aStreamingClient<T>` - Streaming client with type safety
- `TaskAwareHandler::with_logic()` - Wrap AgentLogic implementations
- `ClientBuilder::build_streaming()` - Create streaming client

### Enhanced APIs
- `A2aHandler` - Added `rpc_message_stream()` and `rpc_task_resubscribe()`
- `TaskAwareHandler` - Full streaming support with automatic cleanup

### Deprecated APIs
None. All v0.5.0 APIs remain supported.

## 🐛 Bug Fixes

- Improved error handling in streaming contexts
- Fixed event buffer overflow in high-throughput scenarios
- Enhanced cleanup of completed/cancelled streams

## 📈 Performance

- Efficient event buffering with circular buffer
- Minimal overhead for non-streaming operations
- Connection pooling for HTTP clients
- Proper resource cleanup prevents memory leaks

## 🔒 Security

- No new security vulnerabilities
- Proper validation of SSE event data
- Timeout protection for streaming connections

## 🎯 What's Next: v0.7.0

**Theme:** Push Notifications (Webhooks)  
**Target:** Q2 2026 (8 weeks)

Key features planned:
- Webhook configuration (4 RPC methods: set/get/list/delete)
- Webhook delivery system with retry logic
- Comprehensive SSRF protection
- Rate limiting and security audit
- Target: 310+ total tests (+150)

See [TODO_v0.7.0.md](TODO_v0.7.0.md) for detailed plan.

## 📚 Resources

- **Examples:** [examples/README.md](examples/README.md)
- **Getting Started:** [GETTING_STARTED.md](GETTING_STARTED.md)
- **API Docs:** Run `cargo doc --open`
- **Changelog:** [CHANGELOG.md](CHANGELOG.md)
- **Roadmap:** [IMPLEMENTATION_ROADMAP.md](IMPLEMENTATION_ROADMAP.md)

## 👥 Contributors

Thanks to all contributors who helped make v0.6.0 possible!

## 📄 License

MIT OR Apache-2.0

---

**Full Changelog:** v0.5.0...v0.6.0  
**Release Date:** November 5, 2025  
**Spec Compliance:** ~80% (A2A Protocol v0.3.0)
