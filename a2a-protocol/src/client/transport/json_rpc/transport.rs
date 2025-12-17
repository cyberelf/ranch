//! JSON-RPC transport implementation for A2A client

use crate::client::transport::http_client::HttpClient;
use crate::client::transport::{RequestInfo, Transport, TransportConfig};
use crate::{A2aError, A2aResult, AgentCard, Message, SendResponse, Task, TaskStatus};
use async_trait::async_trait;
use serde_json::{json, Value};
use uuid::Uuid;

/// JSON-RPC transport for A2A protocol client
#[derive(Debug)]
pub struct JsonRpcTransport {
    http_client: HttpClient,
}

impl JsonRpcTransport {
    /// Create a new JSON-RPC transport
    pub fn new<S: Into<String>>(endpoint: S) -> A2aResult<Self> {
        Self::with_config(endpoint, TransportConfig::default())
    }

    /// Create a new JSON-RPC transport with custom configuration
    pub fn with_config<S: Into<String>>(endpoint: S, config: TransportConfig) -> A2aResult<Self> {
        let http_client = HttpClient::with_config(endpoint, config)?;
        Ok(Self { http_client })
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

        let response = self
            .http_client
            .send_request_with_retry(
                RequestInfo::new("")
                    .with_method("POST")
                    .with_header("Content-Type", "application/json"),
                Some(request),
            )
            .await?;

        let response_data: Value = response.json().await.map_err(A2aError::Network)?;

        // Parse JSON-RPC response
        if let Some(error) = response_data.get("error") {
            return Err(Self::parse_json_rpc_error(error));
        }

        response_data.get("result").cloned().ok_or_else(|| {
            A2aError::ProtocolViolation("Missing result in JSON-RPC response".to_string())
        })
    }

