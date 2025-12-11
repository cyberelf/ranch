//! A2A client implementation

use crate::{
    A2aResult, AgentCard, AgentId, Message, SendResponse, Task, TaskStatus, client::transport::{Transport, TransportConfig}
};
use std::sync::Arc;

/// A2A client for communicating with agents
#[derive(Clone, Debug)]
pub struct A2aClient {
    transport: Arc<dyn Transport>,
    agent_id: AgentId,
}

impl A2aClient {
    /// Create a new A2A client
    pub fn new(transport: Arc<dyn Transport>) -> Self {
        Self {
            transport,
            agent_id: AgentId::generate(), // Default client ID
        }
    }

    /// Create a new A2A client with specific agent ID
    pub fn with_agent_id(transport: Arc<dyn Transport>, agent_id: AgentId) -> Self {
        Self {
            transport,
            agent_id,
        }
    }

    /// Get the agent ID used by this client
    pub fn agent_id(&self) -> &AgentId {
        &self.agent_id
    }

    /// Get the transport used by this client
    pub fn transport(&self) -> &Arc<dyn Transport> {
        &self.transport
    }

    /// Send a message and wait for response
    pub async fn send_message(&self, message: Message) -> A2aResult<SendResponse> {
        self.transport.send_message(message).await
    }

    /// Send a text message and wait for response
    pub async fn send_text<S: Into<String>>(&self, text: S) -> A2aResult<SendResponse> {
        let message = Message::user_text(text);
        self.send_message(message).await
    }

    /// Get a task by ID
    pub async fn get_task(&self, request: crate::TaskGetRequest) -> A2aResult<Task> {
        self.transport.get_task(request).await
    }

    /// Get the status of a task
    pub async fn get_task_status(&self, request: crate::TaskStatusRequest) -> A2aResult<TaskStatus> {
        self.transport.get_task_status(request).await
    }

    /// Cancel a task
    pub async fn cancel_task(&self, request: crate::TaskCancelRequest) -> A2aResult<TaskStatus> {
        self.transport.cancel_task(request).await
    }

    /// Fetch the agent card for a target agent
    pub async fn get_agent_card(&self) -> A2aResult<AgentCard> {
        self.transport.get_agent_card(&self.agent_id).await
    }

    /// Check if the client can communicate with the agent
    pub async fn is_available(&self) -> bool {
        self.transport.is_available().await
    }

    /// Get the transport configuration
    pub fn config(&self) -> &TransportConfig {
        self.transport.config()
    }

    /// Get the transport type
    pub fn transport_type(&self) -> &str {
        self.transport.transport_type()
    }

    /// Send a message with retries for retryable errors
    pub async fn send_message_with_retry(
        &self,
        message: Message,
        max_retries: u32,
    ) -> A2aResult<SendResponse> {
        let mut last_error = None;
        let mut retry_count = 0;

        while retry_count <= max_retries {
            match self.send_message(message.clone()).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    let is_retryable = e.is_retryable();
                    last_error = Some(e);

                    if !is_retryable || retry_count >= max_retries {
                        break;
                    }

                    // Exponential backoff
                    let backoff = tokio::time::Duration::from_secs(2u64.pow(retry_count as u32));
                    tokio::time::sleep(backoff).await;
                    retry_count += 1;
                }
            }
        }

        Err(last_error
            .unwrap_or_else(|| crate::A2aError::Internal("Unknown error occurred".to_string())))
    }

    /// Create a new conversation with an agent
    pub async fn start_conversation(&self) -> A2aResult<Conversation> {
        let agent_card = self.get_agent_card().await?;
        Ok(Conversation::new(
            self.clone(),
            self.agent_id.clone(),
            agent_card,
        ))
    }
}

/// A2A conversation helper
#[derive(Clone)]
pub struct Conversation {
    client: A2aClient,
    agent_id: AgentId,
    agent_card: AgentCard,
    messages: Vec<Message>,
}

impl Conversation {
    /// Create a new conversation
    pub fn new(client: A2aClient, agent_id: AgentId, agent_card: AgentCard) -> Self {
        Self {
            client,
            agent_id,
            agent_card,
            messages: Vec::new(),
        }
    }

    /// Get the agent ID for this conversation
    pub fn agent_id(&self) -> &AgentId {
        &self.agent_id
    }

    /// Get the agent card for this conversation
    pub fn agent_card(&self) -> &AgentCard {
        &self.agent_card
    }

    /// Get all messages in this conversation
    pub fn messages(&self) -> &[Message] {
        &self.messages
    }

    /// Send a message in this conversation
    pub async fn send_message(&mut self, message: Message) -> A2aResult<SendResponse> {
        let response = self.client.send_message(message.clone()).await?;
        self.messages.push(message);
        Ok(response)
    }

    /// Send a text message in this conversation
    pub async fn send_text<S: Into<String>>(&mut self, text: S) -> A2aResult<SendResponse> {
        let message = Message::user_text(text);
        self.send_message(message).await
    }

    /// Get the conversation history
    pub fn history(&self) -> Vec<&Message> {
        self.messages.iter().collect()
    }

    /// Clear the conversation history
    pub fn clear_history(&mut self) {
        self.messages.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::transport::JsonRpcTransport;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_client_creation() {
        let transport = Arc::new(JsonRpcTransport::new("https://example.com/rpc").unwrap());
        let client = A2aClient::new(transport);

        assert_eq!(client.transport_type(), "json-rpc");
    }

    #[tokio::test]
    async fn test_client_with_agent_id() {
        let transport = Arc::new(JsonRpcTransport::new("https://example.com/rpc").unwrap());
        let agent_id = AgentId::new("test-agent".to_string()).unwrap();
        let client = A2aClient::with_agent_id(transport, agent_id.clone());

        assert_eq!(client.agent_id(), &agent_id);
    }

}
