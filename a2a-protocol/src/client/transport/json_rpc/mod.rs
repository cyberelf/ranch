//! JSON-RPC 2.0 client transport implementation for A2A protocol

mod transport;

// Re-export transport
pub use transport::JsonRpcTransport;

// Re-export JSON-RPC types from core module
pub use crate::core::{
    is_batch_request, is_notification, map_error_to_rpc, JsonRpcBatchRequest, JsonRpcBatchResponse,
    JsonRpcError, JsonRpcNotification, JsonRpcRequest, JsonRpcResponse,
    AUTHENTICATED_EXTENDED_CARD_NOT_CONFIGURED, CONTENT_TYPE_NOT_SUPPORTED, INVALID_AGENT_RESPONSE,
    PUSH_NOTIFICATION_NOT_SUPPORTED, SERVER_ERROR, TASK_NOT_CANCELABLE, TASK_NOT_FOUND,
    UNSUPPORTED_OPERATION,
};
