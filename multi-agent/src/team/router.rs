//! Router component for dynamic team message routing
//!
//! The Router replaces the Scheduler and enables flexible, metadata-driven message
//! routing between agents. It supports the Client Agent Extension for capable agents
//! and provides fallback routing for agents without extension support.

use super::types::{
    ClientRoutingRequest, ClientRoutingResponse, Recipient, RouterConfig, SimplifiedAgentCard,
    TeamError, EXTENSION_URI,
};
use crate::{Agent, AgentInfo};
use a2a_protocol::prelude::Message;
use std::collections::HashMap;
use std::sync::Arc;

/// Router for dynamic message routing within a team
///
/// The Router orchestrates message flow between agents based on:
/// - Extension-based routing decisions (for capable agents)
/// - Fallback to default agent (for agents without extension support)
/// - Back-to-sender routing (when agents request it)
/// - Maximum hop limits (to prevent infinite loops)
pub struct Router {
    /// ID of the default agent to route to when no routing decision is made
    default_agent_id: String,
    /// Maximum number of routing hops allowed
    max_routing_hops: usize,
    /// Stack of message senders for back-to-sender routing
    sender_stack: Vec<String>,
    /// Current hop count
    hop_count: usize,
    /// Optional list of candidate agents for the next step (handoffs)
    handoffs: Option<Vec<String>>,
}

impl Router {
    /// Create a new Router with the given configuration
    ///
    /// # Arguments
    /// * `config` - Router configuration including default agent and max hops
    ///
    /// # Example
    /// ```
    /// use multi_agent::team::{Router, RouterConfig};
    ///
    /// let config = RouterConfig {
    ///     default_agent_id: "default-agent".to_string(),
    ///     max_routing_hops: 10,
    /// };
    /// let router = Router::new(config);
    /// ```
    pub fn new(config: RouterConfig) -> Self {
        Self {
            default_agent_id: config.default_agent_id,
            max_routing_hops: config.max_routing_hops,
            sender_stack: Vec::new(),
            hop_count: 0,
            handoffs: None,
        }
    }

    /// Check if an agent supports the Client Agent Extension
    ///
    /// # Arguments
    /// * `agent_info` - Agent information including capabilities
    ///
    /// # Returns
    /// `true` if the agent declares support for the extension, `false` otherwise
    pub fn supports_extension(&self, agent_info: &AgentInfo) -> bool {
        agent_info
            .skills
            .iter()
            .any(|skill| skill.name == EXTENSION_URI)
    }

    /// Build simplified agent cards from agent information
    ///
    /// Converts full AgentInfo objects to lightweight SimplifiedAgentCard
    /// for inclusion in extension context.
    ///
    /// # Arguments
    /// * `agents` - Map of agent IDs to Agent trait objects
    ///
    /// # Returns
    /// Vector of SimplifiedAgentCard objects
    pub async fn build_simplified_cards(
        &self,
        agents: &HashMap<String, Arc<dyn Agent>>,
    ) -> Vec<SimplifiedAgentCard> {
        let mut cards = Vec::new();

        for (agent_id, agent) in agents {
            if let Ok(info) = agent.info().await {
                cards.push(SimplifiedAgentCard {
                    id: agent_id.clone(),
                    name: info.name.clone(),
                    description: info.description.clone(),
                    capabilities: info.skills.iter().map(|s| s.name.clone()).collect(),
                    supports_client_routing: self.supports_extension(&info),
                });
            }
        }

        cards
    }

    /// Inject extension context into a message for capable agents
    ///
    /// Adds the Client Agent Extension data to message metadata, including
    /// the list of peer agents and sender information.
    ///
    /// # Arguments
    /// * `message` - Message to inject extension context into
    /// * `agent_cards` - List of available peer agents
    /// * `sender` - ID of message sender ("user" or agent ID)
    pub fn inject_extension_context(
        &self,
        message: &mut Message,
        agent_cards: &[SimplifiedAgentCard],
        sender: &str,
    ) -> Result<(), TeamError> {
        // Build extension request
        let request = ClientRoutingRequest {
            agent_cards: agent_cards.to_vec(),
            sender: sender.to_string(),
        };

        // Add to metadata
        let mut metadata = message.metadata.clone().unwrap_or_default();
        metadata.insert(EXTENSION_URI.to_string(), serde_json::to_value(&request)?);
        message.metadata = Some(metadata);

        Ok(())
    }

