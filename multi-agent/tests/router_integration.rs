//! Integration tests for the Router component
//!
//! Tests dynamic message routing with the Client Agent Extension

use multi_agent::team::{
    ClientRoutingResponse, Recipient, RouterConfig, SimplifiedAgentCard, Team, TeamAgentConfig,
    TeamConfig, EXTENSION_URI,
};
use multi_agent::{extract_text, Agent, AgentInfo, AgentManager};
use a2a_protocol::prelude::*;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

/// Mock agent that supports the Client Agent Extension
struct ExtensionCapableAgent {
    id: String,
    name: String,
    routing_target: Option<String>, // If Some, routes to this agent; if None, routes to user
}

#[async_trait]
impl Agent for ExtensionCapableAgent {
    async fn info(&self) -> A2aResult<AgentInfo> {
        Ok(AgentInfo {
            id: self.id.clone(),
            name: self.name.clone(),
            description: format!("Extension-capable agent: {}", self.name),
            skills: vec![AgentSkill {
                name: EXTENSION_URI.to_string(),
                description: None,
                category: None,
                tags: vec![],
                examples: vec![],
            }],
            metadata: HashMap::new(),
        })
    }

    async fn process(&self, message: Message) -> A2aResult<Message> {
        // Check if extension data is present
        if let Some(metadata) = &message.metadata {
            if metadata.contains_key(EXTENSION_URI) {
                // Agent received extension data - make routing decision
                let routing_response = if let Some(target) = &self.routing_target {
                    ClientRoutingResponse {
                        recipient: target.clone(),
                        reason: Some(format!("Routing to {}", target)),
                        handoffs: None,
                    }
                } else {
                    ClientRoutingResponse {
                        recipient: "user".to_string(),
                        reason: Some("Task complete, returning to user".to_string()),
                        handoffs: None,
                    }
                };

                let mut response = Message::agent_text(&format!(
                    "{} processed message and routing to: {}",
                    self.name, routing_response.recipient
                ));

                // Add routing decision to metadata
                let mut response_metadata = HashMap::new();
                response_metadata.insert(
                    EXTENSION_URI.to_string(),
                    serde_json::to_value(&routing_response).unwrap(),
                );
                response.metadata = Some(response_metadata);

                return Ok(response);
            }
        }

        // No extension data - just process normally
        Ok(Message::agent_text(&format!(
            "{} processed message (no extension)",
            self.name
        )))
    }
}

/// Mock agent that does NOT support the Client Agent Extension
struct BasicAgent {
    id: String,
    name: String,
}

#[async_trait]
impl Agent for BasicAgent {
    async fn info(&self) -> A2aResult<AgentInfo> {
        Ok(AgentInfo {
            id: self.id.clone(),
            name: self.name.clone(),
            description: format!("Basic agent: {}", self.name),
            skills: vec![], // No extension support
            metadata: HashMap::new(),
        })
    }

    async fn process(&self, _message: Message) -> A2aResult<Message> {
        Ok(Message::agent_text(&format!(
            "{} processed message",
            self.name
        )))
    }
}

#[tokio::test]
async fn test_router_with_extension_capable_agent() {
    // Create agent manager and register agents
    let manager = Arc::new(AgentManager::new());

    let default_agent = Arc::new(ExtensionCapableAgent {
        id: "default".to_string(),
        name: "Default Agent".to_string(),
        routing_target: Some("worker".to_string()), // Routes to worker
    });

    let worker_agent = Arc::new(ExtensionCapableAgent {
        id: "worker".to_string(),
        name: "Worker Agent".to_string(),
        routing_target: None, // Routes to user
    });

    manager.register(default_agent).await;
    manager.register(worker_agent).await;

    // Create team with router config
    let config = TeamConfig {
        id: "test-team".to_string(),
        name: "Test Team".to_string(),
        description: "Team with extension-capable agents".to_string(),
        agents: vec![
            TeamAgentConfig {
                agent_id: "default".to_string(),
                role: "default".to_string(),
                capabilities: vec![],
            },
            TeamAgentConfig {
                agent_id: "worker".to_string(),
                role: "worker".to_string(),
                capabilities: vec![],
            },
        ],
        router_config: RouterConfig {
            default_agent_id: "default".to_string(),
            max_routing_hops: 10,
        },
    };

    let team = Team::new(config, manager);

    // Process a message through the team
    let message = Message::user_text("Test message for routing");
    let result = team.process_message(message).await;

    assert!(result.is_ok(), "Failed to process message: {:?}", result.err());
    let response = result.unwrap();

    // Should have been routed through default -> worker -> user
    let text = extract_text(&response).unwrap();
    assert!(text.contains("Worker Agent"));
}

#[tokio::test]
async fn test_router_fallback_to_default() {
    // Create agent manager with only a default agent
    let manager = Arc::new(AgentManager::new());

    let default_agent = Arc::new(ExtensionCapableAgent {
        id: "default".to_string(),
        name: "Default Agent".to_string(),
        routing_target: None, // Routes to user immediately
    });

    manager.register(default_agent).await;

    // Create team
    let config = TeamConfig {
        id: "test-team".to_string(),
        name: "Test Team".to_string(),
        description: "Team with fallback routing".to_string(),
        agents: vec![TeamAgentConfig {
            agent_id: "default".to_string(),
            role: "default".to_string(),
            capabilities: vec![],
        }],
        router_config: RouterConfig {
            default_agent_id: "default".to_string(),
            max_routing_hops: 10,
        },
    };

    let team = Team::new(config, manager);

    // Process message - should go to default and return to user
    let message = Message::user_text("Test message");
    let result = team.process_message(message).await;

    assert!(result.is_ok());
    let response = result.unwrap();
    let text = extract_text(&response).unwrap();
    assert!(text.contains("Default Agent"));
}

