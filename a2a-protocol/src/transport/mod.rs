//! Transport layer implementations

pub mod traits;
pub mod http; // Internal HTTP client
pub mod json_rpc;

pub use traits::{Transport, TransportExt, TransportConfig, RequestInfo};
pub use json_rpc::{
    JsonRpcTransport,
    // Re-export JSON-RPC 2.0 protocol types
    JsonRpcRequest, JsonRpcResponse, JsonRpcError,
    JsonRpcBatchRequest, JsonRpcBatchResponse, JsonRpcNotification,
    is_notification, is_batch_request, map_error_to_rpc,
    SERVER_ERROR, TASK_NOT_FOUND, TASK_CANCELLED, AGENT_NOT_FOUND,
};