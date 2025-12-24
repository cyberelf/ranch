//! A2A Protocol Extension Interfaces
//!
//! This module defines standard interfaces for implementing A2A protocol extensions.
//! Extensions use message metadata to pass additional information between agents with
//! type-safe accessors for better ergonomics.

use serde::{de::DeserializeOwned, Serialize};

/// Trait for A2A Protocol Extensions
///
/// This trait should be implemented for extension-specific types that need to be
/// serialized into message metadata. It provides const metadata about the extension
/// for registration in agent capabilities.
///
/// # Example
///
/// ```
/// use serde::{Serialize, Deserialize};
/// use a2a_protocol::core::extension::AgentExtension;
///
/// #[derive(Debug, Clone, Serialize, Deserialize)]
/// pub struct MyExtensionData {
///     pub some_field: String,
/// }
///
/// impl AgentExtension for MyExtensionData {
///     const URI: &'static str = "https://example.com/extensions/my-extension/v1";
///     const VERSION: &'static str = "v1";
///     const NAME: &'static str = "My Extension";
///     const DESCRIPTION: &'static str = "Description of my extension";
/// }
/// ```
pub trait AgentExtension: Serialize + DeserializeOwned + Send + Sync {
    /// The unique URI identifying this extension
    const URI: &'static str;

    /// The version of this extension
    const VERSION: &'static str;

    /// The human-readable name of this extension
    const NAME: &'static str;

    /// A brief description of what this extension does
    const DESCRIPTION: &'static str;
}
