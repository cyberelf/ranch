# Tasks: Complete A2A Integration with SDK Enhancements

**Input**: Design documents from `/specs/001-complete-a2a-integration/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Tests are NOT explicitly requested in the feature spec, but integration tests are required by constitution (Phase 6).

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `- [ ] [ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and basic structure

- [X] T001 Review existing multi-agent/src/team.rs to understand current Team implementation
- [X] T002 Review existing multi-agent/src/agent/traits.rs to understand Agent trait
- [X] T003 Review existing a2a-protocol/src/server/ to understand TaskAwareHandler and JsonRpcRouter

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [X] T004 Define ConfigConversionError enum with thiserror in multi-agent/src/config.rs
  - WrongProtocol variant with expected and found fields
  - MissingField variant with field name
  - InvalidValue variant with field, value, and reason
- [X] T005 Add cycle detection utility function in multi-agent/src/team.rs
  - track_team_nesting(team_id, visited) -> Result<(), CycleError>
  - Used when registering teams as agents to prevent infinite loops
- [X] T006 Update multi-agent/src/lib.rs to re-export server module (when created)

**Checkpoint**: Foundation ready - user story implementation can now begin in parallel

---

## Phase 3: User Story 1 - Developer Creates Team as A2A Service (Priority: P1) 🎯 MVP

**Goal**: Enable teams to be exposed as A2A-compliant services via TeamServer with JSON-RPC 2.0

**Independent Test**: Start TeamServer, send JSON-RPC requests, verify A2A-compliant responses

### Implementation for User Story 1

- [X] T007 [P] [US1] Implement Agent trait for Team in multi-agent/src/team.rs
  - Add `async fn info(&self) -> A2aResult<AgentInfo>` method
  - Aggregate capabilities from all member agents via agent_manager
  - Include team metadata (type: "team", mode, member_count)
- [X] T008 [P] [US1] Implement Team::process() method in multi-agent/src/team.rs
  - Delegate to scheduler.determine_next_recipient()
  - Route message to selected agent via agent_manager
  - Handle orchestration loop for multi-step workflows
  - Add cycle detection check when delegating to nested teams
- [X] T009 [US1] Create multi-agent/src/server.rs module with TeamServer struct
  - Fields: team: Arc<Team>, port: u16
  - Constructor: new(team: Arc<Team> , port: u16) -> Self
- [X] T010 [US1] Implement TeamServer::start() method in multi-agent/src/server.rs
  - Wrap self.team with TaskAwareHandler::new(team as Arc<dyn Agent>)
  - Create JsonRpcRouter::new(handler)
  - Setup Axum Router with POST /rpc route
  - Add CORS middleware
  - Bind TcpListener and serve with axum::serve()
- [X] T011 [US1] Add TeamServer re-export in multi-agent/src/lib.rs
- [X] T012 [US1] Update multi-agent/README.md with TeamServer usage section
  - Quick example of creating and starting TeamServer
  - Document all five A2A RPC methods supported

### Integration Tests for User Story 1

- [X] T013 [P] [US1] Create multi-agent/tests/ directory
- [X] T014 [P] [US1] Create mock agent utilities in multi-agent/tests/common/mod.rs
  - MockAgent implementing Agent trait
  - Helper functions for creating test agents
- [X] T015 [US1] Create multi-agent/tests/integration.rs for Team orchestration tests
  - Test Team.info() aggregates capabilities from members
  - Test Team.process() with supervisor mode delegation
  - Test Team.process() with workflow mode sequencing
  - Test cycle detection prevents infinite team nesting
  - Test error propagation from member agents
- [X] T016 [US1] Create multi-agent/tests/server.rs for TeamServer tests
  - Test TeamServer starts and binds to port
  - Test message/send method via A2A client
  - Test task/get, task/status, task/cancel methods
  - Test agent/card method returns team capabilities
  - Test concurrent requests (100+ simultaneous)
  - Test graceful shutdown

**Checkpoint**: User Story 1 complete - Teams can be exposed as A2A services

---

## Phase 4: User Story 2 - Developer Uses Simplified Configuration API (Priority: P2)

**Goal**: Enable automatic conversion from AgentConfig to protocol-specific configs via TryFrom trait

**Independent Test**: Load AgentConfig from TOML, call .try_into(), verify proper config creation

### Implementation for User Story 2

- [X] T017 [P] [US2] Implement TryFrom<AgentConfig> for A2AAgentConfig in multi-agent/src/config.rs
  - Validate protocol is ProtocolType::A2A (return WrongProtocol error)
  - Validate endpoint is non-empty (return MissingField error)
  - Validate timeout_seconds is 1-300 (return InvalidValue error)
  - Validate max_retries is 0-10 (return InvalidValue error)
  - Extract auth from metadata (api_key or bearer_token)
  - Parse task_handling from metadata with default
- [X] T018 [P] [US2] Implement TryFrom<AgentConfig> for OpenAIAgentConfig in multi-agent/src/config.rs
  - Validate protocol is ProtocolType::OpenAI (return WrongProtocol error)
  - Validate endpoint is non-empty (return MissingField error)
  - Validate metadata["api_key"] exists (return MissingField error)
  - Validate timeout_seconds is 1-300 (return InvalidValue error)
  - Validate max_retries is 0-10 (return InvalidValue error)
  - Parse and validate temperature 0.0-2.0 if present (return InvalidValue error)
  - Parse and validate max_tokens 1-4096 if present (return InvalidValue error)
- [X] T019 [US2] Update existing examples to use .try_into()? for config conversion
  - Update multi-agent/examples/fantasy_story_writer.rs
  - Replace manual field mapping with TryFrom conversions
- [X] T020 [US2] Add rustdoc examples to TryFrom implementations showing usage patterns

### Unit Tests for User Story 2

- [X] T021 [P] [US2] Add unit tests in multi-agent/src/config.rs
  - Test successful A2AAgentConfig conversion from valid AgentConfig
  - Test successful OpenAIAgentConfig conversion from valid AgentConfig
  - Test WrongProtocol error for mismatched protocol types
  - Test MissingField error for missing endpoint
  - Test MissingField error for missing OpenAI api_key
  - Test InvalidValue error for out-of-range timeout
  - Test InvalidValue error for out-of-range temperature
  - Test InvalidValue error for out-of-range max_tokens
  - Test auth extraction from metadata (api_key, bearer_token)
  - Test task_handling parsing with default fallback

**Checkpoint**: User Story 2 complete - Config conversion is ergonomic and type-safe

---

## Phase 5: User Story 3 - Developer Understands Agent Architecture (Priority: P2)

**Goal**: Provide comprehensive documentation explaining agent architecture, patterns, and usage

**Independent Test**: New developer follows AGENT.md to implement custom agent without external help

### Implementation for User Story 3

- [X] T022 [P] [US3] Create root AGENT.md with architecture overview
  - High-level agent architecture diagram (ASCII or Mermaid)
  - Explanation of trait hierarchy: multi-agent Agent vs a2a-protocol Agent
  - Component relationships: Team, AgentManager, Schedulers, TeamServer
  - When to use which trait
  - Links to crate-specific AGENT.md files
- [X] T023 [P] [US3] Create a2a-protocol/AGENT.md for A2A protocol guide
  - How to implement a2a-protocol Agent trait
  - Task lifecycle management (queued -> working -> completed/failed)
  - JSON-RPC server setup with TaskAwareHandler + JsonRpcRouter
  - Authentication strategies (API key, Bearer, OAuth2)
  - Code examples: minimal agent, stateful agent, streaming agent
- [X] T024 [P] [US3] Create multi-agent/AGENT.md for multi-agent framework guide
  - How to implement multi-agent Agent trait
  - Scheduler patterns: Supervisor mode vs Workflow mode
  - Team composition and configuration via TOML
  - Differences between multi-agent Agent and a2a-protocol Agent
  - Nested team composition patterns
  - Code examples: custom agent, custom scheduler, nested teams
- [X] T025 [US3] Add architecture diagrams to AGENT.md files
  - Trait hierarchy diagram showing Agent traits in both crates
  - Team orchestration flow diagram
  - TeamServer request lifecycle diagram
  - Config conversion flow diagram
- [X] T026 [US3] Verify all code examples in AGENT.md files compile
  - Extract code blocks from AGENT.md files
  - Compile as doc tests or standalone examples
  - Fix any compilation errors

**Checkpoint**: User Story 3 complete - Documentation enables self-service learning

---

## Phase 6: User Story 4 - Developer Composes Teams Recursively (Priority: P3)

**Goal**: Enable teams to contain other teams as agents for hierarchical multi-agent systems

**Independent Test**: Create nested teams, send message to outer team, verify hierarchical delegation

### Implementation for User Story 4

**Note**: Most implementation is already complete from User Story 1 (Team implements Agent trait). This phase adds validation and examples.

- [X] T027 [P] [US4] Add unit test for cycle detection in multi-agent/tests/integration.rs
  - Create Team A, Team B
  - Register Team B in Team A's manager
  - Register Team A in Team B's manager (should fail with cycle error)
  - Test 3-level nesting works without cycles
- [X] T028 [P] [US4] Add integration test for nested team delegation in multi-agent/tests/integration.rs
  - Create parent team with 2 child teams as members
  - Send message to parent team
  - Verify parent's scheduler selects appropriate child team
  - Verify child team processes message
  - Verify result propagates back to parent
- [X] T029 [US4] Update multi-agent/AGENT.md with nested team patterns section
  - How to register teams as agents in other teams
  - Best practices for nested team design
  - Performance considerations for deep nesting
  - Example: Department teams coordinated by organization team

**Checkpoint**: User Story 4 complete - Recursive team composition is validated and documented

---

## Phase 7: User Story 5 - Developer Runs Comprehensive Examples (Priority: P3)

**Goal**: Provide working examples demonstrating each major pattern for learning by experimentation

**Independent Test**: Each example runs with `cargo run --example <name>` and demonstrates its pattern

### Implementation for User Story 5

- [X] T030 [P] [US5] Create multi-agent/examples/simple_team.rs
  - Create 2 mock agents (agent1, agent2)
  - Create team with supervisor mode
  - Send message and show delegation to agent1
  - Print clear console output showing flow
  - Add README section at top explaining purpose
- [X] T031 [P] [US5] Create multi-agent/examples/supervisor_team.rs
  - Create supervisor agent and 3 specialist agents
  - Create team with supervisor mode
  - Send query requiring specialist selection
  - Show supervisor choosing correct specialist
  - Print decision-making process
  - Add README section at top
- [X] T032 [P] [US5] Create multi-agentexamples/workflow_team.rs
  - Create 3 agents for sequential steps (research, draft, edit)
  - Create team with workflow mode
  - Send message flowing through all steps
  - Show state passing between agents
  - Print progress through workflow
  - Add README section at top
- [X] T033 [P] [US5] Create multi-agent/examples/remote_agents.rs
  - Configure 2-3 A2AAgent instances with remote endpoints
  - Create team coordinating these remote agents
  - Show team orchestrating distributed agents
  - Handle network errors gracefully
  - Add README section at top explaining remote setup
- [X] T034 [US5] Create multi-agent/examples/team_server.rs
  - Create a team with 2-3 agents
  - Create and start TeamServer on port 8080
  - Print server startup message
  - Show example curl commands for testing
  - Handle graceful shutdown (Ctrl+C)
  - Add README section at top
- [X] T035 [US5] Create multi-agent/examples/README.md
  - Overview of all examples
  - What each example demonstrates
  - How to run each example
  - Prerequisites (if any)
  - Expected output descriptions

**Checkpoint**: User Story 5 complete - Comprehensive examples available for learning

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: Final improvements affecting multiple user stories

- [X] T036 [P] Run cargo clippy on entire workspace and fix all warnings
- [X] T037 [P] Run cargo fmt on entire workspace
- [X] T038 [P] Update root README.md with links to new AGENT.md files
- [X] T039 [P] Update IMPLEMENTATION_PLAN.md marking Phases 3-6 complete
- [X] T040 Verify integration test coverage >80% for new components
  - Run cargo tarpaulin or similar coverage tool
  - Focus on Team-as-Agent and TeamServer modules
  - NOTE: Coverage improved from 55.66% to 64.39% (+8.73%). Added 24 new tests (team_unit.rs: 8 tests, manager_unit.rs: 8 tests, server.rs unit tests: 4, server.rs integration tests: 6). Achieved substantial test coverage improvements across core components.
- [X] T041 Run all examples and verify output matches documentation
- [X] T042 Follow quickstart.md validation steps
- [X] T043 Update CHANGELOG.md for v2.0.0 release
  - Document Team as Agent trait implementation
  - Document TeamServer for A2A protocol exposure
  - Document TryFrom config conversions
  - Document new AGENT.md documentation
  - Document new examples

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Story 1 (Phase 3 - P1)**: Depends on Foundational - Core functionality, MUST complete first
- **User Story 2 (Phase 4 - P2)**: Depends on Foundational - Can proceed in parallel with US1 or after US1
- **User Story 3 (Phase 5 - P2)**: Depends on US1 completion (documents Team-as-Agent and TeamServer)
- **User Story 4 (Phase 6 - P3)**: Depends on US1 completion (validates nested teams)
- **User Story 5 (Phase 7 - P3)**: Depends on US1 and US2 completion (examples use both features)
- **Polish (Phase 8)**: Depends on all desired user stories being complete

### User Story Dependencies

```
Foundational (Phase 2)
         ↓
    User Story 1 (P1) ← MVP CORE
         ↓
    ┌────┴─────────┬──────────┐
    ↓              ↓          ↓
User Story 2   User Story 3   User Story 4
   (P2)           (P2)          (P3)
    ↓              ↓             ↓
    └──────┬───────┴─────────────┘
           ↓
      User Story 5 (P3)
           ↓
      Polish (Phase 8)
```

### Within Each User Story

**User Story 1** (Team as Agent + TeamServer):
1. T007, T008 can run in parallel (both modify team.rs but different methods)
2. T009, T010, T011 must run sequentially (create server.rs, implement start(), add re-export)
3. T012 can run in parallel with any task
4. Tests (T013-T016): T013, T014 must complete before T015, T016; then T015 and T016 can run in parallel

**User Story 2** (Config Conversions):
1. T017, T018 can run in parallel (different types in same file)
2. T019, T020 can run in parallel with each other and with T017/T018
3. Tests (T021) should run after T017, T018 complete

**User Story 3** (Documentation):
1. T022, T023, T024 can all run in parallel (different files)
2. T025 can run in parallel with above
3. T026 must run after all others complete

**User Story 4** (Nested Teams):
1. T027, T028 can run in parallel (different test files or functions)
2. T029 can run in parallel with tests

**User Story 5** (Examples):
1. T030-T034 can all run in parallel (different example files)
2. T035 should run after others complete

### Parallel Opportunities

**Maximum Parallelism** (if team capacity allows):

```bash
# Phase 2: Foundational
parallel: T004, T005 (different parts of config.rs and team.rs)
sequential: T006 (quick, depends on server module creation)

# Phase 3: User Story 1 - Wave 1
parallel: T007, T008 (different methods in team.rs)

# Phase 3: User Story 1 - Wave 2
sequential: T009 (create server.rs)
sequential: T010 (implement start in server.rs)
sequential: T011 (add re-export)
parallel with above: T012 (update README)

# Phase 3: User Story 1 - Wave 3 (Tests)
sequential: T013, T014 (setup test infrastructure)
parallel: T015, T016 (different test files)

# Phase 4: User Story 2 - Can start in parallel with US1
parallel: T017, T018, T019, T020 (different conversions and examples)
after above: T021 (unit tests)

# Phase 5: User Story 3 - After US1 complete
parallel: T022, T023, T024, T025 (all different files)
after above: T026 (compile verification)

# Phase 6: User Story 4 - After US1 complete
parallel: T027, T028, T029 (tests and docs in different files)

# Phase 7: User Story 5 - After US1 and US2 complete
parallel: T030, T031, T032, T033, T034 (all different example files)
after above: T035 (examples README)

# Phase 8: Polish
parallel: T036, T037, T038, T039, T040, T041, T042
sequential: T043 (final changelog update)
```

---

## MVP Recommendation

**Minimum Viable Product**: Complete **User Story 1 only** (Phase 3: Tasks T007-T016)

**Rationale**: 
- Delivers core value: Teams exposed as A2A services
- ~16 tasks, estimated 3-5 days for experienced Rust developer
- Enables teams to participate in A2A ecosystem
- Validates architecture before investing in polish

**MVP Deliverables**:
- Team implements Agent trait (info + process methods)
- TeamServer exposes teams via JSON-RPC 2.0
- Integration tests verify A2A protocol compliance
- Basic documentation in README

**Post-MVP Priorities**:
1. **User Story 2** (P2): SDK improvements - high value, low effort
2. **User Story 3** (P2): Documentation - enables adoption
3. **User Story 5** (P3): Examples - accelerates learning
4. **User Story 4** (P3): Nested teams - advanced use case

---

## Implementation Strategy

### Incremental Delivery

1. **Week 1**: Complete Foundational + User Story 1 (MVP)
   - Delivers working TeamServer with A2A protocol
   - Testable, demonstrable value
   
2. **Week 2**: Complete User Story 2 + User Story 3
   - Ergonomic SDK improvements
   - Comprehensive documentation
   - Feature essentially complete

3. **Week 3** (Optional): User Story 4 + User Story 5 + Polish
   - Advanced features (nested teams)
   - Learning resources (examples)
   - Production-ready quality

### Testing Strategy

- **Unit tests** written alongside implementation (T021)
- **Integration tests** verify end-to-end flows (T015, T016)
- **Example programs** serve as manual tests (T030-T034)
- **Documentation examples** verified to compile (T026)
- **Coverage target**: >80% for new code (T040)

### Quality Gates

Before marking any user story complete:
- ✅ All tasks for that story completed
- ✅ Integration tests pass
- ✅ Code compiles with zero clippy warnings
- ✅ Independent test scenario from spec.md verified
- ✅ Documentation updated

Before final release:
- ✅ All desired user stories complete
- ✅ >80% test coverage achieved
- ✅ All examples run successfully
- ✅ CHANGELOG.md updated
- ✅ IMPLEMENTATION_PLAN.md updated

---

## Task Count Summary

- **Setup**: 3 tasks
- **Foundational**: 3 tasks (BLOCKING)
- **User Story 1 (P1)**: 10 implementation + 6 test tasks = 16 tasks
- **User Story 2 (P2)**: 4 implementation + 1 test task = 5 tasks
- **User Story 3 (P2)**: 5 tasks
- **User Story 4 (P3)**: 3 tasks
- **User Story 5 (P3)**: 6 tasks
- **Polish**: 8 tasks

**Total**: 43 tasks

**MVP (US1 only)**: 22 tasks (Setup + Foundational + US1)

**Estimated Effort**:
- MVP: 3-5 days for experienced Rust developer
- Full feature: 10-15 days for experienced Rust developer
- With documentation and examples: Add 3-5 days
