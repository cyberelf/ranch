# A2A Protocol v0.7.0 - Push Notifications

# A2A Protocol v0.7.0 - Push Notifications
**Theme:** Async updates via webhooks with security  
**Target:** Q1 2026  
**Status:** 🎯 CORE COMPLETE - 5/7 priorities done (Storage, SSRF, RPC, Delivery, Integration)

## v0.7.0 - Push Notifications & Webhooks

## Overview

Implement the complete push notification system as defined in the A2A Protocol v0.3.0 specification, enabling agents to receive asynchronous updates via webhooks instead of polling.

### Key Goals
1. ✅ Implement all 4 `tasks/pushNotificationConfig/*` RPC methods
2. ✅ Build robust webhook delivery system with retry logic
3. ✅ Implement comprehensive SSRF protection
4. ✅ Support multiple webhook authentication methods (Bearer + CustomHeaders)
5. ⏳ Add webhook configuration persistence (in-memory complete, persistent future)

### Success Criteria
- [x] Core push notification types defined and tested
- [x] All push notification config methods working
- [x] Webhooks delivered reliably with retry logic
- [x] SSRF attacks prevented (comprehensive validation)
- [x] Support Bearer token and custom header authentication
- [x] High test coverage for webhook system (223 total tests)
- [x] Production-ready core implementation

### Progress
**Completed:**
- ✅ Core data structures: `PushNotificationConfig`, `PushNotificationAuth`, `TaskEvent`
- ✅ Basic validation (HTTPS requirement, non-empty events)
- ✅ Comprehensive SSRF protection (IPv4, IPv6, hostnames)
- ✅ PushNotificationStore (in-memory with full CRUD)
- ✅ All 4 RPC methods implemented and tested
- ✅ WebhookQueue with async delivery
- ✅ Retry logic with exponential backoff
- ✅ Authentication support (Bearer, CustomHeaders)
- ✅ Task integration with automatic triggering
- ✅ WebhookPayload format with event, task, timestamp, agentId
- ✅ 223 total tests (62 new tests for push notifications)
- ✅ Serialization/deserialization support
- ✅ Documentation with examples

**Next Steps:**
- Usage examples (webhook server, webhook client)
- WEBHOOKS.md comprehensive guide
- Performance testing
- External security audit
- Final documentation polish

---

## Priority 1: Core Data Structures (Week 1)

### 1.1 Push Notification Configuration
- [x] Create `PushNotificationConfig` struct
  - [x] `url: Url` - Webhook endpoint
  - [x] `events: Vec<TaskEvent>` - Events to trigger notifications
  - [x] `authentication: Option<PushNotificationAuth>` - Auth config
  - [x] Validation for webhook URLs (basic - HTTPS check)
  - [x] Serialization/deserialization with serde
  - [ ] SSRF protection (validate against private IP ranges)

### 1.2 Authentication Support
- [x] Create `PushNotificationAuth` enum
  - [x] `Bearer { token: String }` variant
  - [x] `CustomHeaders { headers: HashMap<String, String> }` variant
  - [ ] `OAuth2 { ... }` variant (optional for v0.7.0)
  - [x] Secure storage considerations (no plaintext in logs)

### 1.3 Task Event Types
- [x] Create `TaskEvent` enum
  - [x] `StatusChanged` - All state transitions
  - [x] `ArtifactAdded` - New artifacts
  - [x] `Completed` - Task completion
  - [x] `Failed` - Task failure
  - [x] `Cancelled` - Task cancellation
  - [x] `matches_transition()` helper method
  - [x] Serialization for webhook payloads

### 1.4 Storage
- [ ] Define `PushNotificationStore` trait
  - [ ] `async fn set(task_id, config) -> Result<()>`
  - [ ] `async fn get(task_id) -> Result<Option<Config>>`
  - [ ] `async fn list(filter?) -> Result<Vec<Config>>`
  - [ ] `async fn delete(task_id) -> Result<bool>`
- [ ] Implement in-memory store for v0.7.0
- [ ] Design for future persistent stores (SQLite, Postgres)

**Tests:** 9/15+ unit tests completed for data structures and validation
- [x] Config creation and validation
- [x] Authentication variants
- [x] Event matching logic
- [x] Serialization/deserialization
- [ ] SSRF protection tests
- [ ] Storage trait tests

