//! Server-side implementations for A2A protocol

pub mod agent_logic;
#[cfg(feature = "json-rpc")]
pub mod builder;
pub mod handler;
#[cfg(feature = "json-rpc")]
pub mod json_rpc;
pub mod push_notification_store;
pub mod task_aware_handler;
pub mod task_store;
pub mod webhook_delivery;

// Re-export server types
pub use agent_logic::AgentLogic;
#[cfg(feature = "json-rpc")]
pub use builder::ServerBuilder;
pub use handler::A2aHandler;
#[cfg(feature = "json-rpc")]
pub use json_rpc::axum::JsonRpcRouter;
pub use push_notification_store::PushNotificationStore;
pub use task_aware_handler::TaskAwareHandler;
pub use task_store::TaskStore;
pub use webhook_delivery::{DeliveryStatus, RetryConfig, WebhookPayload, WebhookQueue};
