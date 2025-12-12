# Webhooks & Push Notifications Guide

> **🎉 NEW in v0.7.0** - Get real-time notifications when your tasks complete!

This guide explains how to use webhooks (push notifications) to receive asynchronous updates about your tasks instead of polling for status.

## Table of Contents

1. [Why Use Webhooks?](#why-use-webhooks)
2. [Quick Start](#quick-start)
3. [How It Works](#how-it-works)
4. [Configuration](#configuration)
5. [Security](#security)
6. [Troubleshooting](#troubleshooting)
7. [Best Practices](#best-practices)
8. [Examples](#examples)

## Why Use Webhooks?

### The Problem: Polling is Inefficient

Without webhooks, you have to repeatedly ask "is it done yet?" (polling):

```rust
// ❌ Inefficient polling approach
loop {
    let status = client.get_task_status(&task_id).await?;
    if status.state == TaskState::Completed {
        break;
    }
    tokio::time::sleep(Duration::from_secs(5)).await; // Waste time waiting
}
```

**Problems:**
- Wastes resources (client keeps checking)
- Wastes bandwidth (unnecessary requests)
- Higher latency (might wait 5 seconds after completion)
- Server load (handling status checks)

### The Solution: Webhooks

With webhooks, the server **tells you** when something happens:

```rust
// ✅ Efficient webhook approach
// 1. Configure webhook once
client.configure_webhook(&task_id, "https://myapp.com/webhook").await?;

// 2. Do other work - you'll be notified when done!
// (Server will POST to your webhook URL when task completes)
```

**Benefits:**
- ✅ Zero polling overhead
- ✅ Instant notifications
- ✅ Scalable to thousands of tasks
- ✅ Lower server load

## Quick Start

### Step 1: Create a Webhook Receiver

First, set up an endpoint to receive webhook notifications:

```rust
use axum::{routing::post, Router, Json};
use serde_json::Value;

async fn handle_webhook(Json(payload): Json<Value>) -> &'static str {
    println!("📨 Webhook received!");
    println!("Event: {}", payload["event"]);
    println!("Task ID: {}", payload["task"]["id"]);
    println!("Status: {}", payload["task"]["status"]["state"]);
    
    "OK"
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/webhook", post(handle_webhook));
    
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080").await.unwrap();
    println!("🎣 Webhook receiver listening on http://127.0.0.1:8080/webhook");
    axum::serve(listener, app).await.unwrap();
}
```

### Step 2: Configure the Webhook

Tell the A2A server to send notifications to your endpoint:

```bash
curl -X POST http://localhost:3000/rpc \
  -H 'Content-Type: application/json' \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "tasks/pushNotificationConfig/set",
    "params": {
      "taskId": "your-task-id",
      "config": {
        "url": "https://myapp.com/webhook",
        "events": ["completed", "failed"],
        "authentication": {
          "type": "bearer",
          "token": "your-secret-token"
        }
      }
    }
  }'
```

### Step 3: Receive Notifications

When your task completes (or fails), you'll receive a POST request:

```json
{
  "event": "completed",
  "task": {
    "id": "task-abc-123",
    "status": {
      "state": "completed",
      "timestamp": "2025-11-11T10:30:00Z"
    },
    "result": {
      "role": "agent",
      "parts": [{"text": "Task result here"}]
    }
  },
  "timestamp": "2025-11-11T10:30:00Z",
  "agentId": "my-agent"
}
```

That's it! 🎉

## How It Works

### Architecture

```
┌─────────────┐         ┌──────────────┐         ┌──────────────┐
│   Client    │         │  A2A Server  │         │   Webhook    │
│             │         │              │         │   Receiver   │
└──────┬──────┘         └──────┬───────┘         └──────┬───────┘
       │                       │                        │
       │  1. Create Task       │                        │
       │──────────────────────>│                        │
       │                       │                        │
       │  2. Configure Webhook │                        │
       │──────────────────────>│                        │
       │                       │                        │
       │  3. Task ID           │                        │
       │<──────────────────────│                        │
       │                       │                        │
       │   ... client does     │   ... task runs ...   │
       │   other work ...      │                        │
       │                       │                        │
       │                       │  4. Task completes     │
       │                       │         │              │
       │                       │  5. Trigger webhook    │
       │                       │──────────────────────> │
       │                       │                        │
       │                       │  6. Webhook processed  │
       │                       │<────────────────────── │
       │                       │                        │
```

### Event Types

You can subscribe to these events:

| Event | When It Fires | Use Case |
|-------|---------------|----------|
| `completed` | Task successfully finishes | Get final results |
| `failed` | Task encounters an error | Handle failures |
| `cancelled` | Task is cancelled | Cleanup resources |
| `statusChanged` | Any status transition | Monitor progress |
| `artifactAdded` | New artifact created | Stream outputs |

**Example: Subscribe to multiple events**

```json
{
  "events": ["completed", "failed", "cancelled"]
}
```

## Configuration

### Full Configuration Example

```rust
use a2a_protocol::core::{
    PushNotificationConfig,
    PushNotificationAuth,
    TaskEvent,
};
use url::Url;

let config = PushNotificationConfig::new(
    Url::parse("https://myapp.com/webhook")?,
    vec![
        TaskEvent::Completed,
        TaskEvent::Failed,
        TaskEvent::Cancelled,
    ],
    Some(PushNotificationAuth::Bearer {
        token: "your-secret-token".to_string(),
    }),
);
```

### Authentication Options

#### 1. Bearer Token (Recommended)

```json
{
  "authentication": {
    "type": "bearer",
    "token": "your-secret-token"
  }
}
```

Server sends:
```http
POST /webhook HTTP/1.1
Host: myapp.com
Authorization: Bearer your-secret-token
Content-Type: application/json
```

#### 2. Custom Headers

```json
{
  "authentication": {
    "type": "customHeaders",
    "headers": {
      "X-API-Key": "your-api-key",
      "X-Custom-Header": "custom-value"
    }
  }
}
```

Server sends:
```http
POST /webhook HTTP/1.1
Host: myapp.com
X-API-Key: your-api-key
X-Custom-Header: custom-value
Content-Type: application/json
```

### Webhook Payload Format

Every webhook POST contains:

```typescript
{
  event: "completed" | "failed" | "cancelled" | "statusChanged" | "artifactAdded",
  task: {
    id: string,
    status: {
      state: string,
      timestamp: string
    },
    result?: Message,
    error?: string
  },
  timestamp: string,  // ISO 8601 format
  agentId: string
}
```

## Security

### 🔒 HTTPS Required

**All webhook URLs MUST use HTTPS.** HTTP URLs are rejected for security:

```rust
// ✅ Valid
Url::parse("https://myapp.com/webhook")?

// ❌ Invalid - will be rejected
Url::parse("http://myapp.com/webhook")?
```

**Why?** HTTP sends data in plain text, exposing:
- Task results
- Authentication tokens
- Sensitive information

### 🛡️ SSRF Protection

The server automatically blocks webhooks to private networks to prevent Server-Side Request Forgery (SSRF) attacks.

**Blocked targets:**

```
❌ Private IPs:
   10.0.0.0/8
   172.16.0.0/12
   192.168.0.0/16

❌ Localhost:
   127.0.0.1
   ::1
   localhost

❌ Link-local:
   169.254.0.0/16
   fe80::/10

❌ Special:
   169.254.169.254 (AWS metadata)
   .local domains
   .internal domains
```

**Allowed targets:**

```
✅ Public domains:
   https://myapp.com
   https://api.example.com

✅ Public IPs:
   https://8.8.8.8
   https://[2001:4860:4860::8888]
```

### 🔐 Authentication Best Practices

1. **Always use authentication** - Don't accept unauthenticated webhooks
2. **Rotate tokens regularly** - Update tokens every 90 days
3. **Use strong tokens** - Generate with `openssl rand -hex 32`
4. **Store securely** - Never hardcode tokens, use environment variables

```rust
// ✅ Good - from environment
let token = std::env::var("WEBHOOK_TOKEN")?;

// ❌ Bad - hardcoded
let token = "hardcoded-secret"; // Don't do this!
```

### Validating Webhooks

Verify incoming webhooks to prevent spoofing:

```rust
use axum::{
    http::StatusCode,
    extract::Request,
    middleware::Next,
    response::Response,
};

async fn verify_webhook(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Check Authorization header
    let auth_header = request
        .headers()
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;
    
    let expected = format!("Bearer {}", std::env::var("WEBHOOK_TOKEN").unwrap());
    
    if auth_header != expected {
        return Err(StatusCode::UNAUTHORIZED);
    }
    
    Ok(next.run(request).await)
}
```

## Troubleshooting

### Common Issues

#### 1. "Webhook URL must use HTTPS"

**Problem:** Trying to use HTTP instead of HTTPS.

**Solution:**
```rust
// ❌ Wrong
url: "http://example.com/webhook"

// ✅ Correct
url: "https://example.com/webhook"
```

**For testing locally:**
- Use ngrok: `ngrok http 8080` → HTTPS tunnel
- Use localhost.run: `ssh -R 80:localhost:8080 nokey@localhost.run`

#### 2. "Webhook URL cannot target private IP addresses"

**Problem:** Trying to send webhooks to internal network.

**Solution:**
- Use a public domain
- For testing, use tunneling services (ngrok, localhost.run)

```bash
# Create HTTPS tunnel to localhost
ngrok http 8080
# Use the HTTPS URL: https://abc123.ngrok.io/webhook
```

#### 3. "No events configured"

**Problem:** Empty events array.

**Solution:**
```json
{
  "events": []  // ❌ Invalid
}

{
  "events": ["completed"]  // ✅ Valid
}
```

#### 4. Webhooks not received

**Checklist:**
- [ ] Is your webhook receiver running?
- [ ] Is the URL publicly accessible?
- [ ] Did you configure the webhook for the correct task?
- [ ] Are you subscribed to the right events?
- [ ] Check authentication token matches

**Debug:**
```bash
# Test webhook receiver is accessible
curl -X POST https://your-webhook-url/webhook \
  -H 'Content-Type: application/json' \
  -d '{"test": true}'
```

#### 5. Authentication failures

**Problem:** Webhook receiver rejects requests.

**Debug:**
```rust
// Add logging to see what's being sent
async fn handle_webhook(
    headers: axum::http::HeaderMap,
    Json(payload): Json<Value>,
) -> &'static str {
    println!("Headers: {:?}", headers);
    println!("Payload: {:?}", payload);
    "OK"
}
```

**Check:**
- Token in configuration matches receiver expectations
- Header name is correct (`Authorization` vs `X-API-Key`)
- Token format is correct (`Bearer token` vs just `token`)

## Best Practices

### 1. Handle Failures Gracefully

The server retries failed webhooks with exponential backoff:

```
Attempt 1: Immediate
Attempt 2: 1 second delay
Attempt 3: 2 seconds delay
Attempt 4: 4 seconds delay
Attempt 5: 8 seconds delay (final attempt)
```

**Your receiver should:**
- Return `200 OK` quickly (< 1 second)
- Process asynchronously if needed
- Be idempotent (handle duplicates)

```rust
use axum::http::StatusCode;

async fn handle_webhook(Json(payload): Json<Value>) -> StatusCode {
    // ✅ Quick response
    tokio::spawn(async move {
        // Process async
        process_webhook_payload(payload).await;
    });
    
    StatusCode::OK  // Return immediately
}
```

### 2. Implement Idempotency

Webhooks might be delivered multiple times (retries, network issues).

```rust
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashSet;

struct WebhookReceiver {
    processed_ids: Arc<RwLock<HashSet<String>>>,
}

impl WebhookReceiver {
    async fn handle(&self, payload: Value) -> Result<(), Error> {
        let task_id = payload["task"]["id"].as_str().unwrap();
        let timestamp = payload["timestamp"].as_str().unwrap();
        let event_id = format!("{}-{}", task_id, timestamp);
        
        // Check if already processed
        {
            let processed = self.processed_ids.read().await;
            if processed.contains(&event_id) {
                println!("⏭️  Already processed: {}", event_id);
                return Ok(());
            }
        }
        
        // Process the webhook
        process_event(&payload).await?;
        
        // Mark as processed
        {
            let mut processed = self.processed_ids.write().await;
            processed.insert(event_id);
        }
        
        Ok(())
    }
}
```

### 3. Monitor Webhook Health

Track webhook delivery success:

```rust
use std::sync::atomic::{AtomicU64, Ordering};

static WEBHOOKS_RECEIVED: AtomicU64 = AtomicU64::new(0);
static WEBHOOKS_PROCESSED: AtomicU64 = AtomicU64::new(0);
static WEBHOOKS_FAILED: AtomicU64 = AtomicU64::new(0);

async fn handle_webhook(Json(payload): Json<Value>) -> StatusCode {
    WEBHOOKS_RECEIVED.fetch_add(1, Ordering::Relaxed);
    
    match process_webhook(payload).await {
        Ok(_) => {
            WEBHOOKS_PROCESSED.fetch_add(1, Ordering::Relaxed);
            StatusCode::OK
        }
        Err(e) => {
            WEBHOOKS_FAILED.fetch_add(1, Ordering::Relaxed);
            eprintln!("Webhook processing error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

// Expose metrics
async fn metrics() -> String {
    format!(
        "webhooks_received {}\nwebhooks_processed {}\nwebhooks_failed {}",
        WEBHOOKS_RECEIVED.load(Ordering::Relaxed),
        WEBHOOKS_PROCESSED.load(Ordering::Relaxed),
        WEBHOOKS_FAILED.load(Ordering::Relaxed),
    )
}
```

### 4. Use Appropriate Events

Don't subscribe to unnecessary events:

```rust
// ❌ Too many events
vec![
    TaskEvent::StatusChanged,  // Fires on every transition
    TaskEvent::Completed,      // Redundant with StatusChanged
    TaskEvent::Failed,         // Redundant with StatusChanged
]

// ✅ Efficient
vec![
    TaskEvent::Completed,  // Only when done
    TaskEvent::Failed,     // Only when error
]
```

### 5. Secure Your Endpoint

```rust
use axum::{
    Router,
    middleware,
    routing::post,
};

// Rate limiting middleware
async fn rate_limit(request: Request, next: Next) -> Result<Response, StatusCode> {
    // Implement rate limiting
    // Max 100 requests per minute per IP
    Ok(next.run(request).await)
}

let app = Router::new()
    .route("/webhook", post(handle_webhook))
    .layer(middleware::from_fn(verify_webhook))  // Auth
    .layer(middleware::from_fn(rate_limit));     // Rate limit
```

## Examples

### Example 1: Simple Webhook Receiver

```rust
use axum::{routing::post, Router, Json};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct WebhookPayload {
    event: String,
    task: TaskInfo,
    timestamp: String,
    #[serde(rename = "agentId")]
    agent_id: String,
}

#[derive(Debug, Deserialize)]
struct TaskInfo {
    id: String,
    status: TaskStatus,
}

#[derive(Debug, Deserialize)]
struct TaskStatus {
    state: String,
}

async fn handle_webhook(Json(payload): Json<WebhookPayload>) -> &'static str {
    println!("📨 Webhook: {} for task {}", payload.event, payload.task.id);
    println!("   Status: {}", payload.task.status.state);
    println!("   Agent: {}", payload.agent_id);
    println!("   Time: {}", payload.timestamp);
    
    // Your logic here
    match payload.event.as_str() {
        "completed" => println!("✅ Task completed!"),
        "failed" => println!("❌ Task failed!"),
        "cancelled" => println!("🚫 Task cancelled!"),
        _ => println!("ℹ️  Other event: {}", payload.event),
    }
    
    "OK"
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/webhook", post(handle_webhook));
    
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
        .await
        .unwrap();
    
    println!("🎣 Listening on http://127.0.0.1:8080/webhook");
    axum::serve(listener, app).await.unwrap();
}
```

### Example 2: Configure Webhook via RPC

```rust
use a2a_protocol::{
    prelude::*,
    client::ClientBuilder,
};
use url::Url;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = ClientBuilder::new()
        .with_json_rpc("http://localhost:3000/rpc")
        .build()?;
    
    // Create a task
    let message = Message::user_text("Long running task");
    let response = client.send_message(message).await?;
    
    let task_id = match response {
        SendResponse::Task(task) => task.id,
        _ => panic!("Expected task"),
    };
    
    // Configure webhook using JSON-RPC
    let config = PushNotificationConfig::new(
        Url::parse("https://myapp.com/webhook")?,
        vec![TaskEvent::Completed, TaskEvent::Failed],
        Some(PushNotificationAuth::Bearer {
            token: std::env::var("WEBHOOK_TOKEN")?,
        }),
    );
    
    // Note: This requires using the raw JSON-RPC transport
    // See examples/webhook_server.rs for full example
    
    println!("✅ Webhook configured for task: {}", task_id);
    println!("🎣 Will receive notification at: https://myapp.com/webhook");
    
    Ok(())
}
```

### Example 3: Complete Integration

See the full working example in `examples/webhook_server.rs`:

```bash
cargo run --example webhook_server --features streaming
```

This example shows:
- A2A server with webhook support
- Complete webhook configuration
- Testing with curl commands
- Proper error handling

## Next Steps

- **Try the example:** Run `cargo run --example webhook_server --features streaming`
- **Set up ngrok:** For testing webhooks locally
- **Read the spec:** [A2A Push Notifications](https://a2a-protocol.org/)
- **Build your receiver:** Use the examples above as templates

Need help? Check the [main documentation](README.md) or [open an issue](https://github.com/your-repo/issues).

Happy webhook-ing! 🎣
