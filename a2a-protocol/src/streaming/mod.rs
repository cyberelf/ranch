//! Streaming support for A2A protocol

pub mod client;
pub mod server;

// Re-export streaming types
pub use client::StreamingClient;
pub use server::StreamingServer;