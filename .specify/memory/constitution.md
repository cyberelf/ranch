<!--
SYNC IMPACT REPORT
Version: 0.0.0 -> 1.0.0
Modified Principles:
- Added: I. Rust-First & Type Safety
- Added: II. Protocol Compliance
- Added: III. Async-Native
- Added: IV. Testing & Quality
- Added: V. Documentation & Standards
Added Sections:
- Security & Performance
- Development Workflow
Templates requiring updates:
- .specify/templates/plan-template.md (✅ updated - implicitly via Constitution Check)
- .specify/templates/spec-template.md (✅ updated - implicitly via Constitution Check)
- .specify/templates/tasks-template.md (✅ updated - implicitly via Constitution Check)
-->
# RANCH Constitution

## Core Principles

### I. Rust-First & Type Safety
All code MUST be written in Rust (Edition 2021) unless explicitly required otherwise. We leverage Rust's strong type system to prevent runtime errors. Use `thiserror` for library errors and `anyhow` for application errors. Shared state MUST use `Arc` and `RwLock` (from `tokio::sync`) for thread safety.

**Trait Infallibility**: Infallible traits (`From`, `Into`) MUST NOT panic or use assertions. For fallible conversions, use `TryFrom` and `TryInto` with explicit error types. This ensures composability and testability.

### II. Protocol Compliance
The system MUST strictly adhere to the A2A Protocol v0.3.0 specification. All transport MUST use JSON-RPC 2.0. Error codes MUST follow the spec (-32001 through -32007 for A2A errors). Message formats and RPC methods (`message/send`, `task/get`, etc.) MUST be validated against the spec.

### III. Async-Native
All I/O-bound operations MUST be asynchronous using the Tokio runtime. Blocking operations in async contexts are strictly FORBIDDEN. Use `#[async_trait]` for async traits. Leverage `tokio::spawn` and `select!` for concurrency.

### IV. Testing & Quality
Public APIs MUST have unit tests. Cross-module functionality MUST be verified with integration tests. External dependencies SHOULD be mocked using traits. `cargo test` MUST pass before any merge. Code MUST be linted with `cargo clippy` and formatted with `cargo fmt`.

### V. Documentation & Standards
All public items MUST have rustdoc comments. Complex APIs MUST include examples. `README.md` and `CHANGELOG.md` MUST be updated for major features. Semantic versioning MUST be followed.

## Security & Performance

Input from external sources MUST be validated. Authentication (API Key, Bearer, OAuth2) is REQUIRED for agent communications. Sensitive credentials MUST NEVER be logged. Rate limiting SHOULD be implemented in servers. Connection pooling and timeouts MUST be used for network operations.

## Development Workflow

Development follows a workspace-based approach. Use `cargo build` to build the entire workspace. Use `cargo test` to run all tests. Dependencies should be managed at the workspace level where possible. Feature flags should be used to make dependencies optional.

## Governance

This Constitution supersedes all other development practices. Amendments require documentation, approval, and a migration plan. All PRs must verify compliance with these principles.

**Version**: 1.1.0 | **Ratified**: 2025-12-11 | **Last Amended**: 2025-12-11

## Amendment History

### v1.1.0 (2025-12-11)
- **Added**: Trait Infallibility principle to Section I (Rust-First & Type Safety)
- **Rationale**: Prevent panic/assert in From/Into implementations; mandate TryFrom/TryInto for fallible conversions
- **Impact**: Config conversion implementations must use TryFrom with explicit error types
