//! A2A protocol request and response parameter types (A2A v0.3.0 compliant)
//!
//! These types represent the parameters for A2A RPC methods.
//! They are transport-agnostic and can be used with any RPC transport
//! (JSON-RPC 2.0, gRPC, etc.)

use crate::Message;
use serde::{Deserialize, Serialize};

/// Request for message/send RPC method
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MessageSendRequest {
    /// The message to send
    pub message: Message,

    /// Whether to wait for immediate response (default: false)
    /// - true: Returns Message with immediate response
    /// - false: Returns Task for async processing
    #[serde(skip_serializing_if = "Option::is_none")]
    pub immediate: Option<bool>,
}

impl MessageSendRequest {
    /// Create a new message send request
    pub fn new(message: Message) -> Self {
        Self {
            message,
            immediate: None,
        }
    }

    /// Create a request for immediate response
    pub fn immediate(message: Message) -> Self {
        Self {
            message,
            immediate: Some(true),
        }
    }

    /// Create a request for async processing (returns Task)
    pub fn async_request(message: Message) -> Self {
        Self {
            message,
            immediate: Some(false),
        }
    }

    /// Set immediate flag
    pub fn with_immediate(mut self, immediate: bool) -> Self {
        self.immediate = Some(immediate);
        self
    }
}

/// Request for task/get RPC method
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskGetRequest {
    /// Task ID to retrieve
    #[serde(rename = "taskId")]
    pub task_id: String,
}

impl TaskGetRequest {
    /// Create a new task get request
    pub fn new<S: Into<String>>(task_id: S) -> Self {
        Self {
            task_id: task_id.into(),
        }
    }
}

/// Request for task/cancel RPC method
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskCancelRequest {
    /// Task ID to cancel
    #[serde(rename = "taskId")]
    pub task_id: String,

    /// Optional reason for cancellation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

impl TaskCancelRequest {
    /// Create a new task cancel request
    pub fn new<S: Into<String>>(task_id: S) -> Self {
        Self {
            task_id: task_id.into(),
            reason: None,
        }
    }

    /// Add a reason for cancellation
    pub fn with_reason<S: Into<String>>(mut self, reason: S) -> Self {
        self.reason = Some(reason.into());
        self
    }
}

/// Request for task/status RPC method
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskStatusRequest {
    /// Task ID to check status
    #[serde(rename = "taskId")]
    pub task_id: String,
}

impl TaskStatusRequest {
    /// Create a new task status request
    pub fn new<S: Into<String>>(task_id: S) -> Self {
        Self {
            task_id: task_id.into(),
        }
    }
}

/// Request for task/resubscribe RPC method
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskResubscribeRequest {
    /// Task ID to resubscribe to
    #[serde(rename = "taskId")]
    pub task_id: String,

    /// Optional metadata for resuming the stream
    /// 
    /// Common fields (transport implementations may use these):
    /// - `lastEventId`: Resume from this event ID (for SSE: mapped to Last-Event-ID header)
    /// - `sequenceNumber`: Resume from this sequence number
    /// - `resumeToken`: Opaque token for resuming the stream
    /// 
    /// The transport implementation handles how to use this metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

impl TaskResubscribeRequest {
    /// Create a new task resubscribe request
    pub fn new<S: Into<String>>(task_id: S) -> Self {
        Self {
            task_id: task_id.into(),
            metadata: None,
        }
    }

    /// Add metadata to the request
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Convenience method: Set Last-Event-ID for SSE reconnection
    pub fn with_last_event_id<S: Into<String>>(self, last_event_id: S) -> Self {
        self.with_metadata(serde_json::json!({
            "lastEventId": last_event_id.into()
        }))
    }

    /// Convenience method: Set sequence number for ordered stream resumption
    pub fn with_sequence_number(self, sequence: u64) -> Self {
        self.with_metadata(serde_json::json!({
            "sequenceNumber": sequence
        }))
    }
}

/// Request for agent/card RPC method
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentCardGetRequest {
    /// Optional agent ID (if not provided, returns current agent's card)
    #[serde(skip_serializing_if = "Option::is_none", rename = "agentId")]
    pub agent_id: Option<String>,
}

impl AgentCardGetRequest {
    /// Create a request for current agent's card
    pub fn current() -> Self {
        Self { agent_id: None }
    }

    /// Create a request for specific agent's card
    pub fn for_agent<S: Into<String>>(agent_id: S) -> Self {
        Self {
            agent_id: Some(agent_id.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_send_request() {
        let msg = Message::user_text("Hello");
        let req = MessageSendRequest::new(msg.clone());

        assert_eq!(req.message, msg);
        assert_eq!(req.immediate, None);
    }

    #[test]
    fn test_message_send_immediate() {
        let msg = Message::user_text("Hello");
        let req = MessageSendRequest::immediate(msg);

        assert_eq!(req.immediate, Some(true));
    }

    #[test]
    fn test_task_requests() {
        let get_req = TaskGetRequest::new("task-123");
        assert_eq!(get_req.task_id, "task-123");

        let cancel_req = TaskCancelRequest::new("task-456").with_reason("User cancelled");
        assert_eq!(cancel_req.task_id, "task-456");
        assert_eq!(cancel_req.reason, Some("User cancelled".to_string()));
    }

    #[test]
    fn test_agent_card_request() {
        let current = AgentCardGetRequest::current();
        assert!(current.agent_id.is_none());

        let specific = AgentCardGetRequest::for_agent("agent-123");
        assert_eq!(specific.agent_id, Some("agent-123".to_string()));
    }

    #[test]
    fn test_task_resubscribe_request() {
        let req = TaskResubscribeRequest::new("task-789");
        assert_eq!(req.task_id, "task-789");
        assert_eq!(req.metadata, None);

        // Test with generic metadata
        let req_with_metadata = TaskResubscribeRequest::new("task-790")
            .with_metadata(serde_json::json!({"custom": "value"}));
        assert_eq!(req_with_metadata.task_id, "task-790");
        assert!(req_with_metadata.metadata.is_some());

        // Test with Last-Event-ID (for SSE)
        let req_with_last_event = TaskResubscribeRequest::new("task-791")
            .with_last_event_id("event-123");
        assert_eq!(req_with_last_event.task_id, "task-791");
        assert_eq!(
            req_with_last_event.metadata.unwrap()["lastEventId"],
            "event-123"
        );

        // Test with sequence number (for gRPC or ordered streams)
        let req_with_seq = TaskResubscribeRequest::new("task-792")
            .with_sequence_number(42);
        assert_eq!(req_with_seq.task_id, "task-792");
        assert_eq!(
            req_with_seq.metadata.unwrap()["sequenceNumber"],
            42
        );
    }
}
