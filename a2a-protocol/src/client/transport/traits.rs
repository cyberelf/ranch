//! Transport layer traits and common types for A2A client

use crate::{A2aResult, AgentCard, AgentId, Message, SendResponse, Task, TaskStatus};
use async_trait::async_trait;

#[cfg(feature = "streaming")]
use crate::{TaskArtifactUpdateEvent, TaskStatusUpdateEvent};
#[cfg(feature = "streaming")]
use futures_util::stream::Stream;

/// Configuration for client transport layer
#[derive(Debug, Clone)]
pub struct TransportConfig {
    /// Request timeout in seconds
    pub timeout_seconds: u64,

    /// Maximum number of retries
    pub max_retries: u32,

    /// Whether to enable compression
    pub enable_compression: bool,

    /// Additional transport-specific configuration
    pub extra: std::collections::HashMap<String, serde_json::Value>,
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self {
            timeout_seconds: 30,
            max_retries: 3,
            enable_compression: true,
            extra: std::collections::HashMap::new(),
        }
    }
}

/// Transport layer trait for A2A protocol client communication
///
/// This trait defines the interface for sending A2A messages and managing tasks.
/// All methods return core domain types, keeping the transport layer protocol-agnostic.
///
/// Implementations handle protocol-specific details (JSON-RPC, gRPC, etc.) internally.
#[async_trait]
pub trait Transport: Send + Sync + std::fmt::Debug {
    /// Send a message and return the response (Task for async or Message for immediate)
    async fn send_message(&self, message: Message) -> A2aResult<SendResponse>;

    /// Fetch an agent's card
    async fn get_agent_card(&self, agent_id: &AgentId) -> A2aResult<AgentCard>;

    /// Get a task by ID
    async fn get_task(&self, request: crate::TaskGetRequest) -> A2aResult<Task>;

    /// Get the status of a task
    async fn get_task_status(&self, request: crate::TaskStatusRequest) -> A2aResult<TaskStatus>;

    /// Cancel a task
    async fn cancel_task(&self, request: crate::TaskCancelRequest) -> A2aResult<TaskStatus>;

    /// Check if the transport is connected/available
    async fn is_available(&self) -> bool;

    /// Get the transport configuration
    fn config(&self) -> &TransportConfig;

    /// Get the transport type name (e.g., "json-rpc", "grpc", "http")
    fn transport_type(&self) -> &'static str;
}

/// Streaming result type for message/stream responses
///
/// Per A2A protocol spec, each streaming event can be:
/// - Message: Immediate message response
/// - Task: Task object (initial or updated)
/// - TaskStatusUpdateEvent: Task status change notification
/// - TaskArtifactUpdateEvent: Task artifact notification
#[cfg(feature = "streaming")]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum StreamingResult {
    /// Immediate message response
    Message(Message),

    /// Task object
    Task(Task),

    /// Task status update event
    TaskStatusUpdate(TaskStatusUpdateEvent),

    /// Task artifact update event
    TaskArtifactUpdate(TaskArtifactUpdateEvent),
}

/// Streaming transport trait for real-time message/task updates
///
/// This trait extends the base Transport with streaming capabilities.
/// Implementations use protocol-specific streaming (SSE for JSON-RPC, gRPC streams, etc.)
#[cfg(feature = "streaming")]
#[async_trait]
pub trait StreamingTransport: Transport {
    /// Send a message and get a stream of responses
    ///
    /// Per A2A protocol spec (message/stream), this returns a stream where each item is one of:
    /// - Message (for immediate responses)
    /// - Task (for initial task or task completion)
    /// - TaskStatusUpdateEvent (for status updates)
    /// - TaskArtifactUpdateEvent (for artifact notifications)
    ///
    /// Each item is wrapped in A2aResult - errors are returned as Err() using standard
    /// JSON-RPC error format, not as a separate event type.
    ///
    /// The stream completes when the task finishes or encounters an error.
    async fn send_streaming_message(
        &self,
        message: Message,
    ) -> A2aResult<Box<dyn Stream<Item = A2aResult<StreamingResult>> + Send + Unpin>>;

    /// Resume a task stream (task/resubscribe RPC method)
    ///
    /// Returns a stream of events for an existing task, optionally resuming from a specific point.
    ///
    /// # Resume Semantics
    /// The `request.metadata` field can contain resume information:
    /// - **SSE (JSON-RPC)**: `lastEventId` is extracted and sent as `Last-Event-ID` HTTP header
    /// - **gRPC**: `lastEventId`, `sequenceNumber`, or `resumeToken` used directly in stream request
    /// - **Other transports**: Implementation-specific
    ///
    /// The transport implementation is responsible for:
    /// 1. Extracting resume information from `metadata`
    /// 2. Mapping it to transport-specific mechanisms
    /// 3. Buffering/replaying events as needed
    ///
    /// # Parameters
    /// - `request`: The task resubscribe request with optional resume metadata
    ///
    /// # Returns
    /// A stream of task updates (same types as send_streaming_message)
    async fn resubscribe_task(
        &self,
        request: crate::TaskResubscribeRequest,
    ) -> A2aResult<Box<dyn Stream<Item = A2aResult<StreamingResult>> + Send + Unpin>>;

    /// Check if streaming is supported by this transport
    fn supports_streaming(&self) -> bool {
        true // Default: assume streaming support
    }
}

/// Request information for client transport implementations
#[derive(Debug, Clone)]
pub struct RequestInfo {
    /// Target URL or endpoint
    pub endpoint: String,

    /// HTTP method (for HTTP-based transports)
    pub method: Option<String>,

    /// Request headers
    pub headers: std::collections::HashMap<String, String>,

    /// Request timeout in milliseconds
    pub timeout_ms: u64,
}

impl RequestInfo {
    /// Create a new request info
    pub fn new<S: Into<String>>(endpoint: S) -> Self {
        Self {
            endpoint: endpoint.into(),
            method: None,
            headers: std::collections::HashMap::new(),
            timeout_ms: 30000,
        }
    }

    /// Set the HTTP method
    pub fn with_method<S: Into<String>>(mut self, method: S) -> Self {
        self.method = Some(method.into());
        self
    }

    /// Add a header
    pub fn with_header<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// Set timeout in milliseconds
    pub fn with_timeout_ms(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }
}
