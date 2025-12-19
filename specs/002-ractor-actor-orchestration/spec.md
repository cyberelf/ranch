# Feature Specification: Ractor Actor-Based Orchestration

**Feature Branch**: `002-ractor-actor-orchestration`  
**Created**: December 19, 2025  
**Status**: Draft  
**Input**: User description: "Refactor the Agent scheduler into actor model, each agent is an actor of ractor, and the scheduler is the orchestration system instead of simple routing decision maker. Reimplement orchestration patterns starting with Supervisor pattern where the Orchestrator's handle function receives all messages, processes them, and calls agent_ref.send_message(...). Integrate rig as the local agent framework to enforce the Supervisor Orchestrator."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Basic Actor Message Routing (Priority: P1)

A developer creates a multi-agent team where each agent runs as an independent ractor actor. When a message is sent to the team, the orchestrator receives it, determines which agent should handle it, and forwards the message to that agent actor. The agent processes the message and returns a response through the actor system.

**Why this priority**: This is the foundational capability - without basic actor-based message routing, none of the advanced orchestration patterns can function. It establishes the core architecture.

**Independent Test**: Can be fully tested by creating two agent actors, sending a message to the orchestrator, and verifying the message reaches the correct agent actor and returns a response.

**Acceptance Scenarios**:

1. **Given** a team with two agent actors (AgentA and AgentB) registered with the orchestrator, **When** a user sends a message that should route to AgentA, **Then** the orchestrator forwards the message to AgentA's actor and returns AgentA's response
2. **Given** an orchestrator with multiple agent actors, **When** an agent actor fails to process a message, **Then** the orchestrator detects the failure and returns an appropriate error without crashing the system
3. **Given** an active orchestrator with agent actors, **When** a message is sent to a non-existent agent, **Then** the orchestrator returns an error indicating the agent was not found

---

### User Story 2 - Supervisor Pattern Orchestration (Priority: P1)

A developer configures a supervisor-based team where a designated supervisor agent (implemented using rig framework) receives all incoming messages. The supervisor analyzes each message, determines which specialist agent should handle it, and the orchestrator routes the message accordingly. This enables intelligent delegation based on message content and agent capabilities.

**Why this priority**: The supervisor pattern is explicitly required in the specification and is the first orchestration pattern to implement. It provides intelligent routing that goes beyond simple round-robin or static rules.

**Independent Test**: Can be fully tested by creating a supervisor agent and three specialist agents, sending various messages (coding questions, writing requests, data analysis tasks), and verifying the supervisor correctly delegates each to the appropriate specialist.

**Acceptance Scenarios**:

1. **Given** a supervisor team with a rig-based supervisor and three specialist agents, **When** a user sends a coding question, **Then** the supervisor analyzes the message, decides to route it to the coding specialist, and the orchestrator forwards the message to that specialist actor
2. **Given** a supervisor making a routing decision, **When** the chosen specialist agent processes the message and responds, **Then** the response flows back through the orchestrator to the supervisor, and the supervisor can either return it to the user or delegate further
3. **Given** a supervisor team in operation, **When** the supervisor's routing decision identifies an agent that is unavailable, **Then** the orchestrator detects this and returns an error or triggers fallback behavior

---

### User Story 3 - Stateful Orchestration Context (Priority: P2)

As messages flow through the orchestrator, the system maintains orchestration context (such as which agents have been consulted, conversation history, and routing decisions made). This context is available to both the orchestrator's routing logic and to agent actors, enabling sophisticated multi-step workflows where decisions depend on prior interactions.

**Why this priority**: Stateful context enables advanced orchestration patterns beyond simple one-shot delegation. It's needed for multi-turn conversations and complex decision trees, but can be built after basic routing works.

**Independent Test**: Can be fully tested by creating a workflow that requires multiple agents in sequence (research → analysis → writing), sending a message, and verifying that context from earlier agents is available to later agents in the chain.

**Acceptance Scenarios**:

1. **Given** an orchestrator managing a multi-step workflow, **When** AgentA completes processing and the orchestrator routes to AgentB, **Then** AgentB receives context about AgentA's decision and can reference AgentA's results
2. **Given** an orchestrator with context tracking enabled, **When** five messages are routed through the same team instance, **Then** the orchestrator maintains conversation history and prevents infinite routing loops by detecting repeated agent visits
3. **Given** context accumulated during orchestration, **When** an agent actor requests current context state, **Then** the orchestrator provides the accumulated context data

---

### User Story 4 - Concurrent Agent Processing (Priority: P2)

Multiple agent actors can process different messages concurrently without blocking each other. When three different users send messages to the same team simultaneously, the orchestrator routes each message to the appropriate agent actors, and all three are processed in parallel by the actor system.

**Why this priority**: Concurrency is essential for production use but builds upon the basic routing infrastructure. It's a natural benefit of the actor model but requires proper orchestrator design to manage.

**Independent Test**: Can be fully tested by sending 10 messages concurrently to a team with 3 agent actors, measuring that processing happens in parallel (total time is not 10x single message time), and verifying all responses are correct.

**Acceptance Scenarios**:

1. **Given** a team orchestrator with three agent actors, **When** five messages arrive simultaneously from different users, **Then** the orchestrator routes them to agent actors concurrently and all five complete without blocking each other
2. **Given** concurrent message processing in progress, **When** one agent actor crashes while processing, **Then** other agent actors continue processing their messages without interruption
3. **Given** heavy concurrent load on an orchestrator, **When** 100 messages arrive within 1 second, **Then** the system maintains stable performance and doesn't drop messages or create deadlocks

---

### User Story 5 - Agent Lifecycle Management (Priority: P3)

Developers can dynamically add or remove agent actors from a running orchestrator without restarting the system. An administrator can register a new agent actor, and the orchestrator immediately includes it in routing decisions. Similarly, removing an agent actor causes the orchestrator to stop routing to it and handle in-flight messages gracefully.

**Why this priority**: Dynamic lifecycle management is valuable for production flexibility but isn't required for the core orchestration patterns to function. Initial implementation can use static agent registration.

**Independent Test**: Can be fully tested by starting an orchestrator with two agents, dynamically registering a third agent, sending a message that should route to it, then removing one agent and verifying messages no longer route there.

**Acceptance Scenarios**:

1. **Given** a running orchestrator with two agent actors, **When** an administrator registers a third agent actor at runtime, **Then** the orchestrator immediately includes the new agent in routing decisions without restart
2. **Given** an agent actor currently processing messages, **When** an administrator requests to remove that agent, **Then** the orchestrator waits for in-flight messages to complete before removing the agent actor
3. **Given** an orchestrator with dynamic agent registration enabled, **When** an agent actor is removed and then re-registered, **Then** the orchestrator correctly handles the lifecycle transition without state corruption

---

### Edge Cases

- What happens when an orchestrator receives a message but all agent actors are busy processing? (Should queue or return "busy" status)
- How does the system handle an agent actor that stops responding (timeout/health check)?
- What happens if the supervisor agent itself crashes or becomes unresponsive? (Needs fallback or error propagation)
- How are circular routing patterns detected and prevented (Agent A → Agent B → Agent A)?
- What happens when an agent actor panics during message processing? (Actor supervision should isolate failure)
- How does the orchestrator handle message rates that exceed agent processing capacity? (Backpressure mechanism needed)
- What happens if context state grows unbounded during long orchestration sessions? (Context cleanup or limits needed)

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST replace the current Scheduler trait with a ractor-based orchestrator that manages agent actors
- **FR-002**: System MUST implement each Agent as a ractor actor with message-passing communication (no direct method calls)
- **FR-003**: System MUST provide an orchestrator actor that receives all incoming messages and routes them to agent actors via `agent_ref.send_message(...)`
- **FR-004**: System MUST implement the Supervisor pattern where a designated supervisor agent (built with rig framework) makes all routing decisions
- **FR-005**: Supervisor agent MUST receive messages, analyze them, and return routing instructions to the orchestrator
- **FR-006**: System MUST integrate the rig framework for implementing the local supervisor agent with LLM-based decision making
- **FR-007**: Orchestrator MUST maintain message flow state (which agents have processed messages, conversation history)
- **FR-008**: System MUST support concurrent message processing across multiple agent actors without blocking
- **FR-009**: Agent actors MUST be isolated such that a crash in one agent does not affect other agents or the orchestrator
- **FR-010**: System MUST detect and prevent infinite routing loops (same agent receiving same message repeatedly)
- **FR-011**: Orchestrator MUST handle agent actor failures gracefully with appropriate error messages
- **FR-012**: System MUST provide a registration mechanism for adding agent actors to the orchestrator
- **FR-013**: Each agent actor MUST have a unique identifier used by the orchestrator for routing
- **FR-014**: Message routing decisions MUST be logged for debugging and monitoring
- **FR-015**: System MUST maintain compatibility with existing A2A protocol message formats

