# Feature Specification: Complete A2A Integration with SDK Enhancements

**Feature Branch**: `001-complete-a2a-integration`  
**Created**: 2025-12-11  
**Status**: Draft  
**Input**: User description: "Complete A2A integration implementation plan with SDK enhancements: add TryFrom trait implementations for config conversions and AGENT.md documentation"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Developer Creates Team as A2A Service (Priority: P1)

As a developer, I want to expose my multi-agent team as an A2A-compliant service so that other agents can discover and interact with it using the standard A2A protocol.

**Why this priority**: This is the core value proposition of the integration - enabling teams to participate in the A2A ecosystem. Without this, teams remain isolated and cannot be composed with other A2A agents.

**Independent Test**: Can be fully tested by starting a TeamServer, making JSON-RPC 2.0 requests to it, and verifying A2A-compliant responses. Delivers immediate value by making any team accessible via standard protocol.

**Acceptance Scenarios**:

1. **Given** a configured team with multiple agents, **When** I create a TeamServer and start it on a port, **Then** the server exposes JSON-RPC 2.0 endpoints at `/rpc`
2. **Given** a running TeamServer, **When** another A2A client sends a `message/send` request, **Then** the team processes the message through its scheduler and returns an A2A-compliant response
3. **Given** a running TeamServer, **When** a client requests `agent/card`, **Then** the server returns the team's capabilities, member agents, and coordination mode
4. **Given** a team using async task processing, **When** a message requires long-running work, **Then** the server returns a Task ID and allows polling via `task/get`

---

### User Story 2 - Developer Uses Simplified Configuration API (Priority: P2)

As a developer, I want to convert configuration objects directly to agent instances without writing boilerplate conversion code, so I can focus on business logic instead of type conversions.

**Why this priority**: This significantly improves developer experience and reduces code duplication. While not blocking core functionality, it makes the SDK more intuitive and reduces common sources of errors.

**Independent Test**: Can be tested by loading an `AgentConfig` from a TOML file and calling `.try_into()?` (or `::try_from`) to create A2AAgent or OpenAIAgent instances. Delivers value by eliminating 10-20 lines of conversion code per agent.

**Acceptance Scenarios**:

1. **Given** an `AgentConfig` with `protocol = "a2a"`, **When** I call `A2AAgentConfig::try_from(config)` or `config.try_into()`, **Then** I receive a properly configured `A2AAgentConfig` with all transport settings
2. **Given** an `AgentConfig` with `protocol = "openai"`, **When** I call `OpenAIAgentConfig::try_from(config)` or `config.try_into()`, **Then** I receive a properly configured `OpenAIAgentConfig` with endpoint and API settings
3. **Given** multiple agent configurations in a TOML file, **When** I iterate and convert each to its specific agent type, **Then** all agents are created without manual field mapping
4. **Given** an `AgentConfig` with invalid or missing required fields, **When** I attempt conversion, **Then** I receive a clear error message indicating what's missing

---

### User Story 3 - Developer Understands Agent Architecture (Priority: P2)

As a developer or contributor, I want comprehensive documentation explaining agent architecture and patterns so I can effectively use, extend, or contribute to the framework.

**Why this priority**: Good documentation is essential for adoption and maintainability but doesn't block immediate functionality. It enables self-service learning and reduces support burden.

**Independent Test**: Can be tested by having a new developer follow AGENT.md to implement a custom agent or team. Success means they complete the task without external help.

**Acceptance Scenarios**:

1. **Given** AGENT.md in the root directory, **When** I read it, **Then** I understand the overall agent architecture, trait hierarchy, and how teams orchestrate agents
2. **Given** AGENT.md in a2a-protocol crate, **When** I read it, **Then** I understand how to implement the A2A Agent trait, handle tasks, and expose agents via JSON-RPC
3. **Given** AGENT.md in multi-agent crate, **When** I read it, **Then** I understand the difference between the multi-agent Agent trait and A2A Agent trait, how to create custom schedulers, and how to compose teams
4. **Given** code examples in AGENT.md files, **When** I copy and adapt them, **Then** the code compiles and demonstrates the documented patterns

---

### User Story 4 - Developer Composes Teams Recursively (Priority: P3)

