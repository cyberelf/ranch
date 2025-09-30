//! Core A2A protocol types and definitions

pub mod error;
pub mod message;
pub mod agent_card;
pub mod agent_id;
pub mod message_id;

// Re-export core types
pub use error::{A2aError, A2aResult};
pub use message::{Message, MessageResponse, MessagePart};
pub use agent_card::{AgentCard, AgentCapability, AgentSkill};
pub use agent_id::AgentId;
pub use message_id::MessageId;