### Key Entities

- **OrchestratorActor**: Central ractor actor that receives messages and manages routing to agent actors. Maintains routing state and conversation context.
- **AgentActor**: Ractor actor wrapper around existing Agent implementations. Receives messages via ractor message passing, processes them, and returns responses.
- **SupervisorAgent**: Special rig-based agent that analyzes messages and makes routing decisions. Returns structured routing instructions (agent_id, reason, context_updates).
- **RoutingDecision**: Message type containing supervisor's decision about which agent should process a message, including routing rationale and context updates.
- **OrchestrationContext**: State maintained by orchestrator including conversation history, visited agents, accumulated context, and routing metrics.
- **AgentRegistry**: Collection of registered agent actors managed by the orchestrator, indexed by agent_id with capability metadata.

### API Design Considerations

**Developer Experience**:
- How will developers use this API? What does typical usage look like?
- What boilerplate can be eliminated? Can we provide convenience functions?
- What are sensible defaults? What should be optional vs required?
- Does the API follow the principle of least surprise?

**Architecture**:
- Are data structures kept pure (separated from runtime logic)?
- Is there a clear dependency direction (no cycles)?
- Developers should create agents as before, then wrap them in agent actors with minimal boilerplate
- Orchestrator creation should accept agent configurations and automatically instantiate agent actors
- Message sending should look like `orchestrator.send_message(msg).await` - simple async interface
- Rig supervisor agent should be configured via builder pattern: `RigSupervisor::builder().with_model(...).with_capabilities(...).build()`

**Architecture**:
- Agent trait remains pure (no ractor dependencies)
- AgentActor wraps Agent implementations, translating between ractor messages and Agent::process calls
- OrchestratorActor maintains actor refs and routing logic
- Supervisor agent implements Agent trait but uses rig internally for LLM reasoning
- Clear separation: Agent (business logic) → AgentActor (actor wrapper) → OrchestratorActor (routing)

**Testing Strategy**:
- Unit tests: Will be co-located in source files testing individual actor message handlers
- Integration tests: Will be in `tests/` directory testing full orchestration flows with multiple agents
- Mock agents needed for testing: Simple echo agent, failing agent, slow agent (in `tests/common/`)
- Key scenarios: Basic routing, supervisor delegation, concurrent processing, failure handling, loop detection

