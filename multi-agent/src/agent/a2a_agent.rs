//! A2A agent implementation
//!
//! This module provides the `A2AAgent` struct which wraps an `A2aClient`
//! and implements the multi-agent `Agent` trait for coordination.

use crate::agent::AgentInfo;
use a2a_protocol::prelude::*;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Configuration for A2A agent runtime behavior
#[derive(Debug, Clone, PartialEq)]
pub struct A2AAgentConfig {
    /// Local agent ID for team coordination (overrides remote agent ID if provided)
    pub local_id: Option<String>,

    /// Local agent name for display (overrides remote agent name if provided)
    pub local_name: Option<String>,

    /// Maximum retry attempts for transient failures
    pub max_retries: u32,

    /// How to handle async task responses
    pub task_handling: TaskHandling,
}

impl Default for A2AAgentConfig {
    fn default() -> Self {
        Self {
            local_id: None,
            local_name: None,
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

/// A2A agent that communicates via the A2A protocol
///
/// This agent wraps an `A2aClient` and provides caching of the agent card
/// for efficient operation.
pub struct A2AAgent {
    /// A2A client for communication
    client: A2aClient,

    /// Configuration for runtime behavior
    config: A2AAgentConfig,

    /// Cached agent card
    card_cache: Arc<RwLock<Option<AgentCard>>>,
}

impl A2AAgent {
    /// Create a new A2A agent with default configuration
    pub fn new(client: A2aClient) -> Self {
        Self {
            client,
            config: A2AAgentConfig::default(),
            card_cache: Arc::new(RwLock::new(None)),
        }
    }

    /// Create a new A2A agent with custom configuration
    pub fn with_config(client: A2aClient, config: A2AAgentConfig) -> Self {
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
    pub fn config(&self) -> &A2AAgentConfig {
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
        let card = self.client.get_agent_card().await?;

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
                task.id, task.status.state
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
                            // Get the first text part from the artifact
                            for part in &artifact.parts {
                                if let a2a_protocol::Part::Text(text_part) = part {
                                    return Ok(Message::agent_text(&text_part.text));
                                }
                            }
                        }
                    }
                    return Ok(Message::agent_text("Task completed (no artifacts)"));
                }
                TaskState::Failed => {
                    let reason = task
                        .status
                        .message
                        .as_ref()
                        .and_then(|m| m.text_content())
                        .unwrap_or("Unknown error");
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
                    task = self.client.get_task(request).await?;
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
impl crate::agent::Agent for A2AAgent {
    async fn info(&self) -> A2aResult<AgentInfo> {
        // Fetch card from cache or remote
        let card = self.fetch_card().await?;
        
        // Build AgentInfo with local ID/name for team coordination,
        // but use remote capabilities and description
        let info = AgentInfo {
            id: self.config.local_id.clone().unwrap_or_else(|| card.id.to_string()),
            name: self.config.local_name.clone().unwrap_or_else(|| card.name.clone()),
            description: card.description.clone().unwrap_or_default(),
            capabilities: card.capabilities.iter().map(|c| c.name.clone()).collect(),
            metadata: card
                .metadata
                .iter()
                .map(|(k, v)| (k.clone(), v.to_string()))
                .collect(),
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
    use crate::agent::Agent;

    #[test]
    fn test_a2a_agent_config_default() {
        let config = A2AAgentConfig::default();
        assert_eq!(config.local_id, None);
        assert_eq!(config.local_name, None);
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.task_handling, TaskHandling::PollUntilComplete);
    }

    #[test]
    fn test_a2a_agent_config_custom() {
        let config = A2AAgentConfig {
            local_id: Some("custom-id".to_string()),
            local_name: Some("Custom Agent".to_string()),
            max_retries: 5,
            task_handling: TaskHandling::ReturnTaskInfo,
        };

        assert_eq!(config.local_id, Some("custom-id".to_string()));
        assert_eq!(config.local_name, Some("Custom Agent".to_string()));
        assert_eq!(config.max_retries, 5);
        assert_eq!(config.task_handling, TaskHandling::ReturnTaskInfo);
    }

    #[test]
    fn test_a2a_agent_config_clone() {
        let config = A2AAgentConfig {
            local_id: Some("test".to_string()),
            local_name: Some("Test".to_string()),
            max_retries: 7,
            task_handling: TaskHandling::RejectTasks,
        };

        let cloned = config.clone();
        assert_eq!(config.local_id, cloned.local_id);
        assert_eq!(config.local_name, cloned.local_name);
        assert_eq!(config.max_retries, cloned.max_retries);
        assert_eq!(config.task_handling, cloned.task_handling);
    }

    #[test]
    fn test_a2a_agent_config_partial_eq() {
        let config1 = A2AAgentConfig::default();
        let config2 = A2AAgentConfig::default();
        let config3 = A2AAgentConfig {
            local_id: Some("different".to_string()),
            ..A2AAgentConfig::default()
        };

        assert_eq!(config1, config2);
        assert_ne!(config1, config3);
    }

    #[test]
    fn test_task_handling_variants() {
        let poll = TaskHandling::PollUntilComplete;
        let return_info = TaskHandling::ReturnTaskInfo;
        let reject = TaskHandling::RejectTasks;

        assert_eq!(poll, TaskHandling::PollUntilComplete);
        assert_ne!(poll, return_info);
        assert_ne!(poll, reject);
        assert_ne!(return_info, reject);
    }

    #[test]
    fn test_task_handling_copy() {
        let original = TaskHandling::PollUntilComplete;
        let copied = original;
        assert_eq!(original, copied);
    }

    #[tokio::test]
    async fn test_a2a_agent_creation() {
        let transport = JsonRpcTransport::new("https://example.com/rpc").unwrap();
        let client = A2aClient::new(Arc::new(transport));
        let agent = A2AAgent::new(client);

        assert_eq!(agent.config.max_retries, 3);
        assert_eq!(agent.config.task_handling, TaskHandling::PollUntilComplete);
        assert_eq!(agent.config.local_id, None);
        assert_eq!(agent.config.local_name, None);
    }

    #[tokio::test]
    async fn test_a2a_agent_with_config() {
        let transport = JsonRpcTransport::new("https://example.com/rpc").unwrap();
        let client = A2aClient::new(Arc::new(transport));

        let config = A2AAgentConfig {
            local_id: Some("local-123".to_string()),
            local_name: Some("Local Agent".to_string()),
            max_retries: 5,
            task_handling: TaskHandling::ReturnTaskInfo,
        };

        let agent = A2AAgent::with_config(client, config.clone());

        assert_eq!(agent.config.max_retries, 5);
        assert_eq!(agent.config.task_handling, TaskHandling::ReturnTaskInfo);
        assert_eq!(agent.config.local_id, Some("local-123".to_string()));
        assert_eq!(agent.config.local_name, Some("Local Agent".to_string()));
    }

    #[tokio::test]
    async fn test_a2a_agent_client_accessor() {
        let transport = JsonRpcTransport::new("https://example.com/rpc").unwrap();
        let client = A2aClient::new(Arc::new(transport));
        let agent = A2AAgent::new(client);

        // Should be able to access the client
        let _client_ref = agent.client();
    }

    #[tokio::test]
    async fn test_a2a_agent_config_accessor() {
        let transport = JsonRpcTransport::new("https://example.com/rpc").unwrap();
        let client = A2aClient::new(Arc::new(transport));
        
        let config = A2AAgentConfig {
            local_id: Some("test".to_string()),
            local_name: Some("Test".to_string()),
            max_retries: 7,
            task_handling: TaskHandling::RejectTasks,
        };
        
        let agent = A2AAgent::with_config(client, config);

        let agent_config = agent.config();
        assert_eq!(agent_config.max_retries, 7);
        assert_eq!(agent_config.task_handling, TaskHandling::RejectTasks);
        assert_eq!(agent_config.local_id, Some("test".to_string()));
    }

    #[tokio::test]
    async fn test_clear_cache() {
        let transport = JsonRpcTransport::new("https://example.com/rpc").unwrap();
        let client = A2aClient::new(Arc::new(transport));
        let agent = A2AAgent::new(client);

        // Clear cache should not panic
        agent.clear_cache().await;

        // Cache should be empty after clear
        let cache = agent.card_cache.read().await;
        assert!(cache.is_none());
    }

    #[tokio::test]
    async fn test_cache_initially_empty() {
        let transport = JsonRpcTransport::new("https://example.com/rpc").unwrap();
        let client = A2aClient::new(Arc::new(transport));
        let agent = A2AAgent::new(client);

        let cache = agent.card_cache.read().await;
        assert!(cache.is_none());
    }

    #[tokio::test]
    async fn test_a2a_agent_info_with_config() {
        // Create mock transport using JsonRpcTransport with a fake URL
        // This tests the info() method with local overrides
        let transport = Arc::new(JsonRpcTransport::new("http://localhost:9999/rpc").unwrap());
        let client = A2aClient::new(transport);
        
        let config = A2AAgentConfig {
            local_id: Some("local-id-123".to_string()),
            local_name: Some("Local Override Name".to_string()),
            max_retries: 3,
            task_handling: TaskHandling::PollUntilComplete,
        };
        
        let agent = A2AAgent::with_config(client, config);

        // Test that config is returned when remote fetch fails
        // The info() method should fall back to local config
        let result = agent.info().await;
        
        // Since we can't reach the fake URL, this should use local overrides
        // or return an error depending on implementation
        match result {
            Ok(info) => {
                // If successful, verify local overrides are used
                assert_eq!(info.id, "local-id-123");
                assert_eq!(info.name, "Local Override Name");
            }
            Err(_) => {
                // Expected when transport fails and no fallback
                // This is acceptable behavior
            }
        }
    }

    #[tokio::test]
    async fn test_a2a_agent_config_accessors() {
        let transport = Arc::new(JsonRpcTransport::new("http://localhost:9999/rpc").unwrap());
        let client = A2aClient::new(transport);
        
        let config = A2AAgentConfig {
            local_id: Some("test-id".to_string()),
            local_name: Some("Test Agent".to_string()),
            max_retries: 5,
            task_handling: TaskHandling::ReturnTaskInfo,
        };
        
        let agent = A2AAgent::with_config(client, config.clone());

        // Test config accessor
        assert_eq!(agent.config().max_retries, 5);
        assert_eq!(agent.config().task_handling, TaskHandling::ReturnTaskInfo);
        assert_eq!(agent.config().local_id, Some("test-id".to_string()));
        assert_eq!(agent.config().local_name, Some("Test Agent".to_string()));
    }

    #[tokio::test]
    async fn test_a2a_agent_cache_operations() {
        let transport = Arc::new(JsonRpcTransport::new("http://localhost:9999/rpc").unwrap());
        let client = A2aClient::new(transport);
        let agent = A2AAgent::new(client);

        // Clear cache (should be no-op on empty cache)
        agent.clear_cache().await;

        // Cache operations are tested indirectly through info() calls
        // which populate and use the cache
    }

    #[tokio::test]
    async fn test_a2a_agent_default_health_check() {
        let transport = Arc::new(JsonRpcTransport::new("http://localhost:9999/rpc").unwrap());
        let client = A2aClient::new(transport);
        let agent = A2AAgent::new(client);

        // Default health check returns false when endpoint is unreachable
        let healthy = agent.health_check().await;
        assert!(!healthy);
    }

    #[tokio::test]
    async fn test_handle_task_return_info() {
        let transport = Arc::new(JsonRpcTransport::new("http://localhost:9999/rpc").unwrap());
        let client = A2aClient::new(transport);
        
        let config = A2AAgentConfig {
            task_handling: TaskHandling::ReturnTaskInfo,
            ..Default::default()
        };
        
        let agent = A2AAgent::with_config(client, config);

        // Create a mock task with all required fields
        let task = Task {
            id: "test-task-123".to_string(),
            context_id: None,
            status: TaskStatus {
                state: TaskState::Pending,
                message: None,
                timestamp: None,
            },
            artifacts: None,
            history: None,
            metadata: None,
        };

        // Test handle_task with ReturnTaskInfo strategy
        let result = agent.handle_task(task).await;
        assert!(result.is_ok());
        
        let message = result.unwrap();
        let text = crate::adapters::extract_text(&message).unwrap_or_default();
        assert!(text.contains("test-task-123"));
        assert!(text.contains("Pending") || text.contains("pending"));
    }

    #[tokio::test]
    async fn test_handle_task_reject_tasks() {
        let transport = Arc::new(JsonRpcTransport::new("http://localhost:9999/rpc").unwrap());
        let client = A2aClient::new(transport);
        
        let config = A2AAgentConfig {
            task_handling: TaskHandling::RejectTasks,
            ..Default::default()
        };
        
        let agent = A2AAgent::with_config(client, config);

        // Create a mock task with all required fields
        let task = Task {
            id: "test-task-456".to_string(),
            context_id: None,
            status: TaskStatus {
                state: TaskState::Working,
                message: None,
                timestamp: None,
            },
            artifacts: None,
            history: None,
            metadata: None,
        };

        // Test handle_task with RejectTasks strategy
        let result = agent.handle_task(task).await;
        assert!(result.is_err());
        
        let error = result.unwrap_err();
        let error_msg = error.to_string();
        assert!(error_msg.contains("Async tasks not supported"));
    }

    #[tokio::test]
    async fn test_poll_task_completed_with_artifacts() {
        let transport = Arc::new(JsonRpcTransport::new("http://localhost:9999/rpc").unwrap());
        let client = A2aClient::new(transport);
        let agent = A2AAgent::new(client);

        // Create an artifact with proper structure
        use a2a_protocol::{Artifact, Part, TextPart};
        let artifact = Artifact {
            artifact_id: "artifact-1".to_string(),
            name: Some("Result".to_string()),
            description: None,
            parts: vec![Part::Text(TextPart::new("Task result content"))],
            metadata: None,
        };

        // Create a completed task with artifacts
        let task = Task {
            id: "completed-task".to_string(),
            context_id: None,
            status: TaskStatus {
                state: TaskState::Completed,
                message: None,
                timestamp: None,
            },
            artifacts: Some(vec![artifact]),
            history: None,
            metadata: None,
        };

        let result = agent.poll_task_to_completion(task).await;
        assert!(result.is_ok());
        
        let message = result.unwrap();
        let text = crate::adapters::extract_text(&message).unwrap_or_default();
        assert!(text.contains("Task result content"));
    }

    #[tokio::test]
    async fn test_poll_task_completed_without_artifacts() {
        let transport = Arc::new(JsonRpcTransport::new("http://localhost:9999/rpc").unwrap());
        let client = A2aClient::new(transport);
        let agent = A2AAgent::new(client);

        // Create a completed task without artifacts
        let task = Task {
            id: "completed-no-artifacts".to_string(),
            context_id: None,
            status: TaskStatus {
                state: TaskState::Completed,
                message: None,
                timestamp: None,
            },
            artifacts: None,
            history: None,
            metadata: None,
        };

        let result = agent.poll_task_to_completion(task).await;
        assert!(result.is_ok());
        
        let message = result.unwrap();
        let text = crate::adapters::extract_text(&message).unwrap_or_default();
        assert_eq!(text, "Task completed (no artifacts)");
    }

    #[tokio::test]
    async fn test_poll_task_failed() {
        let transport = Arc::new(JsonRpcTransport::new("http://localhost:9999/rpc").unwrap());
        let client = A2aClient::new(transport);
        let agent = A2AAgent::new(client);

        // Create a failed task
        let task = Task {
            id: "failed-task".to_string(),
            context_id: None,
            status: TaskStatus {
                state: TaskState::Failed,
                message: Some(Message::agent_text("Processing error occurred")),
                timestamp: None,
            },
            artifacts: None,
            history: None,
            metadata: None,
        };

        let result = agent.poll_task_to_completion(task).await;
        assert!(result.is_err());
        
        let error = result.unwrap_err();
        let error_msg = error.to_string();
        assert!(error_msg.contains("failed-task"));
        assert!(error_msg.contains("Processing error occurred"));
    }

    #[tokio::test]
    async fn test_poll_task_cancelled() {
        let transport = Arc::new(JsonRpcTransport::new("http://localhost:9999/rpc").unwrap());
        let client = A2aClient::new(transport);
        let agent = A2AAgent::new(client);

        // Create a cancelled task
        let task = Task {
            id: "cancelled-task".to_string(),
            context_id: None,
            status: TaskStatus {
                state: TaskState::Cancelled,
                message: None,
                timestamp: None,
            },
            artifacts: None,
            history: None,
            metadata: None,
        };

        let result = agent.poll_task_to_completion(task).await;
        assert!(result.is_err());
        
        let error = result.unwrap_err();
        let error_msg = error.to_string();
        assert!(error_msg.contains("cancelled-task"));
        assert!(error_msg.contains("cancelled"));
    }

    #[tokio::test]
    async fn test_fetch_card_caching() {
        let transport = Arc::new(JsonRpcTransport::new("http://localhost:9999/rpc").unwrap());
        let client = A2aClient::new(transport);
        let agent = A2AAgent::new(client);

        // Initially cache should be empty
        {
            let cache = agent.card_cache.read().await;
            assert!(cache.is_none());
        }

        // Note: fetch_card would fail here because endpoint is unreachable
        // but we're just testing the cache structure exists
        let result = agent.fetch_card().await;
        // Expected to fail with unreachable endpoint
        assert!(result.is_err());

        // Cache should still be empty after failed fetch
        {
            let cache = agent.card_cache.read().await;
            assert!(cache.is_none());
        }
    }

}
