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
//! ### Client Usage
//!
//! Add to your `Cargo.toml`:
//! ```toml
//! [dependencies]
//! a2a-protocol = { version = "0.7", features = ["client"] }
//! ```
//!
//! ```no_run
//! use a2a_protocol::{
//!     prelude::*,
//!     client::{A2aClient, JsonRpcTransport},
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
//!
//! ### Server Usage
//!
//! Add to your `Cargo.toml`:
//! ```toml
//! [dependencies]
//! a2a-protocol = { version = "0.7", features = ["server"] }
//! ```
//!
//! See the examples directory for complete server implementation examples.

// Core protocol types (shared between client and server)
pub mod core;

// Client-side implementations (outbound requests)
#[cfg(feature = "client")]
pub mod client;

// Server-side implementations (inbound request handling)
#[cfg(feature = "server")]
pub mod server;

// Re-export commonly used types
pub mod prelude {
    // Client types (only available with "client" feature)
    #[cfg(feature = "client")]
    pub use crate::client::{
        A2aClient, Authenticator, ApiKeyAuth, BearerAuth,
        JsonRpcTransport, Transport,
    };
    
    #[cfg(all(feature = "client", feature = "streaming"))]
    pub use crate::client::A2aStreamingClient;
    
    // Core protocol types (always available)
    pub use crate::core::{
        agent_card::{
            AgentCapability, AgentProvider, AgentSkill,
            AuthenticationRequirement, RateLimit, RateLimitStrategy, StreamingCapabilities,
            TransportInterface, TransportType,
        },
        agent_id::AgentId,
        message_id::MessageId,
        push_notification::{PushNotificationConfig, TaskEvent},
        A2aError,
        A2aResult,
        AgentCard,
        AgentCardGetRequest,
        DataPart,
        FilePart,
        Message,
        MessageRole,
        // A2A request types
        MessageSendRequest,
        Part,
        SendResponse,
        Task,
        TaskCancelRequest,
        TaskGetRequest,
        TaskResubscribeRequest,
        TaskState,
        TaskStatus,
        TaskStatusRequest,
        TextPart,
    };
    
    // Server types (only available with "server" feature)
    #[cfg(feature = "server")]
    pub use crate::server::{
        Agent, AgentLogic, AgentProfile, PushNotificationSupport, TransportCapabilities, WebhookRetryPolicy,
    };
    
    // JSON-RPC protocol types (for advanced use, only with client feature)
    #[cfg(feature = "client")]
    pub use crate::client::transport::{
        JsonRpcError,
        JsonRpcRequest,
        JsonRpcResponse,
    };

    // Backwards compatibility
    #[allow(deprecated)]
    pub use crate::core::MessageResponse;
}

pub use crate::core::*;
