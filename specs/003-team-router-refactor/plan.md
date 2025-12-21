# Implementation Plan: Refactor Team Scheduler to Router with Client Agent Extension

**Branch**: `003-team-router-refactor` | **Date**: 2025-12-21 | **Spec**: [specs/003-team-router-refactor/spec.md](specs/003-team-router-refactor/spec.md)
**Input**: Feature specification from `specs/003-team-router-refactor/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

Refactor the `Team` architecture to replace the rigid `Scheduler` with a dynamic `Router`. This enables flexible, metadata-driven message routing between agents, including a "Client Agent Extension" that allows capable agents to receive peer lists and make routing decisions. Unifies "Supervisor" and "Workflow" modes into a single router-based mode.

## Technical Context

**Language/Version**: Rust 2021 (Workspace)
**Primary Dependencies**: `tokio` (async runtime), `serde` (serialization), `async-trait`, `thiserror`, `anyhow`
**Storage**: N/A (In-memory routing)
**Testing**: `cargo test` (Unit & Integration)
**Target Platform**: Linux/Cross-platform
**Project Type**: Rust Library (Crate: `multi-agent`)
**Performance Goals**: Minimal overhead for routing logic; efficient cloning of agent lists.
**Constraints**: Must maintain A2A protocol compliance; "Client Agent Extension" must use existing protocol fields (metadata).
**Scale/Scope**: Refactor of `multi-agent` crate core logic; affects `Team` and `Agent` traits.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### Architecture & API Design Review

**SDK/API Usability**:
- [x] APIs minimize boilerplate - `Team::new` should remain simple, config-driven.
- [x] Sensible defaults reduce required configuration - `default_agent_id` is the main config.
- [x] Method names follow principle of least surprise - `Router::route` vs `Scheduler::schedule`.
- [x] Interface design makes incorrect usage difficult - Typed `Recipient` struct.
- [x] Both low-level control and high-level convenience provided.

**Separation of Concerns**:
- [x] Data structures remain pure - `Recipient`, `SimplifiedAgentCard` are pure data.
- [x] Configuration types only handle parsing/validation - `TeamConfig` updates.
- [x] Runtime instantiation at module boundaries - `Router` created in `Team::new`.
- [x] No conceptual cycles - Router depends on Agent traits, not vice versa.
- [x] Clear dependency graph maintained.

**Test Organization** (per Constitution IV):
- [x] Unit tests planned to be co-located in source files (`#[cfg(test)] mod tests`)
- [x] Integration tests planned for `tests/` directory
- [x] No standalone unit test files in `tests/` directory
- [x] Mocking strategy defined (Mock agents for router testing)
- [x] Clear boundary between unit (router logic) and integration (team message flow) tests

## Project Structure

### Documentation (this feature)

```text
specs/003-team-router-refactor/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
└── tasks.md             # Phase 2 output
```

### Source Code (repository root)

```text
multi-agent/
├── src/
│   ├── team/                # NEW: Team logic separation
│   │   ├── mod.rs           # Team struct and public API
│   │   ├── router.rs        # Router implementation
│   │   └── types.rs         # Shared types (Recipient, RouterConfig, etc.)
│   ├── agent/
│   │   └── traits.rs        # Update Agent trait if needed
│   ├── config.rs            # Update TeamConfig to use RouterConfig
│   └── lib.rs               # Re-exports
tests/
├── integration.rs           # Updated integration tests
├── router_integration.rs    # NEW: Router-specific integration tests
└── common/                  # Shared mocks
```