    /// Extract recipient from extension response in message metadata
    ///
    /// Parses the Client Agent Extension response to determine the next recipient.
    /// Resolves special recipients like "sender" and "user".
    ///
    /// # Arguments
    /// * `message` - Message potentially containing routing decision
    ///
    /// # Returns
    /// * `Some(Recipient)` if routing decision found and valid
    /// * `None` if no routing decision present or extension not used
    pub fn extract_recipient(&mut self, message: &Message) -> Option<Recipient> {
        // Extract extension data from metadata
        let metadata = message.metadata.as_ref()?;
        let ext_data = metadata.get(EXTENSION_URI)?;

        // Parse routing response
        let response: ClientRoutingResponse = serde_json::from_value(ext_data.clone()).ok()?;

        // Store handoffs if present
        self.handoffs = response.handoffs;

        // Resolve recipient
        match response.recipient.as_str() {
            "user" => Some(Recipient::User),
            "sender" => {
                // Resolve to actual sender from stack
                self.sender_stack
                    .last()
                    .map(|sender_id| Recipient::agent(sender_id.clone()))
            }
            agent_id => Some(Recipient::agent(agent_id)),
        }
    }

    /// Push a sender onto the sender stack
    ///
    /// Used to track message senders for back-to-sender routing.
    ///
    /// # Arguments
    /// * `sender_id` - ID of the sender ("user" or agent ID)
    pub fn push_sender(&mut self, sender_id: String) {
        self.sender_stack.push(sender_id);
    }

    /// Pop a sender from the sender stack
    ///
    /// Returns and removes the most recent sender.
    ///
    /// # Returns
    /// * `Some(sender_id)` if stack is not empty
    /// * `None` if stack is empty
    pub fn pop_sender(&mut self) -> Option<String> {
        self.sender_stack.pop()
    }

    /// Route a message to the next recipient
    ///
    /// Main routing method that orchestrates:
    /// 1. Extension detection
    /// 2. Context injection (if supported)
    /// 3. Message sending to target agent
    /// 4. Recipient extraction from response
    /// 5. Hop count tracking
    ///
    /// # Arguments
    /// * `message` - Message to route
    /// * `agents` - Map of available agents
    /// * `sender` - ID of current sender
    ///
    /// # Returns
    /// * `Ok(Recipient)` with next routing target
    /// * `Err(TeamError)` if routing fails or max hops exceeded
    pub async fn route(
        &mut self,
        message: &mut Message,
        agents: &HashMap<String, Arc<dyn Agent>>,
        sender: &str,
    ) -> Result<Recipient, TeamError> {
        // Check hop limit
        if self.hop_count >= self.max_routing_hops {
            return Err(TeamError::MaxHopsExceeded(self.max_routing_hops));
        }
        self.hop_count += 1;

        // Push sender to stack for back-to-sender routing
        self.push_sender(sender.to_string());

        // Determine target agent from recipient or default
        let target_agent_id = match &self.extract_recipient(message) {
            Some(Recipient::Agent { agent_id }) => agent_id.clone(),
            Some(Recipient::User) => {
                self.pop_sender(); // Remove from stack since we're not proceeding
                return Ok(Recipient::User);
            }
            None => self.default_agent_id.clone(),
        };

        // Get target agent
        let agent = agents
            .get(&target_agent_id)
            .ok_or_else(|| TeamError::AgentNotFound(target_agent_id.clone()))?;

        // Get agent info to check extension support
        let agent_info = agent
            .info()
            .await
            .map_err(|e| TeamError::Agent(format!("Failed to get agent info: {}", e)))?;

        // Inject extension context if agent supports it
        if self.supports_extension(&agent_info) {
            let mut agent_cards = self.build_simplified_cards(agents).await;

            // Filter by handoffs if present
            if let Some(handoffs) = &self.handoffs {
                agent_cards.retain(|card| handoffs.contains(&card.id));
            }

            self.inject_extension_context(message, &agent_cards, sender)?;
        }

        // Clear handoffs after they have been potentially used for this hop
        self.handoffs = None;

        // Process message with agent
        let response = agent
            .process(message.clone())
            .await
            .map_err(|e| TeamError::Protocol(format!("Agent processing failed: {}", e)))?;

        // Extract routing decision from response
        let recipient = if let Some(recipient) = self.extract_recipient(&response) {
            recipient
        } else {
            // No routing decision provided
            // If agent doesn't support extension (basic agent), return to user
            // Otherwise, route to default agent
            if !self.supports_extension(&agent_info) {
                Recipient::User
            } else {
                Recipient::agent(&self.default_agent_id)
            }
        };

        // Update message with response content for next hop
        *message = response;

        Ok(recipient)
    }

