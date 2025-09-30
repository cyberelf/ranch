//! A2A message types

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::{MessageId, AgentId};

/// A single part of a message content
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MessagePart {
    /// The type of content (e.g., "text", "image", "audio")
    #[serde(rename = "type")]
    pub content_type: String,

    /// The actual content
    pub content: String,

    /// Optional metadata for this part
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// A2A Message structure
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Message {
    /// Unique message identifier
    pub id: MessageId,

    /// Role of the sender (e.g., "user", "assistant", "system")
    pub role: String,

    /// Message content parts
    pub parts: Vec<MessagePart>,

    /// Timestamp when the message was created
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<chrono::DateTime<chrono::Utc>>,

    /// Optional message metadata
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Message {
    /// Create a new text message
    pub fn new_text<S: Into<String>>(role: S, content: S) -> Self {
        Self {
            id: MessageId::generate(),
            role: role.into(),
            parts: vec![MessagePart {
                content_type: "text".to_string(),
                content: content.into(),
                metadata: None,
            }],
            timestamp: Some(chrono::Utc::now()),
            metadata: HashMap::new(),
        }
    }

    /// Add a text part to the message
    pub fn add_text<S: Into<String>>(mut self, content: S) -> Self {
        self.parts.push(MessagePart {
            content_type: "text".to_string(),
            content: content.into(),
            metadata: None,
        });
        self
    }

    /// Add a custom part to the message
    pub fn add_part(mut self, part: MessagePart) -> Self {
        self.parts.push(part);
        self
    }

    /// Add metadata to the message
    pub fn with_metadata<K: Into<String>, V: Into<serde_json::Value>>(
        mut self,
        key: K,
        value: V,
    ) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Get the primary text content of the message
    pub fn text_content(&self) -> Option<&str> {
        self.parts
            .iter()
            .find(|part| part.content_type == "text")
            .map(|part| part.content.as_str())
    }

    /// Check if the message has any text content
    pub fn has_text(&self) -> bool {
        self.parts.iter().any(|part| part.content_type == "text")
    }
}

/// Response to an A2A message
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MessageResponse {
    /// The original message ID this responds to
    pub in_reply_to: MessageId,

    /// Response message
    pub message: Message,

    /// Optional processing status
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<ResponseStatus>,

    /// Optional error information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ResponseError>,
}

/// Response status information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResponseStatus {
    /// Status code
    pub code: u16,

    /// Human-readable status message
    pub message: String,

    /// Optional detailed status information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<HashMap<String, serde_json::Value>>,
}

/// Response error information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResponseError {
    /// Error type
    #[serde(rename = "type")]
    pub error_type: String,

    /// Error message
    pub message: String,

    /// Optional error details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<HashMap<String, serde_json::Value>>,
}

impl MessageResponse {
    /// Create a successful response
    pub fn success(in_reply_to: MessageId, message: Message) -> Self {
        Self {
            in_reply_to,
            message,
            status: Some(ResponseStatus {
                code: 200,
                message: "OK".to_string(),
                details: None,
            }),
            error: None,
        }
    }

    /// Create an error response
    pub fn error(in_reply_to: MessageId, error_type: String, message: String) -> Self {
        Self {
            in_reply_to,
            message: Message::new_text("system", format!("Error: {}", message).as_str()),
            status: Some(ResponseStatus {
                code: 500,
                message: "Internal Server Error".to_string(),
                details: None,
            }),
            error: Some(ResponseError {
                error_type,
                message,
                details: None,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_message() {
        let msg = Message::new_text("user", "Hello, world!");
        assert_eq!(msg.role, "user");
        assert_eq!(msg.text_content(), Some("Hello, world!"));
        assert!(msg.has_text());
    }

    #[test]
    fn test_message_with_metadata() {
        let msg = Message::new_text("user", "Hello")
            .with_metadata("session_id", "12345")
            .with_metadata("priority", "high");

        assert_eq!(msg.metadata.get("session_id").unwrap(), "12345");
        assert_eq!(msg.metadata.get("priority").unwrap(), "high");
    }

    #[test]
    fn test_multi_part_message() {
        let msg = Message::new_text("user", "Hello")
            .add_text(" there!")
            .add_part(MessagePart {
                content_type: "code".to_string(),
                content: "console.log('Hello');".to_string(),
                metadata: None,
            });

        assert_eq!(msg.parts.len(), 3);
        assert!(msg.has_text());
    }

    #[test]
    fn test_response_creation() {
        let msg_id = MessageId::generate();
        let response = Message::new_text("assistant", "Hello back!");
        let response_msg = MessageResponse::success(msg_id.clone(), response);

        assert_eq!(response_msg.in_reply_to, msg_id);
        assert!(response_msg.error.is_none());
    }
}