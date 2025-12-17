//! A2A client implementations

pub mod auth;
pub mod builder;
#[allow(clippy::module_inception)] // client module contains A2aClient type - this is intentional
pub mod client;
pub mod transport;

#[cfg(feature = "streaming")]
pub mod streaming_client;

// Re-export client types
pub use builder::ClientBuilder;
pub use client::A2aClient;

#[cfg(feature = "streaming")]
pub use streaming_client::A2aStreamingClient;

// Re-export commonly used client types
pub use auth::{ApiKeyAuth, Authenticator, BearerAuth};
pub use transport::{
    JsonRpcError, JsonRpcRequest, JsonRpcResponse, JsonRpcTransport, Transport, TransportConfig,
};

#[cfg(feature = "streaming")]
pub use transport::{StreamingResult, StreamingTransport};
