//! Client transport layer implementations

pub(crate) mod http_client; // Internal HTTP client
pub mod json_rpc;
pub mod traits;

#[cfg(feature = "streaming")]
pub mod sse; // SSE (Server-Sent Events) for streaming

// Re-export JSON-RPC types
pub use json_rpc::{
    is_batch_request,
    is_notification,
    map_error_to_rpc,
    JsonRpcBatchRequest,
    JsonRpcBatchResponse,
    JsonRpcError,
    JsonRpcNotification,
    JsonRpcRequest,
    JsonRpcResponse,
    JsonRpcTransport,
    AUTHENTICATED_EXTENDED_CARD_NOT_CONFIGURED,
    CONTENT_TYPE_NOT_SUPPORTED,
    INVALID_AGENT_RESPONSE,
    PUSH_NOTIFICATION_NOT_SUPPORTED,
    SERVER_ERROR,
    TASK_NOT_CANCELABLE,
    TASK_NOT_FOUND,
    UNSUPPORTED_OPERATION,
};

// Re-export traits module types
pub use traits::{RequestInfo, Transport, TransportConfig};

#[cfg(feature = "streaming")]
pub use traits::{StreamingResult, StreamingTransport};

#[cfg(feature = "streaming")]
pub use sse::{EventBuffer, SseEvent, SseEventId};
