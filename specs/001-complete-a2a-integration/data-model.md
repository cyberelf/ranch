# Data Model

**Feature**: Complete A2A Integration with SDK Enhancements  
**Phase**: 1 - Design & Contracts  
**Date**: 2025-12-11

## Entity Definitions

### 1. Team (as Agent)

**Purpose**: A coordinated group of agents that implements the Agent trait, exposing itself as a single agent that internally delegates work.

**Key Attributes**:
- `config: TeamConfig` - Team configuration (name, description, mode, member agents)
- `agent_manager: Arc<AgentManager>` - Registry of available agents
- `scheduler: Arc<dyn Scheduler>` - Strategy for determining which agent handles each message

**Behaviors**:
- `info()` - Generates AgentInfo by aggregating capabilities from all registered member agents
- `process(message)` - Orchestrates message processing through scheduler and member agents
- `health_check()` - Verifies all member agents are responsive

**State Transitions**:
1. **Created** - Team initialized with config and manager
2. **Processing** - Scheduler determines next agent, delegates message
3. **Completed** - Final agent returns result, team returns to caller
4. **Failed** - Agent or scheduler error, propagate with context

**Relationships**:
- Contains many Agents via AgentManager
- Has one Scheduler (Supervisor or Workflow)
- Can be nested as Agent within another Team

**Validation Rules**:
- All agent IDs in config must be registered in manager
- Supervisor mode requires supervisor_agent_id to exist
- Workflow mode requires at least one step
- Cycle detection prevents Team A containing Team B containing Team A

### 2. TeamServer

**Purpose**: HTTP server that exposes a Team via JSON-RPC 2.0, making it accessible as an A2A protocol service.

**Key Attributes**:
- `team: Arc<Team>` - The team being exposed as a service
- `port: u16` - Port number to bind to
- `handler: Arc<TaskAwareHandler>` - A2A protocol handler wrapping team
- `router: JsonRpcRouter` - JSON-RPC 2.0 request router

**Behaviors**:
- `new(team, port)` - Creates server instance
- `start()` - Binds to port, starts Axum server, handles requests
- `shutdown()` - Gracefully stops server

**State Transitions**:
1. **Initialized** - Created with team and port
2. **Binding** - TcpListener binds to port
3. **Running** - Serving requests at /rpc endpoint
4. **Shutdown** - Graceful termination

**Relationships**:
- Wraps one Team
- Uses one TaskAwareHandler (from a2a-protocol)
- Exposes A2A Agent interface over HTTP

**Validation Rules**:
- Port must be available (not in use)
- Team must be valid (non-empty, agents registered)
- Must support all five A2A RPC methods

### 3. A2AAgentConfig

**Purpose**: Configuration structure for A2A protocol agents, created from generic AgentConfig.

**Key Attributes**:
- `endpoint: String` - Agent's JSON-RPC endpoint URL
- `agent_id: Option<String>` - Remote agent ID (optional, can be discovered)
- `timeout: Duration` - Request timeout
- `max_retries: u32` - Maximum retry attempts
- `auth: Option<Authenticator>` - Authentication strategy
- `task_handling: TaskHandling` - How to handle async task responses

**Behaviors**:
- `TryFrom<AgentConfig>` - Fallible conversion from generic config returning `ConfigConversionError`
- `validate()` - Checks endpoint is valid URL, timeout is reasonable

**Validation Rules**:
- Endpoint must be valid HTTP/HTTPS URL
- Timeout must be > 0 and < 300 seconds
- Max retries must be 0-10
- If auth provided, must have valid credentials
- Task handling must be one of: PollUntilComplete, ReturnTaskInfo, RejectTasks

**Relationships**:
- Created from AgentConfig via `TryFrom<AgentConfig>`
- Used to create A2AAgent instances

### 4. OpenAIAgentConfig

**Purpose**: Configuration structure for OpenAI-compatible agents, created from generic AgentConfig.

**Key Attributes**:
- `endpoint: String` - API endpoint (e.g., "https://api.openai.com/v1/chat/completions")
- `api_key: String` - OpenAI API key
- `model: String` - Model name (e.g., "gpt-4", "gpt-3.5-turbo")
- `timeout: Duration` - Request timeout
- `max_retries: u32` - Maximum retry attempts
- `temperature: Option<f32>` - Sampling temperature (0.0-2.0)
- `max_tokens: Option<u32>` - Maximum tokens in response

