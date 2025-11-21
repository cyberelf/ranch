//! Adapter utilities for converting between A2A protocol types
//!
//! This module provides convenience functions for working with A2A protocol messages.

use a2a_protocol::prelude::*;

/// Create a user message from text
pub fn user_message<S: Into<String>>(text: S) -> Message {
    Message::user_text(text)
}

/// Create an agent message from text
pub fn agent_message<S: Into<String>>(text: S) -> Message {
    Message::agent_text(text)
}

/// Extract the first text content from a message
pub fn extract_text(message: &Message) -> Option<String> {
    for part in &message.parts {
        if let Part::Text(text_part) = part {
            return Some(text_part.text.clone());
        }
    }
    None
}

/// Extract all text parts from a message
pub fn extract_all_text(message: &Message) -> Vec<String> {
    message
        .parts
        .iter()
        .filter_map(|part| {
            if let Part::Text(text_part) = part {
                Some(text_part.text.clone())
            } else {
                None
            }
        })
        .collect()
}

/// Join all text parts with a separator
pub fn join_text(message: &Message, separator: &str) -> String {
    extract_all_text(message).join(separator)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_message() {
        let msg = user_message("Hello");
        assert_eq!(msg.role, MessageRole::User);
        assert_eq!(extract_text(&msg), Some("Hello".to_string()));
    }

    #[test]
    fn test_agent_message() {
        let msg = agent_message("Response");
        assert_eq!(msg.role, MessageRole::Agent);
        assert_eq!(extract_text(&msg), Some("Response".to_string()));
    }

    #[test]
    fn test_extract_text() {
        let msg = Message::user_text("Hello");
        assert_eq!(extract_text(&msg), Some("Hello".to_string()));
    }

    #[test]
    fn test_join_text() {
        let msg = Message::user_text("Hello World");
        assert_eq!(join_text(&msg, " "), "Hello World");
    }
}
