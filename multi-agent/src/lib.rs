//! Multi-Agent Framework
//!
//! A framework for orchestrating multiple agents using the A2A (Agent-to-Agent) protocol
//! and OpenAI-compatible APIs. Supports both local CLI interaction and agent management.

// Multi-agent framework modules
pub mod adapters;
pub mod agent;
pub mod config;
pub mod manager;
pub mod server;
pub mod team;

// Re-export commonly used types from a2a-protocol
pub use a2a_protocol::prelude::*;

// Re-export our adapters for convenience
pub use adapters::{agent_message, extract_text, join_text, user_message};

// Re-export multi-agent specific types
pub use agent::{
    A2AAgent, A2AAgentConfig, Agent, AgentInfo, MultiAgentError, MultiAgentResult, OpenAIAgent,
    OpenAIAgentConfig, TaskHandling,
};
pub use config::{AgentConfig, Config, ConfigConversionError, ProtocolType};
pub use manager::AgentManager;
pub use server::TeamServer;
pub use team::{track_team_nesting, CycleError, SchedulerConfig, Team, TeamConfig};

/// Create a complete multi-agent system with agents and team from configuration
///
/// This convenience function:
/// 1. Creates an AgentManager
/// 2. Registers all agents from the config
/// 3. Creates the specified team
///
/// This is a standalone function to maintain proper separation of concerns:
/// - `Config` remains a pure data structure
/// - Runtime instantiation logic lives at the module level
///
/// # Arguments
/// * `config` - Configuration containing agent and team definitions
/// * `team_id` - The ID of the team to create
///
/// # Returns
/// Tuple of (AgentManager, Team) wrapped in Arc for shared ownership
///
/// # Example
/// ```no_run
/// use multi_agent::*;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let config = Config::from_file("config.toml")?;
/// let (agent_manager, team) = create_team_from_config(&config, "my-team").await?;
///
/// // Use the team
/// let response = team.process_message(Message::user_text("Hello!")).await?;
/// # Ok(())
/// # }
/// ```
pub async fn create_team_from_config(
    config: &Config,
    team_id: &str,
) -> Result<(std::sync::Arc<AgentManager>, std::sync::Arc<Team>), Box<dyn std::error::Error>> {
    use std::sync::Arc;

    // Create agent manager and register all agents
    let agent_manager = Arc::new(AgentManager::new());
    agent_manager.register_from_config(config).await?;

    // Create the team
    let team = Arc::new(Team::from_config(config, team_id, agent_manager.clone())?);

    Ok((agent_manager, team))
}
