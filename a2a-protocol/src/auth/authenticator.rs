//! Authenticator trait and common authentication logic

use async_trait::async_trait;
use std::collections::HashMap;

/// Authentication trait for A2A protocol
#[async_trait]
pub trait Authenticator: Send + Sync {
    /// Authenticate a request by adding necessary headers
    async fn authenticate(&self, headers: &mut HashMap<String, String>) -> Result<(), crate::A2aError>;

    /// Get the authentication type name
    fn auth_type(&self) -> &'static str;

    /// Check if authentication is configured
    fn is_configured(&self) -> bool {
        true
    }
}

/// No-op authenticator for when no authentication is required
pub struct NoAuth;

#[async_trait]
impl Authenticator for NoAuth {
    async fn authenticate(&self, _headers: &mut HashMap<String, String>) -> Result<(), crate::A2aError> {
        Ok(())
    }

    fn auth_type(&self) -> &'static str {
        "none"
    }

    fn is_configured(&self) -> bool {
        false
    }
}