**Behaviors**:
- `From<AgentConfig>` - Converts from generic config
- `validate()` - Checks endpoint, API key format, temperature range

**Validation Rules**:
- Endpoint must be valid HTTP/HTTPS URL
- API key must be non-empty
- Model must be non-empty string
- Timeout must be > 0 and < 300 seconds
- Max retries must be 0-10
- Temperature if set must be 0.0-2.0
- Max tokens if set must be 1-4096

**Relationships**:
- Created from AgentConfig
- Used to create OpenAIAgent instances

### 5. AgentInfo

**Purpose**: Multi-agent framework's representation of agent metadata, returned by Agent trait's info() method.

**Key Attributes**:
- `id: String` - Unique agent identifier
- `name: String` - Human-readable agent name
- `description: String` - Agent's purpose and capabilities description
- `capabilities: Vec<String>` - List of capabilities (e.g., "research", "writing", "analysis")
- `metadata: HashMap<String, String>` - Arbitrary key-value pairs for extensibility

**Behaviors**:
- None - data structure only

**Validation Rules**:
- ID must be non-empty and unique within AgentManager
- Name must be non-empty
- Capabilities list should not contain duplicates

**Relationships**:
- Returned by Agent trait's info() method
- Used by AgentManager for agent discovery
- Aggregated by Team to create team-level AgentInfo
- Distinct from A2A protocol's AgentCard (different structure, different purpose)

### 6. AgentCard

**Purpose**: A2A protocol's standard agent discovery format, used for cross-agent discovery and capability negotiation.

**Key Attributes**:
- `agent_id: AgentId` - A2A protocol agent identifier (UUID-based)
- `name: String` - Agent name
- `description: String` - Agent description
- `capabilities: Vec<Capability>` - Structured capabilities with name, description, version
- `skills: Vec<Skill>` - Structured skills with name, proficiency
- `version: String` - Agent version (semver)
- `provider: Option<AgentProvider>` - Agent provider information

**Behaviors**:
- None - data structure only (defined by a2a-protocol)

**Validation Rules**:
- Must conform to A2A protocol v0.3.0 specification
- AgentId must be valid UUID or qualified name
- Capabilities must include name field minimum
- Version should follow semantic versioning

**Relationships**:
- Returned by TeamServer's `agent/card` RPC endpoint
- Generated from Team's AgentInfo plus team-specific metadata
- Used by A2A clients for agent discovery
- Defined in a2a-protocol crate

## Entity Relationships Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                        TeamServer                            │
│  - team: Arc<Team>                                           │
│  - port: u16                                                 │
│  - handler: TaskAwareHandler                                 │
│  └──► start() → Exposes via HTTP JSON-RPC                   │
└────────────────────┬────────────────────────────────────────┘
                     │ wraps
                     ▼
┌─────────────────────────────────────────────────────────────┐
│                          Team                                │
│  - config: TeamConfig                                        │
│  - agent_manager: Arc<AgentManager>                          │
│  - scheduler: Arc<dyn Scheduler>                             │
│  └──► implements Agent trait                                 │
│     ├─► info() → AgentInfo                                   │
│     └─► process(Message) → Message                           │
└────┬──────────────────────┬──────────────────────────────────┘
     │ contains             │ uses
     ▼                      ▼
┌──────────────────┐  ┌──────────────────────────────────────┐
│  AgentManager    │  │      Scheduler                       │
│  - agents: Map   │  │  (Supervisor / Workflow)             │
│  └─► register()  │  │  └─► determine_next_recipient()      │
│      get()       │  └──────────────────────────────────────┘
│      find()      │
└────┬─────────────┘
     │ manages
     ▼
┌──────────────────────────────────────────────────────────────┐
│                    Agent Implementations                      │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐       │
│  │  A2AAgent    │  │ OpenAIAgent  │  │    Team      │       │
│  │              │  │              │  │ (recursive)  │       │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘       │
│         │                 │                  │               │
│         └─────────────────┴──────────────────┘               │
│                All implement Agent trait                     │
└──────────────────────────────────────────────────────────────┘
         ▲                      ▲
         │ created from         │ created from
         │                      │
┌─────────────────┐      ┌─────────────────┐
│ A2AAgentConfig  │      │OpenAIAgentConfig│
│                 │      │                 │
└────┬────────────┘      └────┬────────────┘
     │ From trait             │ From trait
     └────────────┬───────────┘
                  │
            ┌─────────────┐
            │ AgentConfig │
            │ (generic)   │
            └─────────────┘
