//! Core A2A protocol types and definitions

pub mod error;
pub mod message;
pub mod task;
pub mod agent_card;
pub mod agent_id;
pub mod message_id;
pub mod requests;

// Re-export core types
pub use error::{A2aError, A2aResult};
pub use message::{
    Message, MessageRole, Part, TextPart, FilePart, DataPart, 
    File, FileWithBytes, FileWithUri
};
pub use task::{Task, TaskState, TaskStatus, Artifact, SendResponse};
pub use agent_card::{
    AgentCard, AgentCapability, AgentSkill, TransportInterface,
};
pub use agent_id::AgentId;
pub use message_id::MessageId;
pub use requests::{
    MessageSendRequest, TaskGetRequest, TaskCancelRequest, TaskStatusRequest,
    AgentCardGetRequest,
};

// Backwards compatibility alias
#[deprecated(note = "Use SendResponse instead - message/send returns Task or Message per A2A spec")]
pub type MessageResponse = SendResponse;