**Example Usage**:
```rust
// Create orchestrator with supervisor pattern
let orchestrator = OrchestratorBuilder::new()
    .with_supervisor(RigSupervisor::new(model, capabilities))
    .with_agent(coding_agent)
    .with_agent(writing_agent)
    .build()
    .await?;

// Send message - orchestrator handles routing
let response = orchestrator.send_message(Message::user_text("Write a function")).await?;
```

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Teams successfully route 100 test messages through ractor-based orchestration with 100% success rate
- **SC-002**: System processes 50 concurrent messages without deadlocks or dropped messages
- **SC-003**: Message routing latency increases by no more than 10ms compared to current implementation (actor overhead acceptable)
- **SC-004**: Supervisor pattern correctly delegates 95% of messages to the appropriate specialist agent based on content analysis
- **SC-005**: Agent actor failures are isolated - when one agent crashes, remaining agents continue processing 100% of their messages
- **SC-006**: Orchestration context is maintained across 10+ message turns in a conversation without memory leaks
- **SC-007**: Infinite routing loops are detected and terminated within 3 routing attempts
- **SC-008**: All existing integration tests pass with ractor-based implementation (backward compatibility maintained)
- **SC-009**: Developer can create a new ractor-based team with supervisor pattern in under 20 lines of configuration code
- **SC-010**: System documentation includes complete examples of supervisor pattern usage with rig integration

## Assumptions

1. **Ractor Library Stability**: Using ractor 0.9.x with supervision trees for fault isolation. Clustering features deferred to future iterations.
2. **Rig Framework Capabilities**: Using `rig-core` for provider-agnostic LLM integration. Supervisor agent uses rig for routing decision intelligence.
3. **Performance Profile**: Actor message-passing overhead expected <10ms per hop. Acceptable for multi-agent coordination use case.
4. **Ergonomics Over Compatibility**: New actor-native API is the primary interface. Existing Agent trait can be wrapped but ergonomic design takes priority over backward compatibility (no existing production users).
5. **Message Formats**: A2A protocol Message type fully compatible with ractor message channels. Protocol compliance (FR-015) is mandatory.
6. **Supervisor Response Time**: Rig-based supervisor routing decisions complete within 2-3 seconds (LLM inference time acceptable).
7. **Concurrent Load**: Initial implementation targets 5-20 concurrent messages with queue size 100 (configurable).
8. **Development Priority**: Supervisor pattern (P1) implemented first, followed by context management (P2) and lifecycle management (P3).
9. **Testing Requirements**: Co-located unit tests per Constitution Section IV. Integration tests focus on new actor-based implementation.
10. **Configuration Approach**: TOML-based configuration for teams, agents, and supervisor settings. Builder pattern for programmatic setup.

## Design Decisions (from Clarifications)

### Routing Decision Protocol
- Supervisor returns `RoutingDecision` struct with `next_agent: Option<String>` (None = return to user)
- Includes `context_updates: HashMap<String, String>` and optional `allow_revisit: bool`
- Invalid agent IDs trigger `OrchestrationError::AgentNotFound`

### Context Management
- Per-conversation context identified by session/request ID
- Automatically cleared when response returns to user
- Max size: 100 messages (configurable via `OrchestratorConfig`)
- No persistence across restarts in initial implementation

### Loop Detection
- Track (agent_id, SHA-256(message_content)) tuples in context
- Error on second occurrence of same pair
- Max routing depth: 10 hops (configurable)
- Supervisor can override with `allow_revisit: bool` for legitimate multi-pass scenarios

### Agent Registration
- P1: Build-time via `OrchestratorBuilder::with_agent(id, agent).build().await`
- P3: Runtime via `orchestrator.register_agent(id, agent).await`
- Explicit agent IDs required; validation checks for duplicates

### Logging & Observability
- Use `tracing` crate with structured spans per routing decision
- Info level: Routing decisions with agent_id and routing reason
- Debug level: Message metadata (content excluded for security)
- Error level: Failures and timeouts
- Automatic credential redaction in logs
- Configurable via RUST_LOG environment variable

### Message Queue & Timeouts
- Orchestrator maintains internal queue (size: 100, configurable)
- Messages timeout after 30s (configurable)
- When queue full: Return `OrchestrationError::QueueFull`
- Clients await responses asynchronously
- Agent actors have 30s response timeout for health checks

### Ractor Configuration
- Version: `ractor = "0.9"` (latest 0.9.x)
- Use supervision trees for agent fault isolation
- Skip clustering/distribution features initially
- Use `Actor` trait and `ActorRef<T>` for message passing

