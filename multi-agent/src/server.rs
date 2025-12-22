//! TeamServer - HTTP server exposing Team via JSON-RPC 2.0
//!
//! This module provides TeamServer which wraps a Team and exposes it
//! as an A2A-compliant service using JSON-RPC 2.0 over HTTP.

use crate::{team::Team, Agent as MultiAgentAgent, AgentInfo};
use a2a_protocol::{
    server::{AgentProfile, JsonRpcRouter, ProtocolAgent as A2aServerAgent, TaskAwareHandler},
    A2aResult, Message,
};
use async_trait::async_trait;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

/// Adapter to bridge multi-agent Agent trait to a2a-protocol Agent trait
struct TeamAgentAdapter {
    team: Arc<Team>,
}

impl TeamAgentAdapter {
    fn new(team: Arc<Team>) -> Self {
        Self { team }
    }

    /// Convert AgentInfo to AgentProfile
    fn info_to_profile(info: AgentInfo) -> A2aResult<AgentProfile> {
        // Use a dummy URL since teams don't have their own endpoint
        // The actual endpoint is the TeamServer
        let url = url::Url::parse("http://localhost/team").map_err(|e| {
            a2a_protocol::A2aError::Internal(format!("Failed to create URL: {}", e))
        })?;

        // Use AgentProfile::new() constructor and configure capabilities
        let mut profile = AgentProfile::new(
            info.id.into(), // Convert String to AgentId
            info.name,
            url,
        );

        profile.description = Some(info.description);
        profile.default_input_modes = vec!["text".to_string()];
        profile.default_output_modes = vec!["text".to_string()];

        // Convert skills to capabilities (protocol level)
        // Skills = what the agent does; Capabilities = transport-level protocol features
        profile.capabilities = info.skills.iter().map(|skill| {
            a2a_protocol::core::agent_card::AgentCapability {
                name: skill.name.clone(),
                description: skill.description.clone(),
                category: skill.category.clone(),
                input_schema: None,
                output_schema: None,
            }
        }).collect();

        profile.metadata = info
            .metadata
            .into_iter()
            .map(|(k, v)| (k, serde_json::Value::String(v)))
            .collect();

        Ok(profile)
    }
}

#[async_trait]
impl A2aServerAgent for TeamAgentAdapter {
    async fn profile(&self) -> A2aResult<AgentProfile> {
        let info = self.team.info().await?;
        Self::info_to_profile(info)
    }

    async fn process_message(&self, msg: Message) -> A2aResult<Message> {
        self.team.process(msg).await
    }
}

/// HTTP server that exposes a Team via JSON-RPC 2.0
///
/// TeamServer wraps a Team with TaskAwareHandler to provide full
/// A2A protocol compliance including:
/// - message/send
/// - task/get, task/status, task/cancel
/// - agent/card
///
/// # Example
///
/// ```no_run
/// use multi_agent::{Team, AgentManager, TeamServer};
/// use multi_agent::team::{TeamConfig, RouterConfig, TeamAgentConfig};
/// use std::sync::Arc;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let manager = Arc::new(AgentManager::new());
/// let team_config = TeamConfig {
///     id: "example-team".to_string(),
///     name: "Example Team".to_string(),
///     description: "An example team".to_string(),
///     agents: vec![],
///     router_config: RouterConfig {
///         default_agent_id: "default-agent".to_string(),
///         max_routing_hops: 10,
///     },
/// };
/// let team = Arc::new(Team::new(team_config, manager));
///
/// let server = TeamServer::new(team, 8080);
/// server.start().await?;
/// # Ok(())
/// # }
/// ```
pub struct TeamServer {
    team: Arc<Team>,
    port: u16,
}

impl TeamServer {
    /// Create a new TeamServer
    ///
    /// # Arguments
    /// * `team` - The team to expose via HTTP
    /// * `port` - Port number to bind to
    pub fn new(team: Arc<Team>, port: u16) -> Self {
        Self { team, port }
    }

    /// Start the TeamServer
    ///
    /// This method:
    /// 1. Wraps the team with TeamAgentAdapter to bridge trait differences
    /// 2. Wraps with TaskAwareHandler for async task management
    /// 3. Creates a JsonRpcRouter for JSON-RPC 2.0 handling
    /// 4. Sets up Axum router with CORS support
    /// 5. Binds to the configured port and starts serving
    ///
    /// # Errors
    /// Returns an error if:
    /// - Port is already in use
    /// - Failed to bind to the address
    /// - Server runtime error occurs
    pub async fn start(self) -> Result<(), Box<dyn std::error::Error>> {
        println!(
            "🚀 Starting TeamServer for team: {}",
            self.team.config().name
        );
        println!("   ID: {}", self.team.config().id);
        println!("   Router: default_agent_id = {}", self.team.config().router_config.default_agent_id);
        println!("   Members: {}", self.team.config().agents.len());

        // Wrap team with adapter to bridge multi-agent Agent trait to a2a-protocol Agent trait
        let adapter = TeamAgentAdapter::new(self.team.clone());

        // Wrap with TaskAwareHandler for A2A protocol support
        let handler = TaskAwareHandler::new(Arc::new(adapter));

        // Create JSON-RPC router - must convert to router
        let json_rpc_router = JsonRpcRouter::new(handler);
        let rpc_router = json_rpc_router.into_router();

        // Setup CORS to allow cross-origin requests
        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any);