    /// Parse JSON-RPC error
    fn parse_json_rpc_error(error: &Value) -> A2aError {
        if let Some(code) = error.get("code").and_then(|c| c.as_i64()) {
            let message = error
                .get("message")
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
    async fn send_message(&self, message: Message) -> A2aResult<SendResponse> {
        let params = json!({
            "message": message
        });

        // A2A spec-compliant method name: "message/send"
        let result = self.send_json_rpc_request("message/send", params).await?;

        serde_json::from_value(result).map_err(A2aError::Json)
    }

    async fn get_agent_card(&self, agent_id: &crate::AgentId) -> A2aResult<AgentCard> {
        let params = json!({
            "agentId": agent_id.as_str()
        });

        // A2A spec-compliant method name: "agent/card"
        let result = self.send_json_rpc_request("agent/card", params).await?;

        serde_json::from_value(result).map_err(A2aError::Json)
    }

    async fn get_task(&self, request: crate::TaskGetRequest) -> A2aResult<Task> {
        let params = serde_json::to_value(&request).map_err(A2aError::Json)?;

        // A2A spec-compliant method name: "task/get"
        let result = self.send_json_rpc_request("task/get", params).await?;

        serde_json::from_value(result).map_err(A2aError::Json)
    }

    async fn get_task_status(&self, request: crate::TaskStatusRequest) -> A2aResult<TaskStatus> {
        let params = serde_json::to_value(&request).map_err(A2aError::Json)?;

        // A2A spec-compliant method name: "task/status"
        let result = self.send_json_rpc_request("task/status", params).await?;

        serde_json::from_value(result).map_err(A2aError::Json)
    }

    async fn cancel_task(&self, request: crate::TaskCancelRequest) -> A2aResult<TaskStatus> {
        let params = serde_json::to_value(&request).map_err(A2aError::Json)?;

        // A2A spec-compliant method name: "task/cancel"
        let result = self.send_json_rpc_request("task/cancel", params).await?;

        serde_json::from_value(result).map_err(A2aError::Json)
    }

    async fn is_available(&self) -> bool {
        // Try to get agent card as a health check (A2A spec doesn't define a ping method)
        (self.send_json_rpc_request("agent/card", json!({})).await).is_ok()
    }

    fn config(&self) -> &TransportConfig {
        &self.http_client.config
    }

    fn transport_type(&self) -> &'static str {
        "json-rpc"
    }
}

#[cfg(feature = "streaming")]
use crate::client::transport::sse::SseEvent;
#[cfg(feature = "streaming")]
use crate::client::transport::{StreamingResult, StreamingTransport};
#[cfg(feature = "streaming")]
use async_stream::stream;
#[cfg(feature = "streaming")]
use futures_util::stream::{Stream, StreamExt};

#[cfg(feature = "streaming")]
#[async_trait]
impl StreamingTransport for JsonRpcTransport {
    async fn send_streaming_message(
        &self,
        message: Message,
    ) -> A2aResult<Box<dyn Stream<Item = A2aResult<StreamingResult>> + Send + Unpin>> {
        // Build the SSE URL by parsing the base URL and replacing the path
        let base_url = self.http_client.base_url();

        // Parse the base URL to extract scheme://host:port
        let parsed_url = url::Url::parse(base_url)
            .map_err(|e| A2aError::Transport(format!("Invalid base URL: {}", e)))?;

        let mut stream_url = format!(
            "{}://{}/stream",
            parsed_url.scheme(),
            parsed_url.host_str().unwrap_or("localhost")
        );

        if let Some(port) = parsed_url.port() {
            stream_url = format!(
                "{}://{}:{}/stream",
                parsed_url.scheme(),
                parsed_url.host_str().unwrap_or("localhost"),
                port
            );
        }

        // Create the request body using JSON-RPC format
        let request_body = Self::create_request("message/stream", json!({ "message": message }));

        // Use reqwest to establish SSE connection
        let client = reqwest::Client::new();
        let response = client
            .post(&stream_url)
            .header("Accept", "text/event-stream")
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(A2aError::Network)?;

        if !response.status().is_success() {
            return Err(A2aError::Server(format!(
                "Streaming request failed with status: {}",
                response.status()
            )));
        }

        // Parse SSE stream
        let event_stream = stream! {
            let mut bytes_stream = response.bytes_stream();
            let mut buffer = String::new();

            while let Some(chunk_result) = bytes_stream.next().await {
                match chunk_result {
                    Ok(bytes) => {
                        let chunk_str = String::from_utf8_lossy(&bytes);
                        buffer.push_str(&chunk_str);

                        // Process complete events (delimited by double newline)
                        while let Some(pos) = buffer.find("\n\n") {
                            let event_text = buffer[..pos].to_string();
                            buffer = buffer[pos + 2..].to_string();

                            // Parse SSE event using SseEvent
                            match SseEvent::from_sse_format(&event_text) {
                                Ok(sse_event) => {
                                    if let Some(result) = Self::parse_sse_to_streaming_result(&sse_event) {
                                        yield result;
                                    }
                                }
                                Err(e) => {
                                    yield Err(A2aError::ProtocolViolation(format!("Invalid SSE event: {}", e)));
                                }
                            }
                        }
                    }
                    Err(e) => {
                        yield Err(A2aError::Network(e));
                        break;
                    }
                }
            }
        };

        Ok(Box::new(Box::pin(event_stream)))
    }

    async fn resubscribe_task(
        &self,
        request: crate::TaskResubscribeRequest,
    ) -> A2aResult<Box<dyn Stream<Item = A2aResult<StreamingResult>> + Send + Unpin>> {
        // Build the SSE URL by parsing the base URL and replacing the path
        let base_url = self.http_client.base_url();

        // Parse the base URL to extract scheme://host:port
        let parsed_url = url::Url::parse(base_url)
            .map_err(|e| A2aError::Transport(format!("Invalid base URL: {}", e)))?;

        let mut stream_url = format!(
            "{}://{}/stream",
            parsed_url.scheme(),
            parsed_url.host_str().unwrap_or("localhost")
        );

        if let Some(port) = parsed_url.port() {
            stream_url = format!(
                "{}://{}:{}/stream",
                parsed_url.scheme(),
                parsed_url.host_str().unwrap_or("localhost"),
                port
            );
        }

        // Create the request body using JSON-RPC format
        let request_body =
            Self::create_request("task/resubscribe", serde_json::to_value(&request)?);

        // Use reqwest to establish SSE connection
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(300)) // Longer timeout for streaming
            .build()
            .map_err(|e| A2aError::Transport(e.to_string()))?;

