# Implementation Plan: Complete A2A Integration with SDK Enhancements

**Branch**: `001-complete-a2a-integration` | **Date**: 2025-12-11 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/001-complete-a2a-integration/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

Complete the A2A integration implementation plan (Phases 3-6) by implementing Team as Agent trait, exposing teams via TeamServer with JSON-RPC 2.0, adding ergonomic SDK improvements (TryFrom<AgentConfig> trait implementations), creating comprehensive AGENT.md documentation files, and building production-ready examples. This enables teams to participate in the A2A ecosystem, reduces boilerplate code by 15-20 lines per agent, and provides clear onboarding documentation.

## Technical Context

**Language/Version**: Rust Edition 2021  
**Primary Dependencies**: a2a-protocol (workspace local), tokio (async runtime), axum 0.7 (HTTP server), serde/serde_json (serialization)  
**Storage**: N/A (in-memory agent management with RwLock)  
**Testing**: cargo test (unit + integration tests), >80% coverage target  
**Target Platform**: Linux/macOS/Windows server environments  
**Project Type**: Rust workspace with two crates (a2a-protocol, multi-agent)  
**Performance Goals**: 100 concurrent A2A requests without >500ms latency increase, 3-level team nesting support  
**Constraints**: Strict A2A Protocol v0.3.0 compliance, JSON-RPC 2.0 transport, async-only (no blocking)  
**Scale/Scope**: Framework for multi-agent coordination with ~5 new modules, ~1500 LOC, 5 examples

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

**I. Rust-First & Type Safety**
- ✅ All code in Rust Edition 2021
- ✅ Using thiserror for MultiAgentError types
- ✅ Arc<RwLock<>> for shared AgentManager state
- ✅ Strong typing throughout (no `Any`, careful trait object usage)

**II. Protocol Compliance**
- ✅ Team as Agent + TeamServer must strictly follow A2A Protocol v0.3.0
- ✅ JSON-RPC 2.0 transport via existing JsonRpcRouter
- ✅ All five RPC methods: message/send, task/get, task/status, task/cancel, agent/card
- ✅ Error codes -32001 through -32007 for A2A-specific errors

**III. Async-Native**
- ✅ All Agent trait methods are async (info, process)
- ✅ TeamServer uses Tokio TcpListener and Axum
- ✅ No blocking operations in async contexts
- ✅ Using #[async_trait] for trait definitions

**IV. Testing & Quality**
- ⚠️ **VIOLATION**: Integration tests do not currently exist (Phase 6 adds them)
- ✅ Unit tests exist for adapters.rs
- ⚠️ **ACTION REQUIRED**: Add integration tests in Phase 6 with >80% coverage
- ✅ cargo clippy and cargo fmt required

**V. Documentation & Standards**
- ⚠️ **VIOLATION**: AGENT.md files do not exist (Phase 6 adds them)
- ✅ Most public APIs have rustdoc
- ⚠️ **ACTION REQUIRED**: Create comprehensive AGENT.md files
- ✅ CHANGELOG.md will be updated for v2.0.0 release

**Security & Performance**
- ✅ Authentication support via a2a-protocol (ApiKeyAuth, BearerAuth, OAuth2)
- ✅ Input validation in JSON-RPC layer
- ✅ Connection pooling and timeouts in HTTP clients
- ✅ Rate limiting should be added to TeamServer (recommended but not blocking)

**Post-Design Re-check Notes**: After Phase 1, verify that Team implementing Agent trait doesn't violate Rust's async trait safety rules, and TeamServer properly wraps with TaskAwareHandler for async task management.

**Violations Justification**: Integration tests and AGENT.md documentation are intentionally deferred because they document and verify completed implementations. Cannot write tests before implementation exists. This is acceptable as interim state; both must be completed before feature is considered done.

## Project Structure

### Documentation (this feature)

```text
specs/001-complete-a2a-integration/
├── plan.md              # This file
├── research.md          # Phase 0: Architecture decisions and best practices
├── data-model.md        # Phase 1: Entity definitions (Team, TeamServer, configs)
├── quickstart.md        # Phase 1: Getting started guide
├── contracts/           # Phase 1: API contracts
│   ├── team-agent-trait.md      # Agent trait implementation for Team
│   ├── team-server-api.md       # TeamServer HTTP/JSON-RPC API
│   └── config-conversions.md    # From<AgentConfig> trait contracts
└── tasks.md             # Phase 2: Implementation tasks (created by /speckit.tasks)
```

### Source Code (repository root)

```text
multi-agent/
├── Cargo.toml
├── src/
│   ├── lib.rs                    # Re-exports, updated for TeamServer
│   ├── adapters.rs              # ✅ Exists - message helpers
│   ├── agent/                   # ✅ Exists
│   │   ├── mod.rs               
│   │   ├── traits.rs            # Agent trait definition
│   │   ├── a2a_agent.rs         # ✅ Exists - A2A agent implementation
│   │   ├── openai_agent.rs      # ✅ Exists
│   │   └── errors.rs            # ✅ Exists
│   ├── config.rs                # ⚠️ UPDATE - add TryFrom<AgentConfig> impls
│   ├── manager.rs               # ✅ Exists - AgentManager
│   ├── team.rs                  # ⚠️ UPDATE - implement Agent trait
│   └── server.rs                # ❌ NEW - TeamServer implementation
├── examples/
│   ├── simple_team.rs           # ❌ NEW
│   ├── supervisor_team.rs       # ❌ NEW  
│   ├── workflow_team.rs         # ❌ NEW
│   ├── remote_agents.rs         # ❌ NEW
│   ├── team_server.rs           # ❌ NEW
│   └── fantasy_story_writer.rs  # ✅ Exists
├── tests/                       # ❌ NEW directory
│   ├── integration.rs           # ❌ NEW - team orchestration tests
│   └── server.rs                # ❌ NEW - TeamServer tests
├── AGENT.md                     # ❌ NEW - multi-agent architecture docs
└── README.md                    # ⚠️ UPDATE - add TeamServer docs

a2a-protocol/
├── src/
│   ├── lib.rs                   # ✅ Exists
│   ├── core/                    # ✅ Exists - Message, Task, etc.
│   ├── client/                  # ✅ Exists - A2aClient
│   └── server/                  # ✅ Exists - TaskAwareHandler, JsonRpcRouter
├── AGENT.md                     # ❌ NEW - A2A protocol agent docs
└── README.md                    # ⚠️ UPDATE

AGENT.md                         # ❌ NEW - root architecture overview
IMPLEMENTATION_PLAN.md           # ⚠️ UPDATE - mark completed phases
```

**Structure Decision**: Rust workspace with two crates. Multi-agent crate adds server module, From trait implementations, and integration tests. A2A-protocol crate is stable, only documentation added. Examples demonstrate all patterns. Root AGENT.md provides high-level overview with links to crate-specific docs.

## Complexity Tracking

> No constitution violations requiring justification. Testing and documentation violations are expected interim states that will be resolved during implementation.

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| [e.g., 4th project] | [current need] | [why 3 projects insufficient] |
| [e.g., Repository pattern] | [specific problem] | [why direct DB access insufficient] |
