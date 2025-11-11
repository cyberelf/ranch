# A2A Protocol v0.8.0 - Advanced Webhook Features

**Theme:** Enhanced security, performance, and webhook capabilities  
**Target:** Q2 2026  
**Status:** 📋 PLANNING

## Overview

v0.8.0 focuses on hardening the webhook system with advanced security, performance optimizations, and operational features that were deferred from v0.7.0.

### Vision

Transform the webhook system from "working" to "production-grade enterprise-ready" with features that support high-scale deployments, security audits, and operational monitoring.

### Key Goals

1. **Security Hardening**
   - DNS pre-resolution with rebinding protection
   - Webhook signature verification (HMAC-SHA256)
   - OAuth2 token refresh automation
   - Rate limiting (per-webhook and global)

2. **Performance & Scale**
   - Performance testing and benchmarks
   - Persistent webhook configuration storage
   - Batch webhook deliveries
   - Circuit breaker pattern

3. **Operational Excellence**
   - Webhook delivery analytics
   - Dead letter queue for failed webhooks
   - Metrics and monitoring
   - Delivery history persistence

4. **Developer Experience**
   - AgentCard capabilities flag
   - Enhanced examples
   - Production deployment guide
   - Migration tools

---

## Priorities

### Priority 1: Security Hardening (Week 1-2)

**Goal:** Pass external security audit, prevent DNS rebinding attacks

#### 1.1 DNS Pre-Resolution Protection

**Problem:** Current implementation validates hostnames but doesn't protect against DNS rebinding attacks.

**Solution:**
```rust
// Before making HTTP request:
1. Resolve hostname → IP address
2. Validate IP address against SSRF rules
3. Cache resolution with TTL
4. Re-validate before each request
```

**Tasks:**
- [ ] Implement DNS resolver with caching
- [ ] Add IP validation after resolution
- [ ] TTL-based cache invalidation
- [ ] Tests for DNS rebinding scenarios

**Effort:** 8-12 hours  
**Risk:** Medium (DNS complexity)

#### 1.2 Webhook Signature Verification

**Problem:** Webhook receivers can't verify authenticity of incoming requests.

**Solution:**
```rust
// Server signs webhook payload
let signature = hmac_sha256(secret_key, payload);
headers.insert("X-A2A-Signature", signature);

// Client verifies
fn verify_webhook(payload: &str, signature: &str, secret: &str) -> bool {
    hmac_sha256(secret, payload) == signature
}
```

**Tasks:**
- [ ] Add `secret` field to `PushNotificationConfig`
- [ ] Implement HMAC-SHA256 signing
- [ ] Add signature to webhook headers
- [ ] Provide verification helpers
- [ ] Document signature format

**Effort:** 6-8 hours  
**Risk:** Low

#### 1.3 OAuth2 Token Refresh

**Problem:** Bearer tokens expire, requiring manual updates.

**Solution:**
```rust
pub enum PushNotificationAuth {
    Bearer { token: String },
    CustomHeaders { headers: HashMap<String, String> },
    OAuth2 {
        token_url: String,
        client_id: String,
        client_secret: String,
        scope: Option<String>,
        // Auto-refresh before expiry
    },
}
```

**Tasks:**
- [ ] Add OAuth2 variant to `PushNotificationAuth`
- [ ] Implement token refresh logic
- [ ] Cache tokens with expiry
- [ ] Tests for token refresh
- [ ] Documentation

**Effort:** 10-14 hours  
**Risk:** Medium (OAuth2 complexity)

#### 1.4 Rate Limiting

**Problem:** No protection against webhook flooding attacks.

**Solution:**
```rust
// Per-webhook rate limit
struct WebhookRateLimit {
    max_per_minute: u32,  // Default: 100
    burst: u32,           // Default: 10
}

// Global rate limit
struct GlobalRateLimit {
    max_per_minute: u32,  // Default: 1000
}

// Token bucket algorithm
```

**Tasks:**
- [ ] Implement token bucket rate limiter
- [ ] Per-webhook rate limits
- [ ] Global rate limits
- [ ] Configuration options
- [ ] Rate limit exceeded errors
- [ ] Tests for rate limiting

**Effort:** 12-16 hours  
**Risk:** Medium (distributed rate limiting)

---

### Priority 2: Performance & Scalability (Week 3-4)

**Goal:** Handle 10,000+ concurrent webhooks, persistent storage

#### 2.1 Performance Testing

**Tasks:**
- [ ] Load testing harness
- [ ] Benchmark: 100, 500, 1000, 5000, 10000 concurrent webhooks
- [ ] Memory profiling under load
- [ ] Latency measurements (p50, p95, p99)
- [ ] CPU utilization analysis
- [ ] Document performance characteristics
- [ ] Identify bottlenecks

