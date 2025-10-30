# Unimplemented A2A v0.3.0 Features

**Last Updated:** October 30, 2025  
**Crate Version:** v0.6.0 (in progress)  
**Target Spec:** A2A Protocol v0.3.0

---

## Quick Status

| Feature | Status | Priority |
|---------|--------|----------|
| JSON-RPC Transport | ✅ Complete | - |
| Core Types (Message, Task, Part) | ✅ Complete | - |
| AgentCard Metadata | ✅ Complete | - |
| A2A Error Codes | ✅ Complete | - |
| **SSE Streaming (server)** | ✅ Complete | **v0.6.0** |
| **SSE Streaming (client)** | 🚧 In Progress | **v0.6.0** |
| Push Notifications | ❌ Not Started | v0.7.0 |
| Authenticated Extended Card | ❌ Not Started | v0.8.0 |
| File Handling | ⚠️ Basic Only | v0.8.0 |
| gRPC Transport | ❌ Not Started | v1.0.0 |
| HTTP+JSON/REST Transport | ❌ Not Started | v1.0.0 |

**Overall Spec Compliance:** ~75%  
**Tests Passing:** 124 (98 lib + 17 compliance + 8 RPC + 1 doc)

---

## What's Missing

### 1. SSE Streaming Client (v0.6.0) 🚧
**Server implemented, client in progress**

- ✅ Server: `message/stream` and `task/resubscribe` endpoints
- ✅ Server: SSE infrastructure (SseEvent, SseWriter, EventBuffer)
- ✅ Server: Axum integration with `/stream` endpoint
- ❌ Client: SSE parsing and async stream interface
- ❌ Client: Reconnection with Last-Event-ID
- ❌ Documentation and examples

**Workaround:** Poll `task/status` and `task/get` until client ready.

---

### 2. Push Notifications (v0.7.0) ❌

- `tasks/pushNotificationConfig/set|get|list|delete` RPC methods
- Webhook delivery with retries and authentication
- SSRF protection

**Impact:** Agents cannot proactively push updates to clients.

---

### 3. Authenticated Extended Card (v0.8.0) ❌

- `agent/getAuthenticatedExtendedCard` endpoint
- Access control and extended metadata

**Workaround:** Use public `agent/card`.

---

### 4. Advanced File Handling (v0.8.0) ⚠️

- ✅ Basic `FilePart` with URI/bytes
- ❌ Size limits and validation
- ❌ Streaming for large files
- ❌ MIME type validation

---

### 5. Additional Transports (v1.0.0) ❌

- **gRPC:** Requires .proto from spec
- **HTTP+JSON/REST:** Awaiting spec clarification

**Current:** JSON-RPC satisfies spec requirement.

---

## Next Steps

1. **v0.6.0:** Complete client-side SSE streaming API
2. **v0.7.0:** Implement push notification webhooks
3. **v0.8.0:** Add authenticated extended card and file handling
4. **v1.0.0:** Additional transports and production hardening

See [IMPLEMENTATION_ROADMAP.md](IMPLEMENTATION_ROADMAP.md) for detailed schedule.

---

**Maintained by:** a2a-protocol team
