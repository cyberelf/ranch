//! Unified error types for the multi-agent framework
//!
//! This module provides consistent error handling across all agent implementations
//! and framework components.

use thiserror::Error;

/// Unified error type for the multi-agent framework
#[derive(Error, Debug)]
pub enum MultiAgentError {
    /// Network-related errors (HTTP, connectivity, etc.)
    #[error("Network error: {0}")]
    Network(String),

    /// Authentication/authorization errors
    #[error("Authentication error: {0}")]
    Authentication(String),

    /// Configuration errors (invalid config, missing parameters, etc.)
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Protocol-specific errors (invalid format, version mismatch, etc.)
    #[error("Protocol error: {0}")]
    Protocol(String),

    /// Agent-related errors (not found, unavailable, etc.)
    #[error("Agent error: {0}")]
    Agent(String),

    /// Task execution errors
    #[error("Task error: {0}")]
    Task(String),

    /// Validation errors (invalid input, malformed data, etc.)
    #[error("Validation error: {0}")]
    Validation(String),

    /// Timeout errors
    #[error("Operation timed out")]
    Timeout,

    /// Rate limiting errors
    #[error("Rate limited: retry after {0:?} seconds")]
    RateLimited(std::time::Duration),

    /// Internal/Unexpected errors
    #[error("Internal error: {0}")]
    Internal(String),

    /// JSON serialization/deserialization errors
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// IO errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Generic error with custom message
    #[error("{0}")]
    Generic(String),
}

impl MultiAgentError {
    /// Create a network error
    pub fn network<S: Into<String>>(msg: S) -> Self {
        Self::Network(msg.into())
    }

    /// Create an authentication error
    pub fn auth<S: Into<String>>(msg: S) -> Self {
        Self::Authentication(msg.into())
    }

    /// Create a configuration error
    pub fn config<S: Into<String>>(msg: S) -> Self {
        Self::Configuration(msg.into())
    }

    /// Create a protocol error
    pub fn protocol<S: Into<String>>(msg: S) -> Self {
        Self::Protocol(msg.into())
    }

    /// Create an agent error
    pub fn agent<S: Into<String>>(msg: S) -> Self {
        Self::Agent(msg.into())
    }

    /// Create a task error
    pub fn task<S: Into<String>>(msg: S) -> Self {
        Self::Task(msg.into())
    }

    /// Create a validation error
    pub fn validation<S: Into<String>>(msg: S) -> Self {
        Self::Validation(msg.into())
    }

    /// Create an internal error
    pub fn internal<S: Into<String>>(msg: S) -> Self {
        Self::Internal(msg.into())
    }

    /// Create a generic error
    pub fn generic<S: Into<String>>(msg: S) -> Self {
        Self::Generic(msg.into())
    }

    /// Check if this error is retryable
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::Network(_) | Self::Timeout | Self::RateLimited(_) => true,
            Self::Authentication(_) | Self::Configuration(_) | Self::Validation(_) => false,
            Self::Protocol(_) => false, // Protocol errors are usually not retryable
            Self::Agent(msg) => msg.contains("timeout") || msg.contains("unavailable"),
            Self::Task(msg) => msg.contains("timeout") || msg.contains("retry"),
            Self::Internal(_) | Self::Json(_) | Self::Io(_) | Self::Generic(_) => false,
        }
    }

    /// Get a user-friendly error message
    pub fn user_message(&self) -> String {
        match self {
            Self::Network(msg) => format!("Connection failed: {}", msg),
            Self::Authentication(_) => {
                "Authentication failed. Please check your credentials.".to_string()
            }
            Self::Configuration(msg) => format!("Configuration error: {}", msg),
            Self::Protocol(msg) => format!("Protocol error: {}", msg),
            Self::Agent(msg) => format!("Agent error: {}", msg),
            Self::Task(msg) => format!("Task failed: {}", msg),
            Self::Validation(msg) => format!("Invalid input: {}", msg),
            Self::Timeout => "Operation timed out. Please try again.".to_string(),
            Self::RateLimited(duration) => {
                format!("Rate limited. Please wait {} seconds.", duration.as_secs())
            }
            Self::Internal(msg) => format!("An error occurred: {}", msg),
            Self::Json(_) => "Data format error occurred.".to_string(),
            Self::Io(_) => "File system error occurred.".to_string(),
            Self::Generic(msg) => msg.clone(),
        }
    }
}

