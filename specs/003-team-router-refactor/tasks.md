# Tasks: Refactor Team Scheduler to Router with Client Agent Extension

**Input**: Design documents from `/specs/003-team-router-refactor/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, quickstart.md

**Tests**: Tests are OPTIONAL and not included in this task list as they were not explicitly requested in the feature specification.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and basic structure

- [X] T001 Create team module directory structure at multi-agent/src/team/ with mod.rs, router.rs, types.rs
- [X] T002 [P] Add extension constants to multi-agent/src/team/types.rs (EXTENSION_URI, EXTENSION_VERSION)
- [X] T003 [P] Update multi-agent/Cargo.toml dependencies (ensure uuid, serde_json are included)

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core data structures and types that MUST be complete before user story implementation

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [X] T004 [P] Create new Recipient enum in multi-agent/src/team/types.rs (Agent(String), User variants) - Note: Existing Recipient struct in team.rs should be refactored or kept separate for backward compatibility
- [X] T005 [P] Create SimplifiedAgentCard struct in multi-agent/src/team/types.rs with A2A-compliant fields
- [X] T006 [P] Create ClientRoutingRequest struct in multi-agent/src/team/types.rs per data-model.md schema
- [X] T007 [P] Create ClientRoutingResponse struct in multi-agent/src/team/types.rs per data-model.md schema
- [X] T008 [P] Create TeamError enum in multi-agent/src/team/types.rs (InvalidRecipient, MaxHopsExceeded, RouterError variants)
- [X] T009 Create RouterConfig struct in multi-agent/src/team/types.rs (default_agent_id, max_routing_hops fields)
- [X] T010 Update TeamConfig struct in multi-agent/src/config.rs to replace mode with router_config field
- [X] T011 [P] Export all new types from multi-agent/src/team/mod.rs
- [X] T011a Ensure AgentInfo struct tracks extension support from AgentCard.capabilities.extensions array

**Checkpoint**: Foundation ready - user story implementation can now begin in parallel

---

## Phase 3: User Story 1 - Dynamic Routing with Client Agent Extension (Priority: P1) 🎯 MVP

**Goal**: Implement Router component that replaces Scheduler, enables metadata-driven routing, and injects peer agent lists for capable agents

**Independent Test**: Create a team with a default agent and a "smart" agent, verify smart agent receives peer list and can route messages using extension metadata

### Implementation for User Story 1

- [X] T012 Create Router struct in multi-agent/src/team/router.rs with default_agent_id, max_routing_hops, and sender_stack fields
- [X] T013 Implement Router::new() constructor in multi-agent/src/team/router.rs
- [X] T014 Implement Router::supports_extension() helper method in multi-agent/src/team/router.rs (checks agent capabilities)
- [X] T015 Implement Router::build_simplified_cards() in multi-agent/src/team/router.rs (converts AgentInfo to SimplifiedAgentCard)
- [X] T016 Implement Router::inject_extension_context() in multi-agent/src/team/router.rs (adds extension data to message.metadata)
- [X] T017 Implement Router::extract_recipient() in multi-agent/src/team/router.rs (parses extension response from message.metadata)
- [X] T018 Implement Router::route() main method in multi-agent/src/team/router.rs (orchestrates extension detection, injection, and routing)
- [X] T019 Update Team struct in multi-agent/src/team/mod.rs to replace Scheduler with Router field
- [X] T020 Update Team::new() in multi-agent/src/team/mod.rs to instantiate Router from config
- [X] T021 Refactor Team::process_messages() in multi-agent/src/team/mod.rs to use Router::route() instead of Scheduler logic
- [X] T022 Remove Scheduler trait and implementations in multi-agent/src/team/mod.rs (SupervisorScheduler, WorkflowScheduler)
- [X] T023 Remove TeamMode enum from multi-agent/src/team/mod.rs (no longer needed with unified Router)
- [X] T024 Add unit tests in #[cfg(test)] mod tests within multi-agent/src/team/router.rs (supports_extension, build_simplified_cards, inject_extension_context, extract_recipient)
- [X] T025 Add integration test in multi-agent/tests/router_integration.rs (team with extension-capable agent receives peer list and routes messages)
- [X] T026 Update config.example.toml with router_config section (default_agent_id, max_routing_hops)

**Checkpoint**: At this point, dynamic routing with extension support should be fully functional and testable independently

---

## Phase 4: User Story 2 - Fallback Routing to Default Agent (Priority: P2)

**Goal**: Ensure messages without recipient metadata always route to default agent for robustness and backward compatibility

**Independent Test**: Send a message with no x-recipient metadata and verify the default agent receives it

### Implementation for User Story 2

- [X] T027 [US2] Implement default agent fallback logic in Router::extract_recipient() in multi-agent/src/team/router.rs (return default if no metadata)
- [X] T028 [US2] Update Router::route() in multi-agent/src/team/router.rs to handle None from extract_recipient by routing to default agent
- [X] T029 [US2] Add edge case handling in Router::route() for invalid/missing recipient in multi-agent/src/team/router.rs
- [X] T030 [US2] Add unit test in #[cfg(test)] mod tests within multi-agent/src/team/router.rs verifying fallback to default agent when no recipient specified
- [X] T031 [US2] Add integration test in multi-agent/tests/router_integration.rs with simple agent (no extension) verifying default routing

**Checkpoint**: At this point, User Stories 1 AND 2 should both work independently

---

## Phase 5: User Story 3 - Back to Sender Routing (Priority: P2)

**Goal**: Enable agents to return messages to the sender (User or another Agent) using "user" or "sender" recipient

**Independent Test**: Agent sets x-recipient to "sender" and verify Router routes back to actual sender

### Implementation for User Story 3

- [X] T032 [US3] Implement Router::push_sender() method in multi-agent/src/team/router.rs (adds sender to stack)
- [X] T033 [US3] Implement Router::pop_sender() method in multi-agent/src/team/router.rs (removes and returns last sender)
- [X] T034 [US3] Update Router::route() in multi-agent/src/team/router.rs to push sender before routing to next agent
- [X] T035 [US3] Update Router::extract_recipient() in multi-agent/src/team/router.rs to resolve "sender" to actual sender from stack
- [X] T036 [US3] Update Router::extract_recipient() in multi-agent/src/team/router.rs to handle "user" recipient as User variant
- [X] T037 [US3] Update ClientRoutingRequest injection in multi-agent/src/team/router.rs to include sender field
- [X] T038 [US3] Add unit test in #[cfg(test)] mod tests within multi-agent/src/team/router.rs for sender stack push/pop operations
- [X] T039 [US3] Add integration test in multi-agent/tests/router_integration.rs for back-to-sender routing (Agent A → Agent B → Agent A)
- [X] T040 [US3] Add integration test in multi-agent/tests/router_integration.rs for user routing (Agent → User)

**Checkpoint**: All user stories should now be independently functional

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Edge cases, documentation, and improvements affecting multiple user stories

- [X] T041 [P] Implement max_routing_hops limit in Router::route() in multi-agent/src/team/router.rs to prevent infinite loops (default: 10, per FR-008)
- [X] T042 [P] Add error handling for invalid agent IDs in Router::route() in multi-agent/src/team/router.rs
- [X] T043 [P] Update multi-agent/README.md with Router architecture and Client Agent Extension documentation
- [ ] T044 [P] Add example in multi-agent/examples/router_example.rs demonstrating dynamic routing with extension
- [ ] T045 [P] Update CHANGELOG.md with Router refactor and extension support details
- [X] T046 Run quickstart.md validation by creating team and executing routing scenarios from quickstart.md
- [X] T047 Add rustdoc comments to all public Router APIs in multi-agent/src/team/router.rs
- [X] T048 [P] Add rustdoc comments to all public types in multi-agent/src/team/types.rs
- [X] T049 Update .github/copilot-instructions.md with Router and Client Agent Extension guidance
- [X] T050 Create FEATURE_COMPLETE.md in specs/003-team-router-refactor/ with executive summary, deliverables checklist, test results (count/coverage/pass rate), validation evidence, known limitations, release readiness checklist, and status declaration
- [X] T048 Add rustdoc comments to all public Router APIs in multi-agent/src/team/router.rs
- [X] T049 [P] Add rustdoc comments to all public types in multi-agent/src/team/types.rs
- [X] T050 Update .github/copilot-instructions.md with Router and Client Agent Extension guidance
- [X] T051 Create FEATURE_COMPLETE.md in specs/003-team-router-refactor/ with executive summary, deliverables checklist, test results (count/coverage/pass rate), validation evidence, known limitations, release readiness checklist, and status declaration

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3, 4, 5)**: All depend on Foundational phase completion
  - User Story 1 (P1) can start immediately after Phase 2
  - User Story 2 (P2) depends on US1 completion (T018: Router::route implementation)
  - User Story 3 (P2) can start after US1 completion, parallel with US2
- **Polish (Phase 6)**: Depends on all user stories being complete

### User Story Dependencies

**User Story 1 (P1)**: 
- Depends on: Phase 2 (Foundational) completion
- Blocks: US2, US3 (they build on Router implementation)
- Parallel opportunities: T012-T017 (all Router helper methods), T024-T026 (tests and docs)

**User Story 2 (P2)**:
- Depends on: T018 (Router::route main method from US1)
- Independent from: US3 (can implement in parallel)
- Parallel opportunities: T030-T031 (tests can run in parallel)

**User Story 3 (P2)**:
- Depends on: T018 (Router::route main method from US1)
- Independent from: US2 (can implement in parallel with US2)
- Parallel opportunities: T032-T033 (sender stack methods), T038-T040 (tests)

### Parallel Execution Examples

**After Phase 2 completes - User Story 1 tasks can parallelize**:
- Developer A: T012-T014 (Router struct and constructor)
- Developer B: T015-T017 (Helper methods for extension handling)
- Developer C: T024-T026 (Tests and documentation)

**After T018 completes - User Story 2 and 3 can both proceed**:
- Developer A: T027-T031 (US2: Fallback routing)
- Developer B: T032-T040 (US3: Back-to-sender routing)

**Phase 6 tasks can mostly parallelize**:
- Developer A: T041-T042 (Edge case handling)
- Developer B: T043-T045 (Documentation)
- Developer C: T047-T048 (API documentation)
- Developer D: T049-T050 (Agent context and feature finalization)

---

## Implementation Strategy

### MVP Scope (Recommended for first release)

**Phase 1 + Phase 2 + Phase 3 (User Story 1 only)**:
- Delivers core Router functionality with extension support
- Enables dynamic routing between agents
- Provides foundation for P2 stories
- Estimated: ~27 tasks (T001-T026, plus T011a)

### Incremental Delivery

1. **Release 1 (MVP)**: US1 - Dynamic Routing with Extension
2. **Release 2**: US2 - Fallback Routing (adds robustness)
3. **Release 3**: US3 - Back to Sender (adds conversational flows)
4. **Release 4**: Polish & Cross-Cutting Concerns

Each release delivers independently testable functionality.

---

## Validation Checklist

Before marking the feature complete, verify:

- [ ] All tasks marked as completed
- [ ] Router replaces Scheduler in Team struct
- [ ] Agents supporting extension receive peer lists in metadata
- [ ] Agents NOT supporting extension do NOT receive peer lists
- [ ] Messages route to specified recipient from extension response
- [ ] Messages without recipient route to default agent
- [ ] "user" recipient returns to caller
- [ ] "sender" recipient routes to actual sender
- [ ] Max hops limit prevents infinite routing loops
- [ ] Invalid recipient IDs are handled gracefully
- [ ] All examples from quickstart.md execute successfully
- [ ] Integration tests pass for all three user stories
- [ ] Documentation updated (README, CHANGELOG, API docs)
- [ ] Constitution checks still passing (Separation of Concerns, Test Organization)
- [ ] FEATURE_COMPLETE.md created with all required sections per Constitution Section VII
