//! Streaming event types for A2A protocol
//!
//! These are domain event types that can be sent as results in JSON-RPC streaming responses.
//! Per A2A protocol spec, message/stream returns a stream of JSON-RPC responses where
//! result can be: Message | Task | TaskStatusUpdateEvent | TaskArtifactUpdateEvent

use super::{message::Message, TaskState, TaskStatus};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Task status update event - sent when task status changes
///
/// This is a core domain event type that can be returned as the `result` field
/// in a JSON-RPC streaming response from `message/stream`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TaskStatusUpdateEvent {
    /// Task ID
    pub task_id: String,

    /// Updated task status
    pub status: TaskStatus,

    /// Previous state (if changed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_state: Option<TaskState>,

    /// Event timestamp
    pub timestamp: DateTime<Utc>,

    /// Optional progress information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress: Option<TaskProgress>,
}

impl TaskStatusUpdateEvent {
    /// Create a new task status update event
    pub fn new(task_id: impl Into<String>, status: TaskStatus) -> Self {
        Self {
            task_id: task_id.into(),
            status,
            previous_state: None,
            timestamp: Utc::now(),
            progress: None,
        }
    }

    /// Set the previous state
    pub fn with_previous_state(mut self, state: TaskState) -> Self {
        self.previous_state = Some(state);
        self
    }

    /// Set progress information
    pub fn with_progress(mut self, progress: TaskProgress) -> Self {
        self.progress = Some(progress);
        self
    }
}

/// Task progress information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TaskProgress {
    /// Current progress (0-100)
    pub percent: Option<u8>,

    /// Current step description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub step: Option<String>,

    /// Total steps
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_steps: Option<u32>,

    /// Current step number
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_step: Option<u32>,
}

/// Task artifact update event - sent when a task artifact is created or updated
///
/// This is a core domain event type that can be returned as the `result` field
/// in a JSON-RPC streaming response from `message/stream`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TaskArtifactUpdateEvent {
    /// Task ID
    pub task_id: String,

    /// Artifact ID
    pub artifact_id: String,

    /// Artifact type (e.g., "message", "file", "data")
    pub artifact_type: String,

    /// Artifact metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,

    /// Event timestamp
    pub timestamp: DateTime<Utc>,

    /// Whether this artifact is final
    #[serde(default)]
    pub is_final: bool,
}

impl TaskArtifactUpdateEvent {
    /// Create a new artifact update event
    pub fn new(
        task_id: impl Into<String>,
        artifact_id: impl Into<String>,
        artifact_type: impl Into<String>,
    ) -> Self {
        Self {
            task_id: task_id.into(),
            artifact_id: artifact_id.into(),
            artifact_type: artifact_type.into(),
            metadata: None,
            timestamp: Utc::now(),
            is_final: false,
        }
    }

    /// Set metadata
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Mark as final artifact
    pub fn as_final(mut self) -> Self {
        self.is_final = true;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_status_update_event() {
        let status = TaskStatus::new(TaskState::Working)
            .with_message(Message::agent_text("Processing"));

        let event = TaskStatusUpdateEvent::new("task_123", status)
            .with_previous_state(TaskState::Pending)
            .with_progress(TaskProgress {
                percent: Some(50),
                step: Some("Step 2".to_string()),
                total_steps: Some(4),
                current_step: Some(2),
            });

        assert_eq!(event.task_id, "task_123");
        assert_eq!(event.status.state, TaskState::Working);
        assert_eq!(event.previous_state, Some(TaskState::Pending));
        assert_eq!(event.progress.as_ref().unwrap().percent, Some(50));
    }

    #[test]
    fn test_task_artifact_update_event() {
        let event = TaskArtifactUpdateEvent::new("task_456", "artifact_789", "message")
            .with_metadata(serde_json::json!({"size": 1024}))
            .as_final();

        assert_eq!(event.task_id, "task_456");
        assert_eq!(event.artifact_id, "artifact_789");
        assert_eq!(event.artifact_type, "message");
        assert!(event.is_final);
        assert_eq!(event.metadata.as_ref().unwrap()["size"], 1024);
    }

    #[test]
    fn test_event_serialization() {
        let status = TaskStatus::new(TaskState::Working);
        let event = TaskStatusUpdateEvent::new("task_001", status);

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"taskId\":\"task_001\""));
        assert!(json.contains("\"status\":"));
    }
}
