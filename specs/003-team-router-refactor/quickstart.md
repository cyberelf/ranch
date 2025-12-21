# Quickstart: Dynamic Team Routing with Client Agent Extension

## Creating a Team with Router

```rust
use multi_agent::{Team, TeamConfig, RouterConfig};

let config = TeamConfig {
    id: "my-team".to_string(),
    name: "My Team".to_string(),
    description: "A dynamic team with client routing".to_string(),
    agents: vec![...],
    router_config: RouterConfig {
        default_agent_id: "manager".to_string(),
        max_routing_hops: Some(10),  // Prevent infinite loops
    },
};

let team = Team::new(config, agent_manager);
```

## Declaring Extension Support in Agent Card

To enable client routing for an agent, declare the extension in its Agent Card:

```rust
use a2a_protocol::core::{AgentCard, AgentCapabilities, AgentExtension};

let agent_card = AgentCard {
    name: "Manager Agent".to_string(),
    // ... other fields
    capabilities: AgentCapabilities {
        streaming: false,
        push_notifications: false,
        extensions: vec![
            AgentExtension {
                uri: "https://ranch.woi.dev/extensions/client-routing/v1".to_string(),
                description: "Can receive peer list and make routing decisions".to_string(),
                required: false,
                params: None,
            }
        ],
    },
    // ... other fields
};
```

## Implementing a Client-Routing Agent

### 1. Handle Incoming Extension Data

```rust
use multi_agent::team::{ClientRoutingRequest, SimplifiedAgentCard, EXTENSION_URI};

async fn process(&self, message: Message) -> A2aResult<Message> {
    // Check if extension data present
    if let Some(extensions) = &message.extensions {
        if extensions.contains(&EXTENSION_URI.to_string()) {
            if let Some(metadata) = &message.metadata {
                if let Some(ext_data) = metadata.get(EXTENSION_URI) {
                    // Parse extension data
                    let request: ClientRoutingRequest = serde_json::from_value(ext_data.clone())?;
                    
                    // Access peer agents
                    for card in &request.agent_cards {
                        println!("Available agent: {} - {}", card.id, card.description);
                        if card.supports_client_routing {
                            println!("  (supports routing)");
                        }
                    }
                    
                    // Track sender
                    let sender = &request.sender;  // "user" or agent ID
                }
            }
        }
    }
    
    // Process message...
    // Decide routing...
}
```

### 2. Return Routing Decision

```rust
use multi_agent::team::{ClientRoutingResponse, EXTENSION_URI};

// Create response message
let mut response = Message::agent_text("Routing to researcher for web search...");

// Add extension declaration
response.extensions = Some(vec![EXTENSION_URI.to_string()]);

// Add routing decision to metadata
let routing = ClientRoutingResponse {
    recipient: "researcher".to_string(),
    reason: Some("Query requires web search capability".to_string()),
};

let mut metadata = HashMap::new();
metadata.insert(
    EXTENSION_URI.to_string(),
    serde_json::to_value(&routing)?
);
response.metadata = Some(metadata);

Ok(response)
```

## Routing Options

### Route to Specific Agent

```rust
ClientRoutingResponse {
    recipient: "researcher".to_string(),
    reason: Some("Query requires search".to_string()),
}
```

### Return to User

```rust
ClientRoutingResponse {
    recipient: "user".to_string(),
    reason: Some("Task complete".to_string()),
}
```

### Route Back to Sender

```rust
ClientRoutingResponse {
    recipient: "sender".to_string(),
    reason: Some("Requesting clarification".to_string()),
}
```

The Router will resolve "sender" to the actual agent or user who sent the message.

## Client Opt-In (HTTP Example)

When calling an agent that supports the extension, clients declare usage via headers:

```http
POST /v1/message:send HTTP/1.1
Host: agent.example.com
Content-Type: application/json
Authorization: Bearer token
A2A-Extensions: https://ranch.woi.dev/extensions/client-routing/v1

{
  "message": {
    "role": "user",
    "parts": [{"text": "Find restaurants near me"}],
    "extensions": ["https://ranch.woi.dev/extensions/client-routing/v1"],
    "metadata": {
      "https://ranch.woi.dev/extensions/client-routing/v1": {
        "agentCards": [...],
        "sender": "user"
      }
    }
  }
}
```

## Backward Compatibility

Agents without extension support receive regular messages:

```rust
// Without extension support
let message = Message::user_text("Hello");
// No extensions field
// No extension metadata
```

The Router automatically detects support and only injects extension data when the target agent declares it.
