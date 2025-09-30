//! Transport layer traits and common types

use async_trait::async_trait;
use crate::{Message, MessageResponse, AgentCard, A2aResult};

/// Configuration for transport layer
#[derive(Debug, Clone)]
pub struct TransportConfig {
    /// Request timeout in seconds
    pub timeout_seconds: u64,

    /// Maximum number of retries
    pub max_retries: u32,

    /// Whether to enable compression
    pub enable_compression: bool,

    /// Additional transport-specific configuration
    pub extra: std::collections::HashMap<String, serde_json::Value>,
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self {
            timeout_seconds: 30,
            max_retries: 3,
            enable_compression: true,
            extra: std::collections::HashMap::new(),
        }
    }
}

/// Transport layer trait for sending A2A messages
#[async_trait]
pub trait Transport: Send + Sync + std::fmt::Debug {
    /// Send a message and return the response
    async fn send_message(&self, message: Message) -> A2aResult<MessageResponse>;

    /// Fetch an agent's card
    async fn get_agent_card(&self, agent_id: &crate::AgentId) -> A2aResult<AgentCard>;

    /// Check if the transport is connected/available
    async fn is_available(&self) -> bool;

    /// Get the transport configuration
    fn config(&self) -> &TransportConfig;

    /// Get the transport type name
    fn transport_type(&self) -> &'static str;
}

/// Request information for transport implementations
#[derive(Debug, Clone)]
pub struct RequestInfo {
    /// Target URL or endpoint
    pub endpoint: String,

    /// HTTP method (for HTTP-based transports)
    pub method: Option<String>,

    /// Request headers
    pub headers: std::collections::HashMap<String, String>,

    /// Request timeout in milliseconds
    pub timeout_ms: u64,
}

impl RequestInfo {
    /// Create a new request info
    pub fn new<S: Into<String>>(endpoint: S) -> Self {
        Self {
            endpoint: endpoint.into(),
            method: None,
            headers: std::collections::HashMap::new(),
            timeout_ms: 30000,
        }
    }

    /// Set the HTTP method
    pub fn with_method<S: Into<String>>(mut self, method: S) -> Self {
        self.method = Some(method.into());
        self
    }

    /// Add a header
    pub fn with_header<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// Set timeout in milliseconds
    pub fn with_timeout_ms(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }
}