# A2A Protocol Implementation

A comprehensive, production-ready Rust implementation of the A2A (Agent-to-Agent) protocol v0.3.0 specification.

## ⚡ Quick Start (5 Minutes!)

### Create a Simple Server

```rust
use a2a_protocol::{prelude::*, server::{AgentLogic, ServerBuilder, TaskAwareHandler}};
use async_trait::async_trait;

// 1. Define your agent logic
struct EchoAgent;

#[async_trait]
impl AgentLogic for EchoAgent {
    async fn process_message(&self, message: Message) -> A2aResult<Message> {
        let text = message.text_content().unwrap_or("");
        Ok(Message::agent_text(format!("Echo: {}", text)))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 2. Create agent card
    let agent_card = AgentCard::new(
        AgentId::new("echo-agent")?,
        "Echo Agent",
        url::Url::parse("https://example.com")?
    );

    // 3. Wrap in handler and start server
    let handler = TaskAwareHandler::with_logic(agent_card, EchoAgent);
    ServerBuilder::new(handler).with_port(3000).run().await?;
    Ok(())
}
```

**That's it!** You now have a working A2A server. Test it:

```bash
curl -X POST http://localhost:3000/rpc \
  -H 'Content-Type: application/json' \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "message/send",
    "params": {
      "message": {
        "role": "user",
        "parts": [{"kind":"text","text":"hello"}]
      },
      "immediate": true
    }
  }'
```

### Create a Simple Client

```rust
use a2a_protocol::{prelude::*, client::ClientBuilder};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create client
    let client = ClientBuilder::new()
        .with_json_rpc("http://localhost:3000/rpc")
        .build()?;

    // Send message
    let message = Message::user_text("hello");
    let response = client.send_message(message).await?;

    // Handle response
    match response {
        SendResponse::Message(msg) => println!("Got: {}", msg.text_content().unwrap()),
        SendResponse::Task(task) => println!("Task created: {}", task.id),
    }
    Ok(())
}
```

## 📚 Learn More

