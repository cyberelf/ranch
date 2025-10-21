# Migration Guide: v0.3.x → v0.4.0

This guide helps you migrate from `a2a-protocol` v0.3.x to v0.4.0, which focuses on strict A2A v0.3.0 specification compliance.

## Overview of Changes

**v0.4.0** removes non-spec-compliant features and focuses exclusively on **JSON-RPC 2.0** transport as defined in the A2A specification.

### What Was Removed

1. ❌ **A2aRouter** - Non-spec REST-ish endpoints
2. ❌ **Streaming module** - Incomplete/non-SSE-compliant implementation
3. ❌ **health/check endpoint** - Not in A2A spec

### What Remains

✅ **JsonRpcRouter** - Spec-compliant JSON-RPC 2.0 server  
✅ **JsonRpcTransport** - Spec-compliant JSON-RPC 2.0 client  
✅ **All core types** - Message, Task, AgentCard, etc.  
✅ **Task management** - Complete lifecycle support  

---

## Breaking Changes

### 1. Server: A2aRouter Removed

#### Before (v0.3.x)
```rust
use a2a_protocol::server::A2aRouter;

let router = A2aRouter::new(handler);
let app = router.into_router();

// Exposed endpoints:
// POST /messages
// GET /card
// GET /health
```

#### After (v0.4.0)
```rust
use a2a_protocol::server::JsonRpcRouter;

let router = JsonRpcRouter::new(handler);
let app = router.into_router();

// Exposed endpoint:
// POST /rpc (JSON-RPC 2.0)
```

**Why?** The REST-ish endpoints (`/messages`, `/card`, `/health`) are not defined in the A2A specification. The spec requires JSON-RPC 2.0, gRPC, or HTTP+JSON/REST (with different patterns).

---

### 2. Endpoint Changes

| v0.3.x Endpoint | v0.4.0 Method | Description |
|----------------|---------------|-------------|
| `POST /messages` | `message/send` via `/rpc` | Send message |
| `GET /card` | `agent/card` via `/rpc` | Get agent card |
| `GET /health` | ❌ Removed | Add custom endpoint if needed |

#### Client Request Migration

**Before (v0.3.x) - Direct HTTP:**
```bash
curl -X POST http://localhost:3000/messages \
  -H "Content-Type: application/json" \
  -d '{"role": "user", "parts": [{"kind":"text","text":"Hello"}]}'
```

**After (v0.4.0) - JSON-RPC 2.0:**
```bash
curl -X POST http://localhost:3000/rpc \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "message/send",
    "params": {
      "message": {
        "role": "user",
        "parts": [{"kind":"text","text":"Hello"}]
      },
      "immediate": true
    }
  }'
```

---

### 3. Health Checks

The `GET /health` endpoint was not part of the A2A spec and has been removed.

#### Migration Options

**Option A: Add Custom Health Endpoint**
```rust
use axum::{Router, routing::get};
use a2a_protocol::server::JsonRpcRouter;

let a2a_router = JsonRpcRouter::new(handler).into_router();

let app = Router::new()
    .nest("/", a2a_router)
    .route("/health", get(|| async { "OK" }));
```

**Option B: Use Framework Health Checks**
```rust
// Axum example
use tower_http::trace::TraceLayer;

let app = a2a_router
    .layer(TraceLayer::new_for_http())
    .route("/healthz", get(health_check));
```

---

### 4. Streaming APIs Removed

The streaming module (`a2a_protocol::streaming`) has been removed because it was:
- Not W3C Server-Sent Events (SSE) compliant
- Incomplete with TODO comments
- Using deprecated types

**Status:** Will be re-implemented in v0.5.0 following the W3C SSE specification.

#### Before (v0.3.x)
```rust
use a2a_protocol::streaming::StreamingClient;
// This module is now removed
```

#### Migration Path

Use **polling** with the `task/status` or `task/get` methods:

```rust
use a2a_protocol::prelude::*;
use tokio::time::{sleep, Duration};

async fn poll_task(client: &A2aClient, task_id: &str) -> A2aResult<Task> {
    loop {
        let task = client.get_task(task_id).await?;
        
        match task.status.state {
            TaskState::Completed => return Ok(task),
            TaskState::Failed => return Err(A2aError::TaskFailed),
            TaskState::Canceled => return Err(A2aError::TaskCanceled),
            _ => {
                println!("Task state: {:?}", task.status.state);
                sleep(Duration::from_secs(1)).await;
            }
        }
    }
}
```

