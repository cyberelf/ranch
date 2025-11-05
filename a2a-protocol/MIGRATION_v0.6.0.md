# Migration Guide: v0.5.0 → v0.6.0

**Version:** v0.5.0 → v0.6.0  
**Date:** November 5, 2025  
**Breaking Changes:** None (fully backward compatible)

## Overview

Version 0.6.0 is **100% backward compatible** with v0.5.0. All existing code will continue to work without any changes. This guide helps you take advantage of new features.

## No Changes Required

✅ **Your existing code works as-is!**

All v0.5.0 APIs remain supported:
- `A2aHandler` trait - No changes
- `TaskAwareHandler` - Enhanced but backward compatible
- `JsonRpcRouter` - No changes
- `A2aClient` - Enhanced but backward compatible
- All core types (Message, Task, AgentCard) - No changes

## Optional Enhancements

You can optionally adopt new features to simplify your code:

### 1. Simplified Server Setup (Recommended)

#### Before (v0.5.0) - Still works!
```rust
use a2a_protocol::server::{JsonRpcRouter, TaskAwareHandler};
use axum::Server;

let handler = TaskAwareHandler::new(agent_card);
let router = JsonRpcRouter::new(handler);
let app = Router::new().nest("/", router.into_router());
let addr = "0.0.0.0:3000".parse()?;
Server::bind(&addr).serve(app.into_make_service()).await?;
```

#### After (v0.6.0) - Simpler!
```rust
use a2a_protocol::server::{ServerBuilder, TaskAwareHandler};

let handler = TaskAwareHandler::new(agent_card);
ServerBuilder::new(handler).with_port(3000).run().await?;
```

**Migration:**
1. Replace manual Axum setup with `ServerBuilder`
2. Use `.with_port()`, `.with_address()`, or `.with_host_port()`
3. Call `.run().await?` to start server

**Benefits:**
- 5 lines → 1 line
- No need to manually configure Axum
- Handles CORS and routing automatically

### 2. Simplified Agent Implementation (For Basic Agents)

#### Before (v0.5.0) - Still works!
```rust
use a2a_protocol::server::A2aHandler;

struct MyAgent;

#[async_trait]
impl A2aHandler for MyAgent {
    async fn rpc_message_send(&self, params: MessageSendParams) 
        -> Result<SendResponse, A2aError> {
        let text = params.message.text_content().unwrap_or("");
        let response = Message::agent_text(format!("Echo: {}", text));
        Ok(SendResponse::Message(response))
    }
    
    // Must implement all other methods...
    async fn rpc_task_get(&self, params: TaskGetParams) 
        -> Result<Task, A2aError> {
        Err(A2aError::UnsupportedOperation {
            operation: "task/get".to_string(),
        })
    }
    
    // ... more methods
}
```

#### After (v0.6.0) - Much simpler!
```rust
use a2a_protocol::server::AgentLogic;

struct MyAgent;

#[async_trait]
impl AgentLogic for MyAgent {
    async fn process_message(&self, msg: Message) -> A2aResult<Message> {
        let text = msg.text_content().unwrap_or("");
        Ok(Message::agent_text(format!("Echo: {}", text)))
    }
}

// Wrap it
let handler = TaskAwareHandler::with_logic(agent_card, MyAgent);
```

**Migration:**
1. If you only process messages, switch to `AgentLogic`
2. Implement just `process_message()` instead of all RPC methods
3. Use `TaskAwareHandler::with_logic()` instead of `new()`

**Benefits:**
- Focus on business logic, not protocol details
- Automatic task management
- Optional lifecycle hooks (`initialize()`, `shutdown()`)

**When NOT to use AgentLogic:**
- You need custom task management
- You need to implement custom RPC methods
- You need full control over protocol behavior
- In these cases, keep using `A2aHandler`

### 3. SSE Streaming Support (New Feature)

