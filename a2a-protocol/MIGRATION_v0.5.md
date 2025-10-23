# Migration Guide: v0.4.x → v0.5.0

This guide helps you migrate from `a2a-protocol` v0.4.x to v0.5.0, which adds enhanced metadata fields to `AgentCard` and implements A2A-specific JSON-RPC error codes with structured data.

## Overview of Changes

**v0.5.0** enhances agent discovery with optional metadata and improves error handling with A2A-specific error codes.

### What's New

1. ✅ **Enhanced AgentCard** - Optional provider, icon, documentation, and signature fields
2. ✅ **A2A Error Codes** - Seven new error codes (-32001 through -32007) with structured data
3. ✅ **Better Error Context** - Errors include relevant data (taskId, state, contentType)

### Breaking Changes

1. ❌ **AgentCard.protocols** - Removed deprecated field
2. ❌ **ProtocolSupport** - Removed deprecated struct
3. ❌ **with_protocol()** - Removed deprecated helper method
4. ❌ **supports_protocol()** - Removed deprecated helper method
5. ❌ **protocol_endpoints()** - Removed deprecated helper method

---

## AgentCard Enhancements

### New Optional Fields

The `AgentCard` struct now supports optional fields for richer agent metadata:

```rust
pub struct AgentCard {
    // Existing required fields
    pub agent_id: AgentId,
    pub name: String,
    pub version: String,
    pub url: Url,
    pub description: Option<String>,
    pub transports: Vec<TransportInterface>,
    pub capabilities: AgentCapabilities,
    
    // NEW v0.5.0: Optional metadata fields
    pub provider: Option<AgentProvider>,        // Provider info
    pub icon_url: Option<Url>,                  // Agent icon/avatar
    pub documentation_url: Option<Url>,         // Documentation link
    pub signatures: Vec<AgentCardSignature>,    // Cryptographic signatures
}
```

### Migration: Adding Metadata

#### Before (v0.4.x)
```rust
use a2a_protocol::core::{AgentCard, AgentId};
use url::Url;

let card = AgentCard::builder("my-agent", "1.0.0")
    .with_name("My AI Agent")
    .with_url(Url::parse("https://agent.example.com")?)
    .with_description("A helpful assistant")
    .build();
```

#### After (v0.5.0) - With New Fields
```rust
use a2a_protocol::core::{AgentCard, AgentId, AgentProvider, AgentCardSignature};
use url::Url;

let card = AgentCard::builder("my-agent", "1.0.0")
    .with_name("My AI Agent")
    .with_url(Url::parse("https://agent.example.com")?)
    .with_description("A helpful assistant")
    
    // NEW: Provider information
    .with_provider(AgentProvider {
        name: "ACME Corporation".to_string(),
        url: Some(Url::parse("https://acme.com")?),
    })
    
    // NEW: Icon URL
    .with_icon_url(Url::parse("https://acme.com/assets/agent-icon.png")?)
    
    // NEW: Documentation URL
    .with_documentation_url(Url::parse("https://docs.acme.com/agents/my-agent")?)
    
    // NEW: Add cryptographic signatures
    .add_signature(AgentCardSignature {
        algorithm: "ed25519".to_string(),
        signature: "base64_encoded_signature".to_string(),
        public_key: "base64_encoded_public_key".to_string(),
        signed_at: chrono::Utc::now(),
    })
    
    .build();
```

**Note:** All new fields are optional. Existing code continues to work without changes.

### Serialization Example

The new fields are included in JSON serialization when present:

```json
{
  "agentId": "my-agent",
  "name": "My AI Agent",
  "version": "1.0.0",
  "url": "https://agent.example.com",
  "description": "A helpful assistant",
  "provider": {
    "name": "ACME Corporation",
    "url": "https://acme.com"
  },
  "iconUrl": "https://acme.com/assets/agent-icon.png",
  "documentationUrl": "https://docs.acme.com/agents/my-agent",
  "signatures": [
    {
      "algorithm": "ed25519",
      "signature": "base64_encoded_signature",
      "publicKey": "base64_encoded_public_key",
      "signedAt": "2025-01-15T10:30:00Z"
    }
  ],
  "transports": [...],
  "capabilities": {...}
}
```

---

## Breaking Change: Removed `protocols` Field

