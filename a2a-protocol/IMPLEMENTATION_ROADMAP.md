# A2A Protocol Implementation Roadmap

**Current Version:** v0.4.0 (Spec-Compliant Baseline)  
**Target Spec:** A2A Protocol v0.3.0  
**Last Updated:** October 20, 2025

---

## Executive Summary

This roadmap tracks the implementation of the A2A (Agent-to-Agent) protocol v0.3.0 specification in Rust. After the v0.4.0 cleanup (October 2025), we have a **strict spec-compliant baseline** with JSON-RPC 2.0 transport. This document outlines the path to full specification compliance.

### Current Compliance: ~70%

**✅ Implemented:**
- JSON-RPC 2.0 transport (fully compliant)
- Core RPC methods: `message/send`, `task/get`, `task/cancel`, `task/status`, `agent/card`
- Basic Task lifecycle management
- Message & Part schema (spec-aligned)
- AgentCard (partial compliance)
- Authentication strategies (Bearer, API Key, OAuth2 foundation)

**🚧 In Progress / Needs Work:**
- AgentCard (remaining required fields)
- Error codes (need A2A-specific codes)

**❌ Not Implemented:**
- SSE streaming (message/stream, task/resubscribe)
- Push notifications (task/pushNotificationConfig/*)
- Authenticated extended card
- gRPC transport
- HTTP+JSON/REST transport

---

## Current State (v0.4.0 Baseline)

### What Works

#### 1. Server (JSON-RPC 2.0)
```rust
// Fully functional spec-compliant server
use a2a_protocol::server::{JsonRpcRouter, TaskAwareHandler};

let handler = TaskAwareHandler::new(agent_card);
let router = JsonRpcRouter::new(handler);
// Exposes: POST /rpc with all RPC methods
```

**Supported Methods:**
- ✅ `message/send` - Send message, returns Task or Message
- ✅ `task/get` - Get task details and results
- ✅ `task/status` - Get current task status  
- ✅ `task/cancel` - Cancel running task
- ✅ `agent/card` - Get agent capabilities

#### 2. Client (JSON-RPC 2.0)
```rust
// Fully functional spec-compliant client
use a2a_protocol::{client::ClientBuilder, prelude::*};

let client = ClientBuilder::new()
    .with_json_rpc("https://agent.example.com/rpc")
    .build()?;

let message = Message::user_text("Hello");
let response = client.send_message(message).await?;
```

#### 3. Core Types
- ✅ `Message` - Spec-compliant structure
- ✅ `Task` - With lifecycle states
- ✅ `TaskStatus` - With state tracking
- ✅ `TaskState` - Enum with 8 states
- ✅ `AgentCard` - Required core fields, pending metadata additions
- ✅ `SendResponse` - Union of Task | Message

#### 4. Testing
- ✅ 101 tests passing (76 unit + 16 compliance + 8 RPC + 1 doc)
- ✅ Full JSON-RPC 2.0 compliance
- ✅ Task lifecycle tests
- ✅ Integration tests

### What's Missing/Broken

#### 1. AgentCard Remaining Fields
**Implemented:**
- `protocolVersion`
- `preferredTransport`
- `additionalInterfaces`

**Still Missing:**
- ❌ `defaultInputModes` / `defaultOutputModes` (MIME types)
- ❌ `supportsAuthenticatedExtendedCard`
- ❌ Strongly typed transport enum (`JSONRPC` | `GRPC` | `HTTP+JSON`)

#### 2. Error Handling Gaps
**Missing A2A-Specific Error Codes:**
- `-32001`: TaskNotFoundError
- `-32002`: TaskNotCancelableError
- `-32003`: PushNotificationNotSupportedError
- `-32004`: UnsupportedOperationError
- `-32005`: ContentTypeNotSupportedError
- `-32006`: InvalidAgentResponseError
- `-32007`: AuthenticatedExtendedCardNotConfiguredError

---

## Release Roadmap

### v0.4.0 ✅ COMPLETED (October 2025)
**Theme:** Spec Compliance Baseline

**Completed:**
- ✅ Removed non-spec A2aRouter (REST endpoints)
- ✅ Removed non-compliant streaming module
- ✅ JSON-RPC 2.0 only (clean baseline)
- ✅ All tests passing
- ✅ Migration guide created

---

### v0.5.0 🎯 NEXT (Target: Q4 2025)
**Theme:** Core Spec Compliance - Metadata & Errors

**Priority:** Close remaining spec gaps that block full interoperability claims

#### Goals
1. ✅ Message & Part schema parity with spec (complete)
2. ✅ `MessageRole` restricted to `user`/`agent` (complete)
3. 🚧 Finalize AgentCard required fields (`defaultInputModes`, `defaultOutputModes`, `supportsAuthenticatedExtendedCard`)
4. 🚧 Implement A2A-specific JSON-RPC error codes
5. 🚧 Expand compliance testing for new metadata and error paths

#### Detailed Tasks

**1. AgentCard Enhancements (Week 1-2)**
- [ ] Add `defaultInputModes: Vec<String>` (MIME types)
- [ ] Add `defaultOutputModes: Vec<String>` (MIME types)
- [ ] Add `supportsAuthenticatedExtendedCard: bool`
- [ ] Promote `preferredTransport` to a spec-aligned enum (`JSONRPC` | `GRPC` | `HTTP+JSON`)
- [ ] Extend `TransportInterface` to validate transport enum usage
- [ ] Update builders and examples
- [ ] Add validation tests for required fields

**2. Error Code Mapping (Week 2-3)**
- [ ] Implement TaskNotFoundError (-32001)
- [ ] Implement TaskNotCancelableError (-32002)
- [ ] Implement PushNotificationNotSupportedError (-32003)
- [ ] Implement UnsupportedOperationError (-32004)
- [ ] Implement ContentTypeNotSupportedError (-32005)
- [ ] Implement InvalidAgentResponseError (-32006)
- [ ] Implement AuthenticatedExtendedCardNotConfiguredError (-32007)
- [ ] Update dispatcher and transport error mapping
- [ ] Add unit tests that assert correct code emission

**3. Compliance Verification (Week 3-4)**
- [ ] Add schema/serde round-trip tests for AgentCard with new fields
- [ ] Extend JSON fixtures to include new error cases
- [ ] Update integration tests to cover error codes
- [ ] Regenerate API docs and examples
- [ ] Document field additions in README and migration guide

#### Success Criteria
- ✅ AgentCard exposes all required metadata and validates inputs
- ✅ JSON serialization matches spec examples (messages, parts, agent card)
- ✅ Error codes map 1:1 with A2A guidance
- ✅ All existing tests pass
- ✅ New compliance tests pass
- ✅ Can interoperate with spec-compliant agents

#### Estimated Timeline
**4 weeks** (AgentCard and error code work on the critical path)

---

### v0.6.0 📅 (Target: Q1 2026)
**Theme:** SSE Streaming Support

**Priority:** Enable real-time communication for long-running tasks

#### Goals
1. ✅ Implement W3C Server-Sent Events (SSE)
2. ✅ Add `message/stream` RPC method
3. ✅ Add `task/resubscribe` RPC method
4. ✅ Implement event types (TaskStatusUpdate, TaskArtifactUpdate)
5. ✅ Add streaming capabilities to AgentCard

#### Detailed Tasks

**1. SSE Infrastructure (Week 1-2)**
- [ ] Implement W3C SSE writer (text/event-stream)
- [ ] Create SSE event wrapper for JSON-RPC responses
- [ ] Implement connection management
- [ ] Add reconnection handling
- [ ] Implement proper event IDs
- [ ] Add Last-Event-ID support
- [ ] Create SSE client for testing

**2. Streaming Response Types (Week 2)**
- [ ] Create `SendStreamingMessageResponse` type
- [ ] Create `TaskStatusUpdateEvent` struct
- [ ] Create `TaskArtifactUpdateEvent` struct
- [ ] Implement event serialization
- [ ] Add metadata fields per spec

**3. message/stream Implementation (Week 3)**
- [ ] Add `message/stream` method to dispatcher
- [ ] Implement SSE response streaming
- [ ] Stream Task status updates
- [ ] Stream Artifact updates
- [ ] Handle stream termination on completion/error
- [ ] Add backpressure handling

**4. task/resubscribe Implementation (Week 3)**
- [ ] Add `task/resubscribe` method
- [ ] Implement resuming existing task stream
- [ ] Handle Last-Event-ID for catchup
- [ ] Determine backfill strategy
- [ ] Add stream state management

**5. Client Streaming Support (Week 4)**
- [ ] Add streaming client API
- [ ] Implement SSE parser
- [ ] Add async stream interface
- [ ] Handle reconnection
- [ ] Add timeout handling

**6. AgentCard Updates (Week 4)**
- [ ] Add `capabilities.streaming` field
- [ ] Document streaming support in card
- [ ] Add streaming configuration options

**7. Testing (Week 5)**
- [ ] Add SSE format validation tests
- [ ] Add streaming workflow tests
- [ ] Add reconnection tests
- [ ] Test with multiple concurrent streams
- [ ] Load testing for streaming
- [ ] Add streaming examples

#### Success Criteria
- ✅ W3C SSE specification compliant
- ✅ Proper event format per A2A spec
- ✅ Reconnection works correctly
- ✅ No memory leaks in long-running streams
- ✅ Works with standard SSE clients

#### Estimated Timeline
**5 weeks**

---

### v0.7.0 📅 (Target: Q2 2026)
**Theme:** Push Notifications

**Priority:** Support webhook-based async updates

#### Goals
1. ✅ Implement all 4 pushNotificationConfig methods
2. ✅ Add webhook delivery system
3. ✅ Implement SSRF protection
4. ✅ Add retry logic with exponential backoff
5. ✅ Support webhook authentication

#### Detailed Tasks

**1. Data Structures (Week 1)**
- [ ] Create `PushNotificationConfig` struct
- [ ] Create `PushNotificationAuthenticationInfo` struct
- [ ] Create `TaskPushNotificationConfig` struct
- [ ] Add webhook URL validation
- [ ] Implement allowed events configuration

**2. RPC Methods (Week 1-2)**
- [ ] Implement `tasks/pushNotificationConfig/set`
- [ ] Implement `tasks/pushNotificationConfig/get`
- [ ] Implement `tasks/pushNotificationConfig/list`
- [ ] Implement `tasks/pushNotificationConfig/delete`
- [ ] Add config persistence
- [ ] Add config validation

**3. Webhook Delivery (Week 2-3)**
- [ ] Create webhook delivery queue
- [ ] Implement HTTP POST to webhook URL
- [ ] Add authentication (Bearer, custom headers)
- [ ] Implement retry logic (exponential backoff)
- [ ] Add delivery status tracking
- [ ] Handle webhook timeouts

**4. Security (Week 3)**
- [ ] Implement SSRF protection
  - [ ] Disallow private IPs (10.0.0.0/8, 192.168.0.0/16, etc.)
  - [ ] Disallow localhost
  - [ ] Disallow link-local addresses
  - [ ] DNS rebinding protection
- [ ] Validate webhook URLs
- [ ] Add rate limiting for webhooks
- [ ] Implement webhook signature (HMAC)

**5. Task Integration (Week 4)**
- [ ] Trigger webhooks on task state changes
- [ ] Send artifact updates via webhook
- [ ] Include proper event payloads
- [ ] Add webhook error handling

**6. Testing (Week 4-5)**
- [ ] Add webhook delivery tests
- [ ] Add SSRF protection tests
- [ ] Add retry logic tests
- [ ] Test webhook authentication
- [ ] Add security tests
- [ ] Create webhook server example

**7. AgentCard Updates**
- [ ] Add `capabilities.pushNotifications` field
- [ ] Document webhook support

#### Success Criteria
- ✅ All 4 config methods working
- ✅ Webhooks delivered reliably
- ✅ SSRF attacks prevented
- ✅ Retry logic handles failures
- ✅ Proper authentication support

#### Estimated Timeline
**5 weeks**

---

### v0.8.0 📅 (Target: Q3 2026)
**Theme:** Optional Features & Extensions

**Priority:** Add remaining optional spec features

#### Goals
1. ✅ Implement authenticated extended card
2. ✅ Add file handling (FileWithBytes, FileWithUri)
3. ✅ Context management improvements
4. ✅ Performance optimizations

#### Detailed Tasks

**1. Authenticated Extended Card (Week 1)**
- [ ] Implement `agent/getAuthenticatedExtendedCard`
- [ ] Add authentication requirement check
- [ ] Return extended AgentCard with additional fields
- [ ] Add `supportsAuthenticatedExtendedCard` handling
- [ ] Add tests

**2. File Handling (Week 2)**
- [ ] Implement FileWithBytes (base64 encoded)
- [ ] Implement FileWithUri (URL reference)
- [ ] Add file size limits
- [ ] Add MIME type validation
- [ ] Implement file upload in FilePart
- [ ] Implement file download from URI
- [ ] Add streaming for large files
- [ ] Add file handling tests

**3. Context Management (Week 2-3)**
- [ ] Server-side contextId generation
- [ ] Context-based task grouping
- [ ] Context history management
- [ ] Add context cleanup policies
- [ ] Add context tests

**4. Performance & Polish (Week 3-4)**
- [ ] Optimize JSON-RPC parsing
- [ ] Add connection pooling
- [ ] Implement caching where appropriate
- [ ] Add metrics/telemetry hooks
- [ ] Profile and optimize hot paths
- [ ] Memory leak audits

**5. Documentation (Week 4)**
- [ ] Complete API documentation
- [ ] Add advanced examples
- [ ] Create tutorial series
- [ ] Document best practices
- [ ] Add architecture diagrams

#### Success Criteria
- ✅ All optional features working
- ✅ File handling robust
- ✅ Good performance benchmarks
- ✅ Complete documentation

#### Estimated Timeline
**4 weeks**

---

### v1.0.0 🎉 (Target: Q4 2026)
**Theme:** Production Ready & Additional Transports

**Priority:** Full spec compliance + production hardening

#### Goals
1. ✅ Implement gRPC transport
2. ✅ Consider HTTP+JSON/REST transport (if spec clarifies)
3. ✅ Security audit
4. ✅ Performance benchmarking
5. ✅ Full spec compliance verification

#### Detailed Tasks

**1. gRPC Transport (Week 1-3)**
- [ ] Obtain official .proto file from A2A spec
- [ ] Generate Rust code with tonic
- [ ] Implement all RPC methods
- [ ] Map to shared internal handlers
- [ ] Add gRPC streaming support
- [ ] Add gRPC tests
- [ ] Update AgentCard for gRPC

**2. HTTP+JSON/REST Transport (Week 3-5)** *(Conditional)*
- [ ] Verify spec defines endpoint patterns
- [ ] Implement REST endpoints per spec
- [ ] Map to shared internal handlers
- [ ] Add REST tests
- [ ] Update AgentCard for REST

**3. Security Audit (Week 6)**
- [ ] Third-party security review
- [ ] Fix any vulnerabilities
- [ ] Add security documentation
- [ ] Implement recommended hardening

**4. Performance (Week 7)**
- [ ] Comprehensive benchmarking
- [ ] Optimize critical paths
- [ ] Add performance regression tests
- [ ] Document performance characteristics

**5. Final Compliance Check (Week 8)**
- [ ] Test against official A2A test suite (if available)
- [ ] Cross-check all spec requirements
- [ ] Verify interoperability with other implementations
- [ ] Create compliance report
- [ ] Get spec maintainer feedback

**6. Release Preparation (Week 9)**
- [ ] Final documentation review
- [ ] CHANGELOG completion
- [ ] Migration guides
- [ ] Release notes
- [ ] Publish to crates.io
- [ ] Announce v1.0.0

#### Success Criteria
- ✅ 100% A2A v0.3.0 spec compliance
- ✅ Multiple transport support
- ✅ Security audit passed
- ✅ Production-ready
- ✅ Excellent documentation

#### Estimated Timeline
**9 weeks**

---

## Development Guidelines

### Code Quality Standards
- **Test Coverage:** Minimum 80% for new code
- **Documentation:** All public APIs fully documented
- **Spec References:** Link to spec sections in comments
- **Error Handling:** No unwrap() in production code
- **Async:** Tokio-based, no blocking operations

### Testing Strategy
1. **Unit Tests:** Test individual components
2. **Integration Tests:** Test RPC methods end-to-end
3. **Compliance Tests:** Validate against spec requirements
4. **Interop Tests:** Test with other A2A implementations
5. **Performance Tests:** Benchmark critical paths
6. **Security Tests:** Validate security measures

### Review Process
1. All changes require PR review
2. Must pass all tests
3. Must pass clippy with no warnings
4. Must maintain or improve coverage
5. Must update relevant documentation

---

## Tracking & Metrics

### Current Status (v0.4.0)
```
Spec Compliance:  ██████████░░░░░░░░░░  ~70%
Transport:        ████████████████████ 100% (JSON-RPC 2.0)
Core Methods:     ████████████████████ 100% (5/5 required)
Data Structures:  ██████████████░░░░░░  ~85%
Optional Methods: ░░░░░░░░░░░░░░░░░░░░   0%
Documentation:    ████████████████░░░░  ~80%
```

### Target for v0.5.0
```
Spec Compliance:  ████████████████░░░░  ~85%
Agent Metadata:   ████████████████████ 100%
Error Codes:      ████████████████████ 100%
```

### Target for v1.0.0
```
Spec Compliance:  ████████████████████ 100%
All Features:     ████████████████████ 100%
```

---

## Risk Management

### High Risk Items
1. **SSE Streaming Complexity**
   - **Risk:** W3C SSE spec is complex, edge cases
   - **Mitigation:** Start early, thorough testing, use existing SSE libraries
   
2. **Webhook Security (SSRF)**
   - **Risk:** Vulnerability if not properly protected
   - **Mitigation:** Comprehensive security review, use battle-tested patterns

3. **Spec Evolution**
   - **Risk:** A2A spec may change (currently v0.3.0)
   - **Mitigation:** Version agnostic design, feature flags

4. **Performance at Scale**
   - **Risk:** May not perform well with many concurrent streams
   - **Mitigation:** Early benchmarking, load testing, optimization

### Medium Risk Items
1. **gRPC Implementation:** Requires proto file and tonic expertise
2. **File Handling:** Large files may cause memory issues
3. **Cross-platform:** Ensure works on Linux, macOS, Windows

### Mitigation Strategies
- Incremental releases with feature flags
- Extensive testing at each phase
- Community engagement for feedback
- Regular spec compliance checks
- Performance monitoring from day one

---

## Success Metrics

### Technical Metrics
- ✅ All required spec features implemented
- ✅ Test coverage > 80%
- ✅ Zero critical security issues
- ✅ Performance meets benchmarks
- ✅ Works on all major platforms

### Community Metrics
- 📈 Usage/downloads from crates.io
- 📈 GitHub stars and contributors
- 📈 Issues resolved vs opened
- 📈 Documentation quality feedback
- 📈 Interoperability reports

---

## Resources

### Specification
- **A2A Spec:** https://github.com/a2aproject/A2A
- **Current Version:** v0.3.0
- **JSON-RPC 2.0:** https://www.jsonrpc.org/specification
- **W3C SSE:** https://html.spec.whatwg.org/multipage/server-sent-events.html

### Documentation
- **README.md** - Quick start and overview
- **MIGRATION_v0.4.md** - Migration guide from v0.3.x
- **examples/** - Working code examples
- **API Docs:** `cargo doc --open`

### Communication
- **Issues:** GitHub Issues for bugs/features
- **Discussions:** GitHub Discussions for questions
- **Spec Questions:** A2A community channels

---

## Changelog

### v0.4.0 (October 20, 2025)
- ✅ Removed non-spec A2aRouter (REST endpoints)
- ✅ Removed incomplete streaming module
- ✅ Established JSON-RPC 2.0 baseline
- ✅ All 101 tests passing
- ✅ Created migration guide

### v0.3.0 and earlier
- Initial implementation (partially spec-compliant)
- Basic client/server functionality
- See git history for details

---

**Last Updated:** October 20, 2025  
**Maintained By:** a2a-protocol team  
**License:** MIT OR Apache-2.0
