//! A2A protocol request handler trait

use crate::{
    A2aResult, AgentCard, AgentCardGetRequest, Message, MessageSendRequest, Part, SendResponse,
    PushNotificationConfig, PushNotificationDeleteRequest, PushNotificationGetRequest,
    PushNotificationListRequest, PushNotificationListResponse, PushNotificationSetRequest,
    Task, TaskCancelRequest, TaskGetRequest, TaskResubscribeRequest, TaskStatus, TaskStatusRequest,
};
use async_trait::async_trait;

#[cfg(feature = "streaming")]
use crate::transport::StreamingResult;
#[cfg(feature = "streaming")]
use futures_util::stream::Stream;

/// Trait for handling A2A protocol requests
#[async_trait]
pub trait A2aHandler: Send + Sync {
    /// Handle an incoming message - returns Task (async) or Message (immediate)
    async fn handle_message(&self, message: Message) -> A2aResult<SendResponse>;

    /// Get the agent card for this handler
    async fn get_agent_card(&self) -> A2aResult<AgentCard>;

    /// Handle health check requests
    async fn health_check(&self) -> A2aResult<HealthStatus>;

    /// Optional: Handle streaming requests
    async fn handle_streaming_message(&self, _message: Message) -> A2aResult<StreamingResponse> {
        Err(crate::A2aError::ProtocolViolation(
            "Streaming not supported".to_string(),
        ))
    }

    /// RPC method: message/send
    /// Handle a message/send request with optional immediate flag
    async fn rpc_message_send(&self, request: MessageSendRequest) -> A2aResult<SendResponse> {
        // Default implementation delegates to handle_message
        // Subclasses can override to handle the immediate flag differently
        self.handle_message(request.message).await
    }

    /// RPC method: task/get
    /// Retrieve a task by ID
    async fn rpc_task_get(&self, _request: TaskGetRequest) -> A2aResult<Task> {
        Err(crate::A2aError::Server(
            "task/get not implemented".to_string(),
        ))
    }

    /// RPC method: task/cancel
    /// Cancel a running task
    async fn rpc_task_cancel(&self, _request: TaskCancelRequest) -> A2aResult<TaskStatus> {
        Err(crate::A2aError::Server(
            "task/cancel not implemented".to_string(),
        ))
    }

    /// RPC method: task/status
    /// Get the status of a task
    async fn rpc_task_status(&self, _request: TaskStatusRequest) -> A2aResult<TaskStatus> {
        Err(crate::A2aError::Server(
            "task/status not implemented".to_string(),
        ))
    }

    /// RPC method: agent/card
    /// Get agent card (optionally for a specific agent)
    async fn rpc_agent_card(&self, _request: AgentCardGetRequest) -> A2aResult<AgentCard> {
        // Default implementation returns this handler's agent card
        self.get_agent_card().await
    }

    /// RPC method: message/stream (SSE streaming)
    /// Handle a streaming message request - returns a stream of results
    #[cfg(feature = "streaming")]
    async fn rpc_message_stream(
        &self,
        message: Message,
    ) -> A2aResult<Box<dyn Stream<Item = A2aResult<StreamingResult>> + Send + Unpin>>;

    /// RPC method: task/resubscribe (SSE streaming)
    /// Resubscribe to a task's event stream
    #[cfg(feature = "streaming")]
    async fn rpc_task_resubscribe(
        &self,
        request: TaskResubscribeRequest,
    ) -> A2aResult<Box<dyn Stream<Item = A2aResult<StreamingResult>> + Send + Unpin>>;

    /// RPC method: tasks/pushNotificationConfig/set
    /// Set push notification configuration for a task
    async fn rpc_push_notification_set(
        &self,
        _request: PushNotificationSetRequest,
    ) -> A2aResult<()> {
        Err(crate::A2aError::PushNotificationNotSupported)
    }

    /// RPC method: tasks/pushNotificationConfig/get
    /// Get push notification configuration for a task
    async fn rpc_push_notification_get(
        &self,
        _request: PushNotificationGetRequest,
    ) -> A2aResult<Option<PushNotificationConfig>> {
        Err(crate::A2aError::PushNotificationNotSupported)
    }

    /// RPC method: tasks/pushNotificationConfig/list
    /// List all push notification configurations
    async fn rpc_push_notification_list(
        &self,
        _request: PushNotificationListRequest,
    ) -> A2aResult<PushNotificationListResponse> {
        Err(crate::A2aError::PushNotificationNotSupported)
    }

    /// RPC method: tasks/pushNotificationConfig/delete
    /// Delete push notification configuration for a task
    async fn rpc_push_notification_delete(
        &self,
        _request: PushNotificationDeleteRequest,
    ) -> A2aResult<bool> {
        Err(crate::A2aError::PushNotificationNotSupported)
    }
}

