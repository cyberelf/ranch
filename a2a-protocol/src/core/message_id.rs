//! Message ID type and utilities

use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// A2A Message identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MessageId(String);

impl MessageId {
    /// Create a new MessageId
    pub fn new(id: String) -> Self {
        Self(id)
    }

    /// Generate a new random MessageId using UUID v4
    pub fn generate() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    /// Get the MessageId as a string slice
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Parse the MessageId as a UUID
    pub fn as_uuid(&self) -> Option<Uuid> {
        Uuid::parse_str(&self.0).ok()
    }

    /// Validate if this is a valid UUID-based MessageId
    pub fn is_valid_uuid(&self) -> bool {
        self.as_uuid().is_some()
    }
}

impl fmt::Display for MessageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for MessageId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for MessageId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<Uuid> for MessageId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_id_from_string() {
        let id = MessageId::new("test-id".to_string());
        assert_eq!(id.as_str(), "test-id");
    }

    #[test]
    fn test_generate_message_id() {
        let id = MessageId::generate();
        assert!(id.is_valid_uuid());
    }

    #[test]
    fn test_uuid_message_id() {
        let uuid = Uuid::new_v4();
        let id = MessageId::from(uuid);
        assert_eq!(id.as_uuid(), Some(uuid));
    }
}