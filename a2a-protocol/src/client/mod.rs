//! A2A client implementations

pub mod builder;
pub mod client;

#[cfg(feature = "streaming")]
pub mod streaming_client;

// Re-export client types
pub use builder::ClientBuilder;
pub use client::A2aClient;

#[cfg(feature = "streaming")]
pub use streaming_client::A2aStreamingClient;
