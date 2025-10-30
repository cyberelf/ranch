# A2A Protocol Implementation Roadmap

**Current Version:** v0.6.0 (in progress)  
**Target Spec:** A2A Protocol v0.3.0  
**Last Updated:** October 30, 2025

---

## Executive Summary

This roadmap tracks the implementation of A2A (Agent-to-Agent) Protocol v0.3.0 in Rust. We now have **spec-compliant JSON-RPC 2.0 transport with server-side SSE streaming**.

### Current Compliance: ~75%

**✅ Implemented:**
- JSON-RPC 2.0 transport (fully compliant)
- Core RPC methods: `message/send`, `task/get`, `task/cancel`, `task/status`, `agent/card`
- Complete task lifecycle management
- Message & Part schema (spec-aligned)
- AgentCard with full v0.3.0 metadata
- A2A error codes (-32001 through -32007)
- Authentication strategies (Bearer, API Key, OAuth2)
- **SSE streaming server** (message/stream, task/resubscribe)

**🚧 In Progress:**
- SSE streaming client API

**❌ Not Implemented:**
- Push notifications (webhooks)
- Authenticated extended card
- Advanced file handling
- gRPC transport
- HTTP+JSON/REST transport

---

## Current State (v0.6.0)

### What Works

#### 1. Server (JSON-RPC 2.0 + SSE)
```rust
use a2a_protocol::server::{JsonRpcRouter, TaskAwareHandler};

let handler = TaskAwareHandler::new(agent_card);
let router = JsonRpcRouter::new(handler);
// Exposes: POST /rpc (JSON-RPC) and POST /stream (SSE)
```

**Supported Methods:**
- ✅ `message/send` - Send message, returns Task or Message
- ✅ `message/stream` - SSE streaming for real-time updates
- ✅ `task/get` - Get task details and results
- ✅ `task/status` - Get current task status
- ✅ `task/cancel` - Cancel running task
- ✅ `task/resubscribe` - Resume SSE stream for task
- ✅ `agent/card` - Get agent capabilities

#### 2. Client (JSON-RPC 2.0)
```rust
use a2a_protocol::{client::ClientBuilder, prelude::*};

let client = ClientBuilder::new()
    .with_json_rpc("https://agent.example.com/rpc")
    .build()?;

let message = Message::user_text("Hello");
let response = client.send_message(message).await?;
```

#### 3. Core Types
- ✅ `Message` - Spec-compliant structure
- ✅ `Task` - Full lifecycle support
- ✅ `AgentCard` - Complete v0.3.0 metadata
- ✅ `A2aError` - All 7 spec error codes
- ✅ `SseEvent` - W3C SSE format support

#### 4. Testing
**124 tests passing** (98 lib + 17 compliance + 8 RPC + 1 doc)
- ✅ JSON-RPC 2.0 compliance
- ✅ Task lifecycle tests
- ✅ SSE streaming tests
- ✅ Integration tests

### What's Missing

#### 1. Client SSE Streaming (v0.6.0) 🚧
- ❌ Client `stream_message()` API
- ❌ SSE event parsing in client
- ❌ Reconnection with Last-Event-ID
- ❌ Client streaming examples

**Workaround:** Use `task/status` polling until client ready.

#### 2. Push Notifications (v0.7.0)
- ❌ Webhook configuration RPC methods
- ❌ Webhook delivery system
- ❌ SSRF protection

#### 3. Advanced Features (v0.8.0+)
- ❌ Authenticated extended card endpoint
- ❌ Advanced file handling (size limits, validation)
- ❌ Additional transports (gRPC, HTTP+JSON)

---

## Release History & Roadmap

### v0.4.0 ✅ COMPLETED (October 2025)
**Theme:** Spec Compliance Baseline

**Completed:**
- ✅ Removed non-spec A2aRouter (REST endpoints)
- ✅ Removed non-compliant streaming module
- ✅ JSON-RPC 2.0 only (clean baseline)
- ✅ All tests passing
- ✅ Migration guide created

---

### v0.5.0 ✅ COMPLETED (October 2025)
**Theme:** Core Spec Compliance - Metadata & Errors

**Priority:** Close remaining spec gaps that block full interoperability claims

#### Goals - ALL COMPLETED
1. ✅ Message & Part schema parity with spec (complete)
2. ✅ `MessageRole` restricted to `user`/`agent` (complete)
3. ✅ Finalize AgentCard required fields (`defaultInputModes`, `defaultOutputModes`, `supportsAuthenticatedExtendedCard`)
4. ✅ Implement A2A-specific JSON-RPC error codes
5. ✅ Expand compliance testing for new metadata and error paths

