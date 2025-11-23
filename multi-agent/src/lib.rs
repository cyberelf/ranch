//! Multi-Agent Framework
//!
//! A framework for orchestrating multiple agents using the A2A (Agent-to-Agent) protocol
//! and OpenAI-compatible APIs. Supports both local CLI interaction and agent management.

// Multi-agent framework modules
pub mod adapters;
pub mod agent;
pub mod config;
pub mod manager;
pub mod team;

// Re-export commonly used types from a2a-protocol
pub use a2a_protocol::prelude::*;

// Re-export our adapters for convenience
pub use adapters::{agent_message, extract_text, join_text, user_message};

// Re-export multi-agent specific types
pub use agent::{Agent, AgentInfo, A2AAgent, A2AAgentConfig, MultiAgentError, MultiAgentResult, OpenAIAgent, OpenAIAgentConfig, TaskHandling};
pub use config::{AgentConfig, Config, ProtocolType};
pub use manager::AgentManager;
pub use team::{SchedulerConfig, Team, TeamConfig};