As a developer, I want teams to be usable as agents within other teams so I can create hierarchical multi-agent systems with delegated responsibilities.

**Why this priority**: This enables advanced use cases like department-level teams coordinating through organization-level teams. It's powerful but not required for basic team coordination.

**Independent Test**: Can be tested by creating a Team that contains other Teams as members, sending a message to the outer team, and verifying hierarchical delegation works correctly.

**Acceptance Scenarios**:

1. **Given** a Team implementing the Agent trait, **When** I register it in another team's AgentManager, **Then** the parent team can delegate messages to the child team
2. **Given** nested teams (team containing teams), **When** a message flows through the hierarchy, **Then** each level's scheduler makes appropriate delegation decisions
3. **Given** a recursive team structure, **When** I request the top-level team's profile, **Then** it includes information about its capability to coordinate sub-teams
4. **Given** an error in a nested team, **When** it propagates upward, **Then** parent teams can handle it gracefully without exposing internal structure

---

### User Story 5 - Developer Runs Comprehensive Examples (Priority: P3)

As a developer learning the framework, I want working examples demonstrating each major pattern (supervisor, workflow, remote agents, team server) so I can learn by experimentation.

**Why this priority**: Examples accelerate learning and serve as integration tests, but the framework can function with minimal examples. Fantasy story example already demonstrates basic concepts.

**Independent Test**: Each example can be run independently with `cargo run --example <name>`. Success means the example completes without errors and demonstrates the intended pattern.

**Acceptance Scenarios**:

1. **Given** `simple_team.rs` example, **When** I run it, **Then** I see two agents collaborating on a task with clear console output
2. **Given** `supervisor_team.rs` example, **When** I run it, **Then** I see a supervisor agent delegating work to specialist agents
3. **Given** `workflow_team.rs` example, **When** I run it, **Then** I see tasks flowing through agents in sequence
4. **Given** `team_server.rs` example, **When** I run it and send A2A requests, **Then** I receive proper responses demonstrating server functionality
5. **Given** `remote_agents.rs` example, **When** I configure endpoints for real A2A agents, **Then** the team coordinates them successfully

---

### Edge Cases

- What happens when a nested team contains a cycle (Team A contains Team B which contains Team A)? System must detect and prevent infinite delegation loops.
- How does the system handle when a team member agent is unavailable during orchestration? Scheduler must either retry, skip, or fail gracefully with clear error.
- What happens when converting an `AgentConfig` with `protocol = "unknown"`? Conversion must fail with clear error indicating unsupported protocol type.
- How does TeamServer behave when the underlying team's scheduler takes longer than typical HTTP timeout? Must return Task ID for async processing.
- What happens when AGENT.md examples reference APIs that don't exist in the current version? Documentation build process should verify code examples compile.
- How does the system handle when multiple agents in a team have overlapping capabilities? Scheduler must have clear selection criteria (first match, best match, or configured priority).

## Requirements *(mandatory)*

### Functional Requirements

#### Phase 3 & 4 Completion: Team as Agent & Server

- **FR-001**: Team MUST implement the Agent trait with `info()` returning team profile and `process()` performing orchestration
- **FR-002**: Team.info() MUST generate an AgentInfo describing the team's collective capabilities derived from member agents
- **FR-003**: TeamServer MUST expose teams via JSON-RPC 2.0 on a configurable port at `/rpc` endpoint
- **FR-004**: TeamServer MUST wrap teams with TaskAwareHandler to provide async task management
- **FR-005**: TeamServer MUST support all five A2A protocol methods: `message/send`, `task/get`, `task/status`, `task/cancel`, `agent/card`
- **FR-006**: When Team receives a message via Agent trait, it MUST use its configured scheduler to determine which member agent processes it
- **FR-007**: When all workflow steps complete, Team MUST return the final result to the caller

#### Phase 5: SDK Improvements

