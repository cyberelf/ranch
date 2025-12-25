//! Router component for dynamic team message routing
//!
//! The Router replaces the Scheduler and enables flexible, metadata-driven message
//! routing between agents. It supports the Client Agent Extension for capable agents
//! and provides fallback routing for agents without extension support.

use super::types::{
    ClientRoutingExtensionData, Participant, RouterConfig, SimplifiedAgentCard, TeamError,
};
use super::TeamAgentConfig;
use crate::manager::AgentManager;
use crate::{Agent, AgentInfo};
use a2a_protocol::core::extension::AgentExtension;
use a2a_protocol::prelude::Message;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Router for dynamic message routing within a team
///
/// The Router orchestrates message flow between agents based on:
/// - Extension-based routing decisions (for capable agents)
/// - Fallback to default agent (for agents without extension support)
/// - Maximum hop limits (to prevent infinite loops)
pub struct Router {
    /// ID of the default agent to route to when no routing decision is made
    pub default_agent_id: String,
    /// Maximum number of routing hops allowed
    max_routing_hops: usize,
    /// Agent manager for resolving agents
    agent_manager: Arc<AgentManager>,
    /// Configuration of agents in the team
    team_agents_config: Vec<TeamAgentConfig>,
    /// Cache of resolved agents
    agents_cache: RwLock<HashMap<String, Arc<dyn Agent>>>,
    /// Cache of extension support status
    extension_support_cache: RwLock<HashMap<String, bool>>,
    /// Cache of simplified agent cards
    simplified_cards_cache: RwLock<Option<Vec<SimplifiedAgentCard>>>,
}

impl Router {
    /// Create a new Router with the given configuration
    ///
    /// # Arguments
    /// * `config` - Router configuration including default agent and max hops
    /// * `team_agents_config` - Configuration of agents in the team
    /// * `agent_manager` - Agent manager for resolving agents
    pub fn new(
        config: RouterConfig,
        team_agents_config: Vec<TeamAgentConfig>,
        agent_manager: Arc<AgentManager>,
    ) -> Self {
        Self {
            default_agent_id: config.default_agent_id,
            max_routing_hops: config.max_routing_hops,
            agent_manager,
            team_agents_config,
            agents_cache: RwLock::new(HashMap::new()),
            extension_support_cache: RwLock::new(HashMap::new()),
            simplified_cards_cache: RwLock::new(None),
        }
    }

    /// Get resolved agents, populating cache if necessary
    async fn get_agents(&self) -> HashMap<String, Arc<dyn Agent>> {
        let cache = self.agents_cache.read().await;
        if !cache.is_empty() {
            return cache.clone();
        }
        drop(cache);

        let mut cache = self.agents_cache.write().await;
        // Double check
        if cache.is_empty() {
            for agent_config in &self.team_agents_config {
                if let Some(agent) = self.agent_manager.get(&agent_config.agent_id).await {
                    cache.insert(agent_config.agent_id.clone(), agent);
                }
            }
        }
        cache.clone()
    }

    /// Check if an agent supports the Client Agent Extension
    ///
    /// # Arguments
    /// * `agent_info` - Agent information including capabilities
    ///
    /// # Returns
    /// `true` if the agent declares support for the extension, `false` otherwise
    pub fn supports_extension(&self, agent_info: &AgentInfo) -> bool {
        ClientRoutingExtensionData::supported_by(&agent_info.capabilities)
    }

    /// Check if an agent supports the Client Agent Extension (cached)
    async fn check_extension_support(
        &self,
        agent_id: &str,
        agent: &dyn Agent,
    ) -> Result<bool, TeamError> {
        {
            let cache = self.extension_support_cache.read().await;
            if let Some(&support) = cache.get(agent_id) {
                return Ok(support);
            }
        }

        let info = agent
            .info()
            .await
            .map_err(|e| TeamError::Agent(format!("Failed to get agent info: {}", e)))?;
        let support = self.supports_extension(&info);

        let mut cache = self.extension_support_cache.write().await;
        cache.insert(agent_id.to_string(), support);
        Ok(support)
    }