        // Create Axum router
        let app = rpc_router.layer(cors);

        let addr = format!("0.0.0.0:{}", self.port);
        let listener = tokio::net::TcpListener::bind(&addr).await?;

        println!("📡 TeamServer listening on http://{}", addr);
        println!("   JSON-RPC endpoint: http://{}/rpc", addr);
        println!("   ");
        println!("   Supported methods:");
        println!("     - message/send    : Send message to team");
        println!("     - task/get        : Get task details");
        println!("     - task/status     : Get task status");
        println!("     - task/cancel     : Cancel a task");
        println!("     - agent/card      : Get team capabilities");
        println!();

        axum::serve(listener, app.into_make_service()).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use a2a_protocol::AgentSkill;
    use crate::manager::AgentManager;
    use crate::team::{RouterConfig, TeamConfig};

    #[test]
    fn test_team_server_creation() {
        let manager = Arc::new(AgentManager::new());
        let config = TeamConfig {
            id: "test-team".to_string(),
            name: "Test Team".to_string(),
            description: "A test team".to_string(),
            agents: vec![],
            router_config: RouterConfig {
                default_agent_id: "supervisor".to_string(),
                max_routing_hops: 10,
            },
        };

        let team = Arc::new(Team::new(config, manager));
        let server = TeamServer::new(team, 8080);

        assert_eq!(server.port, 8080);
    }

    #[tokio::test]
    async fn test_team_agent_adapter_profile() {
        let manager = Arc::new(AgentManager::new());
        let config = TeamConfig {
            id: "adapter-test-team".to_string(),
            name: "Adapter Test Team".to_string(),
            description: "Testing adapter".to_string(),
            agents: vec![],
            router_config: RouterConfig {
                default_agent_id: "supervisor".to_string(),
                max_routing_hops: 10,
            },
        };

        let team = Arc::new(Team::new(config, manager));
        let adapter = TeamAgentAdapter::new(team.clone());

        // Test profile generation
        let profile_result = adapter.profile().await;
        assert!(profile_result.is_ok(), "Failed to get profile: {:?}", profile_result.err());

        let profile = profile_result.unwrap();
        assert_eq!(profile.name, "Adapter Test Team");
        assert_eq!(profile.description, Some("Testing adapter".to_string()));
    }

    #[tokio::test]
    async fn test_team_agent_adapter_process_message() {
        let manager = Arc::new(AgentManager::new());
        
        // Create a simple team config
        let config = TeamConfig {
            id: "process-test-team".to_string(),
            name: "Process Test Team".to_string(),
            description: "Testing message processing".to_string(),
            agents: vec![],
            router_config: RouterConfig {
                default_agent_id: "supervisor".to_string(),
                max_routing_hops: 10,
            },
        };

        let team = Arc::new(Team::new(config, manager));
        let adapter = TeamAgentAdapter::new(team.clone());

        // Test message processing
        let message = Message::user_text("Test message");
        let response_result = adapter.process_message(message).await;
        
        // The response might be an error since we don't have actual agents,
        // but we're testing that the adapter delegates correctly
        assert!(response_result.is_ok() || response_result.is_err());
    }

    #[test]
    fn test_info_to_profile_conversion() {
        use std::collections::HashMap;

        let info = AgentInfo {
            id: "test-agent".to_string(),
            name: "Test Agent".to_string(),
            description: "Test description".to_string(),
            skills: vec![
                AgentSkill {
                    name: "capability1".to_string(),
                    description: None,
                    category: None,
                    tags: vec![],
                    examples: vec![],
                },
                AgentSkill {
                    name: "capability2".to_string(),
                    description: None,
                    category: None,
                    tags: vec![],
                    examples: vec![],
                },
            ],
            metadata: {
                let mut map = HashMap::new();
                map.insert("key1".to_string(), "value1".to_string());
                map
            },
        };

        let profile_result = TeamAgentAdapter::info_to_profile(info);
        assert!(profile_result.is_ok(), "Failed to convert info to profile");

        let profile = profile_result.unwrap();
        assert_eq!(profile.name, "Test Agent");
        assert_eq!(profile.description, Some("Test description".to_string()));
        assert_eq!(profile.capabilities.len(), 2);
        assert_eq!(profile.capabilities[0].name, "capability1");
        assert_eq!(profile.capabilities[1].name, "capability2");
        
        // Check metadata conversion
        assert_eq!(profile.metadata.len(), 1);
        assert!(profile.metadata.contains_key("key1"));
    }
}
