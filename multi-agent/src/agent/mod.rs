//! Agent implementations for the multi-agent framework
//!
//! This module contains the core `Agent` trait and various agent implementations
//! including A2A and OpenAI-compatible agents.

pub mod a2a_agent;
pub mod errors;
pub mod openai_agent;
pub mod traits;

pub use a2a_agent::{A2AAgent, A2AAgentConfig, TaskHandling};
pub use errors::{MultiAgentError, MultiAgentResult};
pub use openai_agent::{OpenAIAgent, OpenAIAgentConfig};
pub use traits::{Agent, AgentInfo};
