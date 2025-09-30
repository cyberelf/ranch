# A2A Protocol Implementation

A comprehensive, production-ready Rust implementation of the A2A (Agent-to-Agent) protocol specification.

## Features

- **Protocol Compliance**: Full adherence to the A2A specification
- **Multiple Transports**: HTTP, JSON-RPC, gRPC, and WebSocket support
- **Async Native**: Built on tokio for high-performance async communication
- **Type Safe**: Strong typing with serde for serialization
- **Extensible**: Plugin architecture for custom transports and authentication
- **Production Ready**: Comprehensive error handling and testing

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
a2a-protocol = "0.1.0"
```

### Basic Usage

```rust
use a2a_protocol::{
    prelude::*,
    client::A2aClient,
    transport::HttpTransport,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create client
    let transport = HttpTransport::new("https://agent.example.com")?;
    let client = A2aClient::new(Arc::new(transport));

    // Send message
    let response = client.send_text("Hello, agent!").await?;

    println!("Response: {}", response.message.text_content().unwrap_or("No content"));
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
        .with_http("https://agent.example.com")
        .with_timeout(30)
        .with_max_retries(3)
        .build()?;

    let response = client.send_text("Hello!").await?;
    Ok(())
}
```

### Server Implementation

```rust
use a2a_protocol::{
    prelude::*,
    server::{A2aRouter, BasicA2aHandler},
};
use axum::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create agent card
    let agent_card = AgentCard::new(
        AgentId::new("my-agent")?,
        "My Agent",
        Url::parse("https://my-agent.example.com")?
    );

    // Create handler
    let handler = BasicA2aHandler::new(agent_card);

    // Create router
    let router = A2aRouter::new(handler).into_router();

    // Start server
    Server::bind(&"127.0.0.1:3000".parse()?)
        .serve(router.into_make_service())
        .await?;

    Ok(())
}
```

## Architecture

### Core Components

- **Core Types**: Message, AgentCard, error handling
- **Transport Layer**: HTTP, JSON-RPC, gRPC, WebSocket support
- **Client**: High-level client with conversation support
- **Server**: Axum-based server with routing
- **Authentication**: Multiple auth strategies (API key, OAuth2, Bearer)
- **Streaming**: Real-time bidirectional streaming

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