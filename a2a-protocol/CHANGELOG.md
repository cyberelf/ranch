# Changelog

All notable changes to the a2a-protocol crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.7.0] - 2025-11-05 (In Progress)

### Added

#### Push Notifications & Webhooks (Core Implementation Complete)
- Push notification configuration types
  - `PushNotificationConfig` for webhook endpoint configuration
  - `PushNotificationAuth` enum (Bearer tokens, Custom headers)
  - `TaskEvent` enum (StatusChanged, Completed, Failed, Cancelled, ArtifactAdded)
- SSRF (Server-Side Request Forgery) protection
  - Comprehensive URL validation for webhook endpoints
  - Blocks private IPv4 ranges (10.x, 172.16-31.x, 192.168.x)
  - Blocks private IPv6 ranges (::1, fc00::/7, fe80::/10)
  - Blocks localhost, link-local, multicast, and broadcast addresses
  - Blocks cloud metadata endpoints (169.254.169.254)
  - Hostname validation (blocks .local, .internal domains)
- Push notification storage
  - `PushNotificationStore` for in-memory webhook configuration storage
  - Methods: set, get, list, delete
- JSON-RPC methods for push notifications
  - `tasks/pushNotificationConfig/set` - Configure webhook for a task
  - `tasks/pushNotificationConfig/get` - Retrieve webhook configuration
  - `tasks/pushNotificationConfig/list` - List all webhook configurations
  - `tasks/pushNotificationConfig/delete` - Remove webhook configuration
- Webhook delivery system
  - `WebhookQueue` for async, non-blocking webhook delivery
  - `WebhookPayload` with event, task, timestamp, and agent ID
  - HTTP delivery with authentication support (Bearer, Custom headers)
  - Exponential backoff retry logic (configurable max retries)
  - `RetryConfig` for customizable retry behavior
  - `DeliveryStatus` tracking (Pending, Delivering, Delivered, Failed, Retrying)