    /// Get simplified agent cards from agent information (cached)
    ///
    /// Converts full AgentInfo objects to lightweight SimplifiedAgentCard
    /// for inclusion in extension context.
    ///
    /// # Returns
    /// Vector of SimplifiedAgentCard objects
    pub async fn get_simplified_cards(
        &self,
        agents: &HashMap<String, Arc<dyn Agent>>,
    ) -> Vec<SimplifiedAgentCard> {
        {
            let cache = self.simplified_cards_cache.read().await;
            if let Some(cards) = cache.as_ref() {
                return cards.clone();
            }
        }

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

        let mut cache = self.simplified_cards_cache.write().await;
        *cache = Some(cards.clone());
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
        // Build extension data for Router → Agent direction
        let sender_participant = if sender == "user" {
            Participant::user()
        } else {
            Participant::agent(sender)
        };

        let ext_data = ClientRoutingExtensionData {
            sender: Some(sender_participant),
            agent_cards: Some(agent_cards.to_vec()),
            recipient: None,
            handoffs: None,
            reason: None,
        };

        // Use typed accessor to set extension
        message.set_extension(ext_data)
            .map_err(TeamError::ExtensionParseError)?;

        Ok(())
    }

    /// Extract recipient from extension response in message metadata
    ///
    /// Parses the Client Agent Extension response to determine the next recipient.
    ///
    /// # Arguments
    /// * `message` - Message potentially containing routing decision
    ///
    /// # Returns
    /// * `Some(Participant)` if routing decision found and valid
    /// * `None` if no routing decision present or extension not used
    pub fn extract_recipient(&self, message: &Message) -> Option<Participant> {
        // Use typed accessor to get extension data
        let ext_data: ClientRoutingExtensionData = message
            .get_extension::<ClientRoutingExtensionData>()
            .ok()??;

        // Return recipient directly from extension data
        ext_data.recipient
    }

    /// Extract handoffs from extension response in message metadata
    ///
    /// Parses the Client Agent Extension response to determine suggested next agents.
    ///
    /// # Arguments
    /// * `message` - Message potentially containing handoff suggestions
    ///
    /// # Returns
    /// * `Some(Vec<String>)` if handoffs found
    /// * `None` if no handoffs present
    pub fn extract_handoffs(&self, message: &Message) -> Option<Vec<String>> {
        // Use typed accessor to get extension data
        let ext_data: ClientRoutingExtensionData = message
            .get_extension::<ClientRoutingExtensionData>()
            .ok()??;

        ext_data.handoffs
    }



