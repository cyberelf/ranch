//! Task-aware A2A handler implementation

use crate::{
    server::{
        handler::{HealthStatus, HealthStatusType},
        A2aHandler, TaskStore,
    }, A2aResult, AgentCard, AgentCardGetRequest, Message, MessageSendRequest, SendResponse,
    Task, TaskCancelRequest, TaskGetRequest, TaskResubscribeRequest, TaskState, TaskStatus, TaskStatusRequest,
};
use async_trait::async_trait;

#[cfg(feature = "streaming")]
use crate::transport::{StreamingResult, sse::SseWriter};
#[cfg(feature = "streaming")]
use crate::core::streaming_events::TaskStatusUpdateEvent;
#[cfg(feature = "streaming")]
use futures_util::stream::Stream;
#[cfg(feature = "streaming")]
use std::collections::HashMap;
#[cfg(feature = "streaming")]
use std::sync::Arc;
#[cfg(feature = "streaming")]
use tokio::sync::RwLock;

/// Handler that supports full task lifecycle management
#[derive(Clone)]
pub struct TaskAwareHandler {
    agent_card: AgentCard,
    task_store: TaskStore,
    /// Whether to return tasks for messages (true) or immediate responses (false)
    async_by_default: bool,
    /// SSE writers for streaming tasks (task_id -> writer)
    #[cfg(feature = "streaming")]
    stream_writers: Arc<RwLock<HashMap<String, SseWriter>>>,
}

