//! Transport layer traits and common types

use super::json_rpc::{
    JsonRpcBatchRequest, JsonRpcBatchResponse, JsonRpcNotification, JsonRpcRequest, JsonRpcResponse,
};
use crate::{A2aResult, AgentCard, Message, SendResponse};
use async_trait::async_trait;

/// Configuration for transport layer
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

/// Transport layer trait for sending A2A messages
#[async_trait]
pub trait Transport: Send + Sync + std::fmt::Debug {
    /// Send a message and return the response (Task for async or Message for immediate)
    async fn send_message(&self, message: Message) -> A2aResult<SendResponse>;

    /// Fetch an agent's card
    async fn get_agent_card(&self, agent_id: &crate::AgentId) -> A2aResult<AgentCard>;

    /// Check if the transport is connected/available
    async fn is_available(&self) -> bool;

    /// Get the transport configuration
    fn config(&self) -> &TransportConfig;

    /// Get the transport type name
    fn transport_type(&self) -> &'static str;

    /// Send a raw JSON-RPC request and receive a raw JSON response
    /// This is the low-level method for sending arbitrary JSON-RPC requests
    async fn send_raw_rpc_request(
        &self,
        request: serde_json::Value,
    ) -> A2aResult<serde_json::Value>;

    /// Send a batch JSON-RPC request (array of requests) and receive batch responses
    /// Per JSON-RPC 2.0: batch requests can contain notifications (no response expected)
    async fn send_raw_batch_request(
        &self,
        requests: JsonRpcBatchRequest,
    ) -> A2aResult<JsonRpcBatchResponse> {
        // Serialize the batch to JSON
        let batch_json = serde_json::to_value(&requests).map_err(|e| crate::A2aError::Json(e))?;

        // Send the batch request
        let response_json = self.send_raw_rpc_request(batch_json).await?;

        // Deserialize the batch response
        let responses: JsonRpcBatchResponse =
            serde_json::from_value(response_json).map_err(|e| crate::A2aError::Json(e))?;

        Ok(responses)
    }
}

/// Extension methods for Transport to provide typed RPC method calls
#[async_trait]
pub trait TransportExt: Transport {
    /// Send a typed JSON-RPC request and get a typed response
    async fn send_rpc_request<T, R>(
        &self,
        request: JsonRpcRequest<T>,
    ) -> A2aResult<JsonRpcResponse<R>>
    where
        T: serde::Serialize + Send + Sync,
        R: serde::de::DeserializeOwned,
    {
        // Serialize the request to JSON
        let request_json = serde_json::to_value(&request).map_err(|e| crate::A2aError::Json(e))?;

        // Send the raw request
        let response_json = self.send_raw_rpc_request(request_json).await?;

        // Deserialize the response
        let response: JsonRpcResponse<R> =
            serde_json::from_value(response_json).map_err(|e| crate::A2aError::Json(e))?;

        Ok(response)
    }

    /// Convenience method: Send a message/send RPC request
    async fn rpc_send_message(
        &self,
        request: crate::MessageSendRequest,
    ) -> A2aResult<JsonRpcResponse<crate::SendResponse>> {
        let rpc_request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: serde_json::json!(uuid::Uuid::new_v4().to_string()),
            method: "message/send".to_string(),
            params: Some(request),
        };

        self.send_rpc_request(rpc_request).await
    }

    /// Convenience method: Send a task/get RPC request
    async fn rpc_get_task(
        &self,
        request: crate::TaskGetRequest,
    ) -> A2aResult<JsonRpcResponse<crate::Task>> {
        let rpc_request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: serde_json::json!(uuid::Uuid::new_v4().to_string()),
            method: "task/get".to_string(),
            params: Some(request),
        };

        self.send_rpc_request(rpc_request).await
    }

    /// Convenience method: Send a task/cancel RPC request
    async fn rpc_cancel_task(
        &self,
        request: crate::TaskCancelRequest,
    ) -> A2aResult<JsonRpcResponse<crate::TaskStatus>> {
        let rpc_request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: serde_json::json!(uuid::Uuid::new_v4().to_string()),
            method: "task/cancel".to_string(),
            params: Some(request),
        };

        self.send_rpc_request(rpc_request).await
    }

    /// Convenience method: Send a task/status RPC request
    async fn rpc_get_task_status(
        &self,
        request: crate::TaskStatusRequest,
    ) -> A2aResult<JsonRpcResponse<crate::TaskStatus>> {
        let rpc_request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: serde_json::json!(uuid::Uuid::new_v4().to_string()),
            method: "task/status".to_string(),
            params: Some(request),
        };

        self.send_rpc_request(rpc_request).await
    }

    /// Convenience method: Send an agent/card RPC request
    async fn rpc_get_agent_card_by_rpc(
        &self,
        request: crate::AgentCardGetRequest,
    ) -> A2aResult<JsonRpcResponse<crate::AgentCard>> {
        let rpc_request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: serde_json::json!(uuid::Uuid::new_v4().to_string()),
            method: "agent/card".to_string(),
            params: Some(request),
        };

        self.send_rpc_request(rpc_request).await
    }

    /// Send a batch of typed JSON-RPC requests
    /// Returns responses in the same order as requests (excluding notifications)
    /// Per JSON-RPC 2.0: notifications don't receive responses
    async fn send_batch_requests(
        &self,
        requests: Vec<serde_json::Value>,
    ) -> A2aResult<JsonRpcBatchResponse> {
        self.send_raw_batch_request(requests).await
    }

    /// Send a JSON-RPC notification (request without expecting a response)
    async fn send_notification<T>(&self, method: &str, params: T) -> A2aResult<()>
    where
        T: serde::Serialize + Send + Sync,
    {
        let notification = JsonRpcNotification {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params: Some(params),
        };

        let notification_json =
            serde_json::to_value(&notification).map_err(|e| crate::A2aError::Json(e))?;

        // Send as a raw request but don't expect a response
        // The server should not send a response for notifications
        let _ = self.send_raw_rpc_request(notification_json).await;

        Ok(())
    }
}

// Blanket implementation: all Transport implementors automatically get TransportExt
impl<T: Transport + ?Sized> TransportExt for T {}

/// Request information for transport implementations
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
