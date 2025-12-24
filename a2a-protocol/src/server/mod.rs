//! Server-side implementations for A2A protocol

pub mod agent_logic;
pub mod agent_profile;
pub mod builder;
pub mod handler;
pub mod json_rpc;
pub mod push_notification_store;

#[cfg(feature = "streaming")]
pub mod sse;

pub mod task_aware_handler;
pub mod task_store;
pub mod transport_capabilities;
pub mod webhook_delivery;

// Re-export server types
pub use agent_logic::{AgentLogic, ProtocolAgent};
pub use agent_profile::AgentProfile;
pub use builder::ServerBuilder;
pub use handler::A2aHandler;
pub use json_rpc::axum::JsonRpcRouter;
pub use push_notification_store::PushNotificationStore;
pub use task_aware_handler::TaskAwareHandler;
pub use task_store::TaskStore;
pub use transport_capabilities::{
    PushNotificationSupport, TransportCapabilities, WebhookRetryPolicy,
};
pub use webhook_delivery::{DeliveryStatus, RetryConfig, WebhookPayload, WebhookQueue};

#[cfg(feature = "streaming")]
pub use sse::{SseResponse, SseWriter};
