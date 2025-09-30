//! JSON-RPC transport implementation

use async_trait::async_trait;
use crate::{
    Message, MessageResponse, AgentCard, A2aResult, A2aError,
    transport::{Transport, TransportConfig, RequestInfo, HttpTransport},
};
use serde_json::{json, Value};
use uuid::Uuid;

/// JSON-RPC transport for A2A protocol
#[derive(Debug)]
pub struct JsonRpcTransport {
    http_transport: HttpTransport,
}

impl JsonRpcTransport {
    /// Create a new JSON-RPC transport
    pub fn new<S: Into<String>>(endpoint: S) -> A2aResult<Self> {
        Self::with_config(endpoint, TransportConfig::default())
    }

    /// Create a new JSON-RPC transport with custom configuration
    pub fn with_config<S: Into<String>>(endpoint: S, config: TransportConfig) -> A2aResult<Self> {
        let http_transport = HttpTransport::with_config(endpoint, config)?;
        Ok(Self { http_transport })
    }

    /// Create a JSON-RPC request
    fn create_request(method: &str, params: Value) -> Value {
        json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": Uuid::new_v4().to_string()
        })
    }

    /// Send a JSON-RPC request and parse the response
    async fn send_json_rpc_request(&self, method: &str, params: Value) -> A2aResult<Value> {
        let request = Self::create_request(method, params);

        let response = self.http_transport.send_request_with_retry(
            RequestInfo::new("")
                .with_method("POST")
                .with_header("Content-Type", "application/json-rpc"),
            Some(request),
        ).await?;

        let response_data: Value = response.json().await
            .map_err(|e| A2aError::Network(e))?;

        // Parse JSON-RPC response
        if let Some(error) = response_data.get("error") {
            return Err(Self::parse_json_rpc_error(error));
        }

        response_data
            .get("result")
            .cloned()
            .ok_or_else(|| A2aError::ProtocolViolation("Missing result in JSON-RPC response".to_string()))
    }

    /// Parse JSON-RPC error
    fn parse_json_rpc_error(error: &Value) -> A2aError {
        if let Some(code) = error.get("code").and_then(|c| c.as_i64()) {
            let message = error.get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown JSON-RPC error");

            match code {
                -32600 => A2aError::Validation(message.to_string()),
                -32601 => A2aError::ProtocolViolation(format!("Method not found: {}", message)),
                -32602 => A2aError::Validation(format!("Invalid params: {}", message)),
                -32603 => A2aError::Internal(message.to_string()),
                -32700 => A2aError::ProtocolViolation(format!("Parse error: {}", message)),
                _ => A2aError::Server(format!("JSON-RPC error {}: {}", code, message)),
            }
        } else {
            A2aError::ProtocolViolation("Invalid JSON-RPC error format".to_string())
        }
    }
}

#[async_trait]
impl Transport for JsonRpcTransport {
    async fn send_message(&self, message: Message) -> A2aResult<MessageResponse> {
        let params = json!({
            "message": message
        });

        let result = self.send_json_rpc_request("a2a.sendMessage", params).await?;

        serde_json::from_value(result)
            .map_err(|e| A2aError::Json(e))
    }

    async fn get_agent_card(&self, agent_id: &crate::AgentId) -> A2aResult<AgentCard> {
        let params = json!({
            "agentId": agent_id.as_str()
        });

        let result = self.send_json_rpc_request("a2a.getAgentCard", params).await?;

        serde_json::from_value(result)
            .map_err(|e| A2aError::Json(e))
    }

    async fn is_available(&self) -> bool {
        match self.send_json_rpc_request("a2a.ping", json!({})).await {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    fn config(&self) -> &TransportConfig {
        self.http_transport.config()
    }

    fn transport_type(&self) -> &'static str {
        "json-rpc"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_rpc_transport_creation() {
        let transport = JsonRpcTransport::new("https://example.com/rpc").unwrap();
        assert_eq!(transport.transport_type(), "json-rpc");
    }

    #[test]
    fn test_json_rpc_request_creation() {
        let request = JsonRpcTransport::create_request("test.method", json!({"param": "value"}));

        assert_eq!(request.get("jsonrpc"), Some(&json!("2.0")));
        assert_eq!(request.get("method"), Some(&json!("test.method")));
        assert!(request.get("id").is_some());
    }

    #[test]
    fn test_json_rpc_error_parsing() {
        let error = json!({
            "code": -32601,
            "message": "Method not found"
        });

        let a2a_error = JsonRpcTransport::parse_json_rpc_error(&error);
        assert!(matches!(a2a_error, A2aError::ProtocolViolation(_)));
    }
}