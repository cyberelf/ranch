# A2A Protocol Module Refactoring

**Date**: November 12, 2024
**Version**: 0.7.0+

## Overview

This document describes a major refactoring of the `a2a-protocol` crate's module structure to clearly separate client-side and server-side code. The previous structure had confusing module boundaries where `auth` and `transport` modules were at the root level, making it unclear to users when to use them.

## Problem Statement

**Before Refactoring:**
- `auth/` - Authentication strategies (actually only used by client)
- `transport/` - Transport implementations (actually only used by client) 
- `client/` - Client implementations
- `server/` - Server implementations
- `core/` - Shared protocol types

**Issues:**
1. Users were confused about when to import from `auth` vs `client`
2. The `transport` module name suggested it was shared, but it only contained client transport code
3. Server code never used `auth` or `transport` modules
4. No clear separation between client-only and server-only code

## Solution

The refactoring reorganizes modules into a clear hierarchy:

### New Module Structure

```
a2a-protocol/src/
├── core/                       # Shared protocol types (unchanged)
│   ├── agent_card.rs
│   ├── error.rs
│   ├── message.rs
│   └── task.rs
│
├── client/                     # Client-side implementations
│   ├── mod.rs
│   ├── builder.rs
│   ├── client.rs
│   ├── streaming_client.rs
│   ├── auth/                   # ← MOVED from src/auth/
│   │   ├── mod.rs
│   │   ├── authenticator.rs
│   │   └── strategies.rs
│   └── transport/              # ← MOVED from src/transport/
│       ├── mod.rs
│       ├── http_client.rs
│       ├── traits.rs
│       ├── sse.rs
│       └── json_rpc/
│           ├── mod.rs
│           ├── transport.rs
│           └── types.rs
│
└── server/                     # Server-side implementations (unchanged)
    ├── handler.rs
    ├── task_aware_handler.rs
    └── json_rpc/
        ├── axum.rs
        └── dispatcher.rs
```

### Key Changes

1. **`auth/` → `client/auth/`**
   - Authentication is only used by clients to add credentials to outbound requests
   - Moved entire module under `client/`

2. **`transport/` → `client/transport/`**
   - All transport implementations (JsonRpcTransport, HttpClient) are client-side
   - Transport trait defines the client interface
   - Server uses JSON-RPC types for serialization but doesn't use transport implementations

3. **Shared streaming types**
   - `StreamingResult`, `SseEvent`, `SseWriter`, `SseResponse` are used by both client and server
   - Kept in `client/transport/sse` and re-exported for server use
   - Client parses SSE, server generates SSE

## Migration Guide

### For Client Code

**Old imports:**
```rust
use a2a_protocol::{
    auth::{Authenticator, BearerAuth},
    transport::{JsonRpcTransport, Transport},
    client::A2aClient,
};
```

**New imports:**
```rust
use a2a_protocol::{
    client::{
        auth::{Authenticator, BearerAuth},
        transport::{JsonRpcTransport, Transport},
        A2aClient,
    },
};

// Or use the prelude:
use a2a_protocol::prelude::*;
```

### For Server Code

Server code is largely unchanged. Only imports of streaming types need updating:

**Old imports:**
```rust
use crate::transport::{StreamingResult, sse::SseWriter};
```

**New imports:**
```rust
use crate::client::transport::{StreamingResult, sse::SseWriter};
```

### Using the Prelude

The simplest migration is to use the prelude, which re-exports commonly used types:

```rust
use a2a_protocol::prelude::*;

// Now you have access to:
// - A2aClient, Authenticator, JsonRpcTransport (client types)
// - Message, Task, AgentCard (core types)
// - Agent, AgentLogic (server types)
```

## Benefits

1. **Clear Separation**: Client and server code are now clearly separated
2. **Intuitive Imports**: `use a2a_protocol::client::*` for client, `use a2a_protocol::server::*` for server
3. **Better Documentation**: Module docs now clearly state their purpose
4. **Easier Maintenance**: No confusion about which modules are used where
5. **Type Safety**: Compiler helps prevent using client types in server code

## Implementation Details

### Files Changed

- **Moved**: `src/auth/*` → `src/client/auth/*`
- **Moved**: `src/transport/*` → `src/client/transport/*`
- **Updated**: `src/client/mod.rs` - Added re-exports for auth and transport
- **Updated**: `src/lib.rs` - Updated prelude and module structure
- **Updated**: All example files - Updated import paths
- **Updated**: Client files (`builder.rs`, `client.rs`, `streaming_client.rs`) - Updated internal imports
- **Updated**: Server files - Updated SSE imports to use `client::transport::sse`

### Backward Compatibility

⚠️ **Breaking Change**: This is a breaking change as import paths have changed. Users need to update their imports.

However, the functionality is identical - only the module structure changed.

## Testing

All existing tests pass without modification (except for import path updates):
- ✅ 164 unit tests pass
- ✅ All examples compile successfully
- ✅ Library builds without errors

## Future Improvements

1. Consider extracting truly shared streaming types to a `common/streaming` module
2. Add deprecation warnings for old import paths (requires keeping old structure temporarily)
3. Update all documentation and tutorials with new import paths

## Feature Flags (Added in Phase 2)

After the module refactoring, we added Cargo feature flags to enable optional compilation of client and server code.

### Available Features

- **`client`** - Client-side implementations (requires `reqwest`)
- **`server`** - Server-side implementations (requires `axum`, `tower`, etc.)
- **`streaming`** - Streaming support via SSE (requires `client` + async streaming crates)
- **`default`** - All features enabled (`client + server + streaming`)

### Feature Dependencies

