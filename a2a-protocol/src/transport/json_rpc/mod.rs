//! JSON-RPC 2.0 transport implementation for A2A protocol

mod types;
mod transport;

// Re-export JSON-RPC types
pub use types::{
    JsonRpcRequest, JsonRpcResponse, JsonRpcError,
    JsonRpcBatchRequest, JsonRpcBatchResponse, JsonRpcNotification,
    is_notification, is_batch_request, map_error_to_rpc,
    SERVER_ERROR, TASK_NOT_FOUND, TASK_CANCELLED, AGENT_NOT_FOUND,
};

// Re-export transport
pub use transport::JsonRpcTransport;