**When SSE streaming is added in v0.5.0:**
```rust
// Future API (planned)
let stream = client.stream_message(message).await?;
while let Some(event) = stream.next().await {
    match event {
        StreamEvent::TaskStatus(status) => { /* ... */ }
        StreamEvent::TaskArtifact(artifact) => { /* ... */ }
    }
}
```

---

## Code Examples

### Complete Server Migration

#### Before (v0.3.x)
```rust
use a2a_protocol::{
    server::{A2aRouter, BasicA2aHandler},
    AgentId, AgentCard,
};
use axum::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let agent_card = AgentCard::new(
        AgentId::new("my-agent")?,
        "My Agent",
        url::Url::parse("https://example.com")?
    );
    
    let handler = BasicA2aHandler::new(agent_card);
    let router = A2aRouter::new(handler).into_router();
    
    Server::bind(&"127.0.0.1:3000".parse()?)
        .serve(router.into_make_service())
        .await?;
    
    Ok(())
}
```

#### After (v0.4.0)
```rust
use a2a_protocol::{
    server::{JsonRpcRouter, TaskAwareHandler},
    AgentId, AgentCard,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let agent_card = AgentCard::new(
        AgentId::new("my-agent")?,
        "My Agent",
        url::Url::parse("https://example.com")?
    );
    
    let handler = TaskAwareHandler::new(agent_card);
    let router = JsonRpcRouter::new(handler);
    
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    axum::serve(listener, router.into_router()).await?;
    
    Ok(())
}
```

---

### Complete Client Migration

#### Before (v0.3.x)
```rust
use a2a_protocol::{
    client::ClientBuilder,
    prelude::*,
};

let client = ClientBuilder::new()
    .with_http("https://agent.example.com")
    .build()?;

let response = client.send_text("Hello").await?;
```

#### After (v0.4.0)
```rust
use a2a_protocol::{
    client::ClientBuilder,
    prelude::*,
};

let client = ClientBuilder::new()
    .with_json_rpc("https://agent.example.com/rpc")
    .build()?;

let message = Message::user_text("Hello");
let response = client.send_message(message).await?;

match response {
    SendResponse::Message(msg) => println!("{}", msg.text_content().unwrap_or("")),
    SendResponse::Task(task) => println!("Task ID: {}", task.id),
}
```

---

## Benefits of v0.4.0

1. **✅ 100% A2A v0.3.0 Spec Compliance** - Fully interoperable with other A2A agents
2. **✅ Clearer API** - One transport path (JSON-RPC 2.0), less confusion
3. **✅ Better Maintainability** - Removed incomplete/non-compliant code
4. **✅ Future-Proof** - Proper foundation for SSE streaming and other features

---

## Deprecation Timeline

| Version | Status | Notes |
|---------|--------|-------|
| v0.3.x | ⚠️ Deprecated | Contains non-spec features |
| v0.4.0 | ✅ Current | Spec-compliant, stable |
| v0.5.0 | 🚧 Planned | SSE streaming support |
| v0.6.0 | 🚧 Planned | Push notifications |

---

## Need Help?

- **Issues:** Report migration problems at [GitHub Issues](https://github.com/your-org/a2a-protocol/issues)
- **Examples:** See `examples/` directory for complete working examples
- **Spec:** Read the [A2A v0.3.0 specification](https://github.com/a2aproject/A2A)

---

## Quick Reference

### Import Changes
```rust
// Removed
use a2a_protocol::server::A2aRouter;
use a2a_protocol::streaming::StreamingClient;

// Use instead
use a2a_protocol::server::JsonRpcRouter;
// Streaming: Use polling for now
```

### Method Mapping
```rust
// v0.3.x → v0.4.0
router.route("/messages", ...) → JsonRpcRouter with "message/send"
router.route("/card", ...) → JsonRpcRouter with "agent/card"
router.route("/health", ...) → Add custom endpoint
```

### Testing
```bash
# Verify your code compiles with v0.4.0
cargo update
cargo build

# Run tests
cargo test

# Check examples
cargo run --example server
```

---

**Version:** v0.4.0  
**Date:** October 20, 2025  
**Spec:** A2A Protocol v0.3.0
