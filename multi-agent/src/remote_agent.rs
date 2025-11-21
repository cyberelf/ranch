//! Remote agent implementation using A2A protocol
//!
//! This module provides the `RemoteAgent` struct which wraps an `A2aClient`
//! and implements the multi-agent `Agent` trait for coordination.

use crate::AgentInfo;
use a2a_protocol::prelude::*;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Configuration for remote agent runtime behavior
#[derive(Debug, Clone, PartialEq)]
pub struct RemoteAgentConfig {
    /// Maximum retry attempts for transient failures
    pub max_retries: u32,
    
    /// How to handle async task responses
    pub task_handling: TaskHandling,
}

impl Default for RemoteAgentConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            task_handling: TaskHandling::PollUntilComplete,
        }
    }
}

/// Strategy for handling async task responses
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskHandling {
    /// Poll the task until completion (blocking)
    PollUntilComplete,
    
    /// Return task info immediately without waiting
    ReturnTaskInfo,
    
    /// Reject async tasks with an error
    RejectTasks,
}

/// Remote agent that communicates via A2A protocol
///
/// This agent wraps an `A2aClient` and provides caching of the agent card
/// for efficient operation.
pub struct RemoteAgent {
    /// A2A client for communication
    client: A2aClient,
    
    /// Configuration for runtime behavior
    config: RemoteAgentConfig,
    
    /// Cached agent card
    card_cache: Arc<RwLock<Option<AgentCard>>>,
}

impl RemoteAgent {
    /// Create a new remote agent with default configuration
    pub fn new(client: A2aClient) -> Self {
        Self {
            client,
            config: RemoteAgentConfig::default(),
            card_cache: Arc::new(RwLock::new(None)),
        }
    }
    
    /// Create a new remote agent with custom configuration
    pub fn with_config(client: A2aClient, config: RemoteAgentConfig) -> Self {
        Self {
            client,
            config,
            card_cache: Arc::new(RwLock::new(None)),
        }
    }
    
    /// Get the underlying A2A client
    pub fn client(&self) -> &A2aClient {
        &self.client
    }
    
    /// Get the agent configuration
    pub fn config(&self) -> &RemoteAgentConfig {
        &self.config
    }
    
    /// Fetch the agent card (with caching)
    async fn fetch_card(&self) -> A2aResult<AgentCard> {
        // Check cache
        {
            let cache = self.card_cache.read().await;
            if let Some(card) = &*cache {
                return Ok(card.clone());
            }
        }
        
        // Fetch from remote - use our own agent_id to get our card
        let card = self.client.transport().get_agent_card(&self.client.agent_id()).await?;
        
        // Update cache
        {
            let mut cache = self.card_cache.write().await;
            *cache = Some(card.clone());
        }
        
        Ok(card)
    }
    
    /// Handle a task response based on configuration
    async fn handle_task(&self, task: Task) -> A2aResult<Message> {
        match self.config.task_handling {
            TaskHandling::PollUntilComplete => self.poll_task_to_completion(task).await,
            TaskHandling::ReturnTaskInfo => Ok(Message::agent_text(format!(
                "Task created: {} ({:?})",
                task.id,
                task.status.state
            ))),
            TaskHandling::RejectTasks => Err(A2aError::Internal(
                "Async tasks not supported by this agent".to_string(),
            )),
        }
    }
    
    /// Poll a task until completion
    ///
    /// This method will poll the task status every 500ms until the task
    /// reaches a terminal state (Completed, Failed, or Cancelled).
    async fn poll_task_to_completion(&self, mut task: Task) -> A2aResult<Message> {
        loop {
            match task.status.state {
                TaskState::Completed => {
                    // Extract result from artifacts if available
                    if let Some(artifacts) = &task.artifacts {
                        if let Some(artifact) = artifacts.first() {
                            if let Some(data) = &artifact.data {
                                return Ok(Message::agent_text(data.to_string()));
                            }
                        }
                    }
                    return Ok(Message::agent_text("Task completed (no artifacts)"));
                }
                TaskState::Failed => {
                    let reason = task
                        .status
                        .reason
                        .unwrap_or_else(|| "Unknown error".to_string());
                    return Err(A2aError::Internal(format!(
                        "Task {} failed: {}",
                        task.id, reason
                    )));
                }
                TaskState::Cancelled => {
                    return Err(A2aError::Internal(format!(
                        "Task {} was cancelled",
                        task.id
                    )));
                }
                _ => {
                    // Still processing - wait and poll
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                    
                    let request = TaskGetRequest {
                        task_id: task.id.clone(),
                    };
                    task = self.client.transport().get_task(request).await?;
                }
            }
        }
    }
    
    /// Clear the profile and card caches
    ///
    /// This is useful if the remote agent's capabilities have changed
    /// Clear the agent card cache
    pub async fn clear_cache(&self) {
        let mut cache = self.card_cache.write().await;
        *cache = None;
    }
}

#[async_trait]
impl crate::Agent for RemoteAgent {
    async fn info(&self) -> A2aResult<AgentInfo> {
        // Check cache
        {
            let cache = self.card_cache.read().await;
            if let Some(card) = &*cache {
                return Ok(AgentInfo {
                    id: card.id.to_string(),
                    name: card.name.clone(),
                    description: card.description.clone().unwrap_or_default(),
                    capabilities: card.capabilities.iter().map(|c| c.name.clone()).collect(),
                    metadata: card.metadata.iter().map(|(k, v)| (k.clone(), v.to_string())).collect(),
                });
            }
        }
        
        // Fetch card and extract info
        let card = self.fetch_card().await?;
        let info = AgentInfo {
            id: card.id.to_string(),
            name: card.name.clone(),
            description: card.description.clone().unwrap_or_default(),
            capabilities: card.capabilities.iter().map(|c| c.name.clone()).collect(),
            metadata: card.metadata.iter().map(|(k, v)| (k.clone(), v.to_string())).collect(),
        };
        
        Ok(info)
    }
    
    async fn process(&self, message: Message) -> A2aResult<Message> {
        let response = self
            .client
            .send_message_with_retry(message, self.config.max_retries)
            .await?;
        
        match response {
            SendResponse::Message(msg) => Ok(msg),
            SendResponse::Task(task) => self.handle_task(task).await,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_remote_agent_creation() {
        let transport = JsonRpcTransport::new("https://example.com/rpc").unwrap();
        let client = A2aClient::new(Arc::new(transport));
        let agent = RemoteAgent::new(client);
        
        assert_eq!(agent.config.max_retries, 3);
        assert_eq!(agent.config.task_handling, TaskHandling::PollUntilComplete);
    }
    
    #[tokio::test]
    async fn test_remote_agent_with_config() {
        let transport = JsonRpcTransport::new("https://example.com/rpc").unwrap();
        let client = A2aClient::new(Arc::new(transport));
        
        let config = RemoteAgentConfig {
            max_retries: 5,
            task_handling: TaskHandling::ReturnTaskInfo,
        };
        
        let agent = RemoteAgent::with_config(client, config);
        
        assert_eq!(agent.config.max_retries, 5);
        assert_eq!(agent.config.task_handling, TaskHandling::ReturnTaskInfo);
    }
}