/// Result type for multi-agent operations
pub type MultiAgentResult<T> = Result<T, MultiAgentError>;

/// Conversion from reqwest errors
impl From<reqwest::Error> for MultiAgentError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            Self::Timeout
        } else if err.is_connect() || err.is_request() {
            Self::Network(err.to_string())
        } else if err.status() == Some(reqwest::StatusCode::UNAUTHORIZED) {
            Self::Authentication("Unauthorized access".to_string())
        } else if err.status() == Some(reqwest::StatusCode::TOO_MANY_REQUESTS) {
            Self::RateLimited(std::time::Duration::from_secs(60))
        } else {
            Self::Network(err.to_string())
        }
    }
}

/// Conversion from A2A protocol errors
impl From<a2a_protocol::A2aError> for MultiAgentError {
    fn from(err: a2a_protocol::A2aError) -> Self {
        use a2a_protocol::A2aError::*;
        match err {
            Network(net_err) => Self::Network(net_err.to_string()),
            Json(json_err) => Self::Json(json_err),
            InvalidMessage(msg) => Self::Protocol(format!("Invalid message: {}", msg)),
            ProtocolViolation(msg) => Self::Protocol(format!("Protocol violation: {}", msg)),
            Authentication(msg) => Self::Authentication(msg),
            AgentNotFound(id) => Self::Agent(format!("Agent not found: {}", id)),
            InvalidAgentId(id) => Self::Validation(format!("Invalid agent ID: {}", id)),
            TaskNotFound { task_id } => Self::Task(format!("Task not found: {}", task_id)),
            TaskNotCancelable { task_id, state } => Self::Task(format!(
                "Task {} cannot be cancelled from state {:?}",
                task_id, state
            )),
            PushNotificationNotSupported => {
                Self::Protocol("Push notifications not supported".to_string())
            }
            UnsupportedOperation(op) => Self::Protocol(format!("Unsupported operation: {}", op)),
            ContentTypeNotSupported { content_type } => {
                Self::Protocol(format!("Content type not supported: {}", content_type))
            }
            InvalidAgentResponse(msg) => Self::Protocol(format!("Invalid agent response: {}", msg)),
            AuthenticatedExtendedCardNotConfigured => {
                Self::Configuration("Authenticated extended card not configured".to_string())
            }
            Timeout => Self::Timeout,
            RateLimited(duration) => Self::RateLimited(duration),
            Server(msg) => Self::Agent(format!("Agent server error: {}", msg)),
            Transport(msg) => Self::Network(format!("Transport error: {}", msg)),
            Validation(msg) => Self::Validation(msg),
            Configuration(msg) => Self::Configuration(msg),
            Internal(msg) => Self::Internal(msg),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = MultiAgentError::network("Connection failed");
        assert!(matches!(err, MultiAgentError::Network(_)));
    }

    #[test]
    fn test_all_error_constructors() {
        // Test all constructor methods
        let net_err = MultiAgentError::network("net error");
        assert!(matches!(net_err, MultiAgentError::Network(_)));
        assert_eq!(net_err.to_string(), "Network error: net error");

        let auth_err = MultiAgentError::auth("auth error");
        assert!(matches!(auth_err, MultiAgentError::Authentication(_)));
        assert_eq!(auth_err.to_string(), "Authentication error: auth error");

        let config_err = MultiAgentError::config("config error");
        assert!(matches!(config_err, MultiAgentError::Configuration(_)));
        assert_eq!(config_err.to_string(), "Configuration error: config error");

        let proto_err = MultiAgentError::protocol("proto error");
        assert!(matches!(proto_err, MultiAgentError::Protocol(_)));
        assert_eq!(proto_err.to_string(), "Protocol error: proto error");

        let agent_err = MultiAgentError::agent("agent error");
        assert!(matches!(agent_err, MultiAgentError::Agent(_)));
        assert_eq!(agent_err.to_string(), "Agent error: agent error");

        let task_err = MultiAgentError::task("task error");
        assert!(matches!(task_err, MultiAgentError::Task(_)));
        assert_eq!(task_err.to_string(), "Task error: task error");

        let val_err = MultiAgentError::validation("val error");
        assert!(matches!(val_err, MultiAgentError::Validation(_)));
        assert_eq!(val_err.to_string(), "Validation error: val error");

        let int_err = MultiAgentError::internal("int error");
        assert!(matches!(int_err, MultiAgentError::Internal(_)));
        assert_eq!(int_err.to_string(), "Internal error: int error");

        let gen_err = MultiAgentError::generic("gen error");
        assert!(matches!(gen_err, MultiAgentError::Generic(_)));
        assert_eq!(gen_err.to_string(), "gen error");
    }

    #[test]
    fn test_retryable_errors() {
        // Retryable errors
        assert!(MultiAgentError::Timeout.is_retryable());
        assert!(MultiAgentError::RateLimited(std::time::Duration::from_secs(60)).is_retryable());
        assert!(MultiAgentError::Network("Connection failed".to_string()).is_retryable());
        
        // Agent errors with timeout/unavailable keywords
        assert!(MultiAgentError::Agent("timeout occurred".to_string()).is_retryable());
        assert!(MultiAgentError::Agent("service unavailable".to_string()).is_retryable());
        assert!(!MultiAgentError::Agent("invalid request".to_string()).is_retryable());
        
        // Task errors with timeout/retry keywords
        assert!(MultiAgentError::Task("timeout waiting".to_string()).is_retryable());
        assert!(MultiAgentError::Task("please retry".to_string()).is_retryable());
        assert!(!MultiAgentError::Task("invalid task".to_string()).is_retryable());

        // Non-retryable errors
        assert!(!MultiAgentError::Authentication("Invalid token".to_string()).is_retryable());
        assert!(!MultiAgentError::Configuration("Missing field".to_string()).is_retryable());
        assert!(!MultiAgentError::Validation("Invalid input".to_string()).is_retryable());
        assert!(!MultiAgentError::Protocol("Version mismatch".to_string()).is_retryable());
        assert!(!MultiAgentError::Internal("Panic".to_string()).is_retryable());
        assert!(!MultiAgentError::Generic("Error".to_string()).is_retryable());
    }

    #[test]
    fn test_user_messages() {
        let net_err = MultiAgentError::Network("connection reset".into());
        assert_eq!(net_err.user_message(), "Connection failed: connection reset");

        let auth_err = MultiAgentError::Authentication("Invalid token".into());
        assert_eq!(auth_err.user_message(), "Authentication failed. Please check your credentials.");

        let config_err = MultiAgentError::Configuration("Missing API key".into());
        assert_eq!(config_err.user_message(), "Configuration error: Missing API key");

        let proto_err = MultiAgentError::Protocol("Version 1.0".into());
        assert_eq!(proto_err.user_message(), "Protocol error: Version 1.0");

        let agent_err = MultiAgentError::Agent("not found".into());
        assert_eq!(agent_err.user_message(), "Agent error: not found");

        let task_err = MultiAgentError::Task("execution failed".into());
        assert_eq!(task_err.user_message(), "Task failed: execution failed");

        let val_err = MultiAgentError::Validation("invalid email".into());
        assert_eq!(val_err.user_message(), "Invalid input: invalid email");

        let timeout_err = MultiAgentError::Timeout;
        assert_eq!(timeout_err.user_message(), "Operation timed out. Please try again.");

        let rate_err = MultiAgentError::RateLimited(std::time::Duration::from_secs(120));
        assert_eq!(rate_err.user_message(), "Rate limited. Please wait 120 seconds.");

        let int_err = MultiAgentError::Internal("panic".into());
        assert_eq!(int_err.user_message(), "An error occurred: panic");

        let json_err = MultiAgentError::Json(serde_json::from_str::<i32>("invalid").unwrap_err());
        assert_eq!(json_err.user_message(), "Data format error occurred.");

        let io_err = MultiAgentError::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "file not found"));
        assert_eq!(io_err.user_message(), "File system error occurred.");