The deprecated `protocols` field and related methods have been removed from `AgentCard`.

### Migration Path

#### Before (v0.4.x) - Using Deprecated Field
```rust
use a2a_protocol::core::{AgentCard, ProtocolSupport};

let card = AgentCard::builder("my-agent", "1.0.0")
    .with_protocol(ProtocolSupport {
        name: "a2a".to_string(),
        version: "0.3.0".to_string(),
        endpoint: Some("https://agent.example.com/rpc".to_string()),
    })
    .build();

// Check protocol support
if card.supports_protocol("a2a", "0.3.0") {
    let endpoints = card.protocol_endpoints("a2a");
}
```

#### After (v0.5.0) - Using TransportInterface
```rust
use a2a_protocol::core::{AgentCard, TransportInterface};

let card = AgentCard::builder("my-agent", "1.0.0")
    .add_transport(TransportInterface {
        protocol: "json-rpc".to_string(),
        version: "2.0".to_string(),
        url: "https://agent.example.com/rpc".parse()?,
        authentication: None,
        features: vec!["tasks".to_string()],
    })
    .build();

// Check transport support
let json_rpc_transport = card.transports.iter()
    .find(|t| t.protocol == "json-rpc" && t.version == "2.0");
```

**Why?** The `protocols` field was redundant with the `transports` field, which provides more structured information about agent connectivity.

---

## A2A-Specific Error Codes

v0.5.0 implements seven A2A-specific JSON-RPC error codes with structured data fields.

### Error Code Reference

| Code | Constant | Description | Data Fields |
|------|----------|-------------|-------------|
| `-32001` | `TASK_NOT_FOUND` | Task ID does not exist | `taskId: string` |
| `-32002` | `TASK_NOT_CANCELABLE` | Task cannot be cancelled in current state | `taskId: string`, `state: string` |
| `-32003` | `PUSH_NOTIFICATION_NOT_SUPPORTED` | Server doesn't support push notifications | - |
| `-32004` | `UNSUPPORTED_OPERATION` | Operation not supported by this agent | - |
| `-32005` | `CONTENT_TYPE_NOT_SUPPORTED` | Content type not accepted | `contentType: string` |
| `-32006` | `INVALID_AGENT_RESPONSE` | Agent returned invalid response | - |
| `-32007` | `AUTHENTICATED_EXTENDED_CARD_NOT_CONFIGURED` | Authentication required but not configured | - |

### Error Response Format

JSON-RPC error responses now include structured data:

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

### Client Error Handling

#### Before (v0.4.x) - Generic Errors
```rust
use a2a_protocol::prelude::*;

match client.get_task("task_123").await {
    Ok(task) => println!("Task: {:?}", task),
    Err(e) => eprintln!("Error: {}", e), // Generic error message
}
```

#### After (v0.5.0) - Structured Error Matching
```rust
use a2a_protocol::prelude::*;
use a2a_protocol::core::TaskState;

match client.get_task("task_123").await {
    Ok(task) => println!("Task: {:?}", task),
    
    // NEW: Specific error with data
    Err(A2aError::TaskNotFound { task_id }) => {
        eprintln!("Task {} does not exist", task_id);
        // Handle task not found (maybe retry or show user message)
    }
    
    Err(A2aError::TaskNotCancelable { task_id, state }) => {
        eprintln!("Cannot cancel task {} in state {:?}", task_id, state);
        // Handle non-cancelable state (maybe show why to user)
    }
    
    Err(A2aError::ContentTypeNotSupported { content_type }) => {
        eprintln!("Content type {} not supported", content_type);
        // Handle content type issue (convert or reject)
    }
    
    Err(e) => eprintln!("Other error: {}", e),
}
```

### Server Error Handling

Errors automatically map to appropriate error codes:

#### Before (v0.4.x)
```rust
use a2a_protocol::core::A2aError;

async fn get_task(&self, task_id: &str) -> Result<Task, A2aError> {
    self.store
        .get(task_id)
        .ok_or_else(|| A2aError::Other("Task not found".to_string()))
        // ^ Generic error, no structured data
}
```

#### After (v0.5.0)
```rust
use a2a_protocol::core::A2aError;

async fn get_task(&self, task_id: &str) -> Result<Task, A2aError> {
    self.store
        .get(task_id)
        .ok_or_else(|| A2aError::TaskNotFound {
            task_id: task_id.to_string()
        })
        // ^ Specific error with taskId in data field
}
```

