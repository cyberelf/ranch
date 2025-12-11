# TeamServer HTTP/JSON-RPC API Contract

**Purpose**: Define the HTTP server that exposes Team as an A2A protocol service

## Server Structure

```rust
pub struct TeamServer {
    team: Arc<Team>,
    port: u16,
}

impl TeamServer {
    pub fn new(team: Arc<Team>, port: u16) -> Self;
    pub async fn start(self) -> Result<(), Box<dyn std::error::Error>>;
}
```

## HTTP Endpoints

### POST /rpc

**Purpose**: JSON-RPC 2.0 endpoint for all A2A protocol methods

**Content-Type**: `application/json`

**Request Format** (JSON-RPC 2.0):
```json
{
  "jsonrpc": "2.0",
  "method": "<method_name>",
  "params": { /* method-specific parameters */ },
  "id": 1
}
```

**Response Format** (Success):
```json
{
  "jsonrpc": "2.0",
  "result": { /* method-specific result */ },
  "id": 1
}
```

**Response Format** (Error):
```json
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32001,
    "message": "Error description",
    "data": { /* optional additional error info */ }
  },
  "id": 1
}
```

## Supported JSON-RPC Methods

### 1. `message/send`

**Purpose**: Send a message to the team for processing

**Parameters**:
```json
{
  "message": {
    "role": "user",
    "parts": [
      {
        "type": "text",
        "text": "Write a fantasy story about dragons"
      }
    ]
  }
}
```

**Success Response** (Message):
```json
{
  "message": {
    "role": "agent",
    "parts": [
      {
        "type": "text",
        "text": "Once upon a time, in a realm of dragons..."
      }
    ]
  }
}
```

**Success Response** (Task - for async processing):
```json
{
  "task": {
    "id": "task-123e4567-e89b-12d3-a456-426614174000",
    "status": {
      "state": "queued",
      "updated_at": "2025-12-11T10:30:00Z"
    }
  }
}
```

**Behavior**:
1. Validate message format
2. Call team.process(message)
3. If processing completes quickly (<5s), return Message directly
4. If processing takes longer, create Task and return Task ID
5. Client can poll task status via `task/get`

**Error Codes**:
- `-32001`: Invalid message format
- `-32002`: Processing error
- `-32003`: Agent unavailable

---

### 2. `task/get`

**Purpose**: Get complete task information including result if completed

**Parameters**:
```json
{
  "task_id": "task-123e4567-e89b-12d3-a456-426614174000"
}
```

**Success Response** (Completed):
```json
{
  "id": "task-123e4567-e89b-12d3-a456-426614174000",
  "status": {
    "state": "completed",
    "updated_at": "2025-12-11T10:35:00Z"
  },
  "result": {
    "role": "agent",
    "parts": [
      {
        "type": "text",
        "text": "Once upon a time..."
      }
    ]
  }
}
```

**Success Response** (In Progress):
```json
{
  "id": "task-123e4567-e89b-12d3-a456-426614174000",
  "status": {
    "state": "working",
    "updated_at": "2025-12-11T10:32:00Z"
  }
}
```

**Error Codes**:
- `-32004`: Task not found

---

### 3. `task/status`

**Purpose**: Get task status without full result (lighter weight than task/get)

**Parameters**:
```json
{
  "task_id": "task-123e4567-e89b-12d3-a456-426614174000"
}
```

**Success Response**:
```json
{
  "state": "working",
  "updated_at": "2025-12-11T10:32:00Z",
  "reason": null
}
```

**Possible States**:
- `"queued"` - Task created, not yet started
- `"working"` - Task in progress
- `"completed"` - Task finished successfully
- `"failed"` - Task failed with error
- `"cancelled"` - Task was cancelled

**Error Codes**:
- `-32004`: Task not found

---

### 4. `task/cancel`

**Purpose**: Request cancellation of a running task

**Parameters**:
```json
{
  "task_id": "task-123e4567-e89b-12d3-a456-426614174000"
}
```

**Success Response**:
```json
{
  "cancelled": true
}
```

**Behavior**:
- If task is `queued` or `working`, mark as `cancelled`
- If task is already `completed` or `failed`, return `cancelled: false`
- Cancellation is best-effort (may not stop immediately)

**Error Codes**:
- `-32004`: Task not found

---

### 5. `agent/card`

