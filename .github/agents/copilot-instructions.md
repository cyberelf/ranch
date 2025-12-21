# multi_agent Development Guidelines

Auto-generated from all feature plans. Last updated: 2025-12-11

## Active Technologies
- Rust 2021 (Workspace) + `tokio` (async runtime), `serde` (serialization), `async-trait`, `thiserror`, `anyhow` (003-team-router-refactor)
- N/A (In-memory routing) (003-team-router-refactor)

- Rust Edition 2021 + a2a-protocol (workspace local), tokio (async runtime), axum 0.7 (HTTP server), serde/serde_json (serialization) (001-complete-a2a-integration)

## Project Structure

```text
src/
tests/
```

## Commands

cargo test [ONLY COMMANDS FOR ACTIVE TECHNOLOGIES][ONLY COMMANDS FOR ACTIVE TECHNOLOGIES] cargo clippy

## Code Style

Rust Edition 2021: Follow standard conventions

## Recent Changes
- 003-team-router-refactor: Added Rust 2021 (Workspace) + `tokio` (async runtime), `serde` (serialization), `async-trait`, `thiserror`, `anyhow`

- 001-complete-a2a-integration: Added Rust Edition 2021 + a2a-protocol (workspace local), tokio (async runtime), axum 0.7 (HTTP server), serde/serde_json (serialization)

<!-- MANUAL ADDITIONS START -->
<!-- MANUAL ADDITIONS END -->