- **[Examples](examples/)** - 8 runnable examples covering basic to advanced usage
- **[Getting Started Guide](GETTING_STARTED.md)** - Step-by-step tutorial
- **[API Documentation](https://docs.rs/a2a-protocol)** - Full API reference

### When to Use What

- **`AgentLogic` trait** - For simple agents (most common use case)
- **`A2aHandler` trait** - For advanced agents needing full control
- **`A2aClient`** - For non-streaming client operations
- **`A2aStreamingClient`** - For real-time streaming updates

## Features

- **Spec Compliance**: Strict adherence to the A2A v0.3.0 specification
- **JSON-RPC 2.0**: Full JSON-RPC 2.0 transport implementation
- **SSE Streaming**: Server-Sent Events for real-time updates ✅ NEW in v0.6.0
- **Async Native**: Built on tokio for high-performance async communication
- **Type Safe**: Strong typing with serde for serialization
- **Task Management**: Complete task lifecycle support
- **Production Ready**: Comprehensive error handling and testing (161 tests)
- **Easy to Use**: Simple `AgentLogic` trait for quick development ✅ NEW in v0.6.0
- **One-Line Server**: `ServerBuilder` for minimal setup ✅ NEW in v0.6.0

## Specification Compliance

This crate implements **A2A Protocol v0.3.0** with strict spec compliance:

✅ **Supported:**
- JSON-RPC 2.0 transport over HTTP
- All required RPC methods (`message/send`, `task/get`, `task/cancel`, `task/status`, `agent/card`)
- SSE streaming (`message/stream`, `task/resubscribe`) ✅ NEW in v0.6.0
- Complete Task lifecycle management
- A2A Message format with Parts (TextPart, FilePart, DataPart)
- AgentCard discovery

🚧 **Planned:**
- Push notifications (`task/pushNotificationConfig/*`)
- gRPC transport (optional)
- HTTP+JSON/REST transport (if spec clarifies patterns)

## 🎯 Examples

We provide 8 comprehensive examples in the [examples/](examples/) directory:

1. **[basic_echo_server](examples/basic_echo_server.rs)** - Minimal server using `AgentLogic` (⭐ Start here!)
2. **[echo_client](examples/echo_client.rs)** - Simple client for sending messages
3. **[simple_server](examples/simple_server.rs)** - One-line server with `ServerBuilder`
4. **[streaming_server](examples/streaming_server.rs)** - SSE streaming for real-time updates
5. **[streaming_client](examples/streaming_client.rs)** - Client consuming SSE streams
6. **[streaming_type_safety](examples/streaming_type_safety.rs)** - Type-safe streaming patterns
7. **[task_server](examples/task_server.rs)** - Long-running async task handling
8. **[multi_agent](examples/multi_agent.rs)** - Agent-to-agent communication

**Run any example:**
```bash
cargo run --example basic_echo_server --features streaming
```

See [examples/README.md](examples/README.md) for detailed usage instructions.

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
a2a-protocol = "0.6.0"

# For streaming support
a2a-protocol = { version = "0.6.0", features = ["streaming"] }
```

### Simple Server (Using AgentLogic)

The easiest way to create an agent - just implement `process_message`:

```rust
use a2a_protocol::{prelude::*, server::{AgentLogic, ServerBuilder, TaskAwareHandler}};
use async_trait::async_trait;

struct MyAgent;

#[async_trait]
impl AgentLogic for MyAgent {
    async fn process_message(&self, message: Message) -> A2aResult<Message> {
        let text = message.text_content().unwrap_or("");
        Ok(Message::agent_text(format!("Processed: {}", text)))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let agent_card = AgentCard::new(
        AgentId::new("my-agent")?,
        "My Agent",
        url::Url::parse("https://example.com")?
    );

    let handler = TaskAwareHandler::with_logic(agent_card, MyAgent);
    ServerBuilder::new(handler).with_port(3000).run().await?;
    Ok(())
}
```

### Simple Client

```rust
use a2a_protocol::{prelude::*, client::ClientBuilder};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = ClientBuilder::new()
        .with_json_rpc("http://localhost:3000/rpc")
        .build()?;

    let message = Message::user_text("Hello!");
    let response = client.send_message(message).await?;

    match response {
        SendResponse::Message(msg) => println!("{}", msg.text_content().unwrap()),
        SendResponse::Task(task) => {
            println!("Task created: {}", task.id);
            // Poll for completion
            let task = client.get_task(&task.id).await?;
            println!("Status: {:?}", task.status.state);
        }
    }
    Ok(())
}
```

### Streaming Client (NEW in v0.6.0)

Use `A2aStreamingClient` for real-time SSE streaming:

```rust
use a2a_protocol::{
    prelude::*,
    client::A2aStreamingClient,
    transport::{JsonRpcTransport, StreamingResult},
};
use futures_util::StreamExt;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create streaming-capable client
    let transport = Arc::new(JsonRpcTransport::new("http://localhost:3000/rpc")?);
    let client = A2aStreamingClient::new(transport);

    // Stream messages in real-time
    let message = Message::user_text("Stream this");
    let mut stream = client.stream_message(message).await?;

    while let Some(result) = stream.next().await {
        match result? {
            StreamingResult::Message(msg) => println!("Message: {:?}", msg.text_content()),
            StreamingResult::Task(task) => println!("Task: {}", task.id),
            StreamingResult::TaskStatusUpdate(update) => println!("Status: {:?}", update.status.state),
            StreamingResult::TaskArtifactUpdate(artifact) => println!("Artifact: {}", artifact.artifact_id),
        }
    }
    Ok(())
}
```

## What's New in v0.6.0

### ✨ SSE Streaming Support (Complete!)
- Full Server-Sent Events implementation
- `message/stream` and `task/resubscribe` RPC methods
- Type-safe `A2aStreamingClient<T>` with compile-time guarantees
- Reconnection support with Last-Event-ID
- 161 tests passing (110 lib + 8 streaming + 17 compliance + 8 RPC + 18 doc)

### 🚀 Developer Experience Improvements
- **`ServerBuilder`** - One-line server setup (inspired by a2a-go)
- **`AgentLogic` trait** - Simplified agent implementation (just implement `process_message`)
- **8 Complete Examples** - From basic to advanced, all runnable
- **Examples README** - Quick start guide with curl commands

### 📖 Better Documentation
- Quick start guide in main README
- Comprehensive examples with detailed comments
- Clear guidance on when to use AgentLogic vs A2aHandler
- API documentation with examples

## Old Quick Start Examples

These examples show the lower-level API (still fully supported):

### Client Usage

```rust
use a2a_protocol::{
    prelude::*,
    client::A2aClient,
    transport::JsonRpcTransport,
};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create JSON-RPC 2.0 transport (spec-compliant)
    let transport = Arc::new(JsonRpcTransport::new("https://agent.example.com/rpc")?);
    let client = A2aClient::new(transport);

    // Send message using A2A v0.3.0 API
    let message = Message::user_text("Hello, agent!");
    let response = client.send_message(message).await?;

    // Response is either Task (async) or Message (immediate)
    match response {
        SendResponse::Message(msg) => {
            println!("Immediate: {}", msg.text_content().unwrap_or(""));
        }
        SendResponse::Task(task) => {
            println!("Task created: {}", task.id);
            // Poll task status
            let task = client.get_task(&task.id).await?;
            println!("Task state: {:?}", task.status.state);
        }
    }
    Ok(())
}
```

### Using the Client Builder

```rust
use a2a_protocol::client::ClientBuilder;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = ClientBuilder::new()
        .with_agent_id("my-client")?
        .with_json_rpc("https://agent.example.com/rpc")
        .with_timeout(30)
        .with_max_retries(3)
        .build()?;

    let message = Message::user_text("Hello!");
    let response = client.send_message(message).await?;
    Ok(())
}
```

### Server Implementation

```rust
use a2a_protocol::{
    prelude::*,
    server::{JsonRpcRouter, TaskAwareHandler},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create agent card
    let agent_card = AgentCard::new(
        AgentId::new("my-agent")?,
        "My Agent",
        url::Url::parse("https://my-agent.example.com")?
    );

    // Create handler with task support
    let handler = TaskAwareHandler::new(agent_card);

    // Create JSON-RPC 2.0 router (spec-compliant)
    let router = JsonRpcRouter::new(handler);

    // Start server
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    println!("A2A server listening on http://127.0.0.1:3000/rpc");
    axum::serve(listener, router.into_router()).await?;

    Ok(())
}
```

## Supported RPC Methods

The server exposes these A2A v0.3.0 spec-compliant JSON-RPC 2.0 methods:

| Method | Description | Returns |
|--------|-------------|---------|
| `message/send` | Send a message to the agent | `Task` or `Message` |
| `task/get` | Get task details and results | `Task` |
| `task/status` | Get current task status | `TaskStatus` |
| `task/cancel` | Cancel a running task | `TaskStatus` |
| `agent/card` | Get agent capabilities | `AgentCard` |

Example JSON-RPC request:
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "message/send",
  "params": {
    "message": {
      "role": "user",
      "parts": [{"kind": "text", "text": "Hello!"}]
    },
    "immediate": true
  }
}
```

