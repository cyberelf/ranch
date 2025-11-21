//! Multi-Agent Framework
//!
//! A framework for orchestrating multiple agents using the A2A (Agent-to-Agent) protocol.
//! Supports team-based coordination with supervisor and workflow scheduling modes.

// Multi-agent framework modules
pub mod adapters;
pub mod agent;
pub mod config;
pub mod manager;
pub mod remote_agent;
pub mod team;
pub mod server;

// Re-export commonly used types from a2a-protocol
pub use a2a_protocol::prelude::*;

// Re-export our adapters for convenience
pub use adapters::{agent_message, extract_text, join_text, user_message};

// Re-export multi-agent specific types
pub use agent::{Agent, AgentInfo};
pub use config::Config;
pub use manager::AgentManager;
pub use remote_agent::{RemoteAgent, RemoteAgentConfig, TaskHandling};
pub use server::TeamServer;
pub use team::{SchedulerConfig, Team, TeamConfig};