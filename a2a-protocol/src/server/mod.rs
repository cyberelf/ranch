//! Server-side implementations for A2A protocol

pub mod handler;
#[cfg(feature = "json-rpc")]
pub mod json_rpc;
pub mod task_aware_handler;
pub mod task_store;

// Re-export server types
pub use handler::A2aHandler;
#[cfg(feature = "json-rpc")]
pub use json_rpc::axum::JsonRpcRouter;
pub use task_aware_handler::TaskAwareHandler;
pub use task_store::TaskStore;