---

## Priority 2: JSON-RPC Methods (Week 2)

### 2.1 tasks/pushNotificationConfig/set
- [ ] Implement RPC handler in `A2aHandler` trait
  - [ ] Parse and validate `PushNotificationConfig` params
  - [ ] Validate webhook URL (HTTPS required, no private IPs)
  - [ ] Store configuration in store
  - [ ] Return success confirmation
- [ ] Error handling:
  - [ ] Invalid URL format
  - [ ] SSRF-vulnerable URLs
  - [ ] Invalid event types
  - [ ] Storage failures

### 2.2 tasks/pushNotificationConfig/get
- [ ] Implement RPC handler
  - [ ] Accept `taskId` parameter
  - [ ] Retrieve config from store
  - [ ] Return config or `null` if not set
- [ ] Error handling:
  - [ ] Task not found
  - [ ] Storage failures

### 2.3 tasks/pushNotificationConfig/list
- [ ] Implement RPC handler
  - [ ] Optional filters (by event type, active status, etc.)
  - [ ] Pagination support (page, perPage)
  - [ ] Return array of configs with metadata
- [ ] Error handling:
  - [ ] Invalid filter parameters
  - [ ] Storage failures

### 2.4 tasks/pushNotificationConfig/delete
- [ ] Implement RPC handler
  - [ ] Accept `taskId` parameter
  - [ ] Delete config from store
  - [ ] Return boolean success/failure
- [ ] Error handling:
  - [ ] Task not found (still returns true per spec)
  - [ ] Storage failures

### 2.5 Integration with TaskAwareHandler
- [ ] Add push notification support to `TaskAwareHandler`
  - [ ] Implement all 4 RPC methods
  - [ ] Integrate with task lifecycle
  - [ ] Trigger webhooks on relevant task events

**Tests:** 20+ integration tests for all RPC methods

---

## Priority 3: Webhook Delivery System (Week 3)

### 3.1 Webhook Delivery Queue
- [ ] Create `WebhookQueue` struct
  - [ ] Async queue for webhook deliveries
  - [ ] Priority handling (optional)
  - [ ] Concurrent delivery (tokio tasks)
  - [ ] Graceful shutdown

### 3.2 HTTP Delivery
- [ ] Create `WebhookDelivery` module
  - [ ] POST request to webhook URL
  - [ ] JSON payload formatting per spec
  - [ ] Timeout configuration (default 30s)
  - [ ] Connection pooling via reqwest
  - [ ] TLS verification (no self-signed certs)

### 3.3 Authentication
- [ ] Implement `add_authentication()` helper
  - [ ] Bearer token in Authorization header
  - [ ] Custom headers injection
  - [ ] OAuth2 token refresh (if implemented)

### 3.4 Retry Logic
- [ ] Exponential backoff strategy
  - [ ] Initial delay: 1 second
  - [ ] Max retries: 5 attempts
  - [ ] Backoff multiplier: 2x
  - [ ] Max delay: 60 seconds
  - [ ] Jitter to prevent thundering herd
- [ ] Retry on specific HTTP errors:
  - [ ] 5xx server errors
  - [ ] Network timeouts
  - [ ] Connection failures
- [ ] Don't retry on:
  - [ ] 4xx client errors (except 429 rate limit)
  - [ ] Invalid URLs
  - [ ] SSRF violations

### 3.5 Delivery Status Tracking
- [ ] Create `DeliveryStatus` enum
  - [ ] `Pending`, `Delivered`, `Failed`, `Retrying`
- [ ] Track delivery attempts per webhook
- [ ] Optional: Store delivery history for debugging
- [ ] Metrics/logging for monitoring

**Tests:** 25+ tests for delivery, retries, and error handling

---

## Priority 4: Security - SSRF Protection (Week 4)

### 4.1 URL Validation
- [ ] Create `validate_webhook_url()` function
  - [ ] HTTPS required (no HTTP)
  - [ ] Disallow private IPv4 ranges:
    - [ ] 10.0.0.0/8
    - [ ] 172.16.0.0/12
    - [ ] 192.168.0.0/16
    - [ ] 127.0.0.0/8 (localhost)
  - [ ] Disallow private IPv6 ranges:
    - [ ] ::1 (localhost)
    - [ ] fc00::/7 (unique local)
    - [ ] fe80::/10 (link-local)
  - [ ] Disallow link-local addresses
  - [ ] Disallow broadcast addresses
  - [ ] Disallow AWS metadata endpoint (169.254.169.254)