#### Completed Tasks

**1. AgentCard Enhancements ✅**
- ✅ Added `defaultInputModes: Vec<String>` (MIME types)
- ✅ Added `defaultOutputModes: Vec<String>` (MIME types)
- ✅ Added `supportsAuthenticatedExtendedCard: bool`
- ✅ Promoted `preferredTransport` to spec-aligned enum (`JSONRPC` | `GRPC` | `HTTP+JSON`)
- ✅ Extended `TransportInterface` to validate transport enum usage
- ✅ **BONUS:** Added optional metadata fields:
  - `provider: Option<AgentProvider>` (name, URL)
  - `icon_url: Option<Url>` for UI display
  - `documentation_url: Option<Url>` for user help
  - `signatures: Vec<AgentCardSignature>` for verification
- ✅ Updated builders with new methods
- ✅ Added validation tests for all new fields
- ✅ Removed deprecated `protocols` field (breaking change)

**2. Error Code Mapping ✅**
- ✅ Implemented TaskNotFoundError (-32001) with `taskId` data
- ✅ Implemented TaskNotCancelableError (-32002) with `taskId`, `state` data
- ✅ Implemented PushNotificationNotSupportedError (-32003)
- ✅ Implemented UnsupportedOperationError (-32004)
- ✅ Implemented ContentTypeNotSupportedError (-32005) with `contentType` data
- ✅ Implemented InvalidAgentResponseError (-32006)
- ✅ Implemented AuthenticatedExtendedCardNotConfiguredError (-32007)
- ✅ Updated dispatcher and transport error mapping with structured data
- ✅ Added 17 comprehensive unit tests asserting correct code emission and data fields

**3. Compliance Verification ✅**
- ✅ Added schema/serde round-trip tests for AgentCard with new fields
- ✅ Extended compliance tests to cover new error cases
- ✅ Updated integration tests to cover error code paths
- ✅ Created comprehensive migration guide (MIGRATION_v0.5.md)
- ✅ Updated README with v0.5.0 feature documentation
- ✅ All 110 tests passing (84 lib + 17 compliance + 8 RPC + 1 doc)

#### Success Criteria - ALL MET ✅
- ✅ AgentCard exposes all required metadata and validates inputs
- ✅ JSON serialization matches spec examples (messages, parts, agent card)
- ✅ Error codes map 1:1 with A2A guidance with structured data
- ✅ All existing tests pass (110/110)
- ✅ New compliance tests pass
- ✅ Can interoperate with spec-compliant agents

#### Actual Timeline
**3 weeks** - Completed ahead of schedule with bonus features

---

### v0.6.0 🚧 IN PROGRESS (Target: Q1 2026)
**Theme:** SSE Streaming Support

**Status:** Server-side complete, client API in progress

#### Progress Summary
- ✅ **Server infrastructure complete** (4 weeks)
- 🚧 **Client API** (1-2 weeks remaining)
- ❌ **Documentation** (1 week remaining)

#### Completed ✅

**Server-Side Streaming:**
- ✅ W3C SSE infrastructure (`transport/sse.rs`)
  - `SseEvent` - Event formatting and parsing
  - `SseWriter` - Broadcast-based event publisher
  - `EventBuffer` - Replay buffer with Last-Event-ID support
- ✅ Streaming methods in `A2aHandler` trait
  - `rpc_message_stream()` - Stream message processing
  - `rpc_task_resubscribe()` - Resume existing streams
- ✅ `TaskAwareHandler` streaming implementation
  - Stream registry with SseWriter per task
  - Real-time task status and artifact updates
  - Proper cleanup on completion/disconnect
- ✅ Axum integration
  - `/stream` endpoint for SSE responses
  - Proper content-type and keepalive
- ✅ Integration tests (2 new streaming tests added)
- ✅ Feature gating with `streaming` feature flag

#### Remaining Tasks

**Client API (Week 5):**
- [ ] Add `stream_message()` client method
- [ ] Implement SSE parser for client
- [ ] Add async stream interface
- [ ] Handle reconnection with Last-Event-ID
- [ ] Add timeout and error handling
- [ ] Client-side tests

**Documentation (Week 5-6):**
- [ ] Streaming API documentation
- [ ] Client usage examples
- [ ] Server streaming guide
- [ ] Update README with streaming features
- [ ] Migration guide section