    /// Process a message through the team routing logic
    ///
    /// Orchestrates the entire conversation flow:
    /// 1. Resolves agents
    /// 2. Routes message through agents until completion or max hops
    /// 3. Manages conversation state (hops, handoffs)
    ///
    /// # Arguments
    /// * `initial_message` - The starting message from the user
    ///
    /// # Returns
    /// * `Ok(Message)` - The final response message
    /// * `Err(TeamError)` - If routing fails or max hops exceeded
    pub async fn process(&self, initial_message: Message) -> Result<Message, TeamError> {
        let agents = self.get_agents().await;
        
        if agents.is_empty() {
            return Err(TeamError::Configuration("No agents found in team".to_string()));
        }
        
        let mut current_message = initial_message;
        let mut hop_count = 0;
        let mut handoffs: Option<Vec<String>> = None;

        // Initial sender is "user", start with default agent
        let mut sender = "user".to_string();
        let mut target_agent_id = self.default_agent_id.clone();

        loop {
            // Check hop limit
            if hop_count >= self.max_routing_hops {
                return Err(TeamError::MaxHopsExceeded(self.max_routing_hops));
            }
            hop_count += 1;

            // Prevent routing to sender (loop detection)
            if target_agent_id == sender {
                return Err(TeamError::Protocol(format!(
                    "Cannot route message back to sender: {}",
                    sender
                )));
            }

            // Get target agent
            let agent = agents
                .get(&target_agent_id)
                .ok_or_else(|| TeamError::AgentNotFound(target_agent_id.to_string()))?;

            // Check extension support (cached)
            let supports_ext = self
                .check_extension_support(&target_agent_id, agent.as_ref())
                .await?;

            // Inject extension context if agent supports it
            if supports_ext {
                let mut agent_cards = self.get_simplified_cards(&agents).await;

                // Filter out the target agent itself to avoid self-routing suggestions
                agent_cards.retain(|card| card.id != target_agent_id);

                // Filter by handoffs if present
                if let Some(current_handoffs) = &handoffs {
                    agent_cards.retain(|card| current_handoffs.contains(&card.id));
                }

                self.inject_extension_context(&mut current_message, &agent_cards, &sender)?;
            }

            // Clear handoffs after they have been potentially used for this hop
            handoffs = None;

            // Process message with agent
            let response = agent
                .process(current_message.clone())
                .await
                .map_err(|e| TeamError::Protocol(format!("Agent processing failed: {}", e)))?;

            // Extract handoffs from response
            if let Some(new_handoffs) = self.extract_handoffs(&response) {
                handoffs = Some(new_handoffs);
            }

            // Extract routing decision from response
            let recipient = if let Some(recipient) = self.extract_recipient(&response) {
                // Validate routing decision doesn't create loops
                if let Participant::Agent { id } = &recipient {
                    if id == &target_agent_id {
                        // Agent trying to route to itself
                        return Err(TeamError::Protocol(format!(
                            "Agent {} cannot route to itself",
                            target_agent_id
                        )));
                    }
                }
                recipient
            } else {
                // No routing decision provided - apply defaults
                if !supports_ext {
                    // Basic agent without extension support - end conversation
                    Participant::User
                } else if target_agent_id == self.default_agent_id {
                    // Default agent with no routing decision - end conversation
                    Participant::User
                } else {
                    // Non-default agent with no decision - route to default
                    Participant::agent(&self.default_agent_id)
                }
            };

            // Update message with response content for next hop
            current_message = response;

            match recipient {
                Participant::User => {
                    // Return to user - end routing
                    return Ok(current_message);
                }
                Participant::Agent { id: next_agent_id } => {
                    // Continue routing to next agent
                    sender = target_agent_id;
                    target_agent_id = next_agent_id;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use a2a_protocol::{AgentCapabilities, AgentSkill};
    use a2a_protocol::agent_card::AgentExtensionInfo;
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
                name: ClientRoutingExtensionData::URI.to_string(),
                description: None,
                category: None,
                tags: vec![],
                examples: vec![],
            }]
        } else {
            vec![]
        };

        let capabilities = if supports_extension {
            AgentCapabilities {
                extensions: vec![AgentExtensionInfo {
                    uri: ClientRoutingExtensionData::URI.to_string(),
                    name: Some("Client Routing".to_string()),
                    version: Some("1.0".to_string()),
                    description: None,
                }],
                ..Default::default()
            }
        } else {
            AgentCapabilities::default()
        };

        MockAgent {
            info: AgentInfo {
                id: id.to_string(),
                name: format!("{} Agent", id),
                description: format!("Mock agent {}", id),
                skills,
                metadata: HashMap::new(),
                capabilities,
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
        let router = Router::new(config, vec![], Arc::new(AgentManager::new()));

        assert_eq!(router.default_agent_id, "default");
        assert_eq!(router.max_routing_hops, 5);
    }

    #[test]
    fn test_supports_extension() {
        let config = RouterConfig {
            default_agent_id: "default".to_string(),
            max_routing_hops: 10,
        };
        let router = Router::new(config, vec![], Arc::new(AgentManager::new()));

        let agent_with_ext = AgentInfo {
            id: "agent1".to_string(),
            name: "Agent 1".to_string(),
            description: "Test agent".to_string(),
            skills: vec![AgentSkill {
                name: ClientRoutingExtensionData::URI.to_string(),
                description: None,
                category: None,
                tags: vec![],
                examples: vec![],
            }],
            metadata: HashMap::new(),
            capabilities: AgentCapabilities {
                extensions: vec![AgentExtensionInfo {
                    uri: ClientRoutingExtensionData::URI.to_string(),
                    name: Some("Client Routing".to_string()),
                    version: Some("1.0".to_string()),
                    description: None,
                }],
                ..Default::default()
            },
        };

        let agent_without_ext = AgentInfo {
            id: "agent2".to_string(),
            name: "Agent 2".to_string(),
            description: "Test agent".to_string(),
            skills: vec![],
            metadata: HashMap::new(),
            capabilities: AgentCapabilities::default(),
        };

        assert!(router.supports_extension(&agent_with_ext));
        assert!(!router.supports_extension(&agent_without_ext));
    }

    #[tokio::test]
    async fn test_get_simplified_cards() {
        let config = RouterConfig {
            default_agent_id: "default".to_string(),
            max_routing_hops: 10,
        };
        let mut agents: HashMap<String, Arc<dyn Agent>> = HashMap::new();
        agents.insert(
            "agent1".to_string(),
            Arc::new(create_mock_agent("agent1", true)),
        );
        agents.insert(
            "agent2".to_string(),
            Arc::new(create_mock_agent("agent2", false)),
        );

        let router = Router::new(config, vec![], Arc::new(AgentManager::new()));

        let cards = router.get_simplified_cards(&agents).await;

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
        let router = Router::new(config, vec![], Arc::new(AgentManager::new()));

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

        // Verify extension was set using typed accessor
        let ext_data: ClientRoutingExtensionData = message.get_extension().unwrap().unwrap();
        assert_eq!(ext_data.sender, Some(Participant::User));
        assert!(ext_data.agent_cards.is_some());
        assert_eq!(ext_data.agent_cards.unwrap().len(), 1);
    }

    #[test]
    fn test_extract_recipient_agent() {
        let config = RouterConfig {
            default_agent_id: "default".to_string(),
            max_routing_hops: 10,
        };
        let router = Router::new(config, vec![], Arc::new(AgentManager::new()));

        let mut message = Message::agent_text("Response");

        // Set extension data using typed accessor with Participant enum
        let ext_data = ClientRoutingExtensionData {
            sender: None,
            agent_cards: None,
            recipient: Some(Participant::agent("agent2")),
            handoffs: None,
            reason: Some("Test routing".to_string()),
        };
        message.set_extension(ext_data).unwrap();

        let recipient = router.extract_recipient(&message);
        assert!(recipient.is_some());
        match recipient.unwrap() {
            Participant::Agent { id: agent_id } => assert_eq!(agent_id, "agent2"),
            _ => panic!("Expected Agent recipient"),
        }
    }

    #[test]
    fn test_extract_recipient_user() {
        let config = RouterConfig {
            default_agent_id: "default".to_string(),
            max_routing_hops: 10,
        };
        let router = Router::new(config, vec![], Arc::new(AgentManager::new()));

        let mut message = Message::agent_text("Response");

        // Set extension data using typed accessor with Participant enum
        let ext_data = ClientRoutingExtensionData {
            sender: None,
            agent_cards: None,
            recipient: Some(Participant::User),
            handoffs: None,
            reason: None,
        };
        message.set_extension(ext_data).unwrap();

        let recipient = router.extract_recipient(&message);
        assert!(recipient.is_some());
        assert_eq!(recipient.unwrap(), Participant::User);
    }

    #[test]
    fn test_extract_recipient_no_extension() {
        let config = RouterConfig {
            default_agent_id: "default".to_string(),
            max_routing_hops: 10,
        };
        let router = Router::new(config, vec![], Arc::new(AgentManager::new()));

        let message = Message::agent_text("Response without extension");
        let recipient = router.extract_recipient(&message);
        assert!(recipient.is_none());
    }

    fn create_mock_agent_info(id: &str, supports_extension: bool) -> AgentInfo {
        let skills = if supports_extension {
            vec![AgentSkill {
                name: ClientRoutingExtensionData::URI.to_string(),
                description: None,
                category: None,
                tags: vec![],
                examples: vec![],
            }]
        } else {
            vec![]
        };

        let capabilities = if supports_extension {
            AgentCapabilities {
                extensions: vec![AgentExtensionInfo {
                    uri: ClientRoutingExtensionData::URI.to_string(),
                    name: Some("Client Routing".to_string()),
                    version: Some("1.0".to_string()),
                    description: None,
                }],
                ..Default::default()
            }
        } else {
            AgentCapabilities::default()
        };

        AgentInfo {
            id: id.to_string(),
            name: format!("{} Agent", id),
            description: format!("Mock agent {}", id),
            skills,
            metadata: HashMap::new(),
            capabilities,
        }
    }

    #[tokio::test]
    async fn test_process_max_hops() {
        let config = RouterConfig {
            default_agent_id: "agent1".to_string(),
            max_routing_hops: 2,
        };

        let manager = Arc::new(AgentManager::new());

        // Agent 1 routes to Agent 2
        let mut agent1_resp = Message::agent_text("To Agent 2");
        agent1_resp
            .set_extension(ClientRoutingExtensionData {
                recipient: Some(Participant::agent("agent2")),
                ..Default::default()
            })
            .unwrap();

        manager
            .register(Arc::new(MockAgent {
                info: create_mock_agent_info("agent1", true),
                response: agent1_resp,
            }))
            .await
            .unwrap();

        // Agent 2 routes to Agent 3
        let mut agent2_resp = Message::agent_text("To Agent 3");
        agent2_resp
            .set_extension(ClientRoutingExtensionData {
                recipient: Some(Participant::agent("agent3")),
                ..Default::default()
            })
            .unwrap();

        manager
            .register(Arc::new(MockAgent {
                info: create_mock_agent_info("agent2", true),
                response: agent2_resp,
            }))
            .await
            .unwrap();

        let team_agents = vec![
            TeamAgentConfig {
                agent_id: "agent1".to_string(),
                role: "role".to_string(),
                capabilities: vec![],
            },
            TeamAgentConfig {
                agent_id: "agent2".to_string(),
                role: "role".to_string(),
                capabilities: vec![],
            },
        ];

        let router = Router::new(config, team_agents, manager);

        let result = router.process(Message::user_text("Start")).await;

        match result {
            Err(TeamError::MaxHopsExceeded(hops)) => assert_eq!(hops, 2),
            _ => panic!("Expected MaxHopsExceeded error, got {:?}", result),
        }
    }

    #[tokio::test]
    async fn test_process_loop_detection() {
        let config = RouterConfig {
            default_agent_id: "user".to_string(),
            max_routing_hops: 10,
        };

        let manager = Arc::new(AgentManager::new());

        manager
            .register(Arc::new(MockAgent {
                info: create_mock_agent_info("user", true),
                response: Message::agent_text("I am user agent"),
            }))
            .await
            .unwrap();

        let team_agents = vec![TeamAgentConfig {
            agent_id: "user".to_string(),
            role: "role".to_string(),
            capabilities: vec![],
        }];

        let router = Router::new(config, team_agents, manager);

        let result = router.process(Message::user_text("Start")).await;

        match result {
            Err(TeamError::Protocol(msg)) => {
                assert!(msg.contains("Cannot route message back to sender: user"))
            }
            _ => panic!(
                "Expected Protocol error for loop detection, got {:?}",
                result
            ),
        }
    }

    #[tokio::test]
    async fn test_process_handoffs() {
        let config = RouterConfig {
            default_agent_id: "agent1".to_string(),
            max_routing_hops: 10,
        };

        let manager = Arc::new(AgentManager::new());

        // Agent 1 provides handoffs and routes to Agent 2
        let mut agent1_resp = Message::agent_text("To Agent 2 with handoffs");
        agent1_resp
            .set_extension(ClientRoutingExtensionData {
                recipient: Some(Participant::agent("agent2")),
                handoffs: Some(vec!["agent3".to_string()]),
                ..Default::default()
            })
            .unwrap();

        manager
            .register(Arc::new(MockAgent {
                info: create_mock_agent_info("agent1", true),
                response: agent1_resp,
            }))
            .await
            .unwrap();

        struct RecordingMockAgent {
            info: AgentInfo,
            response: Message,
            last_received: Arc<RwLock<Option<Message>>>,
        }

        #[async_trait]
        impl Agent for RecordingMockAgent {
            async fn process(&self, message: Message) -> a2a_protocol::prelude::A2aResult<Message> {
                let mut last = self.last_received.write().await;
                *last = Some(message);
                Ok(self.response.clone())
            }
            async fn info(&self) -> a2a_protocol::prelude::A2aResult<AgentInfo> {
                Ok(self.info.clone())
            }
        }

        let last_received_by_agent2 = Arc::new(RwLock::new(None));

        // Agent 2 returns User to end
        let mut agent2_resp = Message::agent_text("Final");
        agent2_resp
            .set_extension(ClientRoutingExtensionData {
                recipient: Some(Participant::User),
                ..Default::default()
            })
            .unwrap();

        manager
            .register(Arc::new(RecordingMockAgent {
                info: create_mock_agent_info("agent2", true),
                response: agent2_resp,
                last_received: last_received_by_agent2.clone(),
            }))
            .await
            .unwrap();

        manager
            .register(Arc::new(MockAgent {
                info: create_mock_agent_info("agent3", true),
                response: Message::agent_text("Agent 3"),
            }))
            .await
            .unwrap();

        let team_agents = vec![
            TeamAgentConfig {
                agent_id: "agent1".to_string(),
                role: "role".to_string(),
                capabilities: vec![],
            },
            TeamAgentConfig {
                agent_id: "agent2".to_string(),
                role: "role".to_string(),
                capabilities: vec![],
            },
            TeamAgentConfig {
                agent_id: "agent3".to_string(),
                role: "role".to_string(),
                capabilities: vec![],
            },
        ];

        let router = Router::new(config, team_agents, manager);

        router.process(Message::user_text("Start")).await.unwrap();

        let received = last_received_by_agent2.read().await;
        let msg = received.as_ref().unwrap();
        let ext_data: ClientRoutingExtensionData = msg.get_extension().unwrap().unwrap();

        let cards = ext_data.agent_cards.unwrap();
        assert_eq!(cards.len(), 1);
        assert_eq!(cards[0].id, "agent3");
    }

    #[tokio::test]
    async fn test_process_default_routing_logic() {
        let manager = Arc::new(AgentManager::new());

        // 1. Basic agent without extension support -> Participant::User (End)
        let config1 = RouterConfig {
            default_agent_id: "basic".to_string(),
            max_routing_hops: 10,
        };
        manager
            .register(Arc::new(MockAgent {
                info: create_mock_agent_info("basic", false),
                response: Message::agent_text("Basic response"),
            }))
            .await
            .unwrap();

        let router1 = Router::new(
            config1,
            vec![TeamAgentConfig {
                agent_id: "basic".to_string(),
                role: "role".to_string(),
                capabilities: vec![],
            }],
            manager.clone(),
        );

        let resp1 = router1.process(Message::user_text("Start")).await.unwrap();
        assert_eq!(resp1.text_content(), Some("Basic response"));

        // 2. Default agent with extension but no routing decision -> Participant::User (End)
        let config2 = RouterConfig {
            default_agent_id: "default_ext".to_string(),
            max_routing_hops: 10,
        };
        manager
            .register(Arc::new(MockAgent {
                info: create_mock_agent_info("default_ext", true),
                response: Message::agent_text("Default ext response"),
            }))
            .await
            .unwrap();

        let router2 = Router::new(
            config2,
            vec![TeamAgentConfig {
                agent_id: "default_ext".to_string(),
                role: "role".to_string(),
                capabilities: vec![],
            }],
            manager.clone(),
        );

        let resp2 = router2.process(Message::user_text("Start")).await.unwrap();
        assert_eq!(resp2.text_content(), Some("Default ext response"));

        // 3. Non-default agent with no decision -> route to default
        let config3 = RouterConfig {
            default_agent_id: "default".to_string(),
            max_routing_hops: 10,
        };

        // Agent 1 (non-default)
        manager
            .register(Arc::new(MockAgent {
                info: create_mock_agent_info("agent1", true),
                response: Message::agent_text("Agent 1 response"),
            }))
            .await
            .unwrap();

        let mut default_to_agent1 = Message::agent_text("Routing to agent 1");
        default_to_agent1
            .set_extension(ClientRoutingExtensionData {
                recipient: Some(Participant::agent("agent1")),
                ..Default::default()
            })
            .unwrap();

        struct MultiResponseMockAgent {
            info: AgentInfo,
            responses: Arc<RwLock<Vec<Message>>>,
        }

        #[async_trait]
        impl Agent for MultiResponseMockAgent {
            async fn process(&self, _message: Message) -> a2a_protocol::prelude::A2aResult<Message> {
                let mut resps = self.responses.write().await;
                if resps.is_empty() {
                    Ok(Message::agent_text("Final"))
                } else {
                    Ok(resps.remove(0))
                }
            }
            async fn info(&self) -> a2a_protocol::prelude::A2aResult<AgentInfo> {
                Ok(self.info.clone())
            }
        }

        let default_responses = Arc::new(RwLock::new(vec![
            default_to_agent1,
            Message::agent_text("Default final response"),
        ]));

        manager
            .register(Arc::new(MultiResponseMockAgent {
                info: create_mock_agent_info("default", true),
                responses: default_responses,
            }))
            .await
            .unwrap();

        let router3 = Router::new(
            config3,
            vec![
                TeamAgentConfig {
                    agent_id: "agent1".to_string(),
                    role: "role".to_string(),
                    capabilities: vec![],
                },
                TeamAgentConfig {
                    agent_id: "default".to_string(),
                    role: "role".to_string(),
                    capabilities: vec![],
                },
            ],
            manager.clone(),
        );

        let resp3 = router3.process(Message::user_text("Start")).await.unwrap();
        assert_eq!(resp3.text_content(), Some("Default final response"));
    }

    #[tokio::test]
    async fn test_process_filters_target_agent() {
        let config = RouterConfig {
            default_agent_id: "agent1".to_string(),
            max_routing_hops: 10,
        };

        let manager = Arc::new(AgentManager::new());

        struct RecordingMockAgent {
            info: AgentInfo,
            response: Message,
            last_received: Arc<RwLock<Option<Message>>>,
        }

        #[async_trait]
        impl Agent for RecordingMockAgent {
            async fn process(&self, message: Message) -> a2a_protocol::prelude::A2aResult<Message> {
                let mut last = self.last_received.write().await;
                *last = Some(message);
                Ok(self.response.clone())
            }
            async fn info(&self) -> a2a_protocol::prelude::A2aResult<AgentInfo> {
                Ok(self.info.clone())
            }
        }

        let last_received = Arc::new(RwLock::new(None));
        let mut response = Message::agent_text("Final");
        response
            .set_extension(ClientRoutingExtensionData {
                recipient: Some(Participant::User),
                ..Default::default()
            })
            .unwrap();

        manager
            .register(Arc::new(RecordingMockAgent {
                info: create_mock_agent_info("agent1", true),
                response,
                last_received: last_received.clone(),
            }))
            .await
            .unwrap();

        // Register another agent so there's something to see in agent_cards
        manager
            .register(Arc::new(MockAgent {
                info: create_mock_agent_info("agent2", true),
                response: Message::agent_text("Agent 2"),
            }))
            .await
            .unwrap();

        let team_agents = vec![
            TeamAgentConfig {
                agent_id: "agent1".to_string(),
                role: "role".to_string(),
                capabilities: vec![],
            },
            TeamAgentConfig {
                agent_id: "agent2".to_string(),
                role: "role".to_string(),
                capabilities: vec![],
            },
        ];

        let router = Router::new(config, team_agents, manager);

        router.process(Message::user_text("Start")).await.unwrap();

        let received = last_received.read().await;
        let msg = received.as_ref().unwrap();
        let ext_data: ClientRoutingExtensionData = msg.get_extension().unwrap().unwrap();

        let cards = ext_data.agent_cards.unwrap();
        // Should only contain agent2, not agent1
        assert_eq!(cards.len(), 1);
        assert_eq!(cards[0].id, "agent2");
        assert!(!cards.iter().any(|c| c.id == "agent1"));
    }
}
