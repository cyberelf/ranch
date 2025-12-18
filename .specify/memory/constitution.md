<!--
SYNC IMPACT REPORT
Version: 1.3.0 -> 1.4.0
Modified Principles: None
Added Sections:
- Section VII: Feature Finalization & Status Tracking (mandates FEATURE_COMPLETE.md in specs folders)
Removed Sections: None
Templates requiring updates:
- .specify/templates/tasks-template.md (✅ updated - added Phase N: Feature Finalization with mandatory tasks)
- .specify/templates/plan-template.md (✅ updated - added Feature Finalization Requirement section)
- .specify/templates/spec-template.md (✅ no changes needed - acceptance criteria already covered)
- .github/copilot-instructions.md (✅ updated - added Governance section with Constitution reference; deduplicated content)
Follow-up TODOs: None
-->
# RANCH Constitution

## Core Principles

### I. Rust-First & Type Safety
All code MUST be written in Rust (Edition 2021) unless explicitly required otherwise. We leverage Rust's strong type system to prevent runtime errors. Use `thiserror` for library errors and `anyhow` for application errors. Shared state MUST use `Arc` and `RwLock` (from `tokio::sync`) for thread safety.

**Trait Infallibility**: Infallible traits (`From`, `Into`) MUST NOT panic or use assertions. For fallible conversions, use `TryFrom` and `TryInto` with explicit error types. This ensures composability and testability.

**Separation of Concerns**: Data structures MUST remain pure and not depend on runtime constructs. Configuration types SHOULD only handle parsing and validation. Runtime instantiation logic MUST live at module boundaries (lib.rs, standalone functions) or as associated functions on runtime types. This prevents conceptual cycles and maintains a clear dependency graph: data → runtime, not runtime ↔ data.

### II. Protocol Compliance
The system MUST strictly adhere to the A2A Protocol v0.3.0 specification. All transport MUST use JSON-RPC 2.0. Error codes MUST follow the spec (-32001 through -32007 for A2A errors). Message formats and RPC methods (`message/send`, `task/get`, etc.) MUST be validated against the spec.

### III. Async-Native
All I/O-bound operations MUST be asynchronous using the Tokio runtime. Blocking operations in async contexts are strictly FORBIDDEN. Use `#[async_trait]` for async traits. Leverage `tokio::spawn` and `select!` for concurrency.

### IV. Testing & Quality
Public APIs MUST have unit tests. Cross-module functionality MUST be verified with integration tests. External dependencies SHOULD be mocked using traits. `cargo test` MUST pass before any merge. Code MUST be linted with `cargo clippy` and formatted with `cargo fmt`.

**Test Organization & Co-location**: Unit tests MUST be co-located with source code in `#[cfg(test)] mod tests` blocks within the same file. Integration tests MUST reside in the `tests/` directory as separate files. This separation ensures:
- **Proximity**: Tests live next to implementation for easier maintenance
- **Visibility**: Developers see tests when working on code
- **Fast CI**: Unit tests compile with the library
- **Clear Boundaries**: Unit tests verify single modules; integration tests verify component interactions

NEVER create standalone unit test files in `tests/` directory. Use simple mocks within source modules for unit tests; use comprehensive shared mocks in `tests/common/` for integration tests. Detailed guidelines at `.github/TESTING_GUIDELINES.md`.

### V. Documentation & Standards
All public items MUST have rustdoc comments. Complex APIs MUST include examples. `README.md` and `CHANGELOG.md` MUST be updated for major features. Semantic versioning MUST be followed.

### VI. SDK Design & Developer Experience
When designing SDK interfaces, ALWAYS consider the code that will use them. APIs MUST:
- **Minimize boilerplate**: Provide convenience functions that compose common operations
- **Use sensible defaults**: Optional parameters should have reasonable defaults
- **Follow the principle of least surprise**: Method names and behavior should match developer expectations
- **Reduce error-proneness**: Design interfaces that make incorrect usage difficult (e.g., builder patterns, type states)
- **Provide ergonomic helpers**: Offer both low-level control and high-level convenience
- **Maintain consistency**: Similar operations should have similar APIs

**Example**: Instead of requiring users to manually register agents and then create teams, provide `create_team_from_config()` that does both atomically.

