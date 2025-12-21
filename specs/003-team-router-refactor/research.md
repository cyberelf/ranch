# Research: Team Router & Client Agent Extension

## Extension Specification Compliance

Based on [A2A Protocol v1.0 Section 4.6 Extensions](https://a2a-protocol.org/latest/specification/#46-extensions), this research establishes a spec-compliant extension mechanism for client-side agent routing.

## Decisions

### 1. Extension Declaration (Per A2A Spec Section 4.6.1)

**Decision**: Declare extension in `AgentCard.capabilities.extensions` array using `AgentExtension` structure.

**Structure** (per spec):
```json
{
  "uri": "https://ranch.woi.dev/extensions/client-routing/v1",
  "description": "Enables agent to receive peer agent list and make routing decisions",
  "required": false,
  "params": {}
}
```

**Rationale**: A2A spec requires extensions to be declared in the Agent Card with:
- **URI**: Unique identifier for the extension (MUST include version)
- **Description**: Human-readable explanation
- **Required**: Whether clients MUST support this extension
- **Params**: Extension-specific configuration (optional)

**Client Declaration**: Clients opt into extensions via:
- HTTP: `A2A-Extensions` header (comma-separated URIs)
- gRPC: `a2a-extensions` metadata
- JSON-RPC: Service parameter

### 2. Extension Points (Per A2A Spec Section 4.6.2)

**Decision**: Use **Message Extensions** (not Artifact Extensions).

**Message Extension Structure** (per spec):
```json
{
  "role": "user",
  "parts": [{"text": "Route this message"}],
  "extensions": ["https://ranch.woi.dev/extensions/client-routing/v1"],
  "metadata": {
    "https://ranch.woi.dev/extensions/client-routing/v1": {
      "agentCards": [
        {
          "id": "researcher",
          "name": "Research Agent",
          "description": "Searches the web",
          "capabilities": ["search", "summarize"],
          "supportsClientRouting": true
        }
      ],
      "sender": "user"
    }
  }
}
```

**Response Message Structure**:
```json
{
  "role": "agent",
  "parts": [{"text": "Routing to researcher..."}],
  "extensions": ["https://ranch.woi.dev/extensions/client-routing/v1"],
  "metadata": {
    "https://ranch.woi.dev/extensions/client-routing/v1": {
      "recipient": "researcher"
    }
  }
}
```

**Rationale**: Per A2A spec 4.6.2, extensions in Messages provide "additional strongly typed context or parameters relevant to the message being sent". The extension data goes in:
1. `message.extensions[]`: Array of extension URIs in use
2. `message.metadata[extension_uri]`: Keyed by extension URI, contains extension-specific data

### 3. Extension Data Schema

**Decision**: Define extension-specific data schema for request and response.

**Request Schema** (Router → Agent):
```typescript
{
  "agentCards": [
    {
      "id": string,
      "name": string,
      "description": string,
      "capabilities": string[],
      "supportsClientRouting": boolean
    }
  ],
  "sender": "user" | string  // Agent ID or "user"
}
```

**Response Schema** (Agent → Router):
```typescript
{
  "recipient": "user" | string,  // Agent ID or "user" or "sender"
  "reason"?: string  // Optional explanation for routing decision
}
```

**Back to Sender**: Per user requirement, agents can use:
- `"recipient": "sender"` (Router resolves to actual sender)
- `"recipient": "user"` (Always routes to user, who is the initial sender)

**Rationale**: This follows the spec's recommendation for "strongly typed context" and enables:
- Agents to see available peers and their capabilities
- Agents to track conversation initiator ("user" is always the initial sender)
- Agents to route back to sender (including other agents)

### 4. Simplified Agent Card Structure

**Decision**: Use lightweight structure optimized for routing context.

**Fields**:
- `id`: Agent identifier (required)
- `name`: Human-readable name (required)
- `description`: Brief capability description (required)
- `capabilities`: Array of capability tags (required)
- `supportsClientRouting`: Boolean flag (required)

**Rationale**: 
- Full `AgentCard` objects are heavyweight (with schemas, examples, authentication details)
- Routing only needs identity, capability summary, and extension support
- Keeps message metadata under reasonable size limits

### 5. Extension Versioning (Per A2A Spec Section 4.6.3)

**Decision**: Use versioned URI: `https://ranch.woi.dev/extensions/client-routing/v1`

**Rationale**: Per spec:
- Extensions SHOULD include version in URI
- Breaking changes MUST create new URI
- Agents MAY ignore unsupported versions (unless `required: true`)
- No automatic fallback to previous versions

**Future Versions**:
- v2: `/client-routing/v2` (breaking changes)
- v1.1: Backward-compatible, reuse `/v1` URI with additional optional fields

### 6. Routing Logic

**Decision**: Router implements extension-aware message flow.

**Algorithm**:
1. **Check Extension Support**: Query `AgentInfo.capabilities` for extension URI
2. **Inject Context** (if supported):
   - Add extension URI to `message.extensions[]`
   - Build `agentCards` array from team members
   - Set `metadata[extension_uri] = { agentCards, sender }`
3. **Send Message**: Forward to target agent
4. **Extract Recipient**: Read `response.metadata[extension_uri].recipient`
5. **Route Decision**:
   - `"user"`: Return to caller
   - `"sender"`: Route to message sender (tracked in context)
   - `<agent-id>`: Route to specified agent
   - Missing: Route to default agent

**Fallback**: If extension not supported, omit extension fields and rely on default routing.

### 7. Backward Compatibility

**Decision**: Extension is OPTIONAL (`required: false` in AgentCard).

**Implications**:
- Agents without extension support function normally (receive messages without extension data)
- Router detects support per-agent and only injects context when supported
- Mixed teams (with/without extension support) operate correctly

**Rationale**: Per A2A spec, optional extensions maintain compatibility with clients/agents that don't support them. The router acts as a transparent intermediary.

## Unknowns Resolved

### Message Metadata Structure
- **Confirmed**: `Message.metadata` is `google.protobuf.Struct` (arbitrary JSON)
- **Confirmed**: Extension data MUST be keyed by extension URI
- **Confirmed**: `Message.extensions` array declares active extensions

### Agent Card Declaration
- **Confirmed**: Extensions declared in `AgentCard.capabilities.extensions[]`
- **Confirmed**: `AgentExtension` struct requires `uri`, `description`, `required`, and optional `params`

### Service Parameter Transmission
- **Confirmed**: Clients declare extension usage via:
  - HTTP: `A2A-Extensions` header
  - gRPC: `a2a-extensions` metadata
  - JSON-RPC: Service parameter

### Extension Error Handling
- **Confirmed**: If agent declares `required: true` and client doesn't support it, agent returns `ExtensionSupportRequiredError`
- **Confirmed**: Agents SHOULD ignore unsupported optional extensions

## Implementation Notes

### Router Responsibilities
1. Maintain sender tracking across multi-turn conversations
2. Detect extension support from `AgentCard` or `AgentInfo`
3. Inject extension data only when target supports it
4. Parse extension data from responses
5. Handle "sender" recipient by resolving to actual sender agent/user

### Agent Responsibilities
1. Declare extension in Agent Card if supported
2. Parse extension data from incoming messages
3. Include extension URI in `response.extensions[]` when using extension
4. Populate `response.metadata[extension_uri]` with routing decision

### Testing Requirements
1. Extension declaration in Agent Card
2. Extension negotiation (client opt-in)
3. Message metadata injection
4. Response parsing
5. "Back to sender" routing
6. Mixed teams (with/without extension support)