```

## State Machine: Team Message Processing

```
                    ┌─────────┐
                    │  Idle   │
                    └────┬────┘
                         │ process(message)
                         ▼
                  ┌──────────────┐
                  │  Scheduling  │ ←──────┐
                  │              │        │
                  └──┬──────┬────┘        │
                     │      │             │
         ┌───────────┘      └─────────────┼────┐
         │                                │    │
         │ agent_id                return │    │ next step
         ▼                         to user     │
    ┌─────────────┐                    ▲      │
    │  Delegating │                    │      │
    │  to Agent   │                    │      │
    └──┬──────────┘                    │      │
       │ process()                     │      │
       ▼                               │      │
    ┌────────────┐                     │      │
    │ Processing │                     │      │
    │ (in agent) │                     │      │
    └──┬──────────┘                    │      │
       │ result                        │      │
       ▼                               │      │
    ┌────────────┐                     │      │
    │  Complete  │─────────────────────┘      │
    │            │                            │
    └──┬─────────┘                            │
       │ more steps?                          │
       └──────────────────────────────────────┘
              (workflow mode)
```

## Data Flow: TeamServer Request Handling

```
External Client (A2A)
       │
       │ HTTP POST /rpc
       │ {"jsonrpc": "2.0", "method": "message/send", "params": {...}}
       ▼
┌──────────────────┐
│   TeamServer     │
│   (Axum Router)  │
└─────┬────────────┘
      │ JSON-RPC request
      ▼
┌──────────────────────┐
│   JsonRpcRouter      │
│   (parse, validate)  │
└─────┬────────────────┘
      │ method: "message/send"
      ▼
┌──────────────────────────┐
│   TaskAwareHandler       │
│   (implements A2A Agent) │
└─────┬────────────────────┘
      │ delegate to wrapped agent
      ▼
┌──────────────────┐
│      Team        │
│ (multi-agent)    │
└─────┬────────────┘
      │ process(message)
      ▼
┌──────────────────┐
│    Scheduler     │
│ (determine next) │
└─────┬────────────┘
      │ agent_id
      ▼
┌──────────────────┐
│  Member Agent    │
│  (A2A/OpenAI)    │
└─────┬────────────┘
      │ result message
      ▼
      [flows back up]
      │
      ▼
External Client
{"jsonrpc": "2.0", "result": {"message": {...}}, "id": 1}
```

## Configuration Schema

### TeamConfig
```rust
struct TeamConfig {
    id: String,
    name: String,
    description: String,
    mode: TeamMode,                    // Supervisor | Workflow
    agents: Vec<TeamAgentConfig>,
    scheduler_config: SchedulerConfig,
}

struct TeamAgentConfig {
    agent_id: String,
    role: String,
    capabilities: Vec<String>,
}

enum SchedulerConfig {
    Supervisor(SupervisorSchedulerConfig),
    Workflow(WorkflowSchedulerConfig),
}
```

### AgentConfig (Generic)
```rust
struct AgentConfig {
    id: String,
    name: String,
    endpoint: String,
    protocol: ProtocolType,           // A2A | OpenAI
    capabilities: Vec<String>,
    metadata: HashMap<String, String>,
    timeout_seconds: u64,
    max_retries: u32,
}
```

## Persistence & Storage

**Note**: This feature does not require persistent storage. All state is in-memory.

- TeamConfig loaded from TOML files at startup
- AgentManager registry is in-memory with RwLock
- TaskAwareHandler maintains task state in-memory
- For production persistence, users can add their own storage layer

## Concurrency & Thread Safety

### Shared State
- `AgentManager.agents: RwLock<HashMap<String, Arc<dyn Agent>>>` - Multiple readers, single writer
- `Team.agent_manager: Arc<AgentManager>` - Shared across team instances
- `TeamServer.team: Arc<Team>` - Shared by all request handlers

### Locking Strategy
- Read locks for agent lookup (get, list, find_by_capability)
- Write locks only for register/remove (infrequent)
- No lock held across async boundaries (all locks released before .await)

### Message Flow
- Each request gets independent Message instance (no sharing)
- Schedulers maintain internal state with RwLock (WorkflowScheduler.current_step)
- No global mutable state beyond agent registry
