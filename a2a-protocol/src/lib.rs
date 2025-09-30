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
//! ```rust
//! use a2a_protocol::{
//!     prelude::*,
//!     client::A2aClient,
//!     transport::HttpTransport,
//! };
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create client
//!     let transport = HttpTransport::new("https://agent.example.com")?;
//!     let client = A2aClient::new(transport);
//!
//!     // Send message
//!     let response = client.send_message(Message {
//!         id: uuid::Uuid::new_v4().to_string(),
//!         role: "user".to_string(),
//!         content: "Hello, agent!".to_string(),
//!         metadata: std::collections::HashMap::new(),
//!     }).await?;
//!
//!     println!("Response: {}", response.content);
//!     Ok(())
//! }
//! ```

pub mod core;
pub mod transport;
pub mod client;
pub mod server;
pub mod auth;
pub mod streaming;

// Re-export commonly used types
pub mod prelude {
    pub use crate::core::{
        Message, MessageResponse, AgentCard, A2aError, A2aResult,
        agent_id::AgentId, message_id::MessageId,
    };
    pub use crate::client::A2aClient;
    pub use crate::transport::Transport;
    pub use crate::auth::Authenticator;
}

pub use crate::core::*;