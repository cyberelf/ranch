//! A2A protocol error types

use thiserror::Error;
use crate::AgentId;

/// Result type for A2A operations
pub type A2aResult<T> = Result<T, A2aError>;

/// A2A protocol error types
#[derive(Error, Debug)]
pub enum A2aError {
    /// Network-related errors
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    /// JSON serialization/deserialization errors
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Invalid message format
    #[error("Invalid message format: {0}")]
    InvalidMessage(String),

    /// Protocol violation
    #[error("Protocol violation: {0}")]
    ProtocolViolation(String),

    /// Authentication failed
    #[error("Authentication failed: {0}")]
    Authentication(String),

    /// Agent not found
    #[error("Agent not found: {0}")]
    AgentNotFound(AgentId),

    /// Invalid agent ID
    #[error("Invalid agent ID: {0}")]
    InvalidAgentId(String),

    /// Timeout
    #[error("Operation timed out")]
    Timeout,

    /// Rate limited
    #[error("Rate limited: retry after {0:?} seconds")]
    RateLimited(std::time::Duration),

    /// Server error
    #[error("Server error: {0}")]
    Server(String),

    /// Transport-specific error
    #[error("Transport error: {0}")]
    Transport(String),

    /// Validation error
    #[error("Validation error: {0}")]
    Validation(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
}

impl A2aError {
    /// Returns true if this error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            A2aError::Network(_)
                | A2aError::Timeout
                | A2aError::RateLimited(_)
                | A2aError::Server(_)
        )
    }

    /// Returns the HTTP status code if applicable
    pub fn status_code(&self) -> Option<u16> {
        match self {
            A2aError::Network(err) => err.status().map(|s| s.as_u16()),
            A2aError::Authentication(_) => Some(401),
            A2aError::AgentNotFound(_) => Some(404),
            A2aError::RateLimited(_) => Some(429),
            A2aError::Validation(_) => Some(400),
            A2aError::ProtocolViolation(_) => Some(422),
            _ => None,
        }
    }
}