/// Health status response
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HealthStatus {
    /// Overall health status
    pub status: HealthStatusType,

    /// Optional status message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,

    /// Optional version information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    /// Optional detailed status information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

/// Health status types
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatusType {
    /// Service is healthy
    Healthy,

    /// Service is degraded but still functional
    Degraded,

    /// Service is unhealthy
    Unhealthy,
}

impl HealthStatus {
    /// Create a healthy status
    pub fn healthy() -> Self {
        Self {
            status: HealthStatusType::Healthy,
            message: Some("Service is healthy".to_string()),
            version: None,
            details: None,
        }
    }

    /// Create a degraded status
    pub fn degraded<S: Into<String>>(message: S) -> Self {
        Self {
            status: HealthStatusType::Degraded,
            message: Some(message.into()),
            version: None,
            details: None,
        }
    }

    /// Create an unhealthy status
    pub fn unhealthy<S: Into<String>>(message: S) -> Self {
        Self {
            status: HealthStatusType::Unhealthy,
            message: Some(message.into()),
            version: None,
            details: None,
        }
    }

    /// Set version information
    pub fn with_version<S: Into<String>>(mut self, version: S) -> Self {
        self.version = Some(version.into());
        self
    }

    /// Set details
    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }
}

/// Streaming response for handling streaming messages
#[derive(Debug)]
pub struct StreamingResponse {
    /// Stream of message parts
    pub stream: tokio::sync::mpsc::UnboundedReceiver<Part>,

    /// Complete message when streaming is done
    pub final_message: Option<Message>,
}

impl StreamingResponse {
    /// Create a new streaming response
    pub fn new(stream: tokio::sync::mpsc::UnboundedReceiver<Part>) -> Self {
        Self {
            stream,
            final_message: None,
        }
    }

    /// Create a streaming response with final message
    pub fn with_final_message(
        stream: tokio::sync::mpsc::UnboundedReceiver<Part>,
        final_message: Message,
    ) -> Self {
        Self {
            stream,
            final_message: Some(final_message),
        }
    }
}

/// Basic handler implementation
pub struct BasicA2aHandler {
    agent_card: AgentCard,
}

impl BasicA2aHandler {
    /// Create a new basic handler
    pub fn new(agent_card: AgentCard) -> Self {
        Self { agent_card }
    }

    /// Set the agent card
    pub fn with_agent_card(mut self, agent_card: AgentCard) -> Self {
        self.agent_card = agent_card;
        self
    }
}

#[async_trait]
impl A2aHandler for BasicA2aHandler {
    async fn handle_message(&self, message: Message) -> A2aResult<SendResponse> {
        // Simple echo handler for demonstration - returns immediate message response
        let content = message.text_content().unwrap_or("No content");
        let echo_content = format!("Echo: {}", content);
        let response_message = Message::agent_text(echo_content);

        Ok(SendResponse::message(response_message))
    }

    async fn get_agent_card(&self) -> A2aResult<AgentCard> {
        Ok(self.agent_card.clone())
    }

    async fn health_check(&self) -> A2aResult<HealthStatus> {
        Ok(HealthStatus::healthy()
            .with_version(env!("CARGO_PKG_VERSION"))
            .with_details(serde_json::json!({
                "handler": "BasicA2aHandler",
                "timestamp": chrono::Utc::now().to_rfc3339()
            })))
    }

    #[cfg(feature = "streaming")]
    async fn rpc_message_stream(
        &self,
        _message: Message,
    ) -> A2aResult<Box<dyn Stream<Item = A2aResult<StreamingResult>> + Send + Unpin>> {
        Err(crate::A2aError::Server(
            "BasicA2aHandler does not support streaming".to_string(),
        ))
    }

    #[cfg(feature = "streaming")]
    async fn rpc_task_resubscribe(
        &self,
        _request: TaskResubscribeRequest,
    ) -> A2aResult<Box<dyn Stream<Item = A2aResult<StreamingResult>> + Send + Unpin>> {
        Err(crate::A2aError::Server(
            "BasicA2aHandler does not support streaming".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AgentId, MessageId};
    use url::Url;

    #[tokio::test]
    async fn test_basic_handler() {
        let agent_id = AgentId::new("test-agent".to_string()).unwrap();
        let agent_card = AgentCard::new(
            agent_id,
            "Test Agent",
            Url::parse("https://example.com").unwrap(),
        );

        let handler = BasicA2aHandler::new(agent_card);

        let message = Message::user_text("Hello, world!");
        let response = handler.handle_message(message).await.unwrap();

        // Response should be immediate message
        assert!(response.is_message());
        let response_msg = response.as_message().unwrap();
        assert_eq!(response_msg.text_content(), Some("Echo: Hello, world!"));
    }

    #[tokio::test]
    async fn test_health_check() {
        let agent_id = AgentId::new("test-agent".to_string()).unwrap();
        let agent_card = AgentCard::new(
            agent_id,
            "Test Agent",
            Url::parse("https://example.com").unwrap(),
        );

        let handler = BasicA2aHandler::new(agent_card);
        let health = handler.health_check().await.unwrap();

        assert!(matches!(health.status, HealthStatusType::Healthy));
        assert!(health.version.is_some());
    }

    #[test]
    fn test_health_status_creation() {
        let status = HealthStatus::healthy()
            .with_version("1.0.0")
            .with_details(serde_json::json!({"key": "value"}));

        assert!(matches!(status.status, HealthStatusType::Healthy));
        assert_eq!(status.version, Some("1.0.0".to_string()));
        assert!(status.details.is_some());
    }
}
