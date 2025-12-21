# Data Model: Team Router with Client Agent Extension

## Extension Declaration (A2A Spec Compliant)

### AgentExtension
Per A2A spec section 4.6.1, extensions are declared in Agent Cards:

```rust
pub struct AgentExtensionDeclaration {
    pub uri: String,  // "https://ranch.woi.dev/extensions/client-routing/v1"
    pub description: String,
    pub required: bool,  // false for optional
    pub params: Option<serde_json::Value>,
}
```

### Extension URI
```
https://ranch.woi.dev/extensions/client-routing/v1
```

## Core Structures

### Router
The central component replacing `Scheduler`.

```rust
pub struct Router {
    default_agent_id: String,
    sender_stack: Vec<String>,  // Track sender for "back to sender"
}

impl Router {
    pub fn new(default_agent_id: String) -> Self { ... }
    
    pub async fn route(
        &mut self,
        message: &mut Message,
        agents: &HashMap<String, Arc<dyn Agent>>,
        sender: &str,  // "user" or agent ID
    ) -> Result<Recipient, TeamError>;
    
    fn supports_extension(&self, agent_info: &AgentInfo) -> bool {
        agent_info.capabilities.contains(&EXTENSION_URI.to_string())
    }
    
    fn inject_extension_context(&self, message: &mut Message, agents: &[SimplifiedAgentCard]) { ... }
    
    fn extract_recipient(&self, message: &Message) -> Option<String> { ... }
}
```

### Recipient
Destination for a routed message.

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Recipient {
    Agent(String),  // Agent ID
    User,           // Return to caller
}
```

### SimplifiedAgentCard
Lightweight agent info for extension context.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimplifiedAgentCard {
    pub id: String,
    pub name: String,
    pub description: String,
    pub capabilities: Vec<String>,
    #[serde(rename = "supportsClientRouting")]
    pub supports_client_routing: bool,
}
```

### TeamConfig
Updated configuration.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamConfig {
    pub id: String,
    pub name: String,
    pub description: String,
    pub agents: Vec<TeamAgentConfig>,
    pub router_config: RouterConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouterConfig {
    pub default_agent_id: String,
    pub max_routing_hops: Option<usize>,  // Prevent infinite loops
}
```

## Extension Data Schemas

### Request Extension Data (Router → Agent)

Keyed by extension URI in `message.metadata`:

```json
{
  "https://ranch.woi.dev/extensions/client-routing/v1": {
    "agentCards": [
      {
        "id": "researcher",
        "name": "Research Agent",
        "description": "Searches the web and summarizes findings",
        "capabilities": ["search", "summarize", "citations"],
        "supportsClientRouting": true
      },
      {
        "id": "writer",
        "name": "Writer Agent",
        "description": "Creates formatted documents",
        "capabilities": ["write", "format", "markdown"],
        "supportsClientRouting": false
      }
    ],
    "sender": "user"
  }
}
```

**Rust Structure**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientRoutingRequest {
    #[serde(rename = "agentCards")]
    pub agent_cards: Vec<SimplifiedAgentCard>,
    pub sender: String,  // "user" or agent ID
}
```

### Response Extension Data (Agent → Router)

Keyed by extension URI in `message.metadata`:

```json
{
  "https://ranch.woi.dev/extensions/client-routing/v1": {
    "recipient": "researcher",
    "reason": "User query requires web search capabilities"
  }
}
```

**Special Recipients**:
- `"user"`: Route to user (end conversation)
- `"sender"`: Route back to whoever sent this message (tracked by Router)
- `<agent-id>`: Route to specific agent

**Rust Structure**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientRoutingResponse {
    pub recipient: String,
    pub reason: Option<String>,
}
```

## Message Structure (A2A Compliant)

### Outgoing Message (with Extension)

```rust
let mut message = Message::user_text("Find recent papers on quantum computing");
message.message_id = Some(uuid::Uuid::new_v4().to_string());

// Declare extension usage
message.extensions = Some(vec![EXTENSION_URI.to_string()]);

// Add extension data
let extension_data = ClientRoutingRequest {
    agent_cards: vec![
        SimplifiedAgentCard { /* ... */ },
    ],
    sender: "user".to_string(),
};

let mut metadata = HashMap::new();
metadata.insert(
    EXTENSION_URI.to_string(),
    serde_json::to_value(&extension_data)?
);
message.metadata = Some(metadata);
```

### Response Message (with Extension)

```rust
let mut response = Message::agent_text("I'll route this to the researcher");

// Declare extension usage
response.extensions = Some(vec![EXTENSION_URI.to_string()]);

// Add routing decision
let routing_decision = ClientRoutingResponse {
    recipient: "researcher".to_string(),
    reason: Some("Query requires search capability".to_string()),
};

let mut metadata = HashMap::new();
metadata.insert(
    EXTENSION_URI.to_string(),
    serde_json::to_value(&routing_decision)?
);
response.metadata = Some(metadata);
```

## Constants

```rust
pub const EXTENSION_URI: &str = "https://ranch.woi.dev/extensions/client-routing/v1";
pub const EXTENSION_NAME: &str = "Client Agent Routing Extension";
pub const EXTENSION_DESCRIPTION: &str = 
    "Enables agents to receive peer agent list and make routing decisions";
```

## Error Handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum TeamError {
    #[error("Extension not supported by agent: {0}")]
    ExtensionNotSupported(String),
    
    #[error("Invalid recipient: {0}")]
    InvalidRecipient(String),
    
    #[error("Routing loop detected")]
    RoutingLoop,
    
    #[error("Extension data parse error: {0}")]
    ExtensionParseError(#[from] serde_json::Error),
    
    // ... existing errors
}
```