**Metrics to track:**
- Webhook enqueue latency
- Delivery latency (excluding network)
- Memory usage per 1000 webhooks
- Max concurrent webhooks
- Queue overflow handling

**Effort:** 16-20 hours  
**Deliverable:** `PERFORMANCE.md` with benchmarks

#### 2.2 Persistent Webhook Configuration

**Problem:** In-memory storage lost on restart.

**Solution:**
```rust
pub trait PushNotificationStore {
    async fn set(&self, task_id: String, config: PushNotificationConfig) -> A2aResult<()>;
    async fn get(&self, task_id: &str) -> A2aResult<Option<PushNotificationConfig>>;
    async fn list(&self) -> A2aResult<Vec<PushNotificationConfigEntry>>;
    async fn delete(&self, task_id: &str) -> A2aResult<bool>;
}

// Implementations:
struct InMemoryStore { ... }      // v0.7.0
struct SqliteStore { ... }        // v0.8.0
struct PostgresStore { ... }      // v0.8.0 (optional)
```

**Tasks:**
- [ ] Define trait for storage backend
- [ ] SQLite implementation
- [ ] PostgreSQL implementation (optional)
- [ ] Migration from in-memory
- [ ] Tests for each backend
- [ ] Configuration options

**Effort:** 20-24 hours  
**Risk:** Low

#### 2.3 Batch Webhook Deliveries

**Problem:** Sending webhooks one-by-one inefficient for high volume.

**Solution:**
```rust
// Batch multiple webhooks to same endpoint
struct WebhookBatch {
    url: Url,
    webhooks: Vec<WebhookPayload>,
}

// Server sends array of events
POST /webhook
[
  {"event": "completed", "task": {...}},
  {"event": "completed", "task": {...}},
  {"event": "failed", "task": {...}}
]
```

**Tasks:**
- [ ] Batch aggregation logic
- [ ] Configurable batch size/timeout
- [ ] Backward compatibility (optional batching)
- [ ] Tests for batching
- [ ] Documentation

**Effort:** 10-12 hours  
**Risk:** Low

#### 2.4 Circuit Breaker Pattern

**Problem:** Repeatedly calling failing webhooks wastes resources.

**Solution:**
```rust
struct CircuitBreaker {
    state: CircuitState,  // Closed, Open, HalfOpen
    failure_threshold: u32,
    timeout: Duration,
}

// After 5 consecutive failures → Open (stop sending)
// After timeout → HalfOpen (try one request)
// If success → Closed (resume normal)
```

**Tasks:**
- [ ] Circuit breaker state machine
- [ ] Configurable thresholds
- [ ] Metrics for circuit state
- [ ] Tests for state transitions
- [ ] Documentation

**Effort:** 8-10 hours  
**Risk:** Low

---

### Priority 3: Operational Excellence (Week 5-6)

**Goal:** Production monitoring, debugging, and reliability

#### 3.1 Webhook Delivery Analytics

**Features:**
- Delivery success/failure rates
- Average delivery time
- Retry statistics
- Most active webhooks
- Error patterns

**Tasks:**
- [ ] Metrics collection infrastructure
- [ ] Prometheus-compatible metrics
- [ ] Delivery history tracking
- [ ] Dashboard example (Grafana)
- [ ] Documentation

**Effort:** 14-18 hours

#### 3.2 Dead Letter Queue

**Problem:** Failed webhooks disappear after max retries.

**Solution:**
```rust
struct DeadLetterQueue {
    storage: Box<dyn DeadLetterStore>,
}

impl DeadLetterQueue {
    async fn enqueue(&self, webhook: FailedWebhook);
    async fn list(&self) -> Vec<FailedWebhook>;
    async fn retry(&self, id: &str);
    async fn delete(&self, id: &str);
}
```

**Tasks:**
- [ ] Dead letter queue implementation
- [ ] Persistent storage (SQLite)
- [ ] Manual retry API
- [ ] Bulk operations
- [ ] Tests
- [ ] Documentation

**Effort:** 12-16 hours

#### 3.3 Delivery History Persistence

**Features:**
- Log all delivery attempts
- Searchable by task ID, timestamp, status
- Useful for debugging
- Configurable retention

**Tasks:**
- [ ] Delivery log schema
- [ ] Storage backend (SQLite)
- [ ] Query API
- [ ] Retention policy
- [ ] Tests
- [ ] Documentation

**Effort:** 10-14 hours

---

### Priority 4: Developer Experience (Week 7-8)

**Goal:** Make v0.8.0 easy to adopt and deploy

#### 4.1 AgentCard Capabilities

