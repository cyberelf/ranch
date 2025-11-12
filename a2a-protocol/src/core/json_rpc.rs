//! JSON-RPC 2.0 protocol types (spec-compliant)
//!
//! These types implement the JSON-RPC 2.0 specification:
//! https://www.jsonrpc.org/specification

use serde::{Deserialize, Serialize};

// JSON-RPC 2.0 error codes (per spec: -32000 to -32099 are server errors)
/// Server error: internal server error
pub const SERVER_ERROR: i32 = -32000;
/// Server error: task not found
pub const TASK_NOT_FOUND: i32 = -32001;
/// Server error: task not cancelable
pub const TASK_NOT_CANCELABLE: i32 = -32002;
/// Server error: push notifications not supported
pub const PUSH_NOTIFICATION_NOT_SUPPORTED: i32 = -32003;
/// Server error: unsupported operation
pub const UNSUPPORTED_OPERATION: i32 = -32004;
/// Server error: content type not supported
pub const CONTENT_TYPE_NOT_SUPPORTED: i32 = -32005;
/// Server error: invalid agent response
pub const INVALID_AGENT_RESPONSE: i32 = -32006;
/// Server error: authenticated extended card not configured
pub const AUTHENTICATED_EXTENDED_CARD_NOT_CONFIGURED: i32 = -32007;

/// JSON-RPC 2.0 request wrapper
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JsonRpcRequest<T> {
    /// JSON-RPC version (always "2.0")
    pub jsonrpc: String,

    /// Request ID (for matching responses)
    pub id: serde_json::Value,

    /// Method name (e.g., "message/send", "task/get")
    pub method: String,

    /// Method parameters
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<T>,
}

impl<T> JsonRpcRequest<T> {
    /// Create a new JSON-RPC request
    pub fn new<S: Into<String>>(method: S, params: T, id: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            method: method.into(),
            params: Some(params),
        }
    }

    /// Create a notification (no response expected)
    pub fn notification<S: Into<String>>(method: S, params: T) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id: serde_json::Value::Null,
            method: method.into(),
            params: Some(params),
        }
    }
}

/// JSON-RPC 2.0 response wrapper
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JsonRpcResponse<T> {
    /// JSON-RPC version (always "2.0")
    pub jsonrpc: String,

    /// Request ID (matches request)
    pub id: serde_json::Value,

    /// Result (if successful)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<T>,

    /// Error (if failed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

impl<T> JsonRpcResponse<T> {
    /// Create a successful response
    pub fn success(id: serde_json::Value, result: T) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    /// Create an error response
    pub fn error(id: serde_json::Value, error: JsonRpcError) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(error),
        }
    }
}

/// JSON-RPC 2.0 error object
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JsonRpcError {
    /// Error code
    pub code: i32,

    /// Error message
    pub message: String,

    /// Optional error data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl JsonRpcError {
    /// Create a new JSON-RPC error
    pub fn new(code: i32, message: String) -> Self {
        Self {
            code,
            message,
            data: None,
        }
    }

    /// Add error data
    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.data = Some(data);
        self
    }

    /// Parse error (-32700)
    pub fn parse_error() -> Self {
        Self::new(-32700, "Parse error".to_string())
    }

    /// Invalid request (-32600)
    pub fn invalid_request() -> Self {
        Self::new(-32600, "Invalid request".to_string())
    }

    /// Method not found (-32601)
    pub fn method_not_found() -> Self {
        Self::new(-32601, "Method not found".to_string())
    }

    /// Invalid params (-32602)
    pub fn invalid_params() -> Self {
        Self::new(-32602, "Invalid params".to_string())
    }

    /// Internal error (-32603)
    pub fn internal_error() -> Self {
        Self::new(-32603, "Internal error".to_string())
    }

    /// Server error (-32000 to -32099)
    pub fn server_error<S: Into<String>>(code: i32, message: S) -> Self {
        assert!(
            (-32099..=-32000).contains(&code),
            "Server error code must be -32000 to -32099"
        );
        Self::new(code, message.into())
    }
}

/// Batch JSON-RPC request (array of requests)
pub type JsonRpcBatchRequest = Vec<serde_json::Value>;

/// Batch JSON-RPC response (array of responses)
pub type JsonRpcBatchResponse = Vec<serde_json::Value>;

/// JSON-RPC notification (request without ID - no response expected)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JsonRpcNotification<T> {
    /// JSON-RPC version (always "2.0")
    pub jsonrpc: String,

    /// Method name
    pub method: String,

    /// Method parameters
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<T>,
}

impl<T> JsonRpcNotification<T> {
    /// Create a new notification
    pub fn new<S: Into<String>>(method: S, params: T) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            method: method.into(),
            params: Some(params),
        }
    }
}

/// Helper to determine if a request is a notification
/// Per JSON-RPC 2.0: A notification is a request without an 'id' member
pub fn is_notification(request: &serde_json::Value) -> bool {
    !request
        .as_object()
        .map_or(false, |obj| obj.contains_key("id"))
}

/// Helper to determine if a value is a batch request
pub fn is_batch_request(value: &serde_json::Value) -> bool {
    value.is_array()
}