### VII. Feature Finalization & Status Tracking
Every feature implementation MUST be properly finalized with clear status documentation. When a feature is complete, the following MUST be provided:

**Finalization Document**: Create `FEATURE_COMPLETE.md` (or equivalent status document) in the feature's specification folder (`specs/<feature-id>/`). This document MUST include:
- **Executive Summary**: Brief overview of what was delivered
- **Deliverables**: List of all completed user stories and tasks
- **Test Results**: Comprehensive test metrics (count, coverage, pass rate)
- **Validation Results**: Evidence that acceptance criteria were met
- **Known Limitations**: Any scope reductions or deferred work
- **Release Readiness**: Production deployment checklist
- **Status Declaration**: Clear statement (COMPLETE, BLOCKED, DEFERRED, etc.)

**Task Completion**: All tasks in `tasks.md` MUST be marked with completion status (`[X]` for done, `[ ]` for incomplete). Incomplete tasks MUST have documented reasons.

**Traceability**: The finalization document MUST reference:
- Specification files (spec.md, plan.md, etc.)
- Implementation files (source code paths)
- Test files (unit tests, integration tests)
- Documentation files (AGENT.md, README.md, etc.)

**Discoverability**: Feature status MUST be easily identifiable:
- Feature folder contains status document at top level
- IMPLEMENTATION_PLAN.md or equivalent tracking document updated
- CHANGELOG.md includes feature in appropriate version section

**Rationale**: Without formal feature finalization, teams cannot determine which features are production-ready, what testing was performed, or what limitations exist. This leads to deployment of incomplete features, confusion about feature status, and difficulty in onboarding new developers. Structured finalization ensures institutional knowledge is captured and features are verifiably complete before release.

## Security & Performance

Input from external sources MUST be validated. Authentication (API Key, Bearer, OAuth2) is REQUIRED for agent communications. Sensitive credentials MUST NEVER be logged. Rate limiting SHOULD be implemented in servers. Connection pooling and timeouts MUST be used for network operations.

## Development Workflow

Development follows a workspace-based approach. Use `cargo build` to build the entire workspace. Use `cargo test` to run all tests. Dependencies should be managed at the workspace level where possible. Feature flags should be used to make dependencies optional.

## Governance

This Constitution supersedes all other development practices. Amendments require documentation, approval, and a migration plan. All PRs must verify compliance with these principles.

**Version**: 1.4.0 | **Ratified**: 2025-12-11 | **Last Amended**: 2025-12-18

## Amendment History

### v1.4.0 (2025-12-18)
- **Added**: Section VII (Feature Finalization & Status Tracking)
- **Rationale**: Codify mandatory feature finalization practices to ensure production readiness and institutional knowledge capture
- **Impact**: All completed features must include FEATURE_COMPLETE.md in specs/<feature-id>/ folder with deliverables, test results, validation evidence, and status declaration
- **Benefit**: Eliminates ambiguity about feature status; enables easy discovery of feature completion state; ensures verification of acceptance criteria before release

### v1.3.0 (2025-12-17)
- **Enhanced**: Section IV (Testing & Quality) - Added Test Organization & Co-location principle
- **Rationale**: Codify Rust best practices for test placement; unit tests belong in source modules with implementation, not in separate test files
- **Impact**: All future unit tests must be co-located in `#[cfg(test)] mod tests` blocks; `tests/` directory reserved exclusively for integration tests
- **Reference**: Comprehensive guidelines documented in `.github/TESTING_GUIDELINES.md`

### v1.2.0 (2025-12-12)
- **Enhanced**: Section I (Rust-First & Type Safety) - Added Separation of Concerns principle
- **Added**: Section VI (SDK Design & Developer Experience)
- **Rationale**: Codify architectural lessons learned from agent registration refactoring: keep data structures pure, avoid conceptual cycles, prioritize developer ergonomics
- **Impact**: Future API designs must consider usability and minimize boilerplate; configuration types must not depend on runtime constructs
- **Added**: Trait Infallibility principle to Section I (Rust-First & Type Safety)
- **Rationale**: Prevent panic/assert in From/Into implementations; mandate TryFrom/TryInto for fallible conversions
- **Impact**: Config conversion implementations must use TryFrom with explicit error types
