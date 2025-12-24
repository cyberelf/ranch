//! Types for the Team Router and Client Agent Extension
//!
//! This module defines the core types used by the Router component for dynamic
//! message routing between agents, including support for the Client Agent Extension.

use serde::{Deserialize, Serialize};
pub use a2a_protocol::extensions::client_routing::{
    ClientRoutingExtensionData, Participant, SimplifiedAgentCard,
};

/// Router configuration
///
/// Defines behavior for the team router including default routing target
/// and safety limits.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouterConfig {
    /// ID of the default agent to route to when no routing decision is made
    pub default_agent_id: String,
    /// Maximum number of routing hops to prevent infinite loops
    ///
    /// Default: 10
    #[serde(default = "default_max_routing_hops")]
    pub max_routing_hops: usize,
}

fn default_max_routing_hops() -> usize {
    10
}

impl Default for RouterConfig {
    fn default() -> Self {
        Self {
            default_agent_id: String::new(),
            max_routing_hops: 10,
        }
    }
}

/// Errors that can occur during team routing
#[derive(Debug, thiserror::Error)]
pub enum TeamError {
    /// Invalid recipient specified
    #[error("Invalid recipient: {0}")]
    InvalidRecipient(String),

    /// Maximum routing hops exceeded
    #[error("Maximum routing hops exceeded ({0})")]
    MaxHopsExceeded(usize),

    /// Router error during message routing
    #[error("Router error: {0}")]
    RouterError(String),

    /// Agent not found
    #[error("Agent not found: {0}")]
    AgentNotFound(String),

    /// Extension data parsing error
    #[error("Extension data parse error: {0}")]
    ExtensionParseError(#[from] serde_json::Error),

    /// Scheduling error (legacy, for compatibility)
    #[error("Scheduling error: {0}")]
    Scheduling(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Agent-related error
    #[error("Agent error: {0}")]
    Agent(String),

    /// Protocol error
    #[error("Protocol error: {0}")]
    Protocol(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_router_config_defaults() {
        let config = RouterConfig::default();
        assert_eq!(config.max_routing_hops, 10);
        assert_eq!(config.default_agent_id, "");
    }
}
