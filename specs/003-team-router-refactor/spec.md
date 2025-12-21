# Feature Specification: Refactor Team Scheduler to Router with Client Agent Extension

**Feature Branch**: `003-team-router-refactor`  
**Created**: 2025-12-21  
**Status**: Draft  
**Input**: User description: "Refactor Team Scheduler to Router with Client Agent Extension"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Dynamic Routing with Client Agent Extension (Priority: P1)

As a developer using the multi-agent framework, I want my agents to be able to dynamically route messages to other agents based on capabilities and context, so that I can build flexible agent teams without hardcoded workflows.

**Why this priority**: This is the core functionality enabling the new routing architecture.

**Independent Test**: Can be fully tested by creating a team with a default agent and a "smart" agent, and verifying that the smart agent receives the peer list and can route messages.

**Acceptance Scenarios**:

1. **Given** a team with a Default Agent and a Smart Agent (supporting `client-routing`), **When** a message is sent to the team, **Then** the Default Agent receives the message first (if no recipient specified).
2. **Given** the Default Agent supports `client-routing`, **When** it receives a message, **Then** it receives a list of other agents (Smart Agent) in the message metadata.
3. **Given** the Default Agent decides to route to the Smart Agent, **When** it returns a response with `x-recipient: smart-agent-id`, **Then** the Router forwards the message to the Smart Agent.
4. **Given** the Smart Agent processes the message, **When** it returns a response with `x-recipient: user`, **Then** the Router returns the final response to the User.

---

### User Story 2 - Fallback Routing to Default Agent (Priority: P2)

As a user, I want messages without a specific recipient to be handled by a default agent, so that the conversation can always proceed even if no specific routing decision is made.

**Why this priority**: Ensures robustness and backward compatibility for simple agents.

**Independent Test**: Can be tested by sending a message with no recipient metadata and verifying the default agent receives it.

**Acceptance Scenarios**:

1. **Given** a message with no `x-recipient` metadata, **When** the Router processes it, **Then** it routes the message to the Default Agent.
2. **Given** an agent returns a response without `x-recipient` metadata, **When** the Router processes it, **Then** it routes the message to the Default Agent.

---

### User Story 3 - Back to Sender Routing (Priority: P2)

As an agent, I want to be able to return a message to the sender (User or another Agent), so that I can ask for clarification or provide a result.

**Why this priority**: Essential for conversational flows and "human in the loop" scenarios.

**Independent Test**: Can be tested by an agent setting `x-recipient` to the sender's ID.

**Acceptance Scenarios**:

1. **Given** an agent receives a message from "User", **When** it returns a response with `x-recipient: user` (or explicit "back to sender" signal), **Then** the Router returns the message to the User.
2. **Given** Agent A sent a message to Agent B, **When** Agent B returns a response with `x-recipient: agent-a`, **Then** the Router forwards the message to Agent A.

---

### Edge Cases

- **Circular Routing**: What happens if Agent A sends to Agent B, and Agent B sends back to Agent A indefinitely? (System should likely have a max-hop limit).
- **Invalid Recipient**: What happens if an agent specifies a non-existent agent ID? (Router should probably error or fallback to Default Agent with an error note).
- **No Default Agent**: What happens if no default agent is configured? (Configuration validation should prevent this).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST implement a `Router` component that replaces the existing `Scheduler`.
- **FR-002**: The `Router` MUST support a `default_agent_id` configuration.
- **FR-003**: The system MUST support a "Client Agent Extension" via A2A Message metadata using extension URI keying.
    - **FR-003.1**: The extension MUST use the extension URI `https://ranch.woi.dev/extensions/client-routing/v1` as the metadata key, with `agentCards` array to pass simplified agent cards to capable agents.
    - **FR-003.2**: The extension MUST use the same extension URI as metadata key, with `recipient` field to determine the next destination.
- **FR-004**: Agents MUST be able to declare support for the "Client Agent Extension" in their `AgentCard` capabilities.
- **FR-005**: The `Router` MUST inject the list of peer agents (excluding the target agent itself) into the message metadata ONLY if the target agent supports the extension.
    - **FR-005.1**: The injected agent list MUST include the extension support status for each peer.
- **FR-006**: The `Router` MUST route messages to the agent specified in `x-recipient`.
    - **FR-006.1**: If `x-recipient` is missing, route to the Default Agent.
    - **FR-006.2**: If the recipient field is exactly "user", return the result to the caller.
- **FR-007**: The system MUST unify `Supervisor` and `Workflow` team modes into a single mode managed by the `Router`.
- **FR-008**: The Router MUST enforce a maximum routing hop limit of 10 to prevent infinite loops.

### Key Entities

- **Router**: The central component managing message flow within a Team.
- **Client Agent Extension**: A protocol extension definition (metadata keys and values).
- **Simplified Agent Card**: A subset of `AgentCard` containing `id`, `name`, `description`, `capabilities`.
- **Recipient**: A destination identifier (Agent ID or User).

## Success Criteria *(mandatory)*

- **SC-001**: A Team configured with a Default Agent and multiple worker agents can successfully route a task from User -> Default -> Worker -> Default -> User using only metadata-based routing.
- **SC-002**: Agents supporting the extension receive the list of peers in the message metadata.
- **SC-003**: Agents NOT supporting the extension do NOT receive the list of peers (preserving bandwidth/context).
- **SC-004**: The `Scheduler` trait and implementations are successfully removed/deprecated.

## Assumptions

- The "Client Agent Extension" will be implemented using `metadata` fields in the A2A `Message` struct, as modifying the core protocol struct might be out of scope or breaking.
- "Simplified Agent Cards" will be a JSON structure serialized into the metadata.
- The "User" is represented by a specific constant or ID in the routing logic.
