//! Transport layer implementations

pub mod http; // Internal HTTP client
pub mod json_rpc;
pub mod traits;

pub use json_rpc::{
    is_batch_request,
    is_notification,
    map_error_to_rpc,
    JsonRpcBatchRequest,
    JsonRpcBatchResponse,
    JsonRpcError,
    JsonRpcNotification,
    // Re-export JSON-RPC 2.0 protocol types
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
pub use traits::{RequestInfo, Transport, TransportConfig, TransportExt};
