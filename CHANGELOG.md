# Changelog

All notable changes to the RANCH (Rust Agent Networking & Coordination Hub) project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

#### Team as Agent Implementation
- **Team implements Agent trait**: Teams can now be treated as agents themselves, enabling hierarchical multi-agent systems and recursive team composition
- **Team.info()**: Returns aggregated capabilities from all member agents with team metadata (type, mode, member count)
- **Team.process()**: Processes messages through the team's orchestration system with cycle detection to prevent infinite team nesting

#### TeamServer for A2A Protocol Exposure
- **TeamServer**: New HTTP server that exposes teams via JSON-RPC 2.0 as A2A-compliant services
- **Full A2A Protocol Support**: Implements all five A2A RPC methods:
  - `message/send` - Send messages to team for processing
  - `task/get` - Retrieve task details
  - `task/status` - Check task processing status
  - `task/cancel` - Cancel running tasks
  - `agent/card` - Get team capabilities and metadata
- **CORS Support**: Built-in CORS middleware for web application integration
- **Graceful Shutdown**: Proper cleanup and shutdown handling via tokio signals

#### Configuration API Improvements
- **TryFrom<AgentConfig> for A2AAgentConfig**: Automatic conversion from generic agent config to A2A-specific config with validation
- **TryFrom<AgentConfig> for OpenAIAgentConfig**: Automatic conversion to OpenAI-specific config with parameter validation
- **ConfigConversionError**: New error type with specific variants:
  - `WrongProtocol` - Protocol type mismatch
  - `MissingField` - Required configuration field missing
  - `InvalidValue` - Configuration value out of valid range
- **Validation**: Automatic validation of timeouts (1-300s), retries (0-10), temperature (0.0-2.0), and max_tokens (1-4096)

#### Documentation
- **Root AGENT.md**: High-level architecture overview explaining trait hierarchy, component relationships, and when to use each abstraction
- **a2a-protocol/AGENT.md**: Comprehensive guide for implementing A2A protocol agents with examples for minimal agents, stateful agents, and streaming agents
- **multi-agent/AGENT.md**: Multi-agent framework guide covering team composition, scheduler patterns, nested teams, and custom agent implementation
- **Architecture Diagrams**: Visual representations of trait hierarchy, team orchestration flow, TeamServer lifecycle, and config conversion flow

#### Examples
- **supervisor_team.rs**: Demonstrates supervisor pattern with intelligent routing to specialist agents based on content analysis
- **workflow_team.rs**: Shows sequential processing through workflow steps (Research → Draft → Edit) with state transformation
- **remote_agents.rs**: Illustrates distributed agent coordination using A2AAgent with remote endpoints and network error handling
- **team_server.rs**: Complete example of exposing a team via HTTP with curl test commands and graceful shutdown
- **examples/README.md**: Comprehensive guide to all examples with pattern selection criteria, common code patterns, and troubleshooting

### Changed
- **SupervisorScheduler**: Fixed infinite loop bug by adding call counter and proper return-to-user logic
- **Team orchestration**: Enhanced with better logging showing which agent is processing each message

### Fixed
- **Infinite loop in supervisor mode**: SupervisorScheduler now correctly returns control to user after supervisor processes message
- **DocTest compilation**: Fixed TeamServer example in documentation to use correct imports
- **Clippy warnings**: Fixed constant assertions and len() > 0 checks to use is_empty()

### Technical Details

#### Breaking Changes
None - This release is fully backward compatible

#### Performance
- Zero-copy message passing where possible
- Efficient capability aggregation using HashSet
- Connection pooling in HTTP clients
- Async-first design throughout

#### Testing
- 241 total tests passing
- Integration tests for team orchestration (7 tests)
- Unit tests for config conversion (10 tests)
- DocTests for examples (3 tests)
- Comprehensive test coverage for Team-as-Agent functionality

#### Dependencies
- tokio 1.x - Async runtime
- axum 0.7 - HTTP server framework
- serde/serde_json - Serialization
- thiserror/anyhow - Error handling
- async-trait - Async trait support

---

## Development Phases Completed

### Phase 1-2: Setup & Foundation ✅
- Project structure established
- Core types defined
- Error handling implemented

### Phase 3: User Story 1 - Team as A2A Service ✅
- Team implements Agent trait
- TeamServer implementation
- Integration tests

### Phase 4: User Story 2 - SDK Improvements ✅
- TryFrom trait implementations
- Configuration validation
- Error types

### Phase 5: User Story 3 - Documentation ✅
- AGENT.md files created
- Architecture diagrams
- Code examples verified

### Phase 6: User Story 4 - Nested Teams ✅
- Cycle detection
- Integration tests
- Documentation

### Phase 7: User Story 5 - Examples ✅
- 5 comprehensive examples
- examples/README.md
- All examples tested

### Phase 8: Polish ✅
- Clippy warnings addressed
- Code formatted
- Tests passing (241/241)
- Documentation complete

---

## Migration Guide

### From Pre-2.0 Versions

If upgrading from an older version where teams were not agents:

**Before:**
```rust
// Teams could not be used as agents
let team = Team::new(config, manager);
// Could not register teams in other teams
```

**After:**
```rust
// Teams are now full agents
let team = Arc::new(Team::new(config, manager));

// Can register teams as agents in other teams
parent_manager.register(team as Arc<dyn Agent>).await?;

// Can expose teams via TeamServer
let server = TeamServer::new(team, 8080);
server.start().await?;
```

### Configuration Conversion

**Before:**
```rust
// Manual field mapping
let a2a_config = A2AAgentConfig {
    max_retries: agent_config.max_retries,
    task_handling: /* parse from metadata */,
};
```

**After:**
```rust
// Automatic conversion with validation
let a2a_config: A2AAgentConfig = agent_config.try_into()?;
```

---

## Future Roadmap

### Version 2.1.0 (Planned)
- Rate limiting for TeamServer
- Authentication middleware
- Monitoring and observability hooks
- Performance benchmarks

### Version 2.2.0 (Planned)
- Dynamic team reconfiguration
- Health check endpoints
- Metrics collection
- Load balancing support

---

## Contributors

This release represents the completion of the complete A2A integration feature specification.

## License

This project follows the same license as the parent RANCH project.