**✅ COMPLETED IN v0.7.0** - Originally deferred but implemented same day (Nov 11, 2025)

**Implemented:**
```rust
pub struct TransportCapabilities {
    pub push_notifications: bool,
    pub streaming: bool,
}
```

**Completed Tasks:**
- [x] Add `push_notifications: bool` capability field (from v0.7.0)
- [x] Add capabilities field to TransportCapabilities struct
- [x] Update AgentCard builder methods
- [x] Add tests for capability field serialization (7 tests)
- [x] Automatic inclusion via TaskAwareHandler
- [x] All tests passing

**Implementation Notes:**
- Located in `core/agent_card.rs`
- Automatically included in AgentCard metadata by TaskAwareHandler
- Serialized under `transportCapabilities` key in metadata
- Backward compatible (optional field, defaults to false)

**For v0.8.0:**
- [ ] Add `batch_operations: bool` capability (new for v0.8.0)
- [ ] Update examples to explicitly demonstrate capabilities
- [ ] Add to compliance tests

**Effort:** Completed (was 2-3 hours)  
**Priority:** DONE - v0.8.0 only needs to add batch_operations

#### 4.2 Enhanced Examples

**New examples:**
- `webhook_with_auth.rs` - Full authentication example
- `webhook_monitoring.rs` - Metrics and monitoring
- `high_availability.rs` - Multi-instance deployment
- `webhook_batch.rs` - Batch delivery example

**Effort:** 8-10 hours

#### 4.3 Production Deployment Guide

**New doc: `DEPLOYMENT.md`**

Topics:
- Architecture recommendations
- Database setup (SQLite vs Postgres)
- Monitoring and alerting
- Security checklist
- Performance tuning
- High availability
- Disaster recovery

**Effort:** 8-10 hours

#### 4.4 Migration Tools

**From v0.7.0 to v0.8.0:**
- Configuration migration
- Storage backend migration
- Breaking changes guide

**Effort:** 6-8 hours

---

## Breaking Changes

### Potential Breaking Changes

1. **Storage Backend**
   ```rust
   // v0.7.0 - concrete type
   TaskAwareHandler {
       push_notification_store: PushNotificationStore,
   }
   
   // v0.8.0 - trait object
   TaskAwareHandler {
       push_notification_store: Box<dyn PushNotificationStoreBackend>,
   }
   ```
   **Mitigation:** Provide adapter for in-memory store

2. **Auth Config**
   ```rust
   // v0.7.0
   PushNotificationAuth::Bearer { token: String }
   
   // v0.8.0
   PushNotificationAuth::OAuth2 { ... }  // New variant
   ```
   **Impact:** None (backward compatible)

3. **Rate Limiting**
   - New errors: `RateLimitExceeded`
   - May reject webhook configurations

**Recommendation:** Minimize breaking changes, provide migration path

---

## Timeline

### Week 1-2: Security (Dec 2025)
- [ ] DNS pre-resolution
- [ ] Webhook signatures
- [ ] OAuth2 auth
- [ ] Rate limiting

### Week 3-4: Performance (Jan 2026)
- [ ] Performance testing
- [ ] Persistent storage
- [ ] Batch deliveries
- [ ] Circuit breaker

### Week 5-6: Operations (Feb 2026)
- [ ] Analytics
- [ ] Dead letter queue
- [ ] Delivery history
- [ ] Monitoring

### Week 7-8: Polish (Mar 2026)
- [ ] AgentCard updates
- [ ] Examples
- [ ] Documentation
- [ ] Migration tools
- [ ] Release v0.8.0

**Target Release:** End of March 2026

---

## Success Metrics

### Performance
- [ ] Handle 10,000 concurrent webhooks
- [ ] <10ms p95 enqueue latency
- [ ] <100MB memory for 10k webhooks
- [ ] Zero data loss on restart (persistent storage)

### Security
- [ ] Pass external security audit
- [ ] Zero DNS rebinding vulnerabilities
- [ ] Signature verification working
- [ ] Rate limiting effective

### Quality
- [ ] 300+ total tests
- [ ] 95%+ code coverage
- [ ] All examples working
- [ ] Comprehensive documentation

### Adoption
- [ ] Production deployment guide
- [ ] Migration path from v0.7.0
- [ ] Community feedback incorporated

---

## Dependencies

### Required
- `hmac` - For webhook signatures
- `sha2` - For HMAC-SHA256
- `trust-dns-resolver` - For DNS resolution
- `sqlx` or `diesel` - For persistent storage

### Optional
- `prometheus` - For metrics
- `postgres` - For PostgreSQL backend
- `redis` - For distributed rate limiting (future)

---

## Risk Assessment