/// Error mapper from A2aError to JsonRpcError
pub fn map_error_to_rpc(error: crate::A2aError) -> JsonRpcError {
    use crate::A2aError;

    match error {
        A2aError::Json(_) => JsonRpcError::parse_error(),
        A2aError::InvalidMessage(msg) | A2aError::Validation(msg) => {
            JsonRpcError::invalid_params().with_data(serde_json::json!({ "message": msg }))
        }
        A2aError::ProtocolViolation(msg) => {
            if msg.contains("Method") || msg.contains("method") {
                JsonRpcError::method_not_found()
            } else {
                JsonRpcError::invalid_request()
            }
        }
        A2aError::TaskNotFound { ref task_id } => {
            JsonRpcError::server_error(TASK_NOT_FOUND, error.to_string())
                .with_data(serde_json::json!({ "taskId": task_id }))
        }
        A2aError::TaskNotCancelable {
            ref task_id,
            ref state,
        } => JsonRpcError::server_error(TASK_NOT_CANCELABLE, error.to_string()).with_data(
            serde_json::json!({
                "taskId": task_id,
                "state": format!("{:?}", state)
            }),
        ),
        A2aError::PushNotificationNotSupported => {
            JsonRpcError::server_error(PUSH_NOTIFICATION_NOT_SUPPORTED, error.to_string())
        }
        A2aError::UnsupportedOperation(msg) => {
            JsonRpcError::server_error(UNSUPPORTED_OPERATION, msg)
        }
        A2aError::ContentTypeNotSupported { content_type } => JsonRpcError::server_error(
            CONTENT_TYPE_NOT_SUPPORTED,
            format!("Content type not supported: {}", content_type),
        )
        .with_data(serde_json::json!({ "contentType": content_type })),
        A2aError::InvalidAgentResponse(msg) => {
            JsonRpcError::server_error(INVALID_AGENT_RESPONSE, msg)
        }
        A2aError::AuthenticatedExtendedCardNotConfigured => JsonRpcError::server_error(
            AUTHENTICATED_EXTENDED_CARD_NOT_CONFIGURED,
            error.to_string(),
        ),
        A2aError::Server(msg) => JsonRpcError::server_error(SERVER_ERROR, msg),
        A2aError::AgentNotFound(_) => JsonRpcError::server_error(SERVER_ERROR, error.to_string()),
        A2aError::Internal(msg) => {
            JsonRpcError::internal_error().with_data(serde_json::json!({ "message": msg }))
        }
        _ => JsonRpcError::internal_error(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Message, SendResponse};

    #[test]
    fn test_json_rpc_request() {
        let params = serde_json::json!({"test": "value"});
        let req = JsonRpcRequest::new("test/method", params, serde_json::json!(1));

        assert_eq!(req.jsonrpc, "2.0");
        assert_eq!(req.method, "test/method");
        assert_eq!(req.id, serde_json::json!(1));
    }

    #[test]
    fn test_json_rpc_response_success() {
        let msg = Message::agent_text("Response");
        let response = SendResponse::message(msg);
        let rpc_response = JsonRpcResponse::success(serde_json::json!(1), response);

        assert_eq!(rpc_response.jsonrpc, "2.0");
        assert!(rpc_response.result.is_some());
        assert!(rpc_response.error.is_none());
    }

    #[test]
    fn test_json_rpc_error_codes() {
        let parse_err = JsonRpcError::parse_error();
        assert_eq!(parse_err.code, -32700);

        let method_err = JsonRpcError::method_not_found();
        assert_eq!(method_err.code, -32601);

        let server_err = JsonRpcError::server_error(-32000, "Server busy");
        assert_eq!(server_err.code, -32000);
    }

    #[test]
    fn test_json_rpc_notification() {
        let params = serde_json::json!({"test": "value"});
        let notification = JsonRpcNotification::new("test/method", params);

        assert_eq!(notification.jsonrpc, "2.0");
        assert_eq!(notification.method, "test/method");

        // Verify serialization doesn't include an 'id' field
        let json = serde_json::to_value(&notification).unwrap();
        assert!(json.get("id").is_none());
    }

    #[test]
    fn test_is_notification_helper() {
        // Request with id is not a notification
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "message/send",
            "id": 1
        });
        assert!(!is_notification(&request));

        // Request without id is a notification
        let notification = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "message/send"
        });
        assert!(is_notification(&notification));

        // Request with null id is not a notification (per JSON-RPC 2.0 spec)
        let null_id = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "message/send",
            "id": null
        });
        assert!(!is_notification(&null_id));
    }

    #[test]
    fn test_is_batch_request_helper() {
        // Single request
        let single = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "message/send",
            "id": 1
        });
        assert!(!is_batch_request(&single));

        // Batch request (array)
        let batch = serde_json::json!([
            {"jsonrpc": "2.0", "method": "message/send", "id": 1},
            {"jsonrpc": "2.0", "method": "task/get", "id": 2}
        ]);
        assert!(is_batch_request(&batch));
    }

    #[test]
    fn test_error_code_constants() {
        // Verify error codes are in valid server error range (-32000 to -32099)
        assert!(SERVER_ERROR >= -32099 && SERVER_ERROR <= -32000);
        assert!(TASK_NOT_FOUND >= -32099 && TASK_NOT_FOUND <= -32000);
        assert!(TASK_NOT_CANCELABLE >= -32099 && TASK_NOT_CANCELABLE <= -32000);
        assert!(
            PUSH_NOTIFICATION_NOT_SUPPORTED >= -32099 && PUSH_NOTIFICATION_NOT_SUPPORTED <= -32000
        );
        assert!(UNSUPPORTED_OPERATION >= -32099 && UNSUPPORTED_OPERATION <= -32000);
        assert!(CONTENT_TYPE_NOT_SUPPORTED >= -32099 && CONTENT_TYPE_NOT_SUPPORTED <= -32000);
        assert!(INVALID_AGENT_RESPONSE >= -32099 && INVALID_AGENT_RESPONSE <= -32000);
        assert!(
            AUTHENTICATED_EXTENDED_CARD_NOT_CONFIGURED >= -32099
                && AUTHENTICATED_EXTENDED_CARD_NOT_CONFIGURED <= -32000
        );
    }
}
