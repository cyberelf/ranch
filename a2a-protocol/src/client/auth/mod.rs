//! Authentication implementations for A2A protocol client

pub mod authenticator;
pub mod strategies;

// Re-export authentication types
pub use authenticator::Authenticator;
pub use strategies::{ApiKeyAuth, BearerAuth};
