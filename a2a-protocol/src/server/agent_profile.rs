//! Agent Profile - Server-side agent metadata
//!
//! This module defines `AgentProfile`, which is used by server-side agents to provide
//! their descriptive metadata (identity, capabilities, skills) separate from transport-level
//! details. The profile is consumed by the handler layer to assemble a complete `AgentCard`.

use crate::core::{
    agent_card::{AgentCapabilities, AgentProvider, AgentSkill, TransportType},
    AgentCard, AgentId,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use url::Url;

/// Agent Profile - Descriptive metadata provided by the agent's core logic
///
/// This struct contains only the descriptive attributes of an agent (what the agent is,
/// what it can do) without transport-level capabilities (how to communicate with it).
/// The handler layer is responsible for adding transport capabilities and assembling
/// the complete AgentCard.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentProfile {
    /// Agent identifier
    pub id: AgentId,

    /// Agent name
    pub name: String,

    /// Agent description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Agent version
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    /// Agent URL or endpoint
    pub url: Url,

    /// Agent provider details (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<AgentProvider>,

    /// URL to an icon representing the agent
    #[serde(rename = "iconUrl", skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<Url>,

    /// URL to documentation for this agent
    #[serde(rename = "documentationUrl", skip_serializing_if = "Option::is_none")]
    pub documentation_url: Option<Url>,

    /// Default input MIME types supported by this agent
    #[serde(
        rename = "defaultInputModes",
        skip_serializing_if = "Vec::is_empty",
        default
    )]
    pub default_input_modes: Vec<String>,

    /// Default output MIME types produced by this agent
    #[serde(
        rename = "defaultOutputModes",
        skip_serializing_if = "Vec::is_empty",
        default
    )]
    pub default_output_modes: Vec<String>,

    /// Agent capabilities
    #[serde(default)]
    pub capabilities: AgentCapabilities,

    /// Agent skills
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub skills: Vec<AgentSkill>,

    /// Additional metadata
    #[serde(skip_serializing_if = "HashMap::is_empty", default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl AgentProfile {
    /// Create a minimal agent profile
    pub fn new<S: Into<String>>(id: AgentId, name: S, url: Url) -> Self {
        Self {
            id,
            name: name.into(),
            description: None,
            version: None,
            url,
            provider: None,
            icon_url: None,
            documentation_url: None,
            default_input_modes: Vec::new(),
            default_output_modes: Vec::new(),
            capabilities: AgentCapabilities::default(),
            skills: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Set the description
    pub fn with_description<S: Into<String>>(mut self, description: S) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the version
    pub fn with_version<S: Into<String>>(mut self, version: S) -> Self {
        self.version = Some(version.into());
        self
    }

    /// Set the agent provider metadata
    pub fn with_provider(mut self, provider: AgentProvider) -> Self {
        self.provider = Some(provider);
        self
    }

    /// Set or clear the agent icon URL
    pub fn with_icon_url(mut self, icon_url: Option<Url>) -> Self {
        self.icon_url = icon_url;
        self
    }

    /// Set or clear the agent documentation URL
    pub fn with_documentation_url(mut self, documentation_url: Option<Url>) -> Self {
        self.documentation_url = documentation_url;
        self
    }

    /// Set the default input MIME types
    pub fn with_default_input_modes<I, S>(mut self, modes: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.default_input_modes = modes.into_iter().map(Into::into).collect();
        self
    }

    /// Set the default output MIME types
    pub fn with_default_output_modes<I, S>(mut self, modes: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.default_output_modes = modes.into_iter().map(Into::into).collect();
        self
    }

    /// Set the agent capabilities
    pub fn with_capability(mut self, capability: AgentCapabilities) -> Self {
        self.capabilities = capability;
        self
    }

    /// Add a skill
    pub fn with_skill(mut self, skill: AgentSkill) -> Self {
        self.skills.push(skill);
        self
    }

    /// Add metadata
    pub fn with_metadata<K: Into<String>, V: Into<serde_json::Value>>(
        mut self,
        key: K,
        value: V,
    ) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Convert this profile into a complete AgentCard with default transport settings
    ///
    /// This creates an AgentCard with minimal transport-level fields. For production use,
    /// let the handler layer assemble the card with proper transport capabilities.
    pub fn into_agent_card(self) -> AgentCard {
        AgentCard {
            id: self.id,
            name: self.name,
            description: self.description,
            version: self.version,
            url: self.url,
            protocol_version: "0.3.0".to_string(),
            preferred_transport: TransportType::default(),
            additional_interfaces: Vec::new(),
            provider: self.provider,
            icon_url: self.icon_url,
            documentation_url: self.documentation_url,
            signatures: Vec::new(),
            default_input_modes: self.default_input_modes,
            default_output_modes: self.default_output_modes,
            supports_authenticated_extended_card: false,
            capabilities: self.capabilities,
            skills: self.skills,
            authentication: None,
            rate_limits: None,
            metadata: self.metadata,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_profile_to_card() {
        let id = AgentId::new("agent1".to_string()).unwrap();
        let url = Url::parse("https://agent.example.com").unwrap();

        let profile = AgentProfile::new(id.clone(), "Test Agent", url.clone())
            .with_description("A test agent")
            .with_skill(AgentSkill {
                name: "Testing".to_string(),
                description: Some("Testing skill".to_string()),
                category: None,
                tags: vec![],
                examples: vec![],
            });

        let card = profile.into_agent_card();
        assert_eq!(card.name, "Test Agent");
        assert_eq!(card.description, Some("A test agent".to_string()));
        assert_eq!(card.skills.len(), 1);
    }

    #[test]
    fn test_agent_profile_builder() {
        let id = AgentId::new("agent1".to_string()).unwrap();
        let url = Url::parse("https://agent.example.com").unwrap();

        let profile = AgentProfile::new(id, "Test Agent", url)
            .with_description("Test description")
            .with_version("1.0.0")
            .with_default_input_modes(vec!["text/plain", "application/json"])
            .with_default_output_modes(vec!["text/plain"]);

        assert_eq!(profile.description, Some("Test description".to_string()));
        assert_eq!(profile.version, Some("1.0.0".to_string()));
        assert_eq!(profile.default_input_modes.len(), 2);
        assert_eq!(profile.default_output_modes.len(), 1);
    }
}