The error automatically maps to:
```json
{
  "code": -32001,
  "message": "Task not found: task_abc123",
  "data": {"taskId": "task_abc123"}
}
```

---

## Error Code Examples

### TaskNotFound (-32001)

```rust
// Server-side
Err(A2aError::TaskNotFound {
    task_id: "task_xyz".to_string()
})

// JSON-RPC response
{
  "error": {
    "code": -32001,
    "message": "Task not found: task_xyz",
    "data": {"taskId": "task_xyz"}
  }
}

// Client-side handling
match client.cancel_task("task_xyz").await {
    Err(A2aError::TaskNotFound { task_id }) => {
        log::warn!("Task {} doesn't exist, nothing to cancel", task_id);
        // Maybe clean up local state
    }
    Ok(status) => log::info!("Task cancelled: {:?}", status),
    Err(e) => log::error!("Cancellation failed: {}", e),
}
```

### TaskNotCancelable (-32002)

```rust
// Server-side
Err(A2aError::TaskNotCancelable {
    task_id: "task_abc".to_string(),
    state: TaskState::Completed,
})

// JSON-RPC response
{
  "error": {
    "code": -32002,
    "message": "Task cannot be canceled: task_abc is in state Completed",
    "data": {
      "taskId": "task_abc",
      "state": "Completed"
    }
  }
}

// Client-side handling
match client.cancel_task("task_abc").await {
    Err(A2aError::TaskNotCancelable { task_id, state }) => {
        println!("Task {} is already {:?}, can't cancel", task_id, state);
        // Show user why cancellation failed
    }
    Ok(status) => println!("Cancelled: {:?}", status),
    Err(e) => eprintln!("Error: {}", e),
}
```

### ContentTypeNotSupported (-32005)

```rust
// Server-side
if !supported_types.contains(&content_type) {
    return Err(A2aError::ContentTypeNotSupported {
        content_type: content_type.to_string()
    });
}

// JSON-RPC response
{
  "error": {
    "code": -32005,
    "message": "Content type not supported: image/webp",
    "data": {"contentType": "image/webp"}
  }
}

// Client-side handling
match client.send_message(message).await {
    Err(A2aError::ContentTypeNotSupported { content_type }) => {
        eprintln!("Agent doesn't support {}", content_type);
        // Convert to supported format or reject
    }
    Ok(response) => handle_response(response),
    Err(e) => eprintln!("Error: {}", e),
}
```

### PushNotificationNotSupported (-32003)

```rust
// Server-side
if callback_url.is_some() && !self.supports_callbacks {
    return Err(A2aError::PushNotificationNotSupported);
}

// JSON-RPC response
{
  "error": {
    "code": -32003,
    "message": "Push notifications are not supported by this agent"
  }
}

// Client-side handling
let request = MessageSendRequest {
    message,
    immediate: false,
    callback_url: Some("https://client.com/callback".to_string()),
};

match client.send_message_request(request).await {
    Err(A2aError::PushNotificationNotSupported) => {
        log::warn!("Agent doesn't support callbacks, using polling instead");
        // Fall back to polling
        let task = client.send_message_immediate(message).await?;
        poll_until_complete(&client, &task.id).await?;
    }
    Ok(response) => handle_response(response),
    Err(e) => log::error!("Error: {}", e),
}
```

---

## Testing Error Handling

### Unit Tests

```rust
use a2a_protocol::core::{A2aError, TaskState};
use a2a_protocol::transport::json_rpc::map_error_to_rpc;

#[test]
fn test_error_code_mapping() {
    let error = A2aError::TaskNotFound {
        task_id: "test_123".to_string()
    };
    
    let rpc_error = map_error_to_rpc(error);
    
    assert_eq!(rpc_error.code, -32001);
    assert!(rpc_error.message.contains("test_123"));
    
    let data = rpc_error.data.unwrap();
    assert_eq!(data["taskId"], "test_123");
}

#[test]
fn test_task_not_cancelable_error() {
    let error = A2aError::TaskNotCancelable {
        task_id: "task_xyz".to_string(),
        state: TaskState::Completed,
    };
    
    let rpc_error = map_error_to_rpc(error);
    
    assert_eq!(rpc_error.code, -32002);
    
    let data = rpc_error.data.unwrap();
    assert_eq!(data["taskId"], "task_xyz");
    assert_eq!(data["state"], "Completed");
}
```