### 4.2 DNS Resolution Protection
- [ ] Pre-resolve DNS before making requests
  - [ ] Resolve hostname to IP
  - [ ] Re-validate IP address after resolution
  - [ ] Prevent DNS rebinding attacks
  - [ ] Cache resolutions with TTL

### 4.3 Rate Limiting
- [ ] Per-webhook rate limits
  - [ ] Max 100 deliveries per minute per webhook
  - [ ] Configurable limits
  - [ ] Token bucket algorithm
- [ ] Global rate limits
  - [ ] Max 1000 deliveries per minute system-wide
  - [ ] Prevent webhook flood attacks

### 4.4 Additional Security
- [ ] Webhook signature (HMAC-SHA256)
  - [ ] Optional: Sign payloads with secret key
  - [ ] Include signature in `X-A2A-Signature` header
  - [ ] Allow webhook receivers to verify authenticity
- [ ] Size limits
  - [ ] Max webhook payload size: 1MB
  - [ ] Prevent memory exhaustion
- [ ] Timeout enforcement
  - [ ] Hard timeout: 30 seconds per delivery
  - [ ] Prevent hanging connections

**Tests:** 30+ security tests covering all SSRF scenarios

---

## Priority 5: Task Integration (Week 5)

### 5.1 Event Triggers
- [ ] Modify `TaskAwareHandler` to trigger webhooks
  - [ ] On task state changes (queued → working → completed/failed)
  - [ ] On artifact additions
  - [ ] On task cancellation
  - [ ] Check for configured webhooks before triggering

### 5.2 Payload Format
- [ ] Define webhook payload schema per A2A spec
  - [ ] `event: TaskEvent` - Event type
  - [ ] `task: Task` - Full task object
  - [ ] `timestamp: DateTime` - Event timestamp
  - [ ] `agentId: String` - Sending agent ID
- [ ] JSON serialization with proper field names

### 5.3 Error Handling
- [ ] Webhook delivery failures should not block task processing
  - [ ] Fire-and-forget with background retry
  - [ ] Log failures but continue task execution
  - [ ] Optional: Dead letter queue for failed webhooks

### 5.4 Cleanup
- [ ] Delete webhook configs when tasks complete (optional)
- [ ] Auto-expire old webhook configs (optional)
- [ ] Cleanup retry queue on shutdown

**Tests:** 15+ integration tests for event triggering

---

## Priority 6: AgentCard Updates (Week 6)

### 6.1 Capabilities Flag
- [ ] Add to `AgentCard`:
  ```rust
  pub struct AgentCard {
      // ... existing fields
      pub capabilities: AgentCapabilities,
  }
  
  pub struct AgentCapabilities {
      pub push_notifications: bool,
      // ... other capabilities
  }
  ```
- [ ] Set `push_notifications: true` for agents supporting webhooks
- [ ] Update builders and serialization

### 6.2 Documentation
- [ ] Document webhook support in agent card
- [ ] Add example agent cards with push notifications
- [ ] Update compliance tests

**Tests:** 5+ tests for agent card updates

---

## Priority 7: Examples & Documentation (Week 7)

### 7.1 Examples
- [ ] `examples/webhook_server.rs` - Agent that receives webhooks
  - [ ] Simple HTTP server to receive webhooks
  - [ ] Signature verification
  - [ ] Event logging
- [ ] `examples/webhook_client.rs` - Agent that configures webhooks
  - [ ] Set webhook config
  - [ ] Trigger task that sends webhooks
  - [ ] List and delete configs
- [ ] `examples/webhook_integration.rs` - End-to-end demo
  - [ ] Two agents communicating via webhooks
  - [ ] Real-time updates demonstration

### 7.2 Documentation
- [ ] Update README.md
  - [ ] Add push notifications to feature list
  - [ ] Quick start for webhooks
- [ ] Update GETTING_STARTED.md
  - [ ] Webhook configuration tutorial
  - [ ] Security best practices
