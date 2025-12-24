//! Multi-Agent Client Collaboration Framework
//!
//! **Important: This is a CLIENT-SIDE framework for coordinating REMOTE agents.**
//!
//! # Core Principle
//!
//! The multi-agent crate provides a **collaboration ground** for remote agents accessed via
//! the A2A (Agent-to-Agent) protocol. It is **NOT** a framework for implementing agents.
//!
//! ## What multi-agent Provides
//!
//! - **A2AAgent**: Client for connecting to remote A2A protocol servers
//! - **AgentManager**: Registry for discovering and managing remote agent connections
//! - **Team**: Coordination layer for routing messages between remote agents
//! - **Router**: Dynamic routing logic with extension support
//!
//! ## What multi-agent Does NOT Provide
//!
//! - Agent implementation framework (use `a2a-protocol` crate for that)
//! - Local agent execution
//! - Mock agents (use real A2A servers instead)
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │ multi-agent (Client-Side Coordination)                      │
//! │                                                             │
//! │  ┌──────────────┐      ┌──────────────┐                   │
//! │  │ A2AAgent     │──────│ A2AAgent     │                   │
//! │  │ (Client)     │      │ (Client)     │                   │
//! │  └──────┬───────┘      └──────┬───────┘                   │
//! │         │                     │                            │
//! │         │  ┌──────────────────┴───────────┐               │
//! │         └──┤ Team (Router & Coordination) │               │
//! │            └──────────────────────────────┘               │
//! └─────────────────────────────────────────────────────────────┘
//!                         │                │
//!                         │ A2A Protocol   │
//!                         │ (JSON-RPC)     │
//!                         ▼                ▼
//! ┌─────────────────────────────────────────────────────────────┐
//! │ a2a-protocol Servers (Agent Implementations)                │
//! │                                                             │
//! │  ┌──────────────┐      ┌──────────────┐                   │
//! │  │ Agent Server │      │ Agent Server │                   │
//! │  │ (Port 3000)  │      │ (Port 3001)  │                   │
//! │  └──────────────┘      └──────────────┘                   │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Example Usage
//!
//! ```no_run
//! use multi_agent::{A2AAgent, Agent, AgentManager, Team, TeamConfig, RouterConfig};
//! use a2a_protocol::prelude::*;
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Step 1: Connect to remote A2A agent servers
//! let transport1 = Arc::new(JsonRpcTransport::new("http://localhost:3000/rpc")?);
//! let client1 = A2aClient::new(transport1);
//! let agent1 = Arc::new(A2AAgent::new(client1));
//!
//! let transport2 = Arc::new(JsonRpcTransport::new("http://localhost:3001/rpc")?);
//! let client2 = A2aClient::new(transport2);
//! let agent2 = Arc::new(A2AAgent::new(client2));
//!
//! // Step 2: Register remote agents
//! let manager = Arc::new(AgentManager::new());
//! manager.register(agent1).await?;
//! manager.register(agent2).await?;
//!
//! // Step 3: Form team and process messages
//! let config = TeamConfig {
//!     id: "my-team".to_string(),
//!     name: "My Team".to_string(),
//!     description: "Team of remote agents".to_string(),
//!     agents: vec![/* team agent configs */],
//!     router_config: RouterConfig {
//!         default_agent_id: "agent1".to_string(),
//!         max_routing_hops: 10,
//!     },
//! };
//!
//! let team = Team::new(config, manager);
//! let response = team.process(Message::user_text("Hello")).await?;
//! # Ok(())
//! # }
//! ```
//!
//! # See Also
//!
//! - **Agent Implementation**: See `a2a-protocol` crate for implementing agents
//! - **Examples**: See `examples/agent_servers.rs` and `examples/team_client.rs`
//! - **Documentation**: See `AGENT.md` for detailed guide

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
pub use team::{
    track_team_nesting, ClientRoutingExtensionData, CycleError, Participant, Router,
    RouterConfig, SimplifiedAgentCard, Team, TeamAgentConfig,
    TeamConfig, TeamError,
};

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
