# A2A Protocol Agent Implementation Guide

This guide covers implementing agents that comply with the A2A (Agent-to-Agent) Protocol v0.7.0 using the `a2a-protocol` crate.

## Table of Contents

- [Overview](#overview)
- [Agent Trait](#agent-trait)
- [Task Lifecycle Management](#task-lifecycle-management)
- [JSON-RPC Server Setup](#json-rpc-server-setup)
- [Authentication Strategies](#authentication-strategies)
- [Code Examples](#code-examples)
- [Best Practices](#best-practices)

## Overview

The `a2a-protocol` crate provides a complete implementation of the A2A protocol, which defines how agents communicate over JSON-RPC 2.0. Implementing an A2A-compliant agent involves:

1. Implementing the `Agent` trait
2. Setting up a JSON-RPC 2.0 server with `JsonRpcRouter`
3. Using `TaskAwareHandler` for async task management
4. Configuring authentication

## Agent Trait

The core of any A2A agent is the `Agent` trait:

```rust
#[async_trait]
pub trait Agent: Send + Sync {
    /// Return the agent's profile (capabilities, skills, version)
    async fn profile(&self) -> A2aResult<AgentProfile>;
    
    /// Process an incoming message and return a response
    async fn process_message(&self, message: Message) -> A2aResult<Message>;
}
```

### Agent Profile

The `AgentProfile` describes what your agent can do:

```rust
pub struct AgentProfile {
    pub id: AgentId,              // Unique identifier
    pub name: String,             // Human-readable name
    pub description: Option<String>,
    pub capabilities: Vec<Capability>,  // What it can do
    pub skills: Vec<Skill>,       // Specific skills
    pub version: String,          // Agent version
    pub provider: Option<Provider>,
}
```

**Example**:

```rust
async fn profile(&self) -> A2aResult<AgentProfile> {
    Ok(AgentProfile {
        id: AgentId::new("research-agent"),
        name: "Research Agent".to_string(),
        description: Some("Searches and summarizes information".to_string()),
        capabilities: vec![
            Capability {
                name: "research".to_string(),
                description: Some("Research topics and provide summaries".to_string()),
                parameters: None,
            },
        ],
        skills: vec![
            Skill {
                name: "web-search".to_string(),
                description: Some("Search the web for information".to_string()),
            },
        ],
        version: "1.0.0".to_string(),
        provider: Some(Provider {
            name: "My Company".to_string(),
            url: Some("https://example.com".to_string()),
        }),
    })
}
```

### Processing Messages

Messages follow the A2A protocol format:

```rust
pub struct Message {
    pub role: MessageRole,     // User, Agent, or System
    pub parts: Vec<Part>,      // Content (text, data, files)
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}
```

**Example**:

```rust
async fn process_message(&self, message: Message) -> A2aResult<Message> {
    // Extract text content
    let text = message.text_content()
        .ok_or_else(|| A2aError::InvalidMessage {
            reason: "No text content".to_string()
        })?;
    
    // Process the request
    let response = self.process_text(text).await?;
    
    // Return agent message
    Ok(Message::agent_text(response))
}
```

## Task Lifecycle Management

For long-running operations, return a `Task` instead of an immediate response:

### Task States

```
queued → working → completed
                → failed
                → cancelled
```

### Using TaskAwareHandler

The `TaskAwareHandler` automatically manages task lifecycles:

```rust
use a2a_protocol::prelude::*;

struct LongRunningAgent {
    // Your agent state
}

#[async_trait]
impl Agent for LongRunningAgent {
    async fn profile(&self) -> A2aResult<AgentProfile> {
        // ... profile implementation
    }
    
    async fn process_message(&self, message: Message) -> A2aResult<Message> {
        // For quick operations, return immediately
        if self.is_quick_operation(&message) {
            return Ok(Message::agent_text("Quick response"));
        }
        
        // For long operations, spawn async task
        tokio::spawn(async move {
            // Do expensive work
            self.expensive_computation().await
        });
        
        // Return task handle (TaskAwareHandler manages this)
        // The handler will track the task and allow clients to poll
        Ok(Message::agent_text("Processing..."))
    }
}
```

### Task Methods

When using `TaskAwareHandler`, it automatically implements:

- `task/get` - Get task details
- `task/status` - Check task status
- `task/cancel` - Cancel a running task

## JSON-RPC Server Setup

Setting up an A2A-compliant server is straightforward:

### Basic Server

```rust
use a2a_protocol::prelude::*;
use axum::{Router, routing::post};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create your agent
    let agent = Arc::new(MyAgent::new());
    
    // Wrap with TaskAwareHandler
    let handler = TaskAwareHandler::new(agent);
    
    // Create JSON-RPC router
    let rpc_router = JsonRpcRouter::new(Arc::new(handler));
    
    // Create Axum app
    let app = Router::new()
        .route("/rpc", post(move |body| {
            let router = rpc_router.clone();
            async move { router.handle(body).await }
        }))
        .layer(tower_http::cors::CorsLayer::permissive());
    
    // Start server
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 8080));
    println!("A2A server listening on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}
```

### Supported RPC Methods

Your server automatically supports these JSON-RPC 2.0 methods:

1. **`message/send`** - Send a message to the agent
   ```json
   {
     "jsonrpc": "2.0",
     "method": "message/send",
     "params": {
       "message": {
         "role": "user",
         "parts": [{"type": "text", "text": "Hello"}]
       }
     },
     "id": 1
   }
   ```

2. **`task/get`** - Get task details
   ```json
   {
     "jsonrpc": "2.0",
     "method": "task/get",
     "params": {"task_id": "task-123"},
     "id": 2
   }
   ```

3. **`task/status`** - Check task status
   ```json
   {
     "jsonrpc": "2.0",
     "method": "task/status",
     "params": {"task_id": "task-123"},
     "id": 3
   }
   ```

4. **`task/cancel`** - Cancel a task
   ```json
   {
     "jsonrpc": "2.0",
     "method": "task/cancel",
     "params": {"task_id": "task-123"},
     "id": 4
   }
   ```

5. **`agent/card`** - Get agent capabilities
   ```json
   {
     "jsonrpc": "2.0",
     "method": "agent/card",
     "params": {},
     "id": 5
   }
   ```

## Authentication Strategies

The A2A protocol supports multiple authentication methods:

### 1. API Key Authentication

```rust
use a2a_protocol::prelude::*;

let auth = ApiKeyAuth::new("my-api-key");
let transport = JsonRpcTransport::builder("https://agent.example.com/rpc")
    .with_auth(Arc::new(auth))
    .build()?;
```

### 2. Bearer Token Authentication

```rust
use a2a_protocol::prelude::*;

let auth = BearerAuth::new("my-bearer-token");
let transport = JsonRpcTransport::builder("https://agent.example.com/rpc")
    .with_auth(Arc::new(auth))
    .build()?;
```

### 3. OAuth2 Client Credentials

```rust
use a2a_protocol::prelude::*;

let auth = OAuth2ClientCredentials::new(
    "https://auth.example.com/token",
    "client-id",
    "client-secret",
);
let transport = JsonRpcTransport::builder("https://agent.example.com/rpc")
    .with_auth(Arc::new(auth))
    .build()?;
```

### Server-Side Validation

Implement authentication validation in your server:

```rust
use axum::{
    extract::Request,
    http::StatusCode,
    middleware::{self, Next},
    response::Response,
};

async fn auth_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract authorization header
    let auth_header = request
        .headers()
        .get("authorization")
        .and_then(|h| h.to_str().ok());
    
    match auth_header {
        Some(token) if validate_token(token) => Ok(next.run(request).await),
        _ => Err(StatusCode::UNAUTHORIZED),
    }
}

// Add to your router
let app = Router::new()
    .route("/rpc", post(rpc_handler))
    .layer(middleware::from_fn(auth_middleware));
```

## Code Examples

### Minimal Agent

The simplest possible A2A agent:

```rust
use a2a_protocol::prelude::*;

struct EchoAgent;

#[async_trait]
impl Agent for EchoAgent {
    async fn profile(&self) -> A2aResult<AgentProfile> {
        Ok(AgentProfile {
            id: AgentId::new("echo"),
            name: "Echo Agent".to_string(),
            description: Some("Echoes messages back".to_string()),
            capabilities: vec![],
            skills: vec![],
            version: "1.0.0".to_string(),
            provider: None,
        })
    }

    async fn process_message(&self, message: Message) -> A2aResult<Message> {
        let text = message.text_content()
            .unwrap_or("(no text)");
        Ok(Message::agent_text(format!("Echo: {}", text)))
    }
}
```

### Stateful Agent

An agent that maintains state:

```rust
use a2a_protocol::prelude::*;
use tokio::sync::RwLock;
use std::collections::HashMap;

struct StatefulAgent {
    state: RwLock<HashMap<String, String>>,
}

impl StatefulAgent {
    fn new() -> Self {
        Self {
            state: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl Agent for StatefulAgent {
    async fn profile(&self) -> A2aResult<AgentProfile> {
        Ok(AgentProfile {
            id: AgentId::new("stateful"),
            name: "Stateful Agent".to_string(),
            description: Some("Remembers previous interactions".to_string()),
            capabilities: vec![
                Capability {
                    name: "memory".to_string(),
                    description: Some("Store and retrieve values".to_string()),
                    parameters: None,
                },
            ],
            skills: vec![],
            version: "1.0.0".to_string(),
            provider: None,
        })
    }

    async fn process_message(&self, message: Message) -> A2aResult<Message> {
        let text = message.text_content()
            .ok_or_else(|| A2aError::InvalidMessage {
                reason: "No text content".to_string()
            })?;
        
        // Parse command: "set key value" or "get key"
        let parts: Vec<&str> = text.split_whitespace().collect();
        
        match parts.as_slice() {
            ["set", key, value] => {
                let mut state = self.state.write().await;
                state.insert(key.to_string(), value.to_string());
                Ok(Message::agent_text(format!("Set {} = {}", key, value)))
            }
            ["get", key] => {
                let state = self.state.read().await;
                match state.get(*key) {
                    Some(value) => Ok(Message::agent_text(format!("{} = {}", key, value))),
                    None => Ok(Message::agent_text(format!("Key {} not found", key))),
                }
            }
            _ => Ok(Message::agent_text("Usage: 'set key value' or 'get key'")),
        }
    }
}
```

### Streaming Agent

An agent that processes data streams:

```rust
use a2a_protocol::prelude::*;
use tokio::io::AsyncReadExt;

struct StreamingAgent;

#[async_trait]
impl Agent for StreamingAgent {
    async fn profile(&self) -> A2aResult<AgentProfile> {
        Ok(AgentProfile {
            id: AgentId::new("streaming"),
            name: "Streaming Agent".to_string(),
            description: Some("Processes streaming data".to_string()),
            capabilities: vec![
                Capability {
                    name: "stream-processing".to_string(),
                    description: Some("Process data streams".to_string()),
                    parameters: None,
                },
            ],
            skills: vec![],
            version: "1.0.0".to_string(),
            provider: None,
        })
    }

    async fn process_message(&self, message: Message) -> A2aResult<Message> {
        // Extract data parts
        let data_parts: Vec<&DataPart> = message.parts.iter()
            .filter_map(|p| match p {
                Part::Data(data) => Some(data),
                _ => None,
            })
            .collect();
        
        if data_parts.is_empty() {
            return Ok(Message::agent_text("No data to process"));
        }
        
        // Process each data stream
        let mut results = Vec::new();
        for data_part in data_parts {
            let processed = self.process_data(&data_part.data).await?;
            results.push(processed);
        }
        
        // Return summary
        Ok(Message::agent_text(format!(
            "Processed {} data streams", 
            results.len()
        )))
    }
}

impl StreamingAgent {
    async fn process_data(&self, data: &[u8]) -> A2aResult<String> {
        // Your data processing logic here
        Ok(format!("Processed {} bytes", data.len()))
    }
}
```

## Best Practices

### 1. Error Handling

Always use appropriate error types:

```rust
use a2a_protocol::A2aError;

// Invalid input
return Err(A2aError::InvalidMessage {
    reason: "Missing required field".to_string()
});

// Task not found
return Err(A2aError::TaskNotFound {
    task_id: task_id.to_string()
});

// Internal processing error
return Err(A2aError::Internal(
    format!("Database error: {}", e)
));
```

### 2. Timeout Configuration

Set appropriate timeouts for operations:

```rust
use tokio::time::{timeout, Duration};

async fn process_with_timeout(&self, message: Message) -> A2aResult<Message> {
    let result = timeout(
        Duration::from_secs(30),
        self.expensive_operation(message)
    ).await;
    
    match result {
        Ok(Ok(msg)) => Ok(msg),
        Ok(Err(e)) => Err(e),
        Err(_) => Err(A2aError::Timeout {
            operation: "process_message".to_string(),
        }),
    }
}
```

### 3. Resource Management

Clean up resources properly:

```rust
impl Drop for MyAgent {
    fn drop(&mut self) {
        // Clean up connections, files, etc.
        self.cleanup();
    }
}
```

### 4. Logging and Tracing

Use structured logging:

```rust
use tracing::{info, warn, error, instrument};

#[instrument(skip(self, message))]
async fn process_message(&self, message: Message) -> A2aResult<Message> {
    info!("Processing message from role: {:?}", message.role);
    
    match self.handle_message(message).await {
        Ok(response) => {
            info!("Successfully processed message");
            Ok(response)
        }
        Err(e) => {
            error!("Failed to process message: {}", e);
            Err(e)
        }
    }
}
```

### 5. Capability Declaration

Be accurate and complete in your capability declarations:

```rust
capabilities: vec![
    Capability {
        name: "text-analysis".to_string(),
        description: Some("Analyze text for sentiment, entities, and topics".to_string()),
        parameters: Some(serde_json::json!({
            "languages": ["en", "es", "fr"],
            "max_length": 10000,
            "features": ["sentiment", "entities", "topics"]
        })),
    },
]
```

### 6. Testing

Write comprehensive tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_agent_profile() {
        let agent = MyAgent::new();
        let profile = agent.profile().await.unwrap();
        
        assert_eq!(profile.id.as_str(), "my-agent");
        assert!(!profile.capabilities.is_empty());
    }

    #[tokio::test]
    async fn test_process_message() {
        let agent = MyAgent::new();
        let message = Message::user_text("test");
        
        let response = agent.process_message(message).await.unwrap();
        assert_eq!(response.role, MessageRole::Agent);
    }
}
```

## Additional Resources

- [A2A Protocol Specification](https://a2a-protocol.org/)
- [JSON-RPC 2.0 Specification](https://www.jsonrpc.org/specification)
- [Examples in Repository](../examples/)
- [Root AGENT.md](../AGENT.md) - Architecture overview

---

**Last Updated**: 2025-12-11  
**A2A Protocol Version**: 0.7.0