## What's New in v0.5.0

### Enhanced AgentCard Metadata

AgentCard now supports optional fields for richer agent discovery:

```rust
use a2a_protocol::core::{AgentCard, AgentProvider};
use url::Url;

let card = AgentCard::builder("my-agent", "1.0.0")
    .with_description("An advanced AI assistant")
    .with_provider(AgentProvider {
        name: "ACME Corp".to_string(),
        url: Some(Url::parse("https://acme.com").unwrap()),
    })
    .with_icon_url(Url::parse("https://acme.com/icon.png").unwrap())
    .with_documentation_url(Url::parse("https://docs.acme.com").unwrap())
    .add_signature(/* AgentCardSignature */)
    .build();
```

**New fields:**
- `provider`: Information about the agent provider (name, URL)
- `iconUrl`: URL to the agent's icon/avatar
- `documentationUrl`: URL to agent documentation
- `signatures`: Cryptographic signatures for verification

### A2A-Specific Error Codes

The protocol now implements A2A-specific JSON-RPC error codes with structured data:

| Code | Error | Description | Data Fields |
|------|-------|-------------|-------------|
| `-32001` | TaskNotFound | Task ID does not exist | `taskId` |
| `-32002` | TaskNotCancelable | Task cannot be cancelled | `taskId`, `state` |
| `-32003` | PushNotificationNotSupported | Server doesn't support push notifications | - |
| `-32004` | UnsupportedOperation | Operation not supported | - |
| `-32005` | ContentTypeNotSupported | Content type not accepted | `contentType` |
| `-32006` | InvalidAgentResponse | Agent returned invalid response | - |
| `-32007` | AuthenticatedExtendedCardNotConfigured | Auth required but not configured | - |

