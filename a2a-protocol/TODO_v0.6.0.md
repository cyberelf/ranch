# A2A Protocol v0.6.0 - IN PROGRESS

## v0.6.0 - Developer Experience & Streaming
**Theme:** Complete SSE streaming + improve usability (inspired by a2a-go)

### Priority 1: Complete SSE Streaming (CRITICAL)
- ✅ SSE infrastructure (SseEvent, SseWriter, EventBuffer)
- ✅ Streaming methods in A2aHandler trait
- ✅ TaskAwareHandler streaming implementation
- ✅ Axum /stream endpoint
- ✅ Integration tests (98 lib + 17 compliance + 8 RPC + 1 doc = 124 tests)
- ✅ **Client streaming API** (COMPLETE!)
  - ✅ `Client::stream_message()` method
  - ✅ `Client::stream_text()` convenience method
  - ✅ `Client::resubscribe_task()` method
  - ✅ SSE event parsing in client
  - ✅ Reconnection with Last-Event-ID support
  - ✅ Client streaming integration tests (6 new tests)
  - ✅ **Total: 132 tests passing** (98 lib + 6 streaming + 17 compliance + 8 RPC + 3 doc)

### Priority 2: Developer Experience Improvements (NEW - inspired by a2a-go)
**Goal:** Make the library as easy to use as the Go implementation

#### 2.1 Simplified Server API
- [ ] **Create `ServerBuilder`** - One-line server setup
  - [ ] Implement `ServerBuilder<H: A2aHandler>`
  - [ ] `.with_address()` configuration
  - [ ] `.with_cors()` configuration (optional)
  - [ ] `.run()` async method that starts server
  - [ ] Hide axum/tokio complexity from users
  
#### 2.2 Simplified Agent Logic Trait
- [ ] **Create `AgentLogic` trait** - Simpler than `A2aHandler`
  - [ ] `async fn process_message(&self, msg: Message) -> Result<Message, A2aError>`
  - [ ] Update `TaskAwareHandler` to accept `impl AgentLogic`
  - [ ] Keep `A2aHandler` for advanced users who need full control
  - [ ] Migration guide showing when to use which trait

#### 2.3 High-Quality Examples
- [ ] **Create `examples/` directory** with runnable examples
  - [ ] `basic_echo_server.rs` - Minimal working server
  - [ ] `echo_client.rs` - Minimal working client
  - [ ] `streaming_server.rs` - SSE streaming example
  - [ ] `streaming_client.rs` - SSE client example
  - [ ] `task_server.rs` - Long-running task example
  - [ ] `multi_agent.rs` - Agent-to-agent communication
  - [ ] Add clap for CLI configuration in examples
  - [ ] Include README.md in examples/ with quickstart

#### 2.4 Documentation Overhaul
- [ ] **Update README.md** with quick start guide
  - [ ] 5-minute "hello world" server example
  - [ ] 5-minute client example
  - [ ] Architecture diagram showing modules
  - [ ] When to use `AgentLogic` vs `A2aHandler`
- [ ] **Create GETTING_STARTED.md**
  - [ ] Step-by-step tutorial
  - [ ] Common patterns and recipes
  - [ ] Troubleshooting guide
- [ ] **Improve inline docs**
  - [ ] Add examples to all public types
  - [ ] Document trait methods with usage patterns
  - [ ] Add "See also" cross-references

### Success Criteria
- ✅ Client streaming API works end-to-end
- ✅ Can build a working server in <10 lines of code
- ✅ All examples run successfully
- ✅ New user can get started in <5 minutes
- ✅ Documentation covers 90% of common use cases
- ✅ All tests passing (target: 140+ tests)
