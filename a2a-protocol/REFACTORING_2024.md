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

## Rationale

This refactoring follows the principle of **least surprise** - when a user wants to build a client, they look in `client/`. When they want to build a server, they look in `server/`. The `core/` module contains protocol definitions used by both.

The previous structure violated this principle by having client-only code (`auth`, `transport`) at the root level, suggesting they might be shared or used by servers.