        let mut req_builder = client
            .post(&stream_url)
            .header("Accept", "text/event-stream")
            .header("Content-Type", "application/json")
            .json(&request_body);

        // Extract Last-Event-ID from metadata if present
        if let Some(metadata) = &request.metadata {
            if let Some(last_event_id) = metadata.get("lastEventId").and_then(|v| v.as_str()) {
                req_builder = req_builder.header("Last-Event-ID", last_event_id);
            }
        }

        let response = req_builder.send().await.map_err(A2aError::Network)?;

        if !response.status().is_success() {
            return Err(A2aError::Server(format!(
                "Task resubscribe failed with status: {}",
                response.status()
            )));
        }

        // Parse SSE stream
        let event_stream = stream! {
            let mut bytes_stream = response.bytes_stream();
            let mut buffer = String::new();

            while let Some(chunk_result) = bytes_stream.next().await {
                match chunk_result {
                    Ok(bytes) => {
                        let chunk_str = String::from_utf8_lossy(&bytes);
                        buffer.push_str(&chunk_str);

                        // Process complete events (delimited by double newline)
                        while let Some(pos) = buffer.find("\n\n") {
                            let event_text = buffer[..pos].to_string();
                            buffer = buffer[pos + 2..].to_string();

                            // Parse SSE event using SseEvent
                            match SseEvent::from_sse_format(&event_text) {
                                Ok(sse_event) => {
                                    if let Some(result) = Self::parse_sse_to_streaming_result(&sse_event) {
                                        yield result;
                                    }
                                }
                                Err(e) => {
                                    yield Err(A2aError::ProtocolViolation(format!("Invalid SSE event: {}", e)));
                                }
                            }
                        }
                    }
                    Err(e) => {
                        yield Err(A2aError::Network(e));
                        break;
                    }
                }
            }
        };

        Ok(Box::new(Box::pin(event_stream)))
    }

    fn supports_streaming(&self) -> bool {
        true
    }
}

#[cfg(feature = "streaming")]
impl JsonRpcTransport {
    /// Convert an SseEvent into a StreamingResult
    fn parse_sse_to_streaming_result(sse_event: &SseEvent) -> Option<A2aResult<StreamingResult>> {
        let event_type = sse_event.event_type.as_deref().unwrap_or("message");
        let data = &sse_event.data;

        // Parse based on event type
        let result = match event_type {
            "message" => serde_json::from_value::<Message>(data.clone())
                .map(StreamingResult::Message)
                .map_err(A2aError::Json),
            "task" => serde_json::from_value::<Task>(data.clone())
                .map(StreamingResult::Task)
                .map_err(A2aError::Json),
            "task-status-update" => serde_json::from_value::<
                crate::core::streaming_events::TaskStatusUpdateEvent,
            >(data.clone())
            .map(StreamingResult::TaskStatusUpdate)
            .map_err(A2aError::Json),
            "task-artifact-update" => serde_json::from_value::<
                crate::core::streaming_events::TaskArtifactUpdateEvent,
            >(data.clone())
            .map(StreamingResult::TaskArtifactUpdate)
            .map_err(A2aError::Json),
            _ => {
                return Some(Err(A2aError::ProtocolViolation(format!(
                    "Unknown SSE event type: {}",
                    event_type
                ))));
            }
        };

        Some(result)
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
