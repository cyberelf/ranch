//! Agent ID type and validation

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use url::Url;
use uuid::Uuid;

/// A2A Agent identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AgentId(String);

impl AgentId {
    /// Create a new AgentId
    pub fn new(id: String) -> Result<Self, crate::A2aError> {
        Self::validate(&id)?;
        Ok(Self(id))
    }

    /// Create an AgentId from a URL
    pub fn from_url(url: &Url) -> Self {
        Self(url.to_string())
    }

    /// Generate a new random AgentId
    pub fn generate() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    /// Get the AgentId as a string slice
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Get the AgentId as a URL if valid
    pub fn as_url(&self) -> Option<Url> {
        Url::parse(&self.0).ok()
    }

    /// Validate an AgentId string
    fn validate(id: &str) -> Result<(), crate::A2aError> {
        if id.trim().is_empty() {
            return Err(crate::A2aError::InvalidAgentId("Empty agent ID".to_string()));
        }

        // If it looks like a URL, validate it
        if id.contains("://") {
            Url::parse(id)
                .map_err(|e| crate::A2aError::InvalidAgentId(format!("Invalid URL: {}", e)))?;
        }

        Ok(())
    }
}

impl fmt::Display for AgentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for AgentId {
    type Err = crate::A2aError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s.to_string())
    }
}

impl From<String> for AgentId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for AgentId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<Url> for AgentId {
    fn from(url: Url) -> Self {
        Self(url.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_agent_id() {
        let id = AgentId::new("agent1".to_string()).unwrap();
        assert_eq!(id.as_str(), "agent1");
    }

    #[test]
    fn test_empty_agent_id() {
        let result = AgentId::new("".to_string());
        assert!(matches!(result, Err(crate::A2aError::InvalidAgentId(_))));
    }

    #[test]
    fn test_url_agent_id() {
        let url = "https://agent.example.com";
        let id = AgentId::new(url.to_string()).unwrap();
        assert_eq!(id.as_str(), url);
    }

    #[test]
    fn test_from_url() {
        let url = Url::parse("https://agent.example.com").unwrap();
        let id = AgentId::from_url(&url);
        assert_eq!(id.as_str(), "https://agent.example.com/");
    }
}