```
default = ["client", "server", "streaming"]
streaming = ["client", "futures-util", "async-stream", "bytes"]
client = ["reqwest"]
server = ["reqwest", "axum", "tower", "tower-http", "hyper"]
```

Note: `streaming` depends on `client` because `StreamingResult` and SSE types are defined in `client/transport/`.

### Usage Examples

```toml
# Client only (smaller binary, faster compilation)
a2a-protocol = { version = "0.7", default-features = false, features = ["client"] }

# Server only
a2a-protocol = { version = "0.7", default-features = false, features = ["server"] }

# Core types only (no networking)
a2a-protocol = { version = "0.7", default-features = false }

# Server with streaming
a2a-protocol = { version = "0.7", default-features = false, features = ["server", "streaming"] }
```

### Implementation Notes

1. **Module Gating**: Top-level modules are feature-gated in `lib.rs`:
   ```rust
   #[cfg(feature = "client")]
   pub mod client;
   
   #[cfg(feature = "server")]
   pub mod server;
   ```

2. **Error Type Gating**: `A2aError::Network` variant requires `client` or `server` features
   (uses `reqwest::Error` which is only available when these features are enabled)

3. **SSE Types**: `SseResponse` and `SseWriter` require `server` feature since they use `axum`
   types for server-side streaming

4. **Protocol Module**: Shared JSON-RPC wire format types are always available (no feature gate)

### Build Validation

All feature combinations are validated to build successfully:

```bash
✅ Core only:            cargo check --no-default-features
✅ Client only:          cargo check --no-default-features --features client
✅ Server only:          cargo check --no-default-features --features server
✅ Streaming only:       cargo check --no-default-features --features streaming
✅ Server + Streaming:   cargo check --no-default-features --features server,streaming
✅ Default (all):        cargo check
```

All 164 tests pass with default features enabled.

See [FEATURE_FLAGS.md](./FEATURE_FLAGS.md) for complete documentation on using feature flags.

## Phase 3: Module Organization Cleanup (November 12, 2024)

After implementing feature flags, we discovered several architectural issues:

### Issues Found

1. **Unused duplicate code**: `client/transport/json_rpc/types.rs` existed but was never imported (dead code)
2. **Misplaced server code**: `SseResponse` and `SseWriter` were in `client/transport/sse.rs` but only used by server
3. **Wrong module hierarchy**: JSON-RPC types in `protocol/` module instead of `core/` (they're wire format, not transport)
4. **Shared types in wrong place**: SSE types (`SseEvent`, `SseEventId`, `EventBuffer`) in client module but used by both

### Changes Made

1. **Moved JSON-RPC to core**:
   - `protocol/json_rpc.rs` → `core/json_rpc.rs`
   - Rationale: JSON-RPC is a wire format specification, belongs with core protocol types
   - Deleted `protocol/` module entirely

2. **Deleted dead code**:
   - Removed `client/transport/json_rpc/types.rs` (complete duplicate, never used)

3. **Split SSE by usage**:
   - Created `core/sse.rs` for shared types (`SseEvent`, `SseEventId`, `EventBuffer`)
   - Created `server/sse.rs` for server-only types (`SseResponse`, `SseWriter`)
   - Simplified `client/transport/sse.rs` to re-export core types

4. **Fixed module re-exports**:
   - `core/mod.rs` now exports JSON-RPC and SSE types
   - `server/mod.rs` exports SSE server types
   - `client/transport/mod.rs` only re-exports shared SSE types

### New Module Structure

```
a2a-protocol/src/
├── core/                       # Shared protocol types
│   ├── json_rpc.rs            # ← MOVED from protocol/json_rpc.rs
│   ├── sse.rs                 # ← NEW: Shared SSE types
│   ├── agent_card.rs
│   ├── error.rs
│   ├── message.rs
│   └── task.rs
│
├── client/                     # Client-side implementations
│   ├── transport/
│   │   ├── sse.rs             # ← SIMPLIFIED: Re-exports from core
│   │   └── json_rpc/
│   │       ├── mod.rs         # Re-exports from core::json_rpc
│   │       ├── transport.rs
│   │       └── types.rs       # ← DELETED (dead code)
│   └── ...
│
└── server/                     # Server-side implementations
    ├── sse.rs                 # ← NEW: Server-only SSE (SseResponse, SseWriter)
    └── ...
```

### Migration Impact

**Breaking Change**: Import paths changed for SSE server types.

**Before:**
```rust
use a2a_protocol::client::transport::sse::{SseResponse, SseWriter};
```

**After:**
```rust
use a2a_protocol::server::sse::{SseResponse, SseWriter};
// Or via prelude (if we add it)
```

**Shared SSE types** now in core:
```rust
use a2a_protocol::core::{SseEvent, SseEventId, EventBuffer};
// Still available via client::transport::sse for compatibility
```

**JSON-RPC types** moved to core (transparent via re-exports):
```rust
use a2a_protocol::core::{JsonRpcRequest, JsonRpcResponse, JsonRpcError};
// Still available via client::transport::json_rpc
```

### Rationale

This cleanup follows proper architectural layering:

1. **Core** = Domain types + wire formats (shared by all)
2. **Client** = Outbound request implementation
3. **Server** = Inbound request handling

The previous structure violated this by:
- Putting wire formats in a separate `protocol/` module (should be in `core/`)
- Putting server-specific code in `client/` module (SSE response generation)
- Duplicating code across modules (types.rs)

The new structure is cleaner and makes it immediately obvious where each type belongs.

## Rationale

This refactoring follows the principle of **least surprise** - when a user wants to build a client, they look in `client/`. When they want to build a server, they look in `server/`. The `core/` module contains protocol definitions used by both.

The previous structure violated this principle by having client-only code (`auth`, `transport`) at the root level, suggesting they might be shared or used by servers.
