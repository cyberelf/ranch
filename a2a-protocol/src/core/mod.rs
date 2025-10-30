//! Core A2A protocol types and definitions

pub mod agent_card;
pub mod agent_id;
pub mod error;
pub mod message;
pub mod message_id;
pub mod requests;
pub mod streaming_events;
pub mod task;

// Re-export core types
pub use agent_card::{AgentCapability, AgentCard, AgentSkill, StreamingCapabilities, TransportInterface};
pub use agent_id::AgentId;
pub use error::{A2aError, A2aResult};
pub use message::{
    DataPart, File, FilePart, FileWithBytes, FileWithUri, Message, MessageRole, Part, TextPart,
};
pub use message_id::MessageId;
pub use requests::{
    AgentCardGetRequest, MessageSendRequest, TaskCancelRequest, TaskGetRequest,
    TaskResubscribeRequest, TaskStatusRequest,
};
pub use streaming_events::{TaskArtifactUpdateEvent, TaskProgress, TaskStatusUpdateEvent};
pub use task::{Artifact, SendResponse, Task, TaskState, TaskStatus};

// Backwards compatibility alias
#[deprecated(note = "Use SendResponse instead - message/send returns Task or Message per A2A spec")]
pub type MessageResponse = SendResponse;
