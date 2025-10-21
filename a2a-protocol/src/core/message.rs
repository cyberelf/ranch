//! A2A message types (A2A Protocol v0.3.0 compliant)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Message role - A2A spec only allows "user" or "agent"
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    /// Message from the user/client
    User,
    /// Message from the agent/server
    Agent,
}

impl MessageRole {
    /// Get the string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            MessageRole::User => "user",
            MessageRole::Agent => "agent",
        }
    }
}

impl std::fmt::Display for MessageRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Part union type - represents different content types in a message
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Part {
    /// Text content part
    Text(TextPart),
    /// File content part
    File(FilePart),
    /// Structured data part
    Data(DataPart),
}

/// Text part for plain textual content
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextPart {
    /// The text content
    pub text: String,
    
    /// Optional metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

impl TextPart {
    /// Create a new text part
    pub fn new<S: Into<String>>(text: S) -> Self {
        Self {
            text: text.into(),
            metadata: None,
        }
    }
    
    /// Add metadata to this part
    pub fn with_metadata(mut self, metadata: HashMap<String, serde_json::Value>) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

/// File part for file-based content
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FilePart {
    /// The file data
    #[serde(flatten)]
    pub file: File,
    
    /// Optional metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// File representation (either bytes or URI)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum File {
    /// File with embedded bytes (base64 encoded)
    WithBytes(FileWithBytes),
    /// File with URI reference
    WithUri(FileWithUri),
}

/// File with embedded bytes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FileWithBytes {
    /// Optional file name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    
    /// MIME type of the file
    #[serde(rename = "mimeType")]
    pub mime_type: String,
    
    /// Base64 encoded file data
    #[serde(rename = "data")]
    pub bytes: String,
}

/// File with URI reference
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FileWithUri {
    /// Optional file name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    
    /// MIME type of the file
    #[serde(rename = "mimeType")]
    pub mime_type: String,
    
    /// URI to the file
    pub uri: String,
}

/// Data part for structured JSON data
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DataPart {
    /// The structured data
    pub data: serde_json::Value,
    
    /// Optional metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

impl DataPart {
    /// Create a new data part
    pub fn new(data: serde_json::Value) -> Self {
        Self {
            data,
            metadata: None,
        }
    }
    
    /// Add metadata to this part
    pub fn with_metadata(mut self, metadata: HashMap<String, serde_json::Value>) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

/// A2A Message structure (compliant with A2A v0.3.0 spec)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Message {
    /// Role of the sender - "user" or "agent"
    pub role: MessageRole,

    /// Message content parts
    pub parts: Vec<Part>,

    /// Optional unique message identifier
    #[serde(skip_serializing_if = "Option::is_none", rename = "messageId")]
    pub message_id: Option<String>,

    /// Optional task identifier this message belongs to
    #[serde(skip_serializing_if = "Option::is_none", rename = "taskId")]
    pub task_id: Option<String>,

    /// Optional context identifier for grouping related tasks
    #[serde(skip_serializing_if = "Option::is_none", rename = "contextId")]
    pub context_id: Option<String>,

    /// Optional message metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

impl Message {
    /// Create a new text message with user role
    pub fn new_text<S: Into<String>>(role: MessageRole, text: S) -> Self {
        Self {
            role,
            parts: vec![Part::Text(TextPart::new(text))],
            message_id: Some(uuid::Uuid::new_v4().to_string()),
            task_id: None,
            context_id: None,
            metadata: None,
        }
    }

    /// Create a new user text message
    pub fn user_text<S: Into<String>>(text: S) -> Self {
        Self::new_text(MessageRole::User, text)
    }

    /// Create a new agent text message
    pub fn agent_text<S: Into<String>>(text: S) -> Self {
        Self::new_text(MessageRole::Agent, text)
    }

    /// Add a text part to the message
    pub fn add_text<S: Into<String>>(mut self, text: S) -> Self {
        self.parts.push(Part::Text(TextPart::new(text)));
        self
    }

    /// Add a part to the message
    pub fn add_part(mut self, part: Part) -> Self {
        self.parts.push(part);
        self
    }

    /// Set the message ID
    pub fn with_message_id<S: Into<String>>(mut self, message_id: S) -> Self {
        self.message_id = Some(message_id.into());
        self
    }

    /// Set the task ID
    pub fn with_task_id<S: Into<String>>(mut self, task_id: S) -> Self {
        self.task_id = Some(task_id.into());
        self
    }

    /// Set the context ID
    pub fn with_context_id<S: Into<String>>(mut self, context_id: S) -> Self {
        self.context_id = Some(context_id.into());
        self
    }

    /// Add metadata to the message
    pub fn with_metadata(mut self, metadata: HashMap<String, serde_json::Value>) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Add a single metadata entry
    pub fn add_metadata<K: Into<String>, V: Into<serde_json::Value>>(
        mut self,
        key: K,
        value: V,
    ) -> Self {
        let metadata = self.metadata.get_or_insert_with(HashMap::new);
        metadata.insert(key.into(), value.into());
        self
    }

    /// Get the primary text content of the message
    pub fn text_content(&self) -> Option<&str> {
        self.parts.iter().find_map(|part| {
            if let Part::Text(text_part) = part {
                Some(text_part.text.as_str())
            } else {
                None
            }
        })
    }

    /// Get all text parts concatenated
    pub fn all_text(&self) -> String {
        self.parts
            .iter()
            .filter_map(|part| {
                if let Part::Text(text_part) = part {
                    Some(text_part.text.as_str())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join("")
    }

    /// Check if the message has any text content
    pub fn has_text(&self) -> bool {
        self.parts.iter().any(|part| matches!(part, Part::Text(_)))
    }

    /// Check if the message has any file content
    pub fn has_files(&self) -> bool {
        self.parts.iter().any(|part| matches!(part, Part::File(_)))
    }

    /// Check if the message has any data content
    pub fn has_data(&self) -> bool {
        self.parts.iter().any(|part| matches!(part, Part::Data(_)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_message() {
        let msg = Message::user_text("Hello, world!");
        assert_eq!(msg.role, MessageRole::User);
        assert_eq!(msg.text_content(), Some("Hello, world!"));
        assert!(msg.has_text());
    }

    #[test]
    fn test_message_with_metadata() {
        let msg = Message::user_text("Hello")
            .add_metadata("session_id", "12345")
            .add_metadata("priority", "high");

        assert_eq!(
            msg.metadata.as_ref().unwrap().get("session_id").unwrap(),
            "12345"
        );
        assert_eq!(
            msg.metadata.as_ref().unwrap().get("priority").unwrap(),
            "high"
        );
    }

    #[test]
    fn test_multi_part_message() {
        let msg = Message::user_text("Hello")
            .add_text(" there!")
            .add_part(Part::Data(DataPart::new(serde_json::json!({
                "code": "console.log('Hello');"
            }))));

        assert_eq!(msg.parts.len(), 3);
        assert!(msg.has_text());
        assert!(msg.has_data());
    }

    #[test]
    fn test_message_roles() {
        let user_msg = Message::user_text("Hello");
        assert_eq!(user_msg.role, MessageRole::User);

        let agent_msg = Message::agent_text("Hello back");
        assert_eq!(agent_msg.role, MessageRole::Agent);
    }

    #[test]
    fn test_message_with_ids() {
        let msg = Message::user_text("Test")
            .with_message_id("msg-123")
            .with_task_id("task-456")
            .with_context_id("ctx-789");

        assert_eq!(msg.message_id.as_ref().unwrap(), "msg-123");
        assert_eq!(msg.task_id.as_ref().unwrap(), "task-456");
        assert_eq!(msg.context_id.as_ref().unwrap(), "ctx-789");
    }

    #[test]
    fn test_part_types() {
        let text_part = Part::Text(TextPart::new("Hello"));
        assert!(matches!(text_part, Part::Text(_)));

        let data_part = Part::Data(DataPart::new(serde_json::json!({"key": "value"})));
        assert!(matches!(data_part, Part::Data(_)));

        let file_part = Part::File(FilePart {
            file: File::WithUri(FileWithUri {
                name: Some("test.txt".to_string()),
                mime_type: "text/plain".to_string(),
                uri: "https://example.com/test.txt".to_string(),
            }),
            metadata: None,
        });
        assert!(matches!(file_part, Part::File(_)));
    }

    #[test]
    fn test_all_text_concatenation() {
        let msg = Message::user_text("Hello")
            .add_text(" ")
            .add_text("World!");

        assert_eq!(msg.all_text(), "Hello World!");
    }
}
