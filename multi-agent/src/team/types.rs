//! Types for the Team Router and Client Agent Extension
//!
//! This module defines the core types used by the Router component for dynamic
//! message routing between agents, including support for the Client Agent Extension.

use serde::{Deserialize, Serialize};
use a2a_protocol::core::extension::ProtocolExtension;

/// Extension URI for the Client Agent Routing Extension
///
/// This extension enables agents to receive peer agent lists and make routing decisions.
/// Complies with A2A Protocol v0.3.0 Section 4.6 (Extensions).
pub const EXTENSION_URI: &str = "https://ranch.woi.dev/extensions/client-routing/v1";

/// Extension version
pub const EXTENSION_VERSION: &str = "v1";

/// Extension name for human-readable display
pub const EXTENSION_NAME: &str = "Client Agent Routing Extension";

/// Extension description
pub const EXTENSION_DESCRIPTION: &str =
    "Enables agents to receive peer agent list and make routing decisions";

/// The Client Routing Extension definition
pub struct ClientRoutingExtension;

impl ProtocolExtension for ClientRoutingExtension {
    fn uri(&self) -> &str {
        EXTENSION_URI
    }

    fn version(&self) -> &str {
        EXTENSION_VERSION
    }

    fn name(&self) -> &str {
        EXTENSION_NAME
    }

    fn description(&self) -> &str {
        EXTENSION_DESCRIPTION
    }
}

/// Destination for a routed message
///
/// Represents where a message should be sent after routing decision is made.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Recipient {
    /// Route to a specific agent by ID
    Agent { agent_id: String },
    /// Route back to the user (end conversation)
    User,
}

impl Recipient {
    /// Create a recipient targeting a specific agent
    pub fn agent(agent_id: impl Into<String>) -> Self {
        Self::Agent {
            agent_id: agent_id.into(),
        }
    }

    /// Create a recipient targeting the user
    pub fn user() -> Self {
        Self::User
    }
}

/// Lightweight agent information for extension context
///
/// This is a simplified version of AgentCard optimized for routing decisions.
/// Only includes the essential information needed by agents to make routing choices.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimplifiedAgentCard {
    /// Unique agent identifier
    pub id: String,
    /// Human-readable agent name
    pub name: String,
    /// Brief description of agent capabilities
    pub description: String,
    /// List of capability tags (e.g., ["search", "summarize"])
    pub capabilities: Vec<String>,
    /// Whether this agent supports the Client Agent Extension
    #[serde(rename = "supportsClientRouting")]
    pub supports_client_routing: bool,
}

/// Extension request data sent from Router to Agent
///
/// Injected into `message.metadata[EXTENSION_URI]` when the agent supports
/// the Client Agent Extension. Contains the list of peer agents and sender information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientRoutingRequest {
    /// List of available peer agents in the team
    #[serde(rename = "agentCards")]
    pub agent_cards: Vec<SimplifiedAgentCard>,
    /// Identity of the message sender ("user" or agent ID)
    pub sender: String,
}

/// Extension response data returned from Agent to Router
///
/// Extracted from `message.metadata[EXTENSION_URI]` in the agent's response.
/// Specifies where the message should be routed next.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientRoutingResponse {
    /// Target recipient for the message
    ///
    /// Can be:
    /// - Agent ID: Route to specific agent
    /// - "user": Route back to user
    /// - "sender": Route back to the message sender
    pub recipient: String,
    /// Optional explanation for routing decision
    pub reason: Option<String>,
    /// Optional list of candidate agents for the next step
    ///
    /// If provided, the router will convert these IDs into simplified agent cards
    /// and pass them to the next agent if it supports the extension.
    pub handoffs: Option<Vec<String>>,
}

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
    fn test_recipient_creation() {
        let agent_recipient = Recipient::agent("agent-1");
        assert_eq!(
            agent_recipient,
            Recipient::Agent {
                agent_id: "agent-1".to_string()
            }
        );

        let user_recipient = Recipient::user();
        assert_eq!(user_recipient, Recipient::User);
    }

    #[test]
    fn test_router_config_defaults() {
        let config = RouterConfig::default();
        assert_eq!(config.max_routing_hops, 10);
        assert_eq!(config.default_agent_id, "");
    }

    #[test]
    fn test_simplified_agent_card_serialization() {
        let card = SimplifiedAgentCard {
            id: "test-agent".to_string(),
            name: "Test Agent".to_string(),
            description: "A test agent".to_string(),
            capabilities: vec!["test".to_string()],
            supports_client_routing: true,
        };

        let json = serde_json::to_string(&card).unwrap();
        let deserialized: SimplifiedAgentCard = serde_json::from_str(&json).unwrap();

        assert_eq!(card.id, deserialized.id);
        assert_eq!(card.name, deserialized.name);
        assert_eq!(card.supports_client_routing, deserialized.supports_client_routing);
    }

    #[test]
    fn test_client_routing_request_serialization() {
        let request = ClientRoutingRequest {
            agent_cards: vec![SimplifiedAgentCard {
                id: "agent-1".to_string(),
                name: "Agent 1".to_string(),
                description: "First agent".to_string(),
                capabilities: vec!["capability-1".to_string()],
                supports_client_routing: false,
            }],
            sender: "user".to_string(),
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: ClientRoutingRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(request.sender, deserialized.sender);
        assert_eq!(request.agent_cards.len(), deserialized.agent_cards.len());
    }

    #[test]
    fn test_client_routing_response_serialization() {
        let response = ClientRoutingResponse {
            recipient: "agent-2".to_string(),
            reason: Some("Agent 2 has the required capability".to_string()),
            handoffs: Some(vec!["agent-3".to_string(), "agent-4".to_string()]),
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: ClientRoutingResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(response.recipient, deserialized.recipient);
        assert_eq!(response.reason, deserialized.reason);
    }

    #[test]
    fn test_extension_constants() {
        assert_eq!(EXTENSION_URI, "https://ranch.woi.dev/extensions/client-routing/v1");
        assert_eq!(EXTENSION_VERSION, "v1");
        assert!(!EXTENSION_NAME.is_empty());
        assert!(!EXTENSION_DESCRIPTION.is_empty());
    }
}