- **FR-008**: A2AAgentConfig MUST implement `TryFrom<AgentConfig>` trait to enable fallible, validated conversion
- **FR-009**: OpenAIAgentConfig MUST implement `TryFrom<AgentConfig>` trait to enable fallible, validated conversion
- **FR-010**: Config conversions MUST extract protocol-specific settings (endpoint, timeout, max_retries) from AgentConfig
- **FR-011**: Config conversions MUST handle optional fields gracefully with sensible defaults
- **FR-012**: Config conversions MUST validate that protocol type matches expected type (e.g., converting to A2AAgentConfig fails if protocol is "openai")
- **FR-013**: When AgentConfig is missing required fields for a protocol, conversion MUST return clear error identifying missing fields

#### Phase 6: Documentation

- **FR-014**: Root AGENT.md MUST provide high-level overview of agent architecture, trait relationships, and framework components
- **FR-015**: a2a-protocol/AGENT.md MUST document A2A Agent trait, how to implement it, JSON-RPC server setup, and task lifecycle
- **FR-016**: multi-agent/AGENT.md MUST document multi-agent Agent trait, scheduler patterns, team composition, and differences from A2A Agent trait
- **FR-017**: Each AGENT.md MUST include runnable code examples that compile with current codebase
- **FR-018**: AGENT.md files MUST include architecture diagrams showing trait hierarchies and component relationships
- **FR-019**: Integration tests MUST verify team orchestration, server request/response cycles, task handling, and error scenarios
- **FR-020**: Integration tests MUST achieve >80% code coverage for new components (Team as Agent, TeamServer)

#### Phase 5 & 6: Examples

- **FR-021**: `simple_team.rs` example MUST demonstrate basic two-agent team with clear console output
- **FR-022**: `supervisor_team.rs` example MUST demonstrate supervisor mode with delegation decisions
- **FR-023**: `workflow_team.rs` example MUST demonstrate sequential workflow mode
- **FR-024**: `remote_agents.rs` example MUST demonstrate connecting to and coordinating remote A2A agents
- **FR-025**: `team_server.rs` example MUST demonstrate exposing a team via TeamServer and accepting A2A requests
- **FR-026**: All examples MUST include README sections explaining what they demonstrate and how to run them

### Key Entities

- **Team (as Agent)**: A coordinated group of agents that implements Agent trait, exposing itself as a single agent that internally delegates work. Has AgentInfo describing collective capabilities, owns an AgentManager and Scheduler, processes messages by orchestrating member agents.

- **TeamServer**: HTTP server exposing a Team via JSON-RPC 2.0. Wraps Team with TaskAwareHandler, routes A2A protocol methods to appropriate handlers, manages port binding and request lifecycle.

- **A2AAgentConfig**: Configuration structure for A2A agents with fields for endpoint URL, authentication, timeout, retry settings. Created from AgentConfig via From trait, validates A2A-specific requirements.

- **OpenAIAgentConfig**: Configuration structure for OpenAI-compatible agents with fields for API endpoint, model, temperature, API key. Created from AgentConfig via From trait, validates OpenAI-specific requirements.

- **AgentInfo**: Multi-agent framework's representation of agent metadata including ID, name, description, capabilities list, and arbitrary metadata map. Generated by Agent trait's info() method, distinct from A2A protocol's AgentCard.

- **AgentCard**: A2A protocol's standard agent discovery format including agent_id, name, description, capabilities, skills, version, provider. Used for A2A agent discovery, returned by TeamServer's `agent/card` endpoint.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Developers can expose a team as an A2A service with fewer than 10 lines of code (create TeamServer, call start)
- **SC-002**: Converting AgentConfig to protocol-specific config eliminates 15-20 lines of boilerplate per agent configuration
- **SC-003**: New developers can implement a custom agent by following AGENT.md without external help in under 2 hours
- **SC-004**: TeamServer handles at least 100 concurrent A2A requests without errors or significant latency increase (>500ms)
- **SC-005**: All five required examples run successfully and produce expected output
- **SC-006**: Integration test suite achieves >80% code coverage for Team-as-Agent and TeamServer components
- **SC-007**: All code examples in AGENT.md files compile successfully as part of documentation build process
- **SC-008**: Team implementing Agent trait can be nested up to 3 levels deep without errors or stack overflow
- **SC-009**: SDK improvements reduce average lines of code in example programs by 20-30%
- **SC-010**: Zero clippy warnings for new code in Team, TeamServer, and config conversion implementations