impl TaskAwareHandler {
    /// Create a new task-aware handler
    pub fn new(agent_card: AgentCard) -> Self {
        Self {
            agent_card,
            task_store: TaskStore::new(),
            async_by_default: true,
            #[cfg(feature = "streaming")]
            stream_writers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a handler that returns immediate responses by default
    pub fn with_immediate_responses(agent_card: AgentCard) -> Self {
        Self {
            agent_card,
            task_store: TaskStore::new(),
            async_by_default: false,
            #[cfg(feature = "streaming")]
            stream_writers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Set whether to return tasks by default
    pub fn set_async_by_default(&mut self, async_by_default: bool) {
        self.async_by_default = async_by_default;
    }

    /// Get the task store (for testing or advanced usage)
    pub fn task_store(&self) -> &TaskStore {
        &self.task_store
    }

    /// Process a message and create a task
    async fn process_message_as_task(&self, message: Message) -> A2aResult<Task> {
        // Create a task for this message
        let content = message.text_content().unwrap_or("Processing message");
        let task = Task::new(format!("Processing: {}", content));
        let task_id = task.id.clone();

        // Store the task
        self.task_store.store(task).await?;

        // Simulate starting work
        self.task_store
            .update_state(&task_id, TaskState::Working)
            .await?;

        // Return the updated task
        self.task_store.get(&task_id).await
    }

    /// Process a message and return immediate response
    async fn process_message_immediately(&self, message: Message) -> A2aResult<Message> {
        let content = message.text_content().unwrap_or("No content");
        let response_content = format!("Echo: {}", content);
        Ok(Message::agent_text(response_content))
    }
}

#[async_trait]
impl A2aHandler for TaskAwareHandler {
    async fn handle_message(&self, message: Message) -> A2aResult<SendResponse> {
        if self.async_by_default {
            // Return a task
            let task = self.process_message_as_task(message).await?;
            Ok(SendResponse::task(task))
        } else {
            // Return immediate response
            let response = self.process_message_immediately(message).await?;
            Ok(SendResponse::message(response))
        }
    }

    async fn get_agent_card(&self) -> A2aResult<AgentCard> {
        Ok(self.agent_card.clone())
    }

    async fn health_check(&self) -> A2aResult<HealthStatus> {
        let task_count = self.task_store.count().await;

        Ok(HealthStatus {
            status: HealthStatusType::Healthy,
            message: Some(format!(
                "Task-aware handler running with {} tasks",
                task_count
            )),
            version: Some(env!("CARGO_PKG_VERSION").to_string()),
            details: Some(serde_json::json!({
                "handler": "TaskAwareHandler",
                "task_count": task_count,
                "timestamp": chrono::Utc::now().to_rfc3339()
            })),
        })
    }

    async fn rpc_message_send(&self, request: MessageSendRequest) -> A2aResult<SendResponse> {
        // Honor the immediate flag if provided
        let use_immediate = request.immediate.unwrap_or(!self.async_by_default);

        if use_immediate {
            let response = self.process_message_immediately(request.message).await?;
            Ok(SendResponse::message(response))
        } else {
            let task = self.process_message_as_task(request.message).await?;
            Ok(SendResponse::task(task))
        }
    }

    async fn rpc_task_get(&self, request: TaskGetRequest) -> A2aResult<Task> {
        self.task_store.get(&request.task_id).await
    }

    async fn rpc_task_cancel(&self, request: TaskCancelRequest) -> A2aResult<TaskStatus> {
        self.task_store
            .cancel(&request.task_id, request.reason)
            .await
    }

    async fn rpc_task_status(&self, request: TaskStatusRequest) -> A2aResult<TaskStatus> {
        self.task_store.get_status(&request.task_id).await
    }

    async fn rpc_agent_card(&self, _request: AgentCardGetRequest) -> A2aResult<AgentCard> {
        self.get_agent_card().await
    }

    #[cfg(feature = "streaming")]
    async fn rpc_message_stream(
        &self,
        message: Message,
    ) -> A2aResult<Box<dyn Stream<Item = A2aResult<StreamingResult>> + Send + Unpin>> {
        // Create a task for this message
        let task = self.process_message_as_task(message).await?;
        let task_id = task.id.clone();

        // Create an SSE writer for this task
        let writer = SseWriter::new(100); // Buffer up to 100 events
        
        // Store the writer so we can send events to it
        {
            let mut writers = self.stream_writers.write().await;
            writers.insert(task_id.clone(), writer.clone());
        }

        // Get the subscription first (creates a subscriber)
        let stream = writer.subscribe();

        // Now send initial task event (subscriber exists)
        writer.send(StreamingResult::Task(task.clone()))?;

        // Simulate some work and send status updates
        let writer_clone = writer.clone();
        let task_id_clone = task_id.clone();
        let task_store = self.task_store.clone();
        let stream_writers = self.stream_writers.clone();
        
        tokio::spawn(async move {
            // Simulate work with status updates
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            
            // Send a status update
            if let Ok(status) = task_store.get_status(&task_id_clone).await {
                let event = TaskStatusUpdateEvent::new(&task_id_clone, status);
                let _ = writer_clone.send(StreamingResult::TaskStatusUpdate(event));
            }

            // Simulate more work
            tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

            // Update to completed
            if let Ok(_) = task_store.update_state(&task_id_clone, TaskState::Completed).await {
                if let Ok(status) = task_store.get_status(&task_id_clone).await {
                    let event = TaskStatusUpdateEvent::new(&task_id_clone, status);
                    let _ = writer_clone.send(StreamingResult::TaskStatusUpdate(event));
                }
            }

            // Cleanup: remove writer after task completes
            let mut writers = stream_writers.write().await;
            writers.remove(&task_id_clone);
        });

        // Return the stream
        Ok(Box::new(Box::pin(stream)))
    }

    #[cfg(feature = "streaming")]
    async fn rpc_task_resubscribe(
        &self,
        request: TaskResubscribeRequest,
    ) -> A2aResult<Box<dyn Stream<Item = A2aResult<StreamingResult>> + Send + Unpin>> {
        let task_id = &request.task_id;

        // Check if task exists
        let task = self.task_store.get(task_id).await?;

        // Check if we have an active stream for this task
        let writers = self.stream_writers.read().await;
        if let Some(writer) = writers.get(task_id) {
            // Return a new subscription to the existing stream
            Ok(Box::new(Box::pin(writer.subscribe())))
        } else {
            // Task exists but no active stream - send current state and close
            drop(writers); // Release the read lock
            
            let writer = SseWriter::new(10);
            let stream = writer.subscribe(); // Get subscription first
            
            writer.send(StreamingResult::Task(task.clone()))?;
            
            let status = self.task_store.get_status(task_id).await?;
            let event = TaskStatusUpdateEvent::new(task_id, status);
            writer.send(StreamingResult::TaskStatusUpdate(event))?;
            
            Ok(Box::new(Box::pin(stream)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{A2aError, AgentId, MessageRole};
    use url::Url;

    fn create_test_handler() -> TaskAwareHandler {
        let agent_id = AgentId::new("test-agent".to_string()).unwrap();
        let agent_card = AgentCard::new(
            agent_id,
            "Test Agent",
            Url::parse("https://example.com").unwrap(),
        );
        TaskAwareHandler::new(agent_card)
    }

    #[tokio::test]
    async fn test_handle_message_returns_task() {
        let handler = create_test_handler();
        let message = Message::user_text("Test message");

        let response = handler.handle_message(message).await.unwrap();

        assert!(response.is_task());
        let task = response.as_task().unwrap();
        assert_eq!(task.status.state, TaskState::Working);
    }

    #[tokio::test]
    async fn test_handle_message_immediate_mode() {
        let agent_id = AgentId::new("test-agent".to_string()).unwrap();
        let agent_card = AgentCard::new(
            agent_id,
            "Test Agent",
            Url::parse("https://example.com").unwrap(),
        );
        let handler = TaskAwareHandler::with_immediate_responses(agent_card);

        let message = Message::user_text("Test message");
        let response = handler.handle_message(message).await.unwrap();

        assert!(response.is_message());
        let msg = response.as_message().unwrap();
        assert_eq!(msg.role, MessageRole::Agent);
    }

    #[tokio::test]
    async fn test_rpc_message_send_with_immediate_flag() {
        let handler = create_test_handler();
        let message = Message::user_text("Test");

        // Request immediate response
        let request = MessageSendRequest {
            message: message.clone(),
            immediate: Some(true),
        };
        let response = handler.rpc_message_send(request).await.unwrap();
        assert!(response.is_message());

        // Request async task
        let request = MessageSendRequest {
            message,
            immediate: Some(false),
        };
        let response = handler.rpc_message_send(request).await.unwrap();
        assert!(response.is_task());
    }

    #[tokio::test]
    async fn test_rpc_task_get() {
        let handler = create_test_handler();

        // Create a task
        let message = Message::user_text("Test");
        let response = handler.handle_message(message).await.unwrap();
        let task = response.as_task().unwrap();
        let task_id = task.id.clone();

        // Retrieve task via RPC
        let request = TaskGetRequest {
            task_id: task_id.clone(),
        };
        let retrieved = handler.rpc_task_get(request).await.unwrap();

        assert_eq!(retrieved.id, task_id);
        assert_eq!(retrieved.status.state, TaskState::Working);
    }

    #[tokio::test]
    async fn test_rpc_task_status() {
        let handler = create_test_handler();

        // Create a task
        let message = Message::user_text("Test");
        let response = handler.handle_message(message).await.unwrap();
        let task = response.as_task().unwrap();
        let task_id = task.id.clone();

        // Get status via RPC
        let request = TaskStatusRequest { task_id };
        let status = handler.rpc_task_status(request).await.unwrap();

        assert_eq!(status.state, TaskState::Working);
        let task = handler.task_store().get(&task.id).await.unwrap();
        assert!(task.history.is_some());
    }

    #[tokio::test]
    async fn test_rpc_task_cancel() {
        let handler = create_test_handler();

        // Create a task
        let message = Message::user_text("Test");
        let response = handler.handle_message(message).await.unwrap();
        let task = response.as_task().unwrap();
        let task_id = task.id.clone();

        // Cancel via RPC
        let request = TaskCancelRequest {
            task_id: task_id.clone(),
            reason: Some("User requested".to_string()),
        };
        let status = handler.rpc_task_cancel(request).await.unwrap();

        assert_eq!(status.state, TaskState::Cancelled);
        assert!(status.reason.is_some());

        // Verify task is cancelled
        let task = handler.task_store().get(&task_id).await.unwrap();
        assert_eq!(task.status.state, TaskState::Cancelled);
    }

    #[tokio::test]
    async fn test_rpc_task_get_not_found() {
        let handler = create_test_handler();

        let request = TaskGetRequest {
            task_id: "nonexistent-task".to_string(),
        };
        let result = handler.rpc_task_get(request).await;

        match result {
            Err(A2aError::TaskNotFound { task_id }) => {
                assert_eq!(task_id, "nonexistent-task")
            }
            other => panic!("Expected TaskNotFound error, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_health_check() {
        let handler = create_test_handler();

        // Create some tasks
        handler
            .handle_message(Message::user_text("Task 1"))
            .await
            .unwrap();
        handler
            .handle_message(Message::user_text("Task 2"))
            .await
            .unwrap();

        let health = handler.health_check().await.unwrap();

        assert!(matches!(health.status, HealthStatusType::Healthy));
        assert!(health.message.unwrap().contains("2 tasks"));
    }

    #[tokio::test]
    async fn test_async_by_default_flag() {
        let agent_id = AgentId::new("test-agent".to_string()).unwrap();
        let agent_card = AgentCard::new(
            agent_id,
            "Test Agent",
            Url::parse("https://example.com").unwrap(),
        );

        let mut handler = TaskAwareHandler::new(agent_card);

        // Default: async (returns tasks)
        let msg = Message::user_text("Test");
        let response = handler.handle_message(msg.clone()).await.unwrap();
        assert!(response.is_task());

        // Change to immediate
        handler.set_async_by_default(false);
        let response = handler.handle_message(msg).await.unwrap();
        assert!(response.is_message());
    }

    #[cfg(feature = "streaming")]
    #[tokio::test]
    async fn test_rpc_message_stream() {
        use futures_util::StreamExt;

        let handler = create_test_handler();
        let message = Message::user_text("Test streaming");

        let mut stream = handler.rpc_message_stream(message).await.unwrap();

        // Collect events from the stream
        let mut events = Vec::new();
        while let Some(result) = stream.next().await {
            match result {
                Ok(event) => {
                    events.push(event);
                    // Stop after we get a few events to avoid waiting forever
                    if events.len() >= 3 {
                        break;
                    }
                }
                Err(e) => panic!("Stream error: {}", e),
            }
        }

        // Should have at least: initial task + status updates
        assert!(!events.is_empty());
        
        // First event should be the task
        match &events[0] {
            StreamingResult::Task(task) => {
                assert_eq!(task.status.state, TaskState::Working);
            }
            _ => panic!("Expected first event to be Task"),
        }

        // Should have status updates
        let has_status_update = events.iter().any(|e| matches!(e, StreamingResult::TaskStatusUpdate(_)));
        assert!(has_status_update, "Should have at least one status update");
    }

    #[cfg(feature = "streaming")]
    #[tokio::test]
    async fn test_rpc_task_resubscribe() {
        use futures_util::StreamExt;

        let handler = create_test_handler();
        
        // Create a task first
        let message = Message::user_text("Test");
        let response = handler.handle_message(message).await.unwrap();
        let task = response.as_task().unwrap();
        let task_id = task.id.clone();

        // Now resubscribe to it
        let request = TaskResubscribeRequest {
            task_id: task_id.clone(),
            metadata: None,
        };
        
        let mut stream = handler.rpc_task_resubscribe(request).await.unwrap();

        // Should get at least the current task state
        let first_event = stream.next().await;
        assert!(first_event.is_some());
    }
}
