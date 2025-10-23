# A2A Protocol Implementation

A comprehensive, production-ready Rust implementation of the A2A (Agent-to-Agent) protocol v0.3.0 specification.

## Features

- **Spec Compliance**: Strict adherence to the A2A v0.3.0 specification
- **JSON-RPC 2.0**: Full JSON-RPC 2.0 transport implementation
- **Async Native**: Built on tokio for high-performance async communication
- **Type Safe**: Strong typing with serde for serialization
- **Task Management**: Complete task lifecycle support
- **Production Ready**: Comprehensive error handling and testing

## Specification Compliance

This crate implements **A2A Protocol v0.3.0** with strict spec compliance:

✅ **Supported:**
- JSON-RPC 2.0 transport over HTTP
- All required RPC methods (`message/send`, `task/get`, `task/cancel`, `task/status`, `agent/card`)
- Complete Task lifecycle management
- A2A Message format with Parts (TextPart, FilePart, DataPart)
- AgentCard discovery

🚧 **Planned:**
- Server-Sent Events (SSE) streaming (`message/stream`, `task/resubscribe`)
- Push notifications (`task/pushNotificationConfig/*`)
- gRPC transport (optional)
- HTTP+JSON/REST transport (if spec clarifies patterns)

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
a2a-protocol = "0.4.0"
```

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