- [ ] Create WEBHOOKS.md guide
  - [ ] Detailed webhook usage
  - [ ] Security considerations
  - [ ] Troubleshooting common issues
  - [ ] Payload format reference
- [ ] Update API documentation
  - [ ] Document all webhook-related types
  - [ ] Add usage examples to docstrings

**Tests:** 10+ doc tests embedded in examples

---

## Priority 8: Testing & Quality (Week 8)

### 8.1 Unit Tests
- [ ] Data structure tests (15+ tests)
- [ ] RPC method tests (20+ tests)
- [ ] URL validation tests (20+ tests)
- [ ] Delivery tests (15+ tests)

### 8.2 Integration Tests
- [ ] End-to-end webhook delivery (10+ tests)
- [ ] Retry logic (10+ tests)
- [ ] SSRF protection (20+ tests)
- [ ] Authentication (10+ tests)

### 8.3 Security Tests
- [ ] SSRF attack scenarios (15+ tests)
- [ ] Rate limiting (10+ tests)
- [ ] Input validation (15+ tests)

### 8.4 Performance Tests
- [ ] Load testing with 1000+ concurrent webhooks
- [ ] Memory usage under sustained load
- [ ] Retry queue performance

### 8.5 Compliance Tests
- [ ] Verify against A2A spec examples
- [ ] Interoperability with other implementations

**Target:** 150+ new tests, bringing total to 310+ tests

---

## Timeline & Milestones

### Week 1-2: Foundation (Nov 11-24, 2025)
- [ ] Data structures complete
- [ ] RPC methods implemented
- [ ] Basic storage working

### Week 3-4: Delivery & Security (Nov 25 - Dec 8, 2025)
- [ ] Webhook delivery system operational
- [ ] SSRF protection validated
- [ ] Retry logic tested

### Week 5-6: Integration & Polish (Dec 9-22, 2025)
- [ ] Task integration complete
- [ ] AgentCard updated
- [ ] Security audit passed

### Week 7-8: Documentation & Release (Dec 23, 2025 - Jan 5, 2026)
- [ ] Examples working
- [ ] Documentation complete
- [ ] All tests passing (310+)
- [ ] Ready for release

**Estimated Completion:** Early January 2026

---

## Breaking Changes

None expected. This is a pure feature addition.

---

## Future Enhancements (v0.8.0+)

- Webhook signature verification on receiver side
- OAuth2 token refresh automation
- Persistent webhook config storage (SQLite/Postgres)
- Webhook delivery analytics dashboard
- Batch webhook deliveries
- Custom retry policies per webhook
- Circuit breaker pattern for failing webhooks

---

## Dependencies

### New Dependencies
- None required (using existing reqwest, tokio, serde)

### Optional Dependencies
- SQLite/Postgres for persistent storage (future)
- Metrics library for monitoring (future)

---

## Risk Assessment

### High Risk
1. **SSRF Vulnerabilities** - Critical security risk
   - Mitigation: Comprehensive validation and testing
   - External security audit before release

2. **Webhook Flood Attacks** - Resource exhaustion
   - Mitigation: Rate limiting and queue size limits
   - Load testing with realistic scenarios

### Medium Risk
1. **DNS Rebinding** - Complex attack vector
   - Mitigation: Pre-resolve and re-validate IPs
   - Reference existing security libraries

2. **Retry Storm** - Cascading failures
   - Mitigation: Exponential backoff with jitter
   - Circuit breaker pattern

### Low Risk
1. **Storage Performance** - In-memory should be fast
   - Mitigation: Profile and optimize if needed
   - Design for future persistent stores

---

## Metrics for Success

- [ ] 310+ total tests passing (150+ new)
- [ ] 90%+ code coverage for webhook module
- [ ] Zero SSRF vulnerabilities in security audit
- [ ] <100ms p95 latency for webhook delivery (excluding network)
- [ ] Successfully handles 1000+ concurrent webhooks
- [ ] All 4 RPC methods spec-compliant
- [ ] Comprehensive documentation with 3+ examples

---

## Notes

- Focus on security first, performance second
- Start with in-memory storage, design for persistence
- Keep webhook payloads spec-compliant
- Extensive testing is critical for security
- Consider hiring security expert for SSRF audit

**Maintainer:** a2a-protocol team  
**Created:** 2025-11-05  
**Target Release:** v0.7.0 - January 2026