        let gen_err = MultiAgentError::Generic("Custom message".into());
        assert_eq!(gen_err.user_message(), "Custom message");
    }

    #[test]
    fn test_error_display() {
        // Test Display trait implementation (from thiserror)
        assert_eq!(
            format!("{}", MultiAgentError::Timeout),
            "Operation timed out"
        );
        
        assert_eq!(
            format!("{}", MultiAgentError::RateLimited(std::time::Duration::from_secs(30))),
            "Rate limited: retry after 30s seconds"
        );
    }

    #[test]
    fn test_json_error_conversion() {
        let json_err = serde_json::from_str::<i32>("not a number").unwrap_err();
        let multi_err: MultiAgentError = json_err.into();
        assert!(matches!(multi_err, MultiAgentError::Json(_)));
        assert!(!multi_err.is_retryable());
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
        let multi_err: MultiAgentError = io_err.into();
        assert!(matches!(multi_err, MultiAgentError::Io(_)));
        assert!(!multi_err.is_retryable());
    }

    #[test]
    fn test_a2a_error_conversions() {
        use a2a_protocol::A2aError;

        // Network error
        let a2a_err = A2aError::Transport("connection failed".to_string());
        let multi_err: MultiAgentError = a2a_err.into();
        assert!(matches!(multi_err, MultiAgentError::Network(_)));
        assert!(multi_err.is_retryable());

        // Authentication error
        let a2a_err = A2aError::Authentication("invalid token".to_string());
        let multi_err: MultiAgentError = a2a_err.into();
        assert!(matches!(multi_err, MultiAgentError::Authentication(_)));
        assert!(!multi_err.is_retryable());

        // Validation error
        let a2a_err = A2aError::Validation("invalid format".to_string());
        let multi_err: MultiAgentError = a2a_err.into();
        assert!(matches!(multi_err, MultiAgentError::Validation(_)));
        assert!(!multi_err.is_retryable());

        // Configuration error
        let a2a_err = A2aError::Configuration("missing config".to_string());
        let multi_err: MultiAgentError = a2a_err.into();
        assert!(matches!(multi_err, MultiAgentError::Configuration(_)));

        // Internal error
        let a2a_err = A2aError::Internal("internal error".to_string());
        let multi_err: MultiAgentError = a2a_err.into();
        assert!(matches!(multi_err, MultiAgentError::Internal(_)));

        // Timeout
        let a2a_err = A2aError::Timeout;
        let multi_err: MultiAgentError = a2a_err.into();
        assert!(matches!(multi_err, MultiAgentError::Timeout));
        assert!(multi_err.is_retryable());

        // Rate limited
        let a2a_err = A2aError::RateLimited(std::time::Duration::from_secs(60));
        let multi_err: MultiAgentError = a2a_err.into();
        assert!(matches!(multi_err, MultiAgentError::RateLimited(_)));
        assert!(multi_err.is_retryable());

        // AgentNotFound
        let a2a_err = A2aError::AgentNotFound(a2a_protocol::AgentId::generate());
        let multi_err: MultiAgentError = a2a_err.into();
        assert!(matches!(multi_err, MultiAgentError::Agent(_)));
        assert!(multi_err.to_string().contains("not found"));

        // InvalidAgentId
        let a2a_err = A2aError::InvalidAgentId("bad-id".to_string());
        let multi_err: MultiAgentError = a2a_err.into();
        assert!(matches!(multi_err, MultiAgentError::Validation(_)));

        // TaskNotFound
        let a2a_err = A2aError::TaskNotFound { task_id: "task-123".to_string() };
        let multi_err: MultiAgentError = a2a_err.into();
        assert!(matches!(multi_err, MultiAgentError::Task(_)));

        // Protocol violations
        let a2a_err = A2aError::InvalidMessage("bad format".to_string());
        let multi_err: MultiAgentError = a2a_err.into();
        assert!(matches!(multi_err, MultiAgentError::Protocol(_)));

        let a2a_err = A2aError::ProtocolViolation("version mismatch".to_string());
        let multi_err: MultiAgentError = a2a_err.into();
        assert!(matches!(multi_err, MultiAgentError::Protocol(_)));

        let a2a_err = A2aError::UnsupportedOperation("unknown op".to_string());
        let multi_err: MultiAgentError = a2a_err.into();
        assert!(matches!(multi_err, MultiAgentError::Protocol(_)));

        let a2a_err = A2aError::PushNotificationNotSupported;
        let multi_err: MultiAgentError = a2a_err.into();
        assert!(matches!(multi_err, MultiAgentError::Protocol(_)));

        // Server error
        let a2a_err = A2aError::Server("server crashed".to_string());
        let multi_err: MultiAgentError = a2a_err.into();
        assert!(matches!(multi_err, MultiAgentError::Agent(_)));
        assert!(multi_err.to_string().contains("server error"));
    }

    #[tokio::test]
    async fn test_reqwest_conversion() {
        // Test timeout error conversion by forcing a request to time out (async client)
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_millis(1))
            .build()
            .unwrap();
        let res = client.get("http://10.255.255.1").send().await; // non-routable address to provoke a timeout/connect error
        assert!(res.is_err());
        let timeout_err = res.unwrap_err();
        let multi_err = MultiAgentError::from(timeout_err);
        assert!(multi_err.is_retryable());
    }
}