    /// Reset the router state for a new conversation
    ///
    /// Clears sender stack and hop count.
    pub fn reset(&mut self) {
        self.sender_stack.clear();
        self.hop_count = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use a2a_protocol::AgentSkill;
    use crate::AgentInfo;
    use async_trait::async_trait;

    // Mock agent for testing
    struct MockAgent {
        info: AgentInfo,
        response: Message,
    }

    #[async_trait]
    impl Agent for MockAgent {
        async fn process(&self, _message: Message) -> a2a_protocol::prelude::A2aResult<Message> {
            Ok(self.response.clone())
        }

        async fn info(&self) -> a2a_protocol::prelude::A2aResult<AgentInfo> {
            Ok(self.info.clone())
        }
    }

    fn create_mock_agent(id: &str, supports_extension: bool) -> MockAgent {
        let skills = if supports_extension {
            vec![AgentSkill {
                name: EXTENSION_URI.to_string(),
                description: None,
                category: None,
                tags: vec![],
                examples: vec![],
            }]
        } else {
            vec![]
        };

        MockAgent {
            info: AgentInfo {
                id: id.to_string(),
                name: format!("{} Agent", id),
                description: format!("Mock agent {}", id),
                skills,
                metadata: HashMap::new(),
            },
            response: Message::agent_text("Response"),
        }
    }

    #[test]
    fn test_router_new() {
        let config = RouterConfig {
            default_agent_id: "default".to_string(),
            max_routing_hops: 5,
        };
        let router = Router::new(config);

        assert_eq!(router.default_agent_id, "default");
        assert_eq!(router.max_routing_hops, 5);
        assert_eq!(router.hop_count, 0);
        assert!(router.sender_stack.is_empty());
    }

    #[test]
    fn test_supports_extension() {
        let config = RouterConfig {
            default_agent_id: "default".to_string(),
            max_routing_hops: 10,
        };
        let router = Router::new(config);

        let agent_with_ext = AgentInfo {
            id: "agent1".to_string(),
            name: "Agent 1".to_string(),
            description: "Test agent".to_string(),
            skills: vec![AgentSkill {
                name: EXTENSION_URI.to_string(),
                description: None,
                category: None,
                tags: vec![],
                examples: vec![],
            }],
            metadata: HashMap::new(),
        };

        let agent_without_ext = AgentInfo {
            id: "agent2".to_string(),
            name: "Agent 2".to_string(),
            description: "Test agent".to_string(),
            skills: vec![],
            metadata: HashMap::new(),
        };

        assert!(router.supports_extension(&agent_with_ext));
        assert!(!router.supports_extension(&agent_without_ext));
    }

    #[tokio::test]
    async fn test_build_simplified_cards() {
        let config = RouterConfig {
            default_agent_id: "default".to_string(),
            max_routing_hops: 10,
        };
        let router = Router::new(config);

        let mut agents: HashMap<String, Arc<dyn Agent>> = HashMap::new();
        agents.insert(
            "agent1".to_string(),
            Arc::new(create_mock_agent("agent1", true)),
        );
        agents.insert(
            "agent2".to_string(),
            Arc::new(create_mock_agent("agent2", false)),
        );

        let cards = router.build_simplified_cards(&agents).await;

        assert_eq!(cards.len(), 2);
        assert!(cards.iter().any(|c| c.id == "agent1" && c.supports_client_routing));
        assert!(cards.iter().any(|c| c.id == "agent2" && !c.supports_client_routing));
    }

    #[test]
    fn test_inject_extension_context() {
        let config = RouterConfig {
            default_agent_id: "default".to_string(),
            max_routing_hops: 10,
        };
        let router = Router::new(config);

        let mut message = Message::user_text("Test message");
        let agent_cards = vec![SimplifiedAgentCard {
            id: "agent1".to_string(),
            name: "Agent 1".to_string(),
            description: "Test agent".to_string(),
            capabilities: vec![],
            supports_client_routing: false,
        }];

        router
            .inject_extension_context(&mut message, &agent_cards, "user")
            .unwrap();

        assert!(message.metadata.is_some());
        assert!(message
            .metadata
            .as_ref()
            .unwrap()
            .contains_key(EXTENSION_URI));
    }

    #[test]
    fn test_extract_recipient_agent() {
        let config = RouterConfig {
            default_agent_id: "default".to_string(),
            max_routing_hops: 10,
        };
        let mut router = Router::new(config);

        let mut message = Message::agent_text("Response");

        let response = ClientRoutingResponse {
            recipient: "agent2".to_string(),
            reason: Some("Test routing".to_string()),
            handoffs: None,
        };

        let mut metadata = HashMap::new();
        metadata.insert(
            EXTENSION_URI.to_string(),
            serde_json::to_value(&response).unwrap(),
        );
        message.metadata = Some(metadata);

        let recipient = router.extract_recipient(&message);
        assert!(recipient.is_some());
        match recipient.unwrap() {
            Recipient::Agent { agent_id } => assert_eq!(agent_id, "agent2"),
            _ => panic!("Expected Agent recipient"),
        }
    }

    #[test]
    fn test_extract_recipient_user() {
        let config = RouterConfig {
            default_agent_id: "default".to_string(),
            max_routing_hops: 10,
        };
        let mut router = Router::new(config);

        let mut message = Message::agent_text("Response");

        let response = ClientRoutingResponse {
            recipient: "user".to_string(),
            reason: None,
            handoffs: None,
        };

        let mut metadata = HashMap::new();
        metadata.insert(
            EXTENSION_URI.to_string(),
            serde_json::to_value(&response).unwrap(),
        );
        message.metadata = Some(metadata);

        let recipient = router.extract_recipient(&message);
        assert!(recipient.is_some());
        assert_eq!(recipient.unwrap(), Recipient::User);
    }

    #[test]
    fn test_extract_recipient_no_extension() {
        let config = RouterConfig {
            default_agent_id: "default".to_string(),
            max_routing_hops: 10,
        };
        let mut router = Router::new(config);

        let message = Message::agent_text("Response without extension");
        let recipient = router.extract_recipient(&message);
        assert!(recipient.is_none());
    }

    #[test]
    fn test_sender_stack() {
        let config = RouterConfig {
            default_agent_id: "default".to_string(),
            max_routing_hops: 10,
        };
        let mut router = Router::new(config);

        router.push_sender("user".to_string());
        router.push_sender("agent1".to_string());
        router.push_sender("agent2".to_string());

        assert_eq!(router.sender_stack.len(), 3);

        assert_eq!(router.pop_sender(), Some("agent2".to_string()));
        assert_eq!(router.pop_sender(), Some("agent1".to_string()));
        assert_eq!(router.pop_sender(), Some("user".to_string()));
        assert_eq!(router.pop_sender(), None);
    }

    #[test]
    fn test_extract_recipient_sender() {
        let config = RouterConfig {
            default_agent_id: "default".to_string(),
            max_routing_hops: 10,
        };
        let mut router = Router::new(config);

        // Push sender to stack
        router.push_sender("agent1".to_string());

        let mut message = Message::agent_text("Response");

        let response = ClientRoutingResponse {
            recipient: "sender".to_string(),
            reason: Some("Back to sender".to_string()),
            handoffs: None,
        };

        let mut metadata = HashMap::new();
        metadata.insert(
            EXTENSION_URI.to_string(),
            serde_json::to_value(&response).unwrap(),
        );
        message.metadata = Some(metadata);

        let recipient = router.extract_recipient(&message);
        assert!(recipient.is_some());
        match recipient.unwrap() {
            Recipient::Agent { agent_id } => assert_eq!(agent_id, "agent1"),
            _ => panic!("Expected Agent recipient"),
        }
    }

    #[test]
    fn test_reset() {
        let config = RouterConfig {
            default_agent_id: "default".to_string(),
            max_routing_hops: 10,
        };
        let mut router = Router::new(config);

        router.push_sender("user".to_string());
        router.hop_count = 5;

        router.reset();

        assert_eq!(router.hop_count, 0);
        assert!(router.sender_stack.is_empty());
    }
}
