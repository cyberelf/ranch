//! A2A Task types (A2A Protocol v0.3.0 compliant)

use crate::Message;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Task state according to A2A spec
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskState {
    /// Task is pending and hasn't started
    Pending,
    /// Task is currently being worked on
    Working,
    /// Task is blocked waiting for something
    Blocked,
    /// Task is under review
    Review,
    /// Task has been completed successfully
    Completed,
    /// Task has been cancelled
    Cancelled,
    /// Task has failed
    Failed,
    /// Task is suspended/paused
    Suspended,
}

/// Task status information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskStatus {
    /// Current state of the task
    pub state: TaskState,

    /// Optional message associated with this status (e.g., for input-required state)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<Message>,

    /// Optional timestamp when the status was set (ISO 8601)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
}

impl TaskStatus {
    /// Create a new task status
    pub fn new(state: TaskState) -> Self {
        Self {
            state,
            message: None,
            timestamp: Some(chrono::Utc::now().to_rfc3339()),
        }
    }

    /// Create a new task status with a message
    pub fn with_message(mut self, message: Message) -> Self {
        self.message = Some(message);
        self
    }
}

/// Artifact produced by a task
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Artifact {
    /// Unique identifier for the artifact
    #[serde(rename = "artifactId")]
    pub artifact_id: String,

    /// Optional name/title of the artifact
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Optional description of the artifact
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Content parts of the artifact
    pub parts: Vec<crate::Part>,

    /// Optional metadata about the artifact
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// A2A Task representing async work
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Task {
    /// Unique task identifier
    pub id: String,

    /// Optional context this task belongs to
    #[serde(skip_serializing_if = "Option::is_none", rename = "contextId")]
    pub context_id: Option<String>,

    /// Current status of the task
    pub status: TaskStatus,

    /// Optional list of artifacts produced by the task
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifacts: Option<Vec<Artifact>>,

    /// Optional conversation history (list of messages)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub history: Option<Vec<Message>>,

    /// Optional task metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

impl Task {
    /// Create a new task with pending state
    pub fn new<S: Into<String>>(id: S) -> Self {
        Self {
            id: id.into(),
            context_id: None,
            status: TaskStatus::new(TaskState::Pending),
            artifacts: None,
            history: None,
            metadata: None,
        }
    }

    /// Create a new task with a generated UUID
    pub fn generate() -> Self {
        Self::new(uuid::Uuid::new_v4().to_string())
    }

    /// Set the context ID
    pub fn with_context_id<S: Into<String>>(mut self, context_id: S) -> Self {
        self.context_id = Some(context_id.into());
        self
    }

    /// Add a message to the conversation history
    pub fn add_message(&mut self, message: Message) {
        let history = self.history.get_or_insert_with(Vec::new);
        history.push(message);
    }

    /// Add an artifact to the task
    pub fn add_artifact(&mut self, artifact: Artifact) {
        let artifacts = self.artifacts.get_or_insert_with(Vec::new);
        artifacts.push(artifact);
    }
}

/// Response from message/send - either a Task or immediate Message
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SendResponse {
    /// Asynchronous task response
    Task(Task),
    /// Immediate message response
    Message(Message),
}

impl SendResponse {
    /// Create a task response
    pub fn task(task: Task) -> Self {
        Self::Task(task)
    }

    /// Create a message response
    pub fn message(message: Message) -> Self {
        Self::Message(message)
    }

    /// Check if this is a task response
    pub fn is_task(&self) -> bool {
        matches!(self, Self::Task(_))
    }

    /// Check if this is a message response
    pub fn is_message(&self) -> bool {
        matches!(self, Self::Message(_))
    }

    /// Get the task if this is a task response
    pub fn as_task(&self) -> Option<&Task> {
        match self {
            Self::Task(task) => Some(task),
            _ => None,
        }
    }

    /// Get the message if this is a message response
    pub fn as_message(&self) -> Option<&Message> {
        match self {
            Self::Message(msg) => Some(msg),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_creation() {
        let task = Task::generate();
        assert_eq!(task.status.state, TaskState::Pending);
        assert!(!task.id.is_empty());
    }

    #[test]
    fn test_task_message_history() {
        let mut task = Task::new("task-123");
        let msg1 = Message::user_text("Hello");
        let msg2 = Message::agent_text("Hi there");

        task.add_message(msg1);
        task.add_message(msg2);

        assert_eq!(task.history.as_ref().unwrap().len(), 2);
        assert_eq!(
            task.history.as_ref().unwrap()[0].role,
            crate::MessageRole::User
        );
        assert_eq!(
            task.history.as_ref().unwrap()[1].role,
            crate::MessageRole::Agent
        );
    }

    #[test]
    fn test_task_artifacts() {
        let mut task = Task::new("task-123");
        task.add_artifact(Artifact {
            artifact_id: "art-1".to_string(),
            name: Some("Result.txt".to_string()),
            description: Some("A text file result".to_string()),
            parts: vec![crate::Part::Text(crate::TextPart::new("Result content"))],
            metadata: None,
        });

        assert_eq!(task.artifacts.as_ref().unwrap().len(), 1);
        assert_eq!(task.artifacts.as_ref().unwrap()[0].artifact_id, "art-1");
    }

    #[test]
    fn test_send_response_variants() {
        let task_resp = SendResponse::task(Task::new("t1"));
        assert!(task_resp.is_task());
        assert!(!task_resp.is_message());

        let msg_resp = SendResponse::message(crate::Message::user_text("Hello"));
        assert!(!msg_resp.is_task());
        assert!(msg_resp.is_message());
    }
}