### High Risk
1. **DNS Resolution** - Complex, security-critical
   - Mitigation: Extensive testing, review RFC standards
   - Timeline: Allow extra time for security review

2. **Performance at Scale** - May reveal architectural issues
   - Mitigation: Early load testing, incremental improvements
   - Timeline: Start testing early in development

### Medium Risk
1. **OAuth2 Integration** - Many edge cases
   - Mitigation: Use well-tested libraries, follow spec
   - Fallback: Defer to v0.9.0 if complex

2. **Storage Migration** - Data loss risks
   - Mitigation: Thorough testing, backup procedures
   - Document rollback plan

### Low Risk
1. **Webhook Signatures** - Well-understood pattern
2. **Circuit Breaker** - Simple state machine
3. **AgentCard Updates** - Small change

---

## Deferred to v0.9.0+

### Features Not in v0.8.0

1. **gRPC Transport**
   - Reason: Not in immediate demand
   - Complexity: High
   - Timeline: v0.9.0 or when spec clarifies

2. **WebSocket Streaming**
   - Reason: SSE sufficient for now
   - Complexity: Medium
   - Timeline: v0.9.0 if requested

3. **Multi-Agent Orchestration**
   - Reason: Multi-agent crate handles this
   - Complexity: High
   - Timeline: v0.9.0+

4. **Advanced Routing**
   - Message routing based on content
   - Agent selection algorithms
   - Timeline: v0.9.0+

---

## Community Input

### Questions for Community

1. **Storage Backend Priority**
   - SQLite first, or PostgreSQL?
   - Need for Redis?

2. **Rate Limiting Approach**
   - Per-webhook only, or per-task-id too?
   - Global limits necessary?

3. **Batch Deliveries**
   - Opt-in or opt-out?
   - Default batch size?

4. **OAuth2 Providers**
   - Which providers to support first?
   - Auth0, Okta, Azure AD?

### Feedback Channels

- GitHub Discussions
- Discord server (if created)
- Email: maintainers@example.com

---

## Next Steps

### Before Starting v0.8.0

1. **Complete v0.7.0 Release**
   - [ ] WEBHOOKS.md guide ✅
   - [ ] AgentCard capabilities
   - [ ] Final testing
   - [ ] Release announcement

2. **Gather Community Feedback**
   - [ ] Survey existing users
   - [ ] GitHub issue triage
   - [ ] Feature requests review

3. **Security Audit**
   - [ ] Internal security review
   - [ ] Consider external audit
   - [ ] Address findings

4. **Architecture Review**
   - [ ] Review storage abstraction
   - [ ] Plan for distributed deployment
   - [ ] Consider breaking changes

### During v0.8.0 Development

1. **Weekly Progress Updates**
   - Post to GitHub Discussions
   - Update TODO_v0.8.0.md
   - Blog posts for major milestones

2. **Early Testing**
   - Alpha release for early adopters
   - Beta release for wider testing
   - RC release before final

3. **Documentation as You Go**
   - Update guides incrementally
   - Add examples when features ready
   - Keep CHANGELOG.md current

---

## Questions for Decision

### Pre-Development Decisions

1. **Storage Strategy**
   - Use trait from start, or migrate later?
   - **Recommendation:** Trait from start for flexibility

2. **Breaking Changes**
   - Allow some breaking changes for better API?
   - **Recommendation:** Minimize, provide adapters

3. **Performance Targets**
   - 10k concurrent webhooks realistic?
   - **Recommendation:** Start with 1k, optimize to 10k

4. **Security Audit**
   - Internal only, or external firm?
   - **Recommendation:** Internal for v0.8.0, external before v1.0

### Feature Prioritization

If timeline slips, what to cut?

**Must Have:**
1. DNS pre-resolution (security critical)
2. Persistent storage (data loss risk)
3. AgentCard capabilities (should have been in v0.7.0)

**Should Have:**
4. Webhook signatures (security enhancement)
5. Rate limiting (DoS protection)
6. Performance testing (validation)

**Nice to Have:**
7. OAuth2 (can defer to v0.9.0)
8. Batch deliveries (optimization)
9. Circuit breaker (reliability)
10. Analytics (operational)
11. Dead letter queue (debugging)

---

## Maintainer Notes

**Created:** 2025-11-11  
**Last Updated:** 2025-11-11  
**Status:** Planning phase  
**Next Review:** After v0.7.0 release

**Key Contacts:**
- Security Lead: TBD
- Performance Lead: TBD
- Documentation Lead: TBD

**Related Documents:**
- [TODO_v0.7.0.md](progress/TODO_v0.7.0.md) - Current release
- [WEBHOOKS.md](WEBHOOKS.md) - Webhook guide
- [FEATURES.md](FEATURES.md) - Feature overview
