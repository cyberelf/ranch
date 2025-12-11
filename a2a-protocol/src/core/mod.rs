//! Core A2A protocol types and definitions

pub mod agent_card;
pub mod agent_id;
pub mod error;
pub mod json_rpc;
pub mod message;
pub mod message_id;
pub mod push_notification;
pub mod requests;
pub mod sse;
pub mod ssrf_protection;
pub mod streaming_events;
pub mod task;

// Re-export core types
pub use agent_card::{
    AgentCapability, AgentCard, AgentCardSignature, AgentSkill, StreamingCapabilities,
    TransportInterface,
};
pub use agent_id::AgentId;
pub use error::{A2aError, A2aResult};
pub use message::{
    DataPart, File, FilePart, FileWithBytes, FileWithUri, Message, MessageRole, Part, TextPart,
};
pub use message_id::MessageId;
pub use push_notification::{PushNotificationAuth, PushNotificationConfig, TaskEvent};
pub use requests::{
    AgentCardGetRequest, MessageSendRequest, PushNotificationConfigEntry,
    PushNotificationDeleteRequest, PushNotificationGetRequest, PushNotificationListRequest,
    PushNotificationListResponse, PushNotificationSetRequest, TaskCancelRequest, TaskGetRequest,
    TaskResubscribeRequest, TaskStatusRequest,
};
pub use streaming_events::{TaskArtifactUpdateEvent, TaskProgress, TaskStatusUpdateEvent};
pub use task::{Artifact, SendResponse, Task, TaskState, TaskStatus};

// Re-export JSON-RPC wire format types
pub use json_rpc::{
    is_batch_request, is_notification, map_error_to_rpc, JsonRpcBatchRequest, JsonRpcBatchResponse,
    JsonRpcError, JsonRpcNotification, JsonRpcRequest, JsonRpcResponse,
    AUTHENTICATED_EXTENDED_CARD_NOT_CONFIGURED, CONTENT_TYPE_NOT_SUPPORTED, INVALID_AGENT_RESPONSE,
    PUSH_NOTIFICATION_NOT_SUPPORTED, SERVER_ERROR, TASK_NOT_CANCELABLE, TASK_NOT_FOUND,
    UNSUPPORTED_OPERATION,
};

// Re-export SSE types
#[cfg(feature = "streaming")]
pub use sse::EventBuffer;
pub use sse::{SseEvent, SseEventId};

// Backwards compatibility alias
#[deprecated(note = "Use SendResponse instead - message/send returns Task or Message per A2A spec")]
pub type MessageResponse = SendResponse;
