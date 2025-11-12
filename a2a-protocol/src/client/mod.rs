//! A2A client implementations

pub mod auth;
pub mod builder;
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
pub use auth::{Authenticator, ApiKeyAuth, BearerAuth};
pub use transport::{
    JsonRpcTransport, Transport, TransportConfig,
    JsonRpcRequest, JsonRpcResponse, JsonRpcError,
};

#[cfg(feature = "streaming")]
pub use transport::{StreamingTransport, StreamingResult};
