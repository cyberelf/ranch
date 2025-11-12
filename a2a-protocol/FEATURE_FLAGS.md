# Feature Flag Guide

## Overview

The `a2a-protocol` crate supports optional compilation through Cargo feature flags, allowing you to include only the functionality you need. This reduces compile times and binary sizes.

## Available Features

### Core Features

- **`client`** - Client-side implementations for making outbound A2A requests
  - Enables: `reqwest` for HTTP transport
  - Includes: `A2aClient`, authentication, transport layer
  
- **`server`** - Server-side implementations for handling inbound A2A requests
  - Enables: `reqwest`, `axum`, `tower`, `tower-http`, `hyper`
  - Includes: `TaskAwareHandler`, JSON-RPC dispatcher, webhook delivery
  
- **`streaming`** - Streaming support via Server-Sent Events (SSE)
  - Enables: `client`, `futures-util`, `async-stream`, `bytes`
  - Includes: `A2aStreamingClient`, `StreamingResult`, SSE infrastructure
  - Note: Automatically enables `client` since streaming types depend on client transport layer

### Default Features

By default, all features are enabled:
```toml
default = ["client", "server", "streaming"]
```

## Usage Examples

### Client Only

For applications that only need to make A2A requests:

```toml
[dependencies]
a2a-protocol = { version = "0.7", default-features = false, features = ["client"] }
```

```rust
use a2a_protocol::{
    prelude::*,
    client::{A2aClient, JsonRpcTransport},
};
```

### Server Only

For applications that only need to handle A2A requests:

```toml
[dependencies]
a2a-protocol = { version = "0.7", default-features = false, features = ["server"] }
```

```rust
use a2a_protocol::{
    prelude::*,
    server::{Agent, TaskAwareHandler, JsonRpcRouter},
};
```

### Core Types Only

For applications that only need the protocol types (no networking):

```toml
[dependencies]
a2a-protocol = { version = "0.7", default-features = false }
```

```rust
use a2a_protocol::prelude::*;  // Message, Task, AgentCard, etc.
```

### Client with Streaming

For clients that need streaming support:

```toml
[dependencies]
a2a-protocol = { version = "0.7", default-features = false, features = ["streaming"] }
```

Note: `streaming` automatically enables `client`, so you don't need to specify both.

```rust
use a2a_protocol::{
    prelude::*,
    client::{A2aStreamingClient, JsonRpcTransport},
};
```

### Server with Streaming

For servers that support streaming responses:

```toml
[dependencies]
a2a-protocol = { version = "0.7", default-features = false, features = ["server", "streaming"] }
```

```rust
use a2a_protocol::{
    prelude::*,
    server::{Agent, TaskAwareHandler},
    client::transport::{SseResponse, SseWriter},  // Server uses SSE types from client
};
```

### Full Featured (Default)

For applications using both client and server:

```toml
[dependencies]
a2a-protocol = "0.7"  # All features enabled by default
```

## Feature Dependencies

```
streaming -> client -> core
server -> core
```

- `streaming` depends on `client` (for `StreamingResult` and transport types)
- Both `client` and `server` depend on `core`
- `core` is always available (no feature gate)

## Compilation Time Comparison

Approximate compile times on a typical development machine:

| Feature Set | Clean Build Time | Incremental |
|-------------|------------------|-------------|
| Core only | ~0.4s | ~0.1s |
| Client only | ~0.8s | ~0.4s |
| Server only | ~0.6s | ~0.5s |
| Streaming only | ~0.4s | ~0.4s |
| Server + Streaming | ~1.8s | ~1.8s |
| Default (all) | ~2.6s | ~0.5s |

## Binary Size Impact

Feature flags also reduce binary size:

- **Core only**: Minimal size (~200KB with core types)
- **Client only**: Adds `reqwest` (~2MB)
- **Server only**: Adds `axum` stack (~1.5MB)  
- **Streaming**: Adds async-stream (~100KB)
- **Full**: All dependencies (~3MB)

(Sizes are approximate and exclude debug symbols)

## Testing with Features

To test specific feature combinations:

```bash
# Test with default features
cargo test

# Test client only
cargo test --no-default-features --features client

# Test server only  
cargo test --no-default-features --features server

# Test core only
cargo test --no-default-features

# Test streaming
cargo test --no-default-features --features streaming
```

## CI/CD Recommendations

We recommend testing multiple feature combinations in CI:

```yaml
strategy:
  matrix:
    features:
      - ""  # core only
      - "client"
      - "server"
      - "client,server"
      - "streaming"
      - "server,streaming"
      # default features tested separately
```

This ensures the crate compiles correctly with all combinations.

## Migration from Pre-0.7.0

Prior to v0.7.0, all code was compiled unconditionally. Starting in v0.7.0:

- **Breaking**: You must opt-in to features if using `default-features = false`
- **Non-breaking**: Default behavior unchanged (all features enabled)

Update your `Cargo.toml` to be explicit about features:

```toml
# Before (still works)
a2a-protocol = "0.7"

# After (recommended for clarity)
a2a-protocol = { version = "0.7", features = ["client", "server"] }

# Or minimal (client-only app)
a2a-protocol = { version = "0.7", default-features = false, features = ["client"] }
```

## Feature Flag Philosophy

The feature flag design follows these principles:

1. **Default = Full**: All features enabled by default for ease of use
2. **Granular Control**: Users can opt-out for optimization
3. **Logical Grouping**: Features map to clear use cases (client/server/streaming)
4. **Dependency Correctness**: Streaming depends on client (not independent)
5. **Zero Cost**: Unused features add zero overhead when disabled

## Troubleshooting

### Error: `unresolved import 'a2a_protocol::client'`

You're trying to use client features without enabling them:

```toml
a2a-protocol = { version = "0.7", default-features = false, features = ["client"] }
```

### Error: `unresolved import 'a2a_protocol::server'`

You're trying to use server features without enabling them:

```toml
a2a-protocol = { version = "0.7", default-features = false, features = ["server"] }
```

### Error: `StreamingResult` not found

You need the `streaming` feature:

```toml
a2a-protocol = { version = "0.7", default-features = false, features = ["streaming"] }
```

Note: `streaming` automatically enables `client`.
