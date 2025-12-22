//! Team module for multi-agent coordination
//!
//! This module provides the Team abstraction for coordinating multiple agents
//! using the Router component for dynamic message routing.

pub mod router;
pub mod types;

// Re-export main types
pub use router::Router;
pub use types::{
    ClientRoutingRequest, ClientRoutingResponse, Recipient, RouterConfig, SimplifiedAgentCard,
    TeamError, EXTENSION_DESCRIPTION, EXTENSION_NAME, EXTENSION_URI, EXTENSION_VERSION,
};

use crate::manager::AgentManager;
use crate::{Agent, AgentInfo};
use a2a_protocol::prelude::{AgentSkill, Message};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

/// Cycle detection for nested teams
#[derive(Debug, thiserror::Error)]
#[error("Cycle detected in team nesting: {0}")]
pub struct CycleError(pub String);

/// Check if a team can be registered without creating a cycle
///
/// # Arguments
/// * `team_id` - ID of the team being registered
/// * `visited` - Set of already visited team IDs in the current path
///
/// # Returns
/// * `Ok(())` if no cycle detected
/// * `Err(CycleError)` if a cycle would be created
pub fn track_team_nesting(team_id: &str, visited: &mut HashSet<String>) -> Result<(), CycleError> {
    if visited.contains(team_id) {
        return Err(CycleError(format!(
            "Team '{}' would create a cycle (path: {})",
            team_id,
            visited.iter().cloned().collect::<Vec<_>>().join(" -> ")
        )));
    }
    visited.insert(team_id.to_string());
    Ok(())
}

/// Team configuration
///
/// Defines team structure including member agents and routing configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamConfig {
    /// Unique team identifier
    pub id: String,
    /// Human-readable team name
    pub name: String,
    /// Brief description of team purpose
    pub description: String,
    /// List of agents in the team
    pub agents: Vec<TeamAgentConfig>,
    /// Router configuration for message routing
    pub router_config: RouterConfig,
}

/// Agent configuration within a team
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamAgentConfig {
    /// Agent identifier (must match registered agent)
    pub agent_id: String,
    /// Role of the agent within the team
    pub role: String,
    /// Specific capabilities this agent provides to the team
    pub capabilities: Vec<String>,
}

/// Team for coordinating multiple agents
///
/// A Team manages a group of agents and routes messages between them using
/// the Router component. The Router enables dynamic, metadata-driven routing
/// with support for the Client Agent Extension.
pub struct Team {
    config: TeamConfig,
    agent_manager: Arc<AgentManager>,
}

impl Team {
    /// Create a new team from configuration
    ///
    /// # Arguments
    /// * `config` - Team configuration including agents and router config
    /// * `agent_manager` - Agent manager with registered agents
    ///
    /// # Example
    /// ```no_run
    /// use multi_agent::team::{Team, TeamConfig, RouterConfig};
    /// use multi_agent::AgentManager;
    /// use std::sync::Arc;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = TeamConfig {
    ///     id: "team-1".to_string(),
    ///     name: "My Team".to_string(),
    ///     description: "A dynamic team".to_string(),
    ///     agents: vec![],
    ///     router_config: RouterConfig {
    ///         default_agent_id: "agent-1".to_string(),
    ///         max_routing_hops: 10,
    ///     },
    /// };
    ///
    /// let agent_manager = Arc::new(AgentManager::new());
    /// let team = Team::new(config, agent_manager);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(config: TeamConfig, agent_manager: Arc<AgentManager>) -> Self {
        Self {
            config,
            agent_manager,
        }
    }

    /// Create a team from a team ID in the config, using pre-registered agents
    ///
    /// # Arguments
    /// * `config` - The full configuration containing team definitions
    /// * `team_id` - The ID of the team to create
    /// * `agent_manager` - Agent manager with pre-registered agents
    ///
    /// # Returns
    /// * `Ok(Team)` if team found in config
    /// * `Err` if team not found
    ///
    /// # Example
    /// ```no_run
    /// use multi_agent::*;
    /// use std::sync::Arc;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = Config::from_file("config.toml")?;
    /// let agent_manager = Arc::new(AgentManager::new());
    ///
    /// // Register agents...
    ///
    /// let team = Team::from_config(&config, "my-team-id", agent_manager)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_config(
        config: &crate::Config,
        team_id: &str,
        agent_manager: Arc<AgentManager>,
    ) -> Result<Self, TeamError> {
        let team_configs = config.to_team_configs();
        let team_config = team_configs
            .iter()
            .find(|team| team.id == team_id)
            .ok_or_else(|| {
                TeamError::Configuration(format!("Team '{}' not found in configuration", team_id))
            })?
            .clone();

        Ok(Self::new(team_config, agent_manager))
    }

    /// Process a single message through the team
    ///
    /// Convenience method for processing a single message.
    ///
    /// # Arguments
    /// * `message` - Message to process
    ///
    /// # Returns
    /// Final response message after routing through team
    pub async fn process_message(&self, message: Message) -> Result<Message, TeamError> {
        self.process_messages(vec![message]).await
    }

    /// Process messages through the team using Router-based routing
    ///
    /// Routes messages between agents based on extension decisions or fallback logic.
    /// Continues until a User recipient is returned or max hops is reached.
    ///
    /// # Arguments
    /// * `initial_messages` - Initial messages to process
    ///
    /// # Returns
    /// Final response message after routing through team
    pub async fn process_messages(
        &self,
        initial_messages: Vec<Message>,
    ) -> Result<Message, TeamError> {
        let mut router = Router::new(self.config.router_config.clone());
        let mut current_message = initial_messages
            .last()
            .ok_or_else(|| TeamError::RouterError("No messages to process".to_string()))?
            .clone();

        // Get all agents in the team
        let mut agents: HashMap<String, Arc<dyn Agent>> = HashMap::new();
        for agent_config in &self.config.agents {
            if let Some(agent) = self.agent_manager.get(&agent_config.agent_id).await {
                agents.insert(agent_config.agent_id.clone(), agent);
            }
        }

        // Initial sender is "user"
        let mut sender = "user".to_string();

        loop {
            // Route message
            let recipient = router
                .route(&mut current_message, &agents, &sender)
                .await?;

            match recipient {
                Recipient::User => {
                    // Return to user - end routing
                    return Ok(current_message);
                }
                Recipient::Agent { agent_id } => {
                    // Continue routing to next agent
                    sender = agent_id;
                }
            }
        }
    }

    /// Get team configuration
    pub fn config(&self) -> &TeamConfig {
        &self.config
    }

    /// Get team ID
    pub fn id(&self) -> &str {
        &self.config.id
    }

    /// Get team name
    pub fn name(&self) -> &str {
        &self.config.name
    }
}

