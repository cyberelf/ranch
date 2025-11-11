# A2A Protocol Features

Complete overview of all features in the `a2a-protocol` Rust implementation (v0.7.0).

## Quick Reference

| Feature | Version | Use Case | Learn More |
|---------|---------|----------|------------|
| **Basic Messaging** | v0.1+ | Send/receive messages | [Getting Started](GETTING_STARTED.md#your-first-server) |
| **Task Management** | v0.1+ | Long-running operations | [Getting Started](GETTING_STARTED.md#working-with-tasks) |
| **Server Builder** | v0.6.0 | Quick server setup | [Examples](examples/simple_server.rs) |
| **SSE Streaming** | v0.6.0 | Real-time updates | [Getting Started](GETTING_STARTED.md#streaming-responses-v060) |
| **Push Notifications** | v0.7.0 | Async task completion | [Webhooks Guide](WEBHOOKS.md) |
| **SSRF Protection** | v0.7.0 | Webhook security | [Webhooks Guide](WEBHOOKS.md#security) |

## Core Features

### 1. Basic Messaging

Send and receive messages between agents using the A2A protocol.

**What it does:**
- Synchronous request/response communication
- Support for text, files, and structured data
- JSON-RPC 2.0 compliant transport

**Example:**
```rust
let message = Message::user_text("Hello, agent!");
let response = client.send_message(message).await?;
```

**When to use:**
- Quick queries or commands
- When you need immediate responses
- Simple agent interactions

**Learn more:** [Getting Started - Your First Client](GETTING_STARTED.md#your-first-client)

---

### 2. Task Management

Handle long-running operations asynchronously.

**What it does:**
- Create tasks for operations that take time
- Track task status (pending, working, completed, failed)
- Retrieve results when ready
- Cancel running tasks

**Example:**
```rust
// Server creates task automatically
let task = process_heavy_computation().await;

// Client polls for completion
let status = client.get_task_status(&task.id).await?;
```

**When to use:**
- Operations taking > 1 second
- Resource-intensive computations
- Batch processing
- Background jobs

**Task States:**
- `Pending` - Task created, not started
- `Working` - Task in progress
- `Completed` - Task finished successfully
- `Failed` - Task encountered an error
- `Cancelled` - Task was cancelled

**Learn more:** [Getting Started - Working with Tasks](GETTING_STARTED.md#working-with-tasks)

---

### 3. Server Builder (v0.6.0)

Create production-ready servers in one line.

**What it does:**
- Simplified server setup
- Automatic JSON-RPC routing
- Built-in error handling
- Configurable ports and middleware

**Example:**
```rust
ServerBuilder::new(handler)
    .with_port(3000)
    .run()
    .await?;
```

**Features:**
- Auto port binding
- Graceful shutdown
- Request logging
- CORS support

**When to use:**
- Starting new projects
- Rapid prototyping
- Production deployments

**Learn more:** [Examples - Simple Server](examples/simple_server.rs)

---

### 4. SSE Streaming (v0.6.0)

Real-time updates using Server-Sent Events.

**What it does:**
- Stream task progress updates
- Receive artifacts as they're created
- Live status changes
- Reconnection support

**Example:**
```rust
let mut stream = client.stream_message(message).await?;
while let Some(event) = stream.next().await {
    match event? {
        StreamingResult::TaskStatusUpdate(update) => {
            println!("Progress: {:?}", update.status);
        }
        _ => {}
    }
}
```

**Event Types:**
- `Message` - Immediate message response
- `Task` - Task created
- `TaskStatusUpdate` - Status changed
- `TaskArtifactUpdate` - New artifact available

**When to use:**
- Long-running tasks with progress
- Real-time data feeds
- Chat-like interactions
- Live monitoring

**Benefits vs Polling:**
- Lower latency
- Reduced server load
- Real-time updates
- Automatic reconnection

**Learn more:** [Getting Started - Streaming](GETTING_STARTED.md#streaming-responses-v060)

---

### 5. Push Notifications / Webhooks (v0.7.0)

Get notified when tasks complete instead of polling.

**What it does:**
- Configure webhook URLs for tasks
- Receive HTTP POST when events occur
- Automatic retries on failure
- Multiple authentication methods

**Example:**
```bash
# Configure webhook
curl -X POST http://server/rpc -d '{
  "method": "tasks/pushNotificationConfig/set",
  "params": {
    "taskId": "task-123",
    "config": {
      "url": "https://myapp.com/webhook",
      "events": ["completed", "failed"]
    }
  }
}'

# Receive notification
POST https://myapp.com/webhook
{
  "event": "completed",
  "task": {...},
  "timestamp": "2025-11-11T10:30:00Z"
}
```

**Supported Events:**
- `completed` - Task finished successfully
- `failed` - Task encountered error
- `cancelled` - Task was cancelled
- `statusChanged` - Any status transition
- `artifactAdded` - New artifact created

**Authentication:**
- Bearer tokens
- Custom HTTP headers
- (OAuth2 planned for v0.8.0)

**When to use:**
- Eliminating polling overhead
- Integrating with external systems
- Building event-driven workflows
- Scalable task monitoring

**Benefits vs Streaming:**
- No persistent connection needed
- Survive client restarts
- Lower resource usage
- Better for batch jobs

**Learn more:** [Webhooks Guide](WEBHOOKS.md)

---

### 6. SSRF Protection (v0.7.0)

Security layer preventing webhook abuse.

**What it does:**
- Validates webhook URLs before use
- Blocks private IP ranges
- Prevents internal network access
- Enforces HTTPS requirement

**Protections:**
- ✅ Requires HTTPS for all webhooks
- ✅ Blocks RFC1918 private IPs (10.x, 192.168.x, 172.16-31.x)
- ✅ Blocks localhost (127.x, ::1)
- ✅ Blocks link-local (169.254.x.x, fe80::/10)
- ✅ Blocks AWS metadata (169.254.169.254)
- ✅ Blocks .local and .internal domains

**Example:**
```rust
// ✅ Allowed
"https://myapp.com/webhook"
"https://api.example.com/notify"

// ❌ Blocked
"http://example.com"           // HTTP not allowed
"https://192.168.1.1"          // Private IP
"https://localhost"            // Localhost
"https://169.254.169.254"      // AWS metadata
"https://internal.local"       // .local domain
```

**When it matters:**
- Multi-tenant systems
- Public-facing agents
- Production deployments
- Security compliance

**Learn more:** [Webhooks Guide - Security](WEBHOOKS.md#security)

---

## Developer Experience Features

### AgentLogic Trait

Simplified agent development - just implement message processing.

**Benefits:**
- Single method to implement: `process_message()`
- Automatic task management
- Clean, readable code
- Perfect for 90% of use cases

```rust
#[async_trait]
impl AgentLogic for MyAgent {
    async fn process_message(&self, message: Message) -> A2aResult<Message> {
        // Your logic here
    }
}
```

**Learn more:** [Getting Started - Understanding AgentLogic](GETTING_STARTED.md#understanding-agentlogic-vs-a2ahandler)

---

### Client Builder

Fluent API for client configuration.

**Features:**
- Chainable configuration
- Multiple transport options
- Timeout control
- Retry configuration
- Authentication support

```rust
let client = ClientBuilder::new()
    .with_json_rpc("http://localhost:3000/rpc")
    .with_timeout(30)
    .with_max_retries(3)
    .build()?;
```

---

### Comprehensive Examples

8 runnable examples covering common scenarios:

1. **basic_echo_server** - Minimal server (start here!)
2. **echo_client** - Simple client
3. **simple_server** - One-line server
4. **streaming_server** - SSE streaming
5. **streaming_client** - Consuming streams
6. **task_server** - Long-running tasks
7. **webhook_server** - Push notifications
8. **multi_agent** - Agent-to-agent communication

```bash
cargo run --example basic_echo_server --features streaming
```

---

## Comparison Matrix

### Messaging vs Tasks vs Streaming vs Webhooks

| Feature | Latency | Server Load | Client Complexity | Best For |
|---------|---------|-------------|-------------------|----------|
| **Immediate Messages** | Lowest | Low | Simple | Quick queries |
| **Tasks (polling)** | High | Medium | Medium | Background jobs |
| **SSE Streaming** | Low | Medium | Medium | Real-time updates |
| **Webhooks** | Medium | Lowest | Low | Event-driven |

### When to Use What?

```
┌─────────────────────────┬──────────────────────────┐
│     Quick Response      │   Long Processing        │
│     (< 1 second)        │   (> 1 second)           │
├─────────────────────────┼──────────────────────────┤
│                         │                          │
│  Immediate Messaging    │    Tasks                 │
│  ✓ send_message()       │    ✓ Async processing    │
│  ✓ Low latency          │    ✓ Background work     │
│                         │                          │
└─────────────────────────┴──────────────────────────┘

┌─────────────────────────┬──────────────────────────┐
│   Need Real-Time        │   Event-Driven           │
│   Updates?              │   Architecture?          │
├─────────────────────────┼──────────────────────────┤
│                         │                          │
│  SSE Streaming          │    Webhooks              │
│  ✓ Live progress        │    ✓ No polling          │
│  ✓ Persistent conn      │    ✓ Scalable            │
│                         │                          │
└─────────────────────────┴──────────────────────────┘
```

**Decision Tree:**

```
Is response immediate (< 1s)?
├─ Yes → Use immediate messaging
└─ No → Is client online waiting?
    ├─ Yes → Use SSE streaming
    └─ No → Use webhooks + tasks
```

---

## Protocol Compliance

### A2A v0.3.0 Specification

✅ **Fully Implemented:**
- JSON-RPC 2.0 transport
- All required RPC methods
- Message format with parts (Text, File, Data)
- Task lifecycle management
- Agent card discovery
- SSE streaming
- Push notifications (v0.7.0)

🚧 **Planned:**
- gRPC transport (optional)
- HTTP+JSON/REST (if spec clarifies)
- OAuth2 webhook auth (v0.8.0)
- Rate limiting (v0.8.0)

### RPC Methods

| Method | Description | Implemented |
|--------|-------------|-------------|
| `message/send` | Send message to agent | ✅ |
| `message/stream` | Stream message response | ✅ v0.6.0 |
| `task/get` | Get task details | ✅ |
| `task/status` | Get task status | ✅ |
| `task/cancel` | Cancel running task | ✅ |
| `task/resubscribe` | Resubscribe to task stream | ✅ v0.6.0 |
| `agent/card` | Get agent capabilities | ✅ |
| `tasks/pushNotificationConfig/set` | Configure webhook | ✅ v0.7.0 |
| `tasks/pushNotificationConfig/get` | Get webhook config | ✅ v0.7.0 |
| `tasks/pushNotificationConfig/list` | List webhooks | ✅ v0.7.0 |
| `tasks/pushNotificationConfig/delete` | Delete webhook | ✅ v0.7.0 |

---

## Version History

### v0.7.0 (November 2025)
- ✨ **Push Notifications / Webhooks**
- 🔒 **SSRF Protection**
- 📊 **223 tests passing**

### v0.6.0 (October 2025)
- ✨ **SSE Streaming**
- 🚀 **ServerBuilder**
- 💡 **AgentLogic Trait**
- 📚 **8 Examples**

### v0.5.0
- Enhanced AgentCard metadata
- A2A-specific error codes

### v0.4.0
- JSON-RPC 2.0 compliance
- Task-aware handler

### v0.1.0
- Initial release
- Basic messaging
- Task management

---

## Next Steps

**New to A2A?**
1. Start with [Getting Started Guide](GETTING_STARTED.md)
2. Run the [basic examples](examples/)
3. Build your first agent

**Ready for production?**
1. Review [Webhooks Guide](WEBHOOKS.md) for scalability
2. Implement [SSRF protection](WEBHOOKS.md#security)
3. Add monitoring and metrics

**Need help?**
- 📖 [API Documentation](https://docs.rs/a2a-protocol)
- 💬 [GitHub Issues](https://github.com/your-repo/issues)
- 📋 [A2A Specification](https://a2a-protocol.org/)