#### New: Streaming Client
```rust
use a2a_protocol::client::ClientBuilder;

// Build streaming-enabled client
let client = ClientBuilder::new()
    .with_json_rpc("http://localhost:3000/rpc")
    .build_streaming()?;  // New in v0.6.0

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

**Migration:**
1. Use `build_streaming()` instead of `build()` to enable streaming
2. Call `stream_message()` or `stream_text()` for real-time updates
3. Process events as they arrive

**Benefits:**
- Real-time updates instead of polling
- Automatic reconnection with Last-Event-ID
- Type-safe event handling

### 4. Feature Flags

No changes to feature flags, but streaming is now available:

```toml
[dependencies]
a2a-protocol = { version = "0.6", features = ["streaming"] }
```

**Default features:** `["http-client", "json-rpc", "streaming"]`

Streaming is enabled by default, but can be disabled if not needed.

## Deprecation Notices

**None.** No APIs were deprecated in v0.6.0.

## New APIs Summary

### Server
- `ServerBuilder` - Fluent API for server setup
- `AgentLogic` trait - Simplified agent interface
- `TaskAwareHandler::with_logic()` - Wrap AgentLogic implementations

### Client
- `ClientBuilder::build_streaming()` - Create streaming client
- `A2aStreamingClient` - Streaming-enabled client
- `stream_message()` - Stream message processing
- `stream_text()` - Stream text message (convenience)
- `resubscribe_task()` - Resume streaming for existing task

### Streaming (Feature-gated)
- `SseEvent` - SSE event types
- `SseWriter` - Event publisher
- `EventBuffer` - Event replay buffer

### Handler Extensions
- `A2aHandler::rpc_message_stream()` - Stream message processing
- `A2aHandler::rpc_task_resubscribe()` - Resume stream

## Testing Changes

No changes required to your tests. All v0.5.0 tests should pass.

### Optional: Test New Features
```rust
#[tokio::test]
async fn test_server_builder() {
    let handler = TaskAwareHandler::new(agent_card);
    let server = ServerBuilder::new(handler)
        .with_port(0)  // Random port
        .build()?;
    // Test server...
}

#[tokio::test]
async fn test_streaming() {
    let client = ClientBuilder::new()
        .with_json_rpc(url)
        .build_streaming()?;
    let mut stream = client.stream_text("test").await?;
    // Test stream...
}
```

## Examples

See the new `examples/` directory for working code:

```bash
# Basic examples
cargo run --example basic_echo_server --features streaming
cargo run --example echo_client --features streaming

# Streaming examples
cargo run --example streaming_server --features streaming
cargo run --example streaming_client --features streaming

# Advanced examples
cargo run --example task_server --features streaming
cargo run --example multi_agent --features streaming
```

All examples are documented in [examples/README.md](examples/README.md).

## Documentation Updates

New documentation files:
- **GETTING_STARTED.md** - Step-by-step tutorial
- **examples/README.md** - Examples guide
- **CHANGELOG.md** - Version history
- **RELEASE_NOTES_v0.6.0.md** - What's new
- **DOCS_INDEX.md** - Documentation index

## Common Migration Patterns

### Pattern 1: Basic Echo Server
```rust
// v0.5.0
let handler = TaskAwareHandler::new(agent_card);
// ... manual axum setup

// v0.6.0 (simpler)
struct EchoAgent;
impl AgentLogic for EchoAgent {
    async fn process_message(&self, msg: Message) -> A2aResult<Message> {
        Ok(Message::agent_text(format!("Echo: {}", msg.text())))
    }
}
let handler = TaskAwareHandler::with_logic(agent_card, EchoAgent);
ServerBuilder::new(handler).with_port(3000).run().await?;
```

### Pattern 2: Client with Streaming
```rust
// v0.5.0 (polling)
let client = ClientBuilder::new().with_json_rpc(url).build()?;
let response = client.send_message(msg).await?;
if let SendResponse::Task(task) = response {
    loop {
        let status = client.get_task_status(&task.id).await?;
        if status.state.is_terminal() { break; }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

// v0.6.0 (streaming - no polling!)
let client = ClientBuilder::new().with_json_rpc(url).build_streaming()?;
let mut stream = client.stream_message(msg).await?;
while let Some(event) = stream.next().await {
    if let SseEvent::TaskUpdate(task) = event {
        if task.status.state.is_terminal() { break; }
    }
}
```

## Troubleshooting

### "Method `build_streaming` not found"
**Cause:** Streaming feature not enabled  
**Solution:** Add `features = ["streaming"]` to dependency or use default features

### "Trait `AgentLogic` not in scope"
**Cause:** Need to import from `server` module  
**Solution:** `use a2a_protocol::server::AgentLogic;`

### "Type `A2aStreamingClient` not found"
**Cause:** Streaming feature not enabled  
**Solution:** Enable `streaming` feature flag

## Questions?

- See [GETTING_STARTED.md](GETTING_STARTED.md) for tutorials
- See [examples/README.md](examples/README.md) for working code
- Check [DOCS_INDEX.md](DOCS_INDEX.md) for all documentation
- Run `cargo doc --open` for API reference

## Summary

**✅ Zero breaking changes** - All v0.5.0 code works as-is  
**✅ Optional enhancements** - Adopt new features at your own pace  
**✅ Better docs** - Comprehensive guides and examples  
**✅ More tests** - 161 tests (+51 from v0.5.0)  

**Recommended steps:**
1. Update dependency: `a2a-protocol = "0.6"`
2. Run tests to verify compatibility
3. Optionally adopt `ServerBuilder` for cleaner code
4. Optionally adopt `AgentLogic` for simpler agents
5. Optionally enable streaming for real-time updates

**Happy coding! 🚀**