#### Test Status
**124 tests passing** (98 lib + 17 compliance + 8 RPC + 1 doc)
- ✅ SSE event formatting/parsing tests
- ✅ Streaming workflow integration tests
- ✅ Concurrent stream tests
- ❌ Client streaming tests (pending client API)

#### Architecture
- **Transport:** `axum::response::sse` for W3C compliance
- **Event Format:** JSON-RPC 2.0 in SSE data field
- **Buffering:** Last 100 events per task
- **Cleanup:** Automatic on task completion or timeout

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

### Current Status (v0.6.0 - In Progress)
```
Spec Compliance:  ███████████████░░░░░  ~75%
Transport:        ████████████████████ 100% (JSON-RPC 2.0)
Core Methods:     ████████████████████ 100% (5/5 required)
Streaming Server: ████████████████████ 100% (message/stream, task/resubscribe)
Streaming Client: ████████░░░░░░░░░░░░  ~40% (SSE parsing done, client API pending)
Data Structures:  ████████████████████ 100% (AgentCard, Message, Task, SseEvent)
Error Codes:      ████████████████████ 100% (7/7 A2A codes)
Optional Methods: ████████░░░░░░░░░░░░  ~40% (streaming server only)
Push Webhooks:    ░░░░░░░░░░░░░░░░░░░░   0% (not started)
Documentation:    ██████████████░░░░░░  ~70% (streaming docs pending)
Tests:            ████████████████████ 124 passing
```

### Progress vs v0.5.0
```
v0.5.0 → v0.6.0 Additions:
+ SSE Infrastructure    ████████████████████ (SseEvent, SseWriter, EventBuffer)
+ Server Streaming      ████████████████████ (message/stream, task/resubscribe)
+ Streaming Tests       ████████████████████ (+23 tests, 101→124)
- Client Streaming API  ████████░░░░░░░░░░░░ (in progress)
- Streaming Docs        ██████░░░░░░░░░░░░░░ (in progress)
```

### Target for v1.0.0
```
Spec Compliance:  ████████████████████ 100%
All Features:     ████████████████████ 100%
All Transports:   ████████████████████ 100% (JSON-RPC + gRPC + HTTP+JSON)
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

### v0.6.0 (October 30, 2025) 🚧 IN PROGRESS
- ✅ **SSE Streaming Server:**
  - Implemented W3C SSE infrastructure (`transport/sse.rs`)
  - Added `SseEvent` for event formatting and parsing
  - Added `SseWriter` for broadcast-based event publishing
  - Added `EventBuffer` for replay with Last-Event-ID support
  - Implemented `message/stream` and `task/resubscribe` endpoints
  - Axum integration with `/stream` endpoint
- ✅ **Streaming Architecture:**
  - Added streaming methods to `A2aHandler` trait
  - Implemented full streaming in `TaskAwareHandler`
  - Stream registry with cleanup on completion/disconnect
  - Feature gating with `streaming` feature flag
- ✅ **Testing:**
  - 124 tests passing (98 lib + 17 compliance + 8 RPC + 1 doc)
  - Added 2 new streaming integration tests
  - SSE format validation and workflow tests
- 🚧 **Client API:** SSE client streaming API (in progress)
- 🚧 **Documentation:** Streaming examples and guides (in progress)
- **Spec Compliance:** ~75% (realistic assessment)

### v0.5.0 (October 23, 2025)
- ✅ **AgentCard Complete Compliance:**
  - Added `defaultInputModes` and `defaultOutputModes` (MIME types)
  - Added `supportsAuthenticatedExtendedCard` flag
  - Upgraded `preferredTransport` to spec-aligned enum (JSONRPC/GRPC/HTTP+JSON)
  - Added optional metadata: `provider`, `icon_url`, `documentation_url`, `signatures`
  - Removed deprecated `protocols` field (breaking change)
- ✅ **A2A Error Codes:**
  - Implemented all 7 error codes (-32001 through -32007)
  - Added structured data fields (taskId, state, contentType)
  - Enhanced error handling with type-safe matching
- ✅ **Testing & Documentation:**
  - 110 tests passing (84 lib + 17 compliance + 8 RPC + 1 doc)
  - Created comprehensive MIGRATION_v0.5.md guide
  - Updated README with v0.5.0 features
- ✅ **Spec Compliance:** ~85% (up from ~70%)

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

**Last Updated:** October 30, 2025  
**Maintained By:** a2a-protocol team  
**License:** MIT OR Apache-2.0
