# Getting Started with A2A Protocol

This guide will help you build your first A2A agent in Rust, step by step. By the end, you'll understand all the key features through v0.7.0.

## Prerequisites

- Rust 1.70 or later
- Basic understanding of async Rust (tokio)
- Familiarity with JSON-RPC 2.0 (optional but helpful)

## Table of Contents

1. [Installation](#installation)
2. [Your First Server](#your-first-server)
3. [Your First Client](#your-first-client)
4. [Understanding AgentLogic vs A2aHandler](#understanding-agentlogic-vs-a2ahandler)
5. [Working with Tasks](#working-with-tasks)
6. [Streaming Responses (v0.6.0)](#streaming-responses-v060)
7. [Webhooks & Push Notifications (v0.7.0)](#webhooks--push-notifications-v070)
8. [Agent-to-Agent Communication](#agent-to-agent-communication)
9. [Common Patterns](#common-patterns)
10. [Troubleshooting](#troubleshooting)

## Installation

Add the A2A protocol library to your `Cargo.toml`:

```toml
[dependencies]
a2a-protocol = { version = "0.7.0", features = ["streaming"] }
tokio = { version = "1.0", features = ["full"] }
async-trait = "0.1"
url = "2.4"
```

## Your First Server

Let's create a simple echo server that responds to messages.

### Step 1: Create a new project

```bash
cargo new my-agent
cd my-agent
```

### Step 2: Implement AgentLogic

Create `src/main.rs`:

```rust
use a2a_protocol::{
    prelude::*,
    server::{AgentLogic, ServerBuilder, TaskAwareHandler},
};
use async_trait::async_trait;
use url::Url;

/// Our simple echo agent
struct EchoAgent;

#[async_trait]
impl AgentLogic for EchoAgent {
    /// Process incoming messages
    async fn process_message(&self, message: Message) -> A2aResult<Message> {
        // Extract text from the message
        let text = message
            .text_content()
            .ok_or_else(|| A2aError::Validation("No text content".to_string()))?;

        // Process and respond
        let response = format!("You said: {}", text);
        Ok(Message::agent_text(response))
    }

    /// Optional: Initialize the agent
    async fn initialize(&self) -> A2aResult<()> {
        println!("🤖 Echo Agent is ready!");
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create agent card (identity)
    let agent_id = AgentId::new("echo-agent".to_string())?;
    let agent_card = AgentCard::new(
        agent_id,
        "Echo Agent",
        Url::parse("https://example.com")?,
    );

    // Wrap logic in task-aware handler
    let handler = TaskAwareHandler::with_logic(agent_card, EchoAgent);

    // Start server
    println!("🚀 Starting server on http://localhost:3000");
    ServerBuilder::new(handler)
        .with_port(3000)
        .run()
        .await?;

    Ok(())
}
```

### Step 3: Run the server

```bash
cargo run
```

You should see:
```
🤖 Echo Agent is ready!
🚀 Starting server on http://localhost:3000
```

### Step 4: Test with curl

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
        "parts": [{"text":"Hello, Agent!"}]
      },
      "immediate": true
    }
  }'
```

Expected response:
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "role": "agent",
    "parts": [
      {
        "text": "You said: Hello, Agent!"
      }
    ]
  }
}
```

🎉 Congratulations! You've built your first A2A agent!

## Your First Client

Now let's create a client to talk to our server.

### Step 1: Create a new binary

Add to `Cargo.toml`:
```toml
[[bin]]
name = "client"
path = "src/client.rs"
```

### Step 2: Implement the client

Create `src/client.rs`:

```rust
use a2a_protocol::{
    prelude::*,
    client::ClientBuilder,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create client
    let client = ClientBuilder::new()
        .with_json_rpc("http://localhost:3000/rpc")
        .build()?;

    println!("📡 Connected to server");

    // Send a message
    let message = Message::user_text("Hello from client!");
    println!("📤 Sending: Hello from client!");

    let response = client.send_message(message).await?;

    // Handle response
    match response {
        SendResponse::Message(msg) => {
            println!("📥 Received: {}", msg.text_content().unwrap_or(""));
        }
        SendResponse::Task(task) => {
            println!("📋 Task created: {}", task.id);
        }
    }

    Ok(())
}
```

### Step 3: Run the client

Make sure your server is still running, then:

```bash
cargo run --bin client
```

Output:
```
📡 Connected to server
📤 Sending: Hello from client!
📥 Received: You said: Hello from client!
```

## Understanding AgentLogic vs A2aHandler

The library provides two ways to implement agents:

### AgentLogic (Recommended for Most Use Cases)

**Use when:**
- You want simple, clean code
- You're building a typical agent that processes messages
- You don't need custom task management

```rust
#[async_trait]
impl AgentLogic for MyAgent {
    async fn process_message(&self, message: Message) -> A2aResult<Message> {
        // Your logic here
    }
}
```

**Benefits:**
- Simple API - just implement `process_message`
- Task management handled automatically
- Clean, readable code

### A2aHandler (Advanced)

**Use when:**
- You need full control over the request/response cycle
- You have custom task management requirements
- You need access to raw JSON-RPC context

```rust
#[async_trait]
impl A2aHandler for MyAdvancedAgent {
    async fn handle_message(&self, request: MessageSendRequest) -> A2aResult<SendResponse> {
        // Custom logic with full control
    }
    
    async fn handle_task_get(&self, request: TaskGetRequest) -> A2aResult<Task> {
        // Custom task retrieval
    }
    
    // ... other methods
}
```

**When to upgrade:**
- You find yourself fighting against `AgentLogic`
- You need streaming control
- You need custom error handling

**Rule of thumb:** Start with `AgentLogic`. Upgrade to `A2aHandler` only if needed.

## Working with Tasks

Tasks represent long-running operations in A2A. Here's how to use them:

### Server: Creating Tasks

```rust
#[async_trait]
impl AgentLogic for ComputeAgent {
    async fn process_message(&self, message: Message) -> A2aResult<Message> {
        let text = message.text_content().unwrap_or("");
        
        // Simulate long computation
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        
        let result = format!("Computed result for: {}", text);
        Ok(Message::agent_text(result))
    }
}
```

The `TaskAwareHandler` automatically:
- Creates tasks for requests without `immediate: true`
- Manages task state (pending → running → completed)
- Stores results for later retrieval

### Client: Polling Tasks

```rust
let message = Message::user_text("Heavy computation");
let response = client.send_message(message).await?;

match response {
    SendResponse::Task(task) => {
        println!("Task ID: {}", task.id);
        
        // Poll for completion
        loop {
            let status = client.get_task_status(&task.id).await?;
            
            match status.state {
                TaskState::Completed => {
                    // Get final result
                    let task = client.get_task(&task.id).await?;
                    println!("Result: {:?}", task.result);
                    break;
                }
                TaskState::Failed => {
                    println!("Task failed!");
                    break;
                }
                _ => {
                    // Still running
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                }
            }
        }
    }
    SendResponse::Message(msg) => {
        // Immediate response
        println!("Immediate: {}", msg.text_content().unwrap());
    }
}
```

## Streaming Responses (v0.6.0)

> **NEW in v0.6.0** - Real-time updates using Server-Sent Events (SSE)

For real-time updates, use SSE streaming:

### Server

The `TaskAwareHandler` automatically supports streaming when you use `ServerBuilder`:

```rust
// Streaming is automatically enabled!
ServerBuilder::new(handler)
    .with_port(3000)
    .run()
    .await?;
```

### Client

Use `A2aStreamingClient` for streaming:

```rust
use a2a_protocol::{
    client::A2aStreamingClient,
    transport::{JsonRpcTransport, StreamingResult},
};
use futures_util::StreamExt;
use std::sync::Arc;

// Create streaming client
let transport = Arc::new(JsonRpcTransport::new("http://localhost:3000/rpc")?);
let client = A2aStreamingClient::new(transport);

// Send streaming message
let message = Message::user_text("Stream this");
let mut stream = client.stream_message(message).await?;

// Process events
while let Some(result) = stream.next().await {
    match result? {
        StreamingResult::Message(msg) => {
            println!("Message: {}", msg.text_content().unwrap_or(""));
        }
        StreamingResult::TaskStatusUpdate(update) => {
            println!("Status: {:?}", update.status.state);
        }
        _ => {}
    }
}
```

**When to use streaming:**
- Real-time progress updates
- Long-running tasks with intermediate results
- Chat-like interactions
- Live data feeds

## Webhooks & Push Notifications (v0.7.0)

> **NEW in v0.7.0** - Get notified when tasks complete instead of polling!

Webhooks let the server notify you when something happens, eliminating the need for polling.

### Why Webhooks?

**Without webhooks (polling):**
```rust
// ❌ Inefficient
loop {
    let status = client.get_task_status(&task_id).await?;
    if status.state == TaskState::Completed {
        break;
    }
    tokio::time::sleep(Duration::from_secs(5)).await;
}
```

**With webhooks:**
```rust
// ✅ Efficient - server notifies you!
client.configure_webhook(&task_id, webhook_config).await?;
// Do other work... server will POST to your webhook when done
```

### Quick Webhook Setup

**1. Create a webhook receiver:**

```rust
use axum::{routing::post, Router, Json};
use serde_json::Value;

async fn handle_webhook(Json(payload): Json<Value>) -> &'static str {
    println!("📨 Task {} {}", 
        payload["task"]["id"], 
        payload["event"]
    );
    "OK"
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/webhook", post(handle_webhook));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
        .await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

**2. Configure webhook (JSON-RPC):**

```bash
curl -X POST http://localhost:3000/rpc \
  -H 'Content-Type: application/json' \
  -d '{
    "jsonrpc": "2.0",
    "method": "tasks/pushNotificationConfig/set",
    "params": {
      "taskId": "your-task-id",
      "config": {
        "url": "https://myapp.com/webhook",
        "events": ["completed", "failed"],
        "authentication": {
          "type": "bearer",
          "token": "secret-token"
        }
      }
    }
  }'
```

**3. Receive notification:**

When your task completes, you'll receive:

```json
{
  "event": "completed",
  "task": {
    "id": "task-123",
    "status": {"state": "completed"},
    "result": {...}
  },
  "timestamp": "2025-11-11T10:30:00Z",
  "agentId": "my-agent"
}
```

### Webhook Events

Subscribe to the events you care about:

- `completed` - Task finishes successfully
- `failed` - Task encounters an error
- `cancelled` - Task is cancelled
- `statusChanged` - Any status transition
- `artifactAdded` - New artifact created

### Webhook Security

✅ **HTTPS required** - All webhook URLs must use HTTPS  
✅ **SSRF protection** - Blocks private IPs and internal networks  
✅ **Authentication** - Bearer tokens or custom headers  
✅ **Retry logic** - Automatic retries with exponential backoff

**For local testing**, use ngrok:
```bash
ngrok http 8080
# Use the HTTPS URL: https://abc123.ngrok.io/webhook
```

📖 **Learn more:** See the complete [WEBHOOKS.md](WEBHOOKS.md) guide for detailed examples, security best practices, and troubleshooting.

## Agent-to-Agent Communication

Agents can communicate with each other:

```rust
use a2a_protocol::{
    prelude::*,
    client::ClientBuilder,
    server::{AgentLogic, ServerBuilder, TaskAwareHandler},
};
use async_trait::async_trait;

/// Agent that calls another agent
struct ManagerAgent {
    worker_client: A2aClient,
}

impl ManagerAgent {
    fn new(worker_url: &str) -> A2aResult<Self> {
        let worker_client = ClientBuilder::new()
            .with_json_rpc(worker_url)
            .build()?;
        Ok(Self { worker_client })
    }
}

#[async_trait]
impl AgentLogic for ManagerAgent {
    async fn process_message(&self, message: Message) -> A2aResult<Message> {
        let text = message.text_content().unwrap_or("");
        
        // Delegate to worker agent
        let worker_msg = Message::user_text(format!("Process: {}", text));
        let response = self.worker_client.send_message(worker_msg).await?;
        
        match response {
            SendResponse::Message(msg) => {
                let result = format!("Manager got result: {}", msg.text_content().unwrap_or(""));
                Ok(Message::agent_text(result))
            }
            SendResponse::Task(task) => {
                Ok(Message::agent_text(format!("Worker created task: {}", task.id)))
            }
        }
    }
}
```

## Common Patterns

### Pattern 1: Error Handling

```rust
async fn process_message(&self, message: Message) -> A2aResult<Message> {
    let text = message
        .text_content()
        .ok_or_else(|| A2aError::Validation("Missing text".to_string()))?;
    
    // Your logic with ? operator
    let result = some_fallible_operation(text)?;
    
    Ok(Message::agent_text(result))
}
```

### Pattern 2: Multi-Part Messages

```rust
use a2a_protocol::core::{Part, DataPart};

async fn process_message(&self, message: Message) -> A2aResult<Message> {
    // Create response with multiple parts
    let parts = vec![
        Part::Text(TextPart {
            text: "Here's your data:".to_string(),
        }),
        Part::Data(DataPart {
            data: serde_json::json!({"result": 42}),
            mime_type: Some("application/json".to_string()),
        }),
    ];
    
    let mut response = Message::agent_text("");
    response.parts = parts;
    Ok(response)
}
```

### Pattern 3: Configuration

```rust
struct ConfigurableAgent {
    api_key: String,
    timeout: u64,
}

impl ConfigurableAgent {
    fn new(api_key: String, timeout: u64) -> Self {
        Self { api_key, timeout }
    }
}

#[async_trait]
impl AgentLogic for ConfigurableAgent {
    async fn process_message(&self, message: Message) -> A2aResult<Message> {
        // Example: Use configuration in your logic
        let text = message.text_content().unwrap_or("");
        let response = format!(
            "Processed '{}' with timeout {}s",
            text, self.timeout
        );
        Ok(Message::agent_text(response))
    }
}
```

## Troubleshooting

### Common Issues

#### 1. "Connection refused"

**Problem:** Client can't connect to server.

**Solution:**
- Make sure server is running
- Check the port number matches
- Verify firewall settings

```bash
# Check if server is listening
netstat -an | grep 3000
```

#### 2. "No text content in message"

**Problem:** Message doesn't have a text part.

**Solution:**
```rust
// Instead of unwrap():
let text = message.text_content().unwrap();

// Use proper error handling:
let text = message
    .text_content()
    .ok_or_else(|| A2aError::Validation("No text content".to_string()))?;
```

#### 3. "Task not found"

**Problem:** Trying to retrieve a task that doesn't exist.

**Solution:**
```rust
match client.get_task(&task_id).await {
    Ok(task) => { /* handle task */ }
    Err(A2aError::TaskNotFound { task_id }) => {
        println!("Task {} not found", task_id);
    }
    Err(e) => { /* other errors */ }
}
```

#### 4. Streaming not working

**Problem:** Streaming client errors or no events received.

**Solution:**
- Enable the `streaming` feature: `features = ["streaming"]`
- Use `A2aStreamingClient`, not `A2aClient`
- Check that server has streaming enabled (automatic with `ServerBuilder`)

#### 5. Build errors with examples

**Problem:** Examples fail to compile.

**Solution:**
```bash
# Always build examples with streaming feature
cargo build --example basic_echo_server --features streaming
cargo run --example echo_client --features streaming
```

### Getting Help

1. Check the [examples/](examples/) directory for working code
2. Read the [API documentation](https://docs.rs/a2a-protocol)
3. Review the [A2A specification](https://a2a-protocol.org/)
4. Search existing issues on GitHub
5. Open a new issue with:
   - Your code
   - Error messages
   - Expected vs actual behavior

## Next Steps

Now that you understand the basics:

1. **Explore examples:** Run all 8 examples to see different patterns
2. **Build something:** Create your own agent with custom logic
3. **Add streaming:** Try the streaming examples for real-time updates
4. **Go multi-agent:** Connect multiple agents together
5. **Read the spec:** Understand the full A2A protocol capabilities

Happy building! 🚀