- Task integration
  - Automatic webhook triggering on task state changes
  - Fire-and-forget delivery (doesn't block task processing)
  - Event filtering (only configured events trigger webhooks)
  - Support for all task lifecycle events

#### Request/Response Types
- `PushNotificationSetRequest` - Parameters for webhook configuration
- `PushNotificationGetRequest` - Parameters for retrieving webhook config
- `PushNotificationListRequest` - Parameters for listing webhook configs
- `PushNotificationDeleteRequest` - Parameters for deleting webhook config
- `PushNotificationListResponse` - Response with list of configurations
- `PushNotificationConfigEntry` - Entry in list response

### Security
- HTTPS requirement for all webhook URLs
- Comprehensive SSRF attack prevention
- 27 security-focused tests for URL validation
- Safe handling of authentication credentials

### Testing
- 223 total tests (up from 161 in v0.6.0)
  - 162 library tests (up from 110)
  - 8 streaming integration tests
  - 17 compliance tests
  - 9 push notification RPC tests
  - 8 RPC integration tests
  - 19 documentation tests

## [0.6.0] - 2025-11-05

### Added

#### SSE Streaming (Complete)
- Server-side SSE (Server-Sent Events) streaming implementation
  - `SseEvent` for W3C-compliant event formatting and parsing
  - `SseWriter` for broadcast-based event publishing
  - `EventBuffer` with Last-Event-ID replay support
  - `/stream` endpoint in Axum integration
- Client-side SSE streaming API
  - `A2aStreamingClient` with `stream_message()` and `stream_text()` methods
  - `resubscribe_task()` for resuming streams with Last-Event-ID
  - Clean Deref pattern for accessing base client methods
- Streaming methods in `A2aHandler` trait
  - `rpc_message_stream()` for real-time message processing
  - `rpc_task_resubscribe()` for stream resumption
- Full `TaskAwareHandler` streaming implementation with automatic cleanup

#### Developer Experience Improvements
- `ServerBuilder` - Fluent API for one-line server setup
  - `.with_port()`, `.with_address()`, `.with_host_port()` methods
  - `.run()` for async server start
  - `.build()` for advanced configuration
- `AgentLogic` trait - Simplified agent implementation
  - Single `process_message()` method for business logic
  - Optional `initialize()` and `shutdown()` lifecycle hooks
  - `TaskAwareHandler::with_logic()` wrapper
- 8 comprehensive runnable examples
  - `basic_echo_server.rs` - Minimal AgentLogic demonstration
  - `echo_client.rs` - ClientBuilder and message handling
  - `simple_server.rs` - ServerBuilder one-liner
  - `streaming_server.rs` - SSE streaming server
  - `streaming_client.rs` - SSE client with reconnection
  - `streaming_type_safety.rs` - Type-safe streaming patterns
  - `task_server.rs` - Long-running task management
  - `multi_agent.rs` - Agent-to-agent communication
- Comprehensive documentation
  - Updated README.md with 5-minute quick start
  - New GETTING_STARTED.md with step-by-step tutorial
  - examples/README.md with usage guide for all examples
  - DOCS_INDEX.md for documentation navigation

#### Testing
- 161 total tests (up from 110 in v0.5.0)
  - 110 library tests
  - 8 streaming integration tests  
  - 17 compliance tests
  - 8 RPC tests
  - 18 documentation tests

### Changed
- Enhanced type safety with `A2aStreamingClient<T>` generic over transport
- Improved error handling in streaming contexts
- Feature flag `streaming` for optional SSE support

### Documentation
- Complete API documentation with examples
- Trait selection guide (AgentLogic vs A2aHandler)
- Common patterns and troubleshooting guide
- Performance tips and best practices

## [0.5.0] - 2025-10-23

### Added

#### AgentCard Enhancements
- `defaultInputModes: Vec<String>` field for MIME type specification
- `defaultOutputModes: Vec<String>` field for MIME type specification
- `supportsAuthenticatedExtendedCard: bool` flag
- `preferredTransport` upgraded to spec-aligned enum (JSONRPC | GRPC | HTTP+JSON)
- Optional metadata fields:
  - `provider: Option<AgentProvider>` (name, URL)
  - `icon_url: Option<Url>` for UI display
  - `documentation_url: Option<Url>` for help
  - `signatures: Vec<AgentCardSignature>` for verification

#### Error Codes
- Complete A2A-specific JSON-RPC error codes (-32001 through -32007):
  - `TaskNotFoundError` (-32001) with `taskId` data
  - `TaskNotCancelableError` (-32002) with `taskId` and `state` data
  - `PushNotificationNotSupportedError` (-32003)
  - `UnsupportedOperationError` (-32004)
  - `ContentTypeNotSupportedError` (-32005) with `contentType` data
  - `InvalidAgentResponseError` (-32006)
  - `AuthenticatedExtendedCardNotConfiguredError` (-32007)
- Structured error data in JSON-RPC responses
- Enhanced error matching and handling

#### Testing & Documentation
- 110 tests (84 lib + 17 compliance + 8 RPC + 1 doc)
- Comprehensive MIGRATION_v0.5.md guide
- Updated README with v0.5.0 features
- Extended compliance tests for new error paths

### Removed
- Deprecated `protocols` field from AgentCard (breaking change)

### Changed
- `TransportInterface` validation for transport enum usage
- Improved AgentCard builders with new field methods

## [0.4.0] - 2025-10-20

### Removed
- Non-spec compliant `A2aRouter` (REST endpoints)
- Incomplete streaming module (pre-SSE version)

### Changed
- Established JSON-RPC 2.0 as sole baseline transport
- Clean architecture for future spec-compliant features

### Added
- Migration guide (MIGRATION_v0.4.md)
- 101 tests passing baseline

### Fixed
- Spec compliance issues from v0.3.x

## [0.3.0] and earlier

See git history for details on initial implementation.

---

**Note:** Versions 0.1.0-0.3.0 were development versions with partial spec compliance.
Starting from v0.4.0, strict adherence to A2A Protocol v0.3.0 specification.

[0.6.0]: https://github.com/cyberelf/ranch/releases/tag/v0.6.0
[0.5.0]: https://github.com/cyberelf/ranch/releases/tag/v0.5.0
[0.4.0]: https://github.com/cyberelf/ranch/releases/tag/v0.4.0