// Implement Agent trait for Team to enable teams as agents
#[async_trait]
impl Agent for Team {
    /// Get agent information for the team
    ///
    /// Returns aggregated skills from all member agents
    async fn info(&self) -> a2a_protocol::prelude::A2aResult<AgentInfo> {
        let mut skills = Vec::new();

        // Aggregate skills from all member agents
        for agent_config in &self.config.agents {
            if let Some(agent) = self.agent_manager.get(&agent_config.agent_id).await {
                match agent.info().await {
                    Ok(info) => {
                        skills.extend(info.skills);
                    }
                    Err(e) => {
                        // Log warning but continue - graceful degradation
                        eprintln!(
                            "Warning: Failed to get info from agent {}: {}",
                            agent_config.agent_id, e
                        );
                    }
                }
            }
        }

        // Add team-specific skills from config
        for agent_config in &self.config.agents {
            for skill_name in &agent_config.capabilities {
                skills.push(AgentSkill {
                    name: skill_name.clone(),
                    description: None,
                    category: None,
                    tags: vec![],
                    examples: vec![],
                });
            }
        }

        let mut metadata = HashMap::new();
        metadata.insert("type".to_string(), "team".to_string());
        metadata.insert(
            "router_default_agent".to_string(),
            self.config.router_config.default_agent_id.clone(),
        );
        metadata.insert(
            "member_count".to_string(),
            self.config.agents.len().to_string(),
        );

        Ok(AgentInfo {
            id: self.config.id.clone(),
            name: self.config.name.clone(),
            description: self.config.description.clone(),
            skills,
            metadata,
        })
    }

    /// Process a message through the team's orchestration
    ///
    /// Delegates to router which routes to appropriate member agents
    async fn process(&self, message: Message) -> a2a_protocol::prelude::A2aResult<Message> {
        self.process_message(message)
            .await
            .map_err(|e| a2a_protocol::prelude::A2aError::Internal(e.to_string()))
    }

    /// Check if team is healthy - all member agents responsive
    async fn health_check(&self) -> bool {
        // Check if at least one agent is healthy
        for agent_config in &self.config.agents {
            if let Some(agent) = self.agent_manager.get(&agent_config.agent_id).await {
                if agent.health_check().await {
                    return true;
                }
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_track_team_nesting_no_cycle() {
        let mut visited = HashSet::new();
        assert!(track_team_nesting("team1", &mut visited).is_ok());
        assert!(visited.contains("team1"));
    }

    #[test]
    fn test_track_team_nesting_with_cycle() {
        let mut visited = HashSet::new();
        visited.insert("team1".to_string());

        let result = track_team_nesting("team1", &mut visited);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cycle"));
    }

    #[test]
    fn test_team_config_serialization() {
        let config = TeamConfig {
            id: "team-1".to_string(),
            name: "Test Team".to_string(),
            description: "A test team".to_string(),
            agents: vec![TeamAgentConfig {
                agent_id: "agent-1".to_string(),
                role: "worker".to_string(),
                capabilities: vec!["test".to_string()],
            }],
            router_config: RouterConfig {
                default_agent_id: "agent-1".to_string(),
                max_routing_hops: 5,
            },
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: TeamConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.id, deserialized.id);
        assert_eq!(config.name, deserialized.name);
        assert_eq!(config.agents.len(), deserialized.agents.len());
    }
}
