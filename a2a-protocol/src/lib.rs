//! # A2A Protocol Implementation
//!
//! A Rust implementation of the A2A (Agent-to-Agent) protocol specification.
//! This crate provides a comprehensive, standards-compliant implementation
//! of the A2A protocol for agent communication.
//!
//! ## Features
//!
//! - **Protocol Compliance**: Full adherence to the A2A specification
//! - **Multiple Transports**: HTTP, JSON-RPC, gRPC, and WebSocket support
//! - **Async Native**: Built on tokio for high-performance async communication
//! - **Type Safe**: Strong typing with serde for serialization
//! - **Extensible**: Plugin architecture for custom transports and authentication
//! - **Production Ready**: Comprehensive error handling and testing
//!
//! ## Quick Start
//!
//! ```no_run
//! use a2a_protocol::{
//!     prelude::*,
//!     client::A2aClient,
//!     transport::JsonRpcTransport,
//! };
//! use std::sync::Arc;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create client
//!     let transport = Arc::new(JsonRpcTransport::new("https://agent.example.com/rpc")?);
//!     let client = A2aClient::new(transport);
//!
//!     // Send message using new A2A v0.3.0 compliant API
//!     let message = Message::user_text("Hello, agent!");
//!     let response = client.send_message(message).await?;
//!
//!     // Response is either a Task (async) or Message (immediate)
//!     match response {
//!         SendResponse::Message(msg) => {
//!             println!("Immediate response: {}", msg.text_content().unwrap_or(""));
//!         }
//!         SendResponse::Task(task) => {
//!             println!("Task created: {}", task.id);
//!         }
//!     }
//!     Ok(())
//! }
//! ```

pub mod core;
pub mod transport;
pub mod client;
pub mod server;
pub mod auth;

// Re-export commonly used types
pub mod prelude {
    pub use crate::core::{
        Message, MessageRole, Part, TextPart, FilePart, DataPart,
        Task, TaskState, TaskStatus, SendResponse,
        AgentCard, A2aError, A2aResult,
        agent_id::AgentId, message_id::MessageId,
        // A2A request types
        MessageSendRequest, TaskGetRequest, TaskCancelRequest, TaskStatusRequest,
        AgentCardGetRequest,
    };
    pub use crate::client::A2aClient;
    pub use crate::transport::{
        Transport, TransportExt,
        // JSON-RPC 2.0 protocol types
        JsonRpcRequest, JsonRpcResponse, JsonRpcError,
    };
    pub use crate::auth::Authenticator;
    
    // Backwards compatibility
    #[allow(deprecated)]
    pub use crate::core::MessageResponse;
}

pub use crate::core::*;