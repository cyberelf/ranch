## Unimplemented A2A v0.3.0 Features

**Last Updated:** October 20, 2025  
**Crate Version:** v0.4.0  
**Target Spec:** A2A Protocol v0.3.0

This document highlights the remaining gaps between the `a2a-protocol` crate and the A2A v0.3.0 specification. Use it alongside the detailed schedule in [IMPLEMENTATION_ROADMAP.md](IMPLEMENTATION_ROADMAP.md).

---

## Quick Status

| Area | Status | Notes |
|------|--------|-------|
| JSON-RPC Transport | ✅ Complete | Spec-compliant request/response handling |
| Core Message/Part Types | ✅ Complete | `messageId`, `taskId`, `contextId`, and part discriminators implemented |
| AgentCard Metadata | 🚧 Partial | Missing `defaultInputModes`, `defaultOutputModes`, `supportsAuthenticatedExtendedCard`, and strong transport enum |
| Spec Error Codes | 🚧 Partial | Custom JSON-RPC codes `-32001` → `-32007` not mapped yet |
| Streaming (SSE) | ❌ Missing | `message/stream`, `task/resubscribe`, event types |
| Push Notifications | ❌ Missing | `tasks/pushNotificationConfig/*` RPC methods and webhook delivery |
| Authenticated Extended Card | ❌ Missing | `agent/getAuthenticatedExtendedCard` handler and card metadata |
| Additional Transports | ❌ Missing | HTTP+JSON/REST and gRPC transports |
| Observability & Ops | ⚠️ Minimal | Logging, metrics, tracing, health endpoints |

**Overall JSON-RPC compliance:** ~70% of spec features  
**Tests passing:** 101

---

## 1. AgentCard Metadata Gaps (MUST)

- `defaultInputModes: Vec<String>` (MIME types supported for inbound content)
- `defaultOutputModes: Vec<String>` (MIME types for outputs)
- `supportsAuthenticatedExtendedCard: bool`
- Strongly typed `preferredTransport` enum (`JSONRPC`, `GRPC`, `HTTP+JSON`) and validation for `additionalInterfaces`
- Documentation + tests covering new fields

**Impact:** Without these fields the card fails strict validation against the spec schema.

---

## 2. A2A-Specific Error Codes (MUST)

JSON-RPC mapping currently returns generic server errors. Implement explicit codes and data payloads for:

- `-32001` `TaskNotFoundError`
- `-32002` `TaskNotCancelableError`
- `-32003` `PushNotificationNotSupportedError`
- `-32004` `UnsupportedOperationError`
- `-32005` `ContentTypeNotSupportedError`
- `-32006` `InvalidAgentResponseError`
- `-32007` `AuthenticatedExtendedCardNotConfiguredError`

**Impact:** Clients cannot reliably distinguish recoverable vs fatal conditions.

---

## 3. Streaming (SSE) Support (MAY)

Required to implement the optional streaming paths in the spec:

- `message/stream` RPC method producing W3C SSE responses
- `task/resubscribe` RPC method with Last-Event-ID semantics
- Event payloads: `TaskStatusUpdateEvent`, `TaskArtifactUpdateEvent`
- Client helpers for consuming SSE streams
- AgentCard capability flags for streaming

**Workaround:** Poll `task/status` + `task/get` until SSE landing.

---

## 4. Push Notification Webhooks (MAY)

- `tasks/pushNotificationConfig/set|get|list|delete`
- Persisted configuration with SSRF protection and validation
- Webhook dispatcher with retries, authentication, and observability
- AgentCard capability disclosure

**Impact:** Agents cannot proactively notify clients of task updates.

---

## 5. Authenticated Extended Card (MAY)

- `agent/getAuthenticatedExtendedCard` endpoint
- Storage for extended metadata
- Access control hooks and `supportsAuthenticatedExtendedCard` flag

**Workaround:** Rely on unauthenticated `agent/card`.

---

## 6. Additional Transports (MAY)

- **gRPC:** Proto definitions, tonic-based server/client, streaming compatibility
- **HTTP+JSON/REST:** Endpoint layout per spec, request/response wrappers, tests

**Current stance:** JSON-RPC transport satisfies the “one transport” requirement; other transports prioritized after v0.5.0.

---

## 7. Advanced File Handling (SHOULD)

- Size limits and validation for `FileWithBytes`
- MIME type validation utilities
- Streaming/downloading helpers for `FileWithUri`
- Conformance tests for large payloads

---

## 8. Observability & Operations (SHOULD)

- Structured logging across server and client
- Metrics + tracing (OpenTelemetry)
- Health/readiness endpoints for server integrations
- Error telemetry dashboards / hooks

---

## 9. Performance & Scaling Enhancements (NICE TO HAVE)

- Connection pooling, request pipelining, compression
- Task store persistence and sharding
- Batch request handling on the server side

---

## Quick Win Suggestions

1. Finish AgentCard fields + validation suite
2. Implement JSON-RPC error code mapping with targeted tests
3. Add compliance fixtures to lock down message/part serialization (regression guard)

---

**Maintained by:** a2a-protocol team  
**Next planned update:** After v0.5.0 feature work lands
