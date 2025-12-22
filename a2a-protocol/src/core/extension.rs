//! A2A Protocol Extension Interfaces
//!
//! This module defines standard interfaces for implementing A2A protocol extensions.
//! Extensions typically use message metadata to pass additional information between agents.

use serde::{Deserialize, Serialize};

/// Trait for A2A Protocol Extensions
pub trait ProtocolExtension {
    /// The unique URI identifying this extension
    fn uri(&self) -> &str;

    /// The version of this extension
    fn version(&self) -> &str;

    /// The human-readable name of this extension
    fn name(&self) -> &str;

    /// A brief description of what this extension does
    fn description(&self) -> &str;
}

/// Common structure for extension metadata
///
/// Extensions should serialize their data into the message metadata
/// using their URI as the key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionMetadata<T> {
    /// The extension data payload
    pub data: T,
}
