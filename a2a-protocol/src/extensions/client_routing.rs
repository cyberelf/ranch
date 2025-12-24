//! Client Agent Routing Extension
//!
//! This extension enables agents to receive a list of peer agents and make
//! dynamic routing decisions by specifying a recipient in their response.

use serde::{Deserialize, Serialize};
use crate::core::extension::AgentExtension;

/// Lightweight agent information for extension context
///
/// This is a simplified version of AgentCard optimized for routing decisions.
/// Only includes the essential information needed by agents to make routing choices.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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

/// Participant in a routing exchange (sender or recipient)
///
/// Used for both sender identification and recipient targeting.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Participant {
    /// The user/client
    User,
    /// A specific agent identified by ID
    Agent { id: String },
}

impl Participant {
    /// Create a participant for an agent
    pub fn agent(id: impl Into<String>) -> Self {
        Self::Agent { id: id.into() }
    }

    /// Create a participant for the user
    pub fn user() -> Self {
        Self::User
    }
}

/// Unified extension data for Client Routing Extension
///
/// This structure is used bidirectionally:
/// - Router → Agent: Sets `sender` and optional `agent_cards` to provide context
/// - Agent → Router: Sets `recipient` (and optionally `reason`) to specify routing
///
/// Follows A2A extension pattern of using a single data structure for both directions.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ClientRoutingExtensionData {
    /// Identity of the message sender
    /// Set by Router when sending to Agent
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sender: Option<Participant>,

    /// Target recipient for the message
    /// Set by Agent when responding to Router
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recipient: Option<Participant>,

    /// List of suggested agents for the next step (handoffs)
    /// Set by Agent when responding to Router
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handoffs: Option<Vec<String>>,

    /// List of available peer agents in the team
    /// Set by Router when sending to Agent (optional)
    #[serde(rename = "agentCards", skip_serializing_if = "Option::is_none")]
    pub agent_cards: Option<Vec<SimplifiedAgentCard>>,

    /// Optional explanation for routing decision
    /// Set by Agent when responding to Router (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

impl AgentExtension for ClientRoutingExtensionData {
    const URI: &'static str = "https://ranch.woi.dev/extensions/client-routing/v1";
    const VERSION: &'static str = "v1";
    const NAME: &'static str = "Client Agent Routing Extension";
    const DESCRIPTION: &'static str = "Enables agents to receive peer agent list and make routing decisions";
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::extension::AgentExtension;

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
    fn test_client_routing_extension_data_serialization() {
        // Test Router → Agent direction
        let router_to_agent = ClientRoutingExtensionData {
            sender: Some(Participant::user()),
            agent_cards: Some(vec![SimplifiedAgentCard {
                id: "agent-1".to_string(),
                name: "Agent 1".to_string(),
                description: "First agent".to_string(),
                capabilities: vec!["capability-1".to_string()],
                supports_client_routing: false,
            }]),
            recipient: None,
            handoffs: None,
            reason: None,
        };

        let json = serde_json::to_string(&router_to_agent).unwrap();
        let deserialized: ClientRoutingExtensionData = serde_json::from_str(&json).unwrap();

        assert_eq!(router_to_agent.sender, deserialized.sender);
        assert_eq!(router_to_agent.agent_cards.as_ref().unwrap().len(), 
                   deserialized.agent_cards.as_ref().unwrap().len());
    }

    #[test]
    fn test_client_routing_extension_data_agent_response() {
        // Test Agent → Router direction
        let agent_to_router = ClientRoutingExtensionData {
            sender: None,
            agent_cards: None,
            recipient: Some(Participant::agent("agent-2")),
            handoffs: None,
            reason: Some("Agent 2 has the required capability".to_string()),
        };

        let json = serde_json::to_string(&agent_to_router).unwrap();
        let deserialized: ClientRoutingExtensionData = serde_json::from_str(&json).unwrap();

        assert_eq!(agent_to_router.recipient, deserialized.recipient);
        assert_eq!(agent_to_router.reason, deserialized.reason);
    }

    #[test]
    fn test_participant_enum() {
        let user_participant = Participant::user();
        assert_eq!(user_participant, Participant::User);

        let agent_participant = Participant::agent("agent-1");
        match agent_participant {
            Participant::Agent { id } => assert_eq!(id, "agent-1"),
            _ => panic!("Expected Agent participant"),
        }

        // Test serialization
        let json = serde_json::to_string(&user_participant).unwrap();
        let deserialized: Participant = serde_json::from_str(&json).unwrap();
        assert_eq!(user_participant, deserialized);
    }

    #[test]
    fn test_extension_data_with_typed_enums() {
        // Test with agent sender and user recipient
        let data = ClientRoutingExtensionData {
            sender: Some(Participant::agent("agent-1")),
            recipient: Some(Participant::User),
            agent_cards: None,
            handoffs: None,
            reason: Some("Routing to user".to_string()),
        };

        let json = serde_json::to_string(&data).unwrap();
        let deserialized: ClientRoutingExtensionData = serde_json::from_str(&json).unwrap();

        assert_eq!(data.sender, deserialized.sender);
        assert_eq!(data.recipient, deserialized.recipient);

        // Test with user sender and agent recipient
        let data2 = ClientRoutingExtensionData {
            sender: Some(Participant::User),
            recipient: Some(Participant::agent("agent-2")),
            agent_cards: None,
            handoffs: None,
            reason: Some("Routing to agent".to_string()),
        };

        let json2 = serde_json::to_string(&data2).unwrap();
        let deserialized2: ClientRoutingExtensionData = serde_json::from_str(&json2).unwrap();

        match deserialized2.recipient.unwrap() {
            Participant::Agent { id } => assert_eq!(id, "agent-2"),
            _ => panic!("Expected agent recipient"),
        }
    }

    #[test]
    fn test_extension_constants() {
        assert_eq!(
            ClientRoutingExtensionData::URI,
            "https://ranch.woi.dev/extensions/client-routing/v1"
        );
        assert_eq!(ClientRoutingExtensionData::VERSION, "v1");
        assert!(!ClientRoutingExtensionData::NAME.is_empty());
        assert!(!ClientRoutingExtensionData::DESCRIPTION.is_empty());
    }
}
