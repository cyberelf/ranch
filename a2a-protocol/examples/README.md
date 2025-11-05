# A2A Protocol Examples

This directory contains runnable examples demonstrating the A2A (Agent-to-Agent) protocol implementation.

## Quick Start

All examples require the `streaming` feature to be enabled:

```bash
cargo run --example <example_name> --features streaming
```

## Available Examples

### Basic Examples

#### 1. `basic_echo_server.rs` - Simple Echo Server
A minimal server using the simplified `AgentLogic` trait. Perfect for getting started!

**Start the server:**
```bash
cargo run --example basic_echo_server --features streaming
```

**Test it:**
```bash
curl -X POST http://localhost:3000/rpc \
  -H 'Content-Type: application/json' \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "message/send",
    "params": {
      "message": {
        "role": "user",
        "parts": [{"kind":"text","text":"hello world"}]
      },
      "immediate": true
    }
  }'
```

#### 2. `echo_client.rs` - Simple Client
Demonstrates how to create a client and send messages to an A2A server.

**Prerequisites:** Start `basic_echo_server` first

**Run:**
```bash
cargo run --example echo_client --features streaming
```

#### 3. `simple_server.rs` - ServerBuilder Example
Shows the one-line server setup using `ServerBuilder`.

```bash
cargo run --example simple_server --features streaming
```

### Advanced Examples

#### 4. `streaming_server.rs` - SSE Streaming Server
Demonstrates server-side SSE (Server-Sent Events) streaming for real-time updates.

**Start the server:**
```bash
cargo run --example streaming_server --features streaming
```

The server runs on port 3001 and supports:
- Regular message/send requests
- message/stream for SSE streaming
- All standard task/* methods

#### 5. `streaming_client.rs` - SSE Streaming Client
Shows how to use `A2aStreamingClient` to receive real-time updates via SSE.

**Prerequisites:** Start `streaming_server` first

**Run:**
```bash
cargo run --example streaming_client --features streaming
```

#### 6. `streaming_type_safety.rs` - Type-Safe Streaming
Demonstrates compile-time type safety with `A2aStreamingClient<T>` and the Deref pattern for accessing base client methods.

```bash
cargo run --example streaming_type_safety --features streaming
```

Shows how the type system ensures streaming support at compile time rather than runtime.

#### 7. `task_server.rs` - Long-Running Tasks
Shows how to handle long-running async tasks with status polling.

**Start the server:**
```bash
cargo run --example task_server --features streaming
```

The server runs on port 3002. Try:
- Quick messages (immediate response with `"immediate": true`)
- Long-running tasks (async processing)
- Task status polling (`task/status` method)
- Task result retrieval (`task/get` method)

#### 8. `multi_agent.rs` - Agent-to-Agent Communication
Demonstrates two agents communicating with each other:
- **Calculator Agent** (port 3003) - Performs math operations
- **Reporter Agent** (port 3004) - Uses Calculator to generate reports

**Run:**
```bash
cargo run --example multi_agent --features streaming
```

**Test the Calculator:**
```bash
curl -X POST http://localhost:3003/rpc \
  -H 'Content-Type: application/json' \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "message/send",
    "params": {
      "message": {
        "role": "user",
        "parts": [{"kind":"text","text":"add 10 20"}]
      },
      "immediate": true
    }
  }'
```

**Test the Reporter (which calls Calculator internally):**
```bash
curl -X POST http://localhost:3004/rpc \
  -H 'Content-Type: application/json' \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "message/send",
    "params": {
      "message": {
        "role": "user",
        "parts": [{"kind":"text","text":"generate report"}]
      },
      "immediate": true
    }
  }'
```

## JSON-RPC 2.0 API Reference

All examples use JSON-RPC 2.0 over HTTP POST. Here are the available methods:

### `message/send`
Send a message to the agent. Returns either an immediate `Message` or a `Task`.

**Parameters:**
- `message`: Message object with `role` and `parts`
- `immediate` (optional): Boolean to request immediate response

**Example:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "message/send",
  "params": {
    "message": {
      "role": "user",
      "parts": [{"kind":"text","text":"hello"}]
    },
    "immediate": true
  }
}
```

### `message/stream`
Send a message and receive SSE stream of updates.

**Parameters:**
- `message`: Message object with `role` and `parts`

### `task/get`
Get task details and results.

**Parameters:**
- `taskId`: String task ID

### `task/status`
Get current task status.

**Parameters:**
- `taskId`: String task ID

### `task/cancel`
Cancel a running task.

**Parameters:**
- `taskId`: String task ID

### `agent/card`
Get agent card information.

**Parameters:** None

## Development Tips

### Running Multiple Examples
You can run multiple examples simultaneously on different ports:

```bash
# Terminal 1
cargo run --example streaming_server --features streaming

# Terminal 2
cargo run --example streaming_client --features streaming
```

### Testing with curl
All examples expose JSON-RPC 2.0 endpoints that can be tested with curl:

```bash
curl -X POST http://localhost:PORT/rpc \
  -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","id":1,"method":"METHOD","params":PARAMS}'
```

### Debugging
Enable trace-level logging:

```bash
RUST_LOG=trace cargo run --example basic_echo_server --features streaming
```

## Next Steps

1. **Try the basic examples first** (`basic_echo_server` + `echo_client`)
2. **Explore streaming** with `streaming_server` + `streaming_client`
3. **Learn task management** with `task_server`
4. **Study agent communication** with `multi_agent`

For more information, see:
- [Main README](../README.md)
- [A2A Protocol Specification](https://github.com/a2a-protocol/specification)
- [API Documentation](https://docs.rs/a2a-protocol)