### Integration Tests

```rust
use a2a_protocol::client::ClientBuilder;
use a2a_protocol::core::A2aError;

#[tokio::test]
async fn test_task_not_found_handling() {
    let client = ClientBuilder::new()
        .with_json_rpc("http://localhost:3000/rpc")
        .build()
        .unwrap();
    
    match client.get_task("nonexistent_task").await {
        Err(A2aError::TaskNotFound { task_id }) => {
            assert_eq!(task_id, "nonexistent_task");
        }
        Ok(_) => panic!("Expected TaskNotFound error"),
        Err(e) => panic!("Unexpected error: {}", e),
    }
}
```

---

## Compatibility Notes

### Backward Compatibility

- ✅ **New AgentCard fields are optional** - Existing code works unchanged
- ✅ **Error codes are additive** - Generic errors still work
- ❌ **Removed `protocols` field** - Must migrate to `transports`

### Forward Compatibility

Clients should handle unknown error codes gracefully:

```rust
use a2a_protocol::transport::json_rpc::JsonRpcError;

match client.send_message(message).await {
    Ok(response) => handle_response(response),
    Err(A2aError::RpcError(JsonRpcError { code, message, data })) => {
        match code {
            -32001..=-32007 => {
                // Handle known A2A error codes
                log::error!("A2A error {}: {}", code, message);
            }
            _ => {
                // Handle unknown/future error codes
                log::error!("Unknown error {}: {}", code, message);
            }
        }
    }
    Err(e) => log::error!("Other error: {}", e),
}
```

---

## Migration Checklist

### Required Changes

- [ ] Remove usage of `AgentCard.protocols` field
- [ ] Replace `with_protocol()` with `add_transport()`
- [ ] Replace `supports_protocol()` with transport iteration
- [ ] Replace `protocol_endpoints()` with transport URL access

### Optional Enhancements

- [ ] Add `provider` information to AgentCard
- [ ] Add `icon_url` for UI display
- [ ] Add `documentation_url` for user help
- [ ] Add `signatures` for verification
- [ ] Update error handling to match specific error types
- [ ] Add error data field access in client code
- [ ] Update tests to verify error codes and data

### Testing

```bash
# Update dependencies
cargo update

# Run full test suite
cargo test

# Check for deprecation warnings
cargo build --all-features 2>&1 | grep -i deprecat

# Test error handling
cargo test error

# Run compliance tests
cargo test --test compliance
```

---

## Benefits of v0.5.0

1. **✅ Richer Agent Metadata** - Better discovery and user experience
2. **✅ Structured Error Handling** - Easier debugging and better UX
3. **✅ Spec Compliance** - Follows A2A error code conventions
4. **✅ Type Safety** - Errors carry typed data fields
5. **✅ Better Interoperability** - Standard error codes across agents

---

## Need Help?

- **Issues:** Report migration problems at [GitHub Issues](https://github.com/your-org/a2a-protocol/issues)
- **Examples:** See `examples/` directory for complete working examples
- **Spec:** Read the [A2A v0.3.0 specification](https://github.com/a2aproject/A2A)
- **API Docs:** Run `cargo doc --open` for detailed documentation

---

## Quick Reference

### Import Changes
```rust
// Removed
use a2a_protocol::core::ProtocolSupport;
agent_card.with_protocol(...)
agent_card.supports_protocol(...)
agent_card.protocol_endpoints(...)

// Use instead
use a2a_protocol::core::TransportInterface;
agent_card.add_transport(...)
agent_card.transports.iter().find(|t| ...)
agent_card.transports[0].url
```

### Error Handling Pattern
```rust
// Old style
Err(e) => eprintln!("Error: {}", e)

// New style
Err(A2aError::TaskNotFound { task_id }) => {
    eprintln!("Task {} not found", task_id);
}
Err(A2aError::TaskNotCancelable { task_id, state }) => {
    eprintln!("Cannot cancel {} in state {:?}", task_id, state);
}
```

---

**Version:** v0.5.0  
**Date:** January 2025  
**Spec:** A2A Protocol v0.3.0