**Purpose**: Get agent discovery card (team's capabilities and metadata)

**Parameters**: None (empty object `{}`)

**Success Response**:
```json
{
  "agent_id": "fantasy-writing-team",
  "name": "Fantasy Story Writing Team",
  "description": "Collaborative team for fantasy story creation",
  "capabilities": [
    {
      "name": "world-building",
      "description": "Create rich fantasy world settings",
      "version": "1.0.0"
    },
    {
      "name": "character-development",
      "description": "Develop compelling characters",
      "version": "1.0.0"
    },
    {
      "name": "plot-generation",
      "description": "Generate engaging plot structures",
      "version": "1.0.0"
    },
    {
      "name": "prose-writing",
      "description": "Write polished narrative prose",
      "version": "1.0.0"
    }
  ],
  "skills": [
    {
      "name": "creative-writing",
      "proficiency": "expert"
    }
  ],
  "version": "2.0.0",
  "provider": {
    "name": "RANCH Multi-Agent Framework",
    "url": "https://github.com/example/ranch"
  }
}
```

**Behavior**:
1. Call team.info() to get AgentInfo
2. Convert AgentInfo to AgentCard format
3. Map capabilities list to Capability structs
4. Add team-specific metadata as skills
5. Include framework version and provider info

**Error Codes**: None (always succeeds if server is running)

## CORS Support

**Headers** (permissive for development):
- `Access-Control-Allow-Origin: *`
- `Access-Control-Allow-Methods: POST, OPTIONS`
- `Access-Control-Allow-Headers: Content-Type`

**Production**: Configure restrictive CORS based on deployment needs

## Implementation Details

### Server Lifecycle

```rust
let team = Arc::new(team);
let server = TeamServer::new(team, 3000);

// Start server (blocks until shutdown)
server.start().await?;
```

### Internal Architecture

```
HTTP Request
    ↓
Axum Router (/rpc endpoint)
    ↓
JsonRpcRouter (validate JSON-RPC 2.0, parse method)
    ↓
TaskAwareHandler (implements a2a-protocol Agent trait)
    ↓ delegate to wrapped agent
Team (implements multi-agent Agent trait)
    ↓ process()
Scheduler + Member Agents
    ↓
Response (Message or Task)
    ↓
JsonRpcRouter (format JSON-RPC response)
    ↓
HTTP Response
```

### Code Structure

```rust
impl TeamServer {
    pub async fn start(self) -> Result<(), Box<dyn std::error::Error>> {
        // 1. Wrap Team with TaskAwareHandler
        let handler = TaskAwareHandler::new(self.team.clone() as Arc<dyn Agent>);
        let handler = Arc::new(handler);
        
        // 2. Create JSON-RPC router
        let rpc_router = JsonRpcRouter::new(handler);
        
        // 3. Create Axum app
        let app = Router::new()
            .route("/rpc", post(move |body| {
                let router = rpc_router.clone();
                async move { router.handle(body).await }
            }))
            .layer(tower_http::cors::CorsLayer::permissive());
        
        // 4. Bind and serve
        let addr = std::net::SocketAddr::from(([0, 0, 0, 0], self.port));
        tracing::info!("Starting TeamServer on {}", addr);
        
        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, app).await?;
        
        Ok(())
    }
}
```

## Testing Contract

### Unit Tests
- Test TeamServer construction
- Test handler wrapping
- Test router setup

### Integration Tests
```rust
#[tokio::test]
async fn test_message_send_integration() {
    // Start TeamServer on random port
    let port = find_free_port();
    let server = TeamServer::new(team, port);
    tokio::spawn(async move { server.start().await });
    
    // Wait for server to start
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Create A2A client
    let client = A2aClient::new(/* ... */);
    
    // Send message
    let message = user_message("Test message");
    let response = client.send_message(message).await.unwrap();
    
    // Verify response
    assert_eq!(response.role, MessageRole::Agent);
}
```

## Performance Requirements

- **Throughput**: Handle 100 concurrent requests without degradation
- **Latency**: <500ms p95 latency increase under load
- **Timeout**: Default request timeout 30 seconds
- **Task Polling**: Support polling at 1Hz (1 req/sec) per task
- **Memory**: <100MB overhead for server infrastructure

## Security Considerations

- **Authentication**: Support ApiKeyAuth, BearerAuth, OAuth2 (via a2a-protocol)
- **Rate Limiting**: Implement per-client rate limiting (recommended)
- **Input Validation**: All messages validated before processing
- **Error Hiding**: Don't leak internal paths or stack traces in errors
- **CORS**: Configure restrictively in production

## Error Code Reference

A2A Protocol Standard Error Codes:
- `-32001`: Invalid request format
- `-32002`: Agent processing error
- `-32003`: Agent unavailable
- `-32004`: Task not found
- `-32005`: Task failed
- `-32006`: Task cancelled
- `-32007`: Protocol version mismatch

JSON-RPC 2.0 Standard Error Codes:
- `-32700`: Parse error (invalid JSON)
- `-32600`: Invalid request (invalid JSON-RPC)
- `-32601`: Method not found
- `-32602`: Invalid params
- `-32603`: Internal error

## Example Client Usage

### Python Client
```python
import requests

response = requests.post(
    "http://localhost:3000/rpc",
    json={
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
)

result = response.json()
print(result["result"])
```

### Rust Client
```rust
let transport = JsonRpcTransport::new("http://localhost:3000/rpc")?;
let client = A2aClient::new(Arc::new(transport));

let message = user_message("Hello");
let response = client.send_message(message).await?;
```

### curl
```bash
curl -X POST http://localhost:3000/rpc \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "agent/card",
    "params": {},
    "id": 1
  }'
```