#[tokio::test]
async fn test_router_with_basic_agent_no_extension() {
    // Create agent manager with basic (non-extension) agent
    let manager = Arc::new(AgentManager::new());

    let basic_agent = Arc::new(BasicAgent {
        id: "basic".to_string(),
        name: "Basic Agent".to_string(),
    });

    manager.register(basic_agent).await;

    // Create team
    let config = TeamConfig {
        id: "test-team".to_string(),
        name: "Test Team".to_string(),
        description: "Team with basic agent".to_string(),
        agents: vec![TeamAgentConfig {
            agent_id: "basic".to_string(),
            role: "worker".to_string(),
            capabilities: vec![],
        }],
        router_config: RouterConfig {
            default_agent_id: "basic".to_string(),
            max_routing_hops: 10,
        },
    };

    let team = Team::new(config, manager);

    // Process message - basic agent doesn't support extension, so should return immediately
    let message = Message::user_text("Test message");
    let result = team.process_message(message).await;

    assert!(result.is_ok());
    let response = result.unwrap();
    let text = extract_text(&response).unwrap();
    assert!(text.contains("Basic Agent"));
}

#[tokio::test]
async fn test_router_max_hops_limit() {
    // Create agent that always routes to itself (infinite loop scenario)
    let manager = Arc::new(AgentManager::new());

    let loop_agent = Arc::new(ExtensionCapableAgent {
        id: "loop".to_string(),
        name: "Loop Agent".to_string(),
        routing_target: Some("loop".to_string()), // Routes to itself
    });

    manager.register(loop_agent).await;

    // Create team with low max hops
    let config = TeamConfig {
        id: "test-team".to_string(),
        name: "Test Team".to_string(),
        description: "Team to test max hops".to_string(),
        agents: vec![TeamAgentConfig {
            agent_id: "loop".to_string(),
            role: "worker".to_string(),
            capabilities: vec![],
        }],
        router_config: RouterConfig {
            default_agent_id: "loop".to_string(),
            max_routing_hops: 3, // Low limit to test
        },
    };

    let team = Team::new(config, manager);

    // Process message - should hit max hops and fail
    let message = Message::user_text("Test message");
    let result = team.process_message(message).await;

    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("Maximum routing hops exceeded"));
}

#[tokio::test]
async fn test_router_mixed_team() {
    // Create team with both extension-capable and basic agents
    let manager = Arc::new(AgentManager::new());

    let smart_agent = Arc::new(ExtensionCapableAgent {
        id: "smart".to_string(),
        name: "Smart Agent".to_string(),
        routing_target: Some("basic".to_string()), // Routes to basic agent
    });

    let basic_agent = Arc::new(BasicAgent {
        id: "basic".to_string(),
        name: "Basic Agent".to_string(),
    });

    manager.register(smart_agent).await;
    manager.register(basic_agent).await;

    // Create team
    let config = TeamConfig {
        id: "test-team".to_string(),
        name: "Test Team".to_string(),
        description: "Mixed team".to_string(),
        agents: vec![
            TeamAgentConfig {
                agent_id: "smart".to_string(),
                role: "coordinator".to_string(),
                capabilities: vec![],
            },
            TeamAgentConfig {
                agent_id: "basic".to_string(),
                role: "worker".to_string(),
                capabilities: vec![],
            },
        ],
        router_config: RouterConfig {
            default_agent_id: "smart".to_string(),
            max_routing_hops: 10,
        },
    };

    let team = Team::new(config, manager);

    // Process message - smart agent should receive extension data and route to basic
    let message = Message::user_text("Test message");
    let result = team.process_message(message).await;

    assert!(result.is_ok(), "Failed to process message: {:?}", result.err());
    let response = result.unwrap();
    let text = extract_text(&response).unwrap();
    assert!(text.contains("Basic Agent"));
}

#[tokio::test]
async fn test_team_as_agent_trait() {
    // Test that Team implements Agent trait correctly
    let manager = Arc::new(AgentManager::new());

    let agent = Arc::new(ExtensionCapableAgent {
        id: "worker".to_string(),
        name: "Worker".to_string(),
        routing_target: None,
    });

    manager.register(agent).await;

    let config = TeamConfig {
        id: "test-team".to_string(),
        name: "Test Team".to_string(),
        description: "Test team description".to_string(),
        agents: vec![TeamAgentConfig {
            agent_id: "worker".to_string(),
            role: "worker".to_string(),
            capabilities: vec!["test-capability".to_string()],
        }],
        router_config: RouterConfig {
            default_agent_id: "worker".to_string(),
            max_routing_hops: 10,
        },
    };

    let team = Team::new(config, manager);

    // Test info() method
    let info = team.info().await;
    assert!(info.is_ok());
    let info = info.unwrap();
    assert_eq!(info.id, "test-team");
    assert_eq!(info.name, "Test Team");
    assert!(info
        .skills
        .iter()
        .any(|s| s.name == "test-capability"));

    // Test process() method
    let message = Message::user_text("Test");
    let result = team.process(message).await;
    assert!(result.is_ok());

    // Test health_check() method
    let healthy = team.health_check().await;
    assert!(healthy);
}