**Example error response:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32001,
    "message": "Task not found: task_abc123",
    "data": {
      "taskId": "task_abc123"
    }
  }
}
```

**Client error handling:**
```rust
match client.get_task("task_abc123").await {
    Err(A2aError::TaskNotFound { task_id }) => {
        println!("Task {} not found", task_id);
    }
    Err(A2aError::TaskNotCancelable { task_id, state }) => {
        println!("Cannot cancel task {} in state {:?}", task_id, state);
    }
    Ok(task) => { /* handle task */ }
    Err(e) => { /* other errors */ }
}
```

## Architecture

### Core Components

- **Core Types**: Message, Task, AgentCard, error handling
- **Transport Layer**: JSON-RPC 2.0 over HTTP (spec-compliant)
- **Client**: High-level client with task management
- **Server**: Axum-based JSON-RPC 2.0 server
- **Authentication**: Multiple auth strategies (API key, OAuth2, Bearer)

### Module Structure

```
a2a-protocol/
├── core/           # Core types and error handling
├── transport/      # JSON-RPC 2.0 transport
├── client/         # Client implementations
├── server/         # Server implementations (JSON-RPC)
└── auth/           # Authentication strategies
```

## Migration from v0.3.x

If upgrading from v0.3.x, note these breaking changes:

### Removed Components
- ❌ `A2aRouter` - Use `JsonRpcRouter` instead
- ❌ Streaming module - Will be re-implemented as SSE in future
- ❌ Direct REST endpoints - Use JSON-RPC 2.0 methods

### Server Changes
```rust
// OLD (v0.3.x) - Non-spec REST endpoints
use a2a_protocol::server::A2aRouter;
let router = A2aRouter::new(handler);

// NEW (v0.4.0) - Spec-compliant JSON-RPC 2.0
use a2a_protocol::server::JsonRpcRouter;
let router = JsonRpcRouter::new(handler);
```

### Client Changes
```rust
// OLD - Direct HTTP POST to /messages
POST /messages
{"role": "user", "parts": [...]}

// NEW - JSON-RPC 2.0 to /rpc
POST /rpc
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "message/send",
  "params": {"message": {"role": "user", "parts": [...]}}
}
```

### Module Structure

```
a2a-protocol/
├── core/           # Core types and error handling
├── transport/      # Transport implementations
├── client/         # Client implementations
├── server/         # Server implementations
├── auth/           # Authentication strategies
└── streaming/      # Streaming support
```

## Features

### Transport Features

- **http-client**: HTTP client support (enabled by default)
- **json-rpc**: JSON-RPC client support (enabled by default)
- **grpc**: gRPC support (requires tonic)
- **websocket**: WebSocket streaming support
- **full**: Enables all features

### Feature Flags

```toml
[dependencies.a2a-protocol]
version = "0.1.0"
features = ["full"]  # Enable all features
```

## Authentication

### API Key Authentication

```rust
use a2a_protocol::auth::ApiKeyAuth;

let auth = ApiKeyAuth::x_api_key_header("your-api-key");
```

### Bearer Token Authentication

```rust
use a2a_protocol::auth::BearerAuth;

let auth = BearerAuth::new("your-bearer-token");
```

### OAuth2 Client Credentials

```rust
use a2a_protocol::auth::OAuth2ClientCredentials;

let auth = OAuth2ClientCredentials::new(
    "https://auth.example.com/token",
    "client-id",
    "client-secret",
    Some("scope"),
);
```

## Streaming

### Client Streaming

```rust
use a2a_protocol::streaming::StreamingClient;

let (client, _) = StreamingClient::new();
let message = Message::new_text("user", "Stream this response");

// Send message for streaming response
client.send_message(message)?;

// Get response stream
let mut response_stream = client.response_stream();
while let Some(part) = response_stream.next().await {
    println!("Received part: {}", part.content);
}
```

## Error Handling

The crate provides comprehensive error handling:

```rust
match client.send_message(message).await {
    Ok(response) => println!("Success: {}", response.message.content),
    Err(a2a_protocol::A2aError::Authentication(msg)) => {
        eprintln!("Authentication failed: {}", msg);
    }
    Err(a2a_protocol::A2aError::Network(err)) => {
        eprintln!("Network error: {}", err);
    }
    Err(err) => {
        eprintln!("Other error: {}", err);
    }
}
```

## Testing

Run tests with:

```bash
cargo test
```

Run tests with specific features:

```bash
cargo test --features "websocket"
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass
5. Submit a pull request

## License

This project is licensed under either of:

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
  https://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or
  https://opensource.org/licenses/MIT)

at your option.

## Acknowledgments

- [A2A Protocol Specification](https://a2a-protocol.org/)
- The Rust community for excellent async primitives