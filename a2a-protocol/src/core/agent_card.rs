//! A2A Agent Card definition

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use url::Url;
use crate::AgentId;

/// A2A Agent Card - metadata about an agent
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentCard {
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

    /// List of supported protocols
    pub protocols: Vec<ProtocolSupport>,

    /// Agent capabilities
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub capabilities: Vec<AgentCapability>,

    /// Agent skills
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub skills: Vec<AgentSkill>,

    /// Authentication requirements
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authentication: Option<AuthenticationRequirement>,

    /// Rate limiting information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rate_limits: Option<RateLimit>,

    /// Additional metadata
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Protocol support information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProtocolSupport {
    /// Protocol name (e.g., "a2a-http", "a2a-jsonrpc")
    pub name: String,

    /// Protocol version
    pub version: String,

    /// Protocol endpoint URL
    pub endpoint: Url,

    /// Optional protocol-specific configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<HashMap<String, serde_json::Value>>,
}

/// Agent capability definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentCapability {
    /// Capability name
    pub name: String,

    /// Capability description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Capability category
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,

    /// Input schema for this capability
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_schema: Option<serde_json::Value>,

    /// Output schema for this capability
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_schema: Option<serde_json::Value>,
}

/// Agent skill definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentSkill {
    /// Skill name
    pub name: String,

    /// Skill description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Skill category
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,

    /// Skill tags
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,

    /// Examples of using this skill
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub examples: Vec<String>,
}

/// Authentication requirement
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AuthenticationRequirement {
    /// No authentication required
    #[serde(rename = "none")]
    None,

    /// API key authentication
    #[serde(rename = "api_key")]
    ApiKey {
        /// API key location (header, query, cookie)
        location: String,
        /// API key name
        name: String,
    },

    /// Bearer token authentication
    #[serde(rename = "bearer")]
    Bearer {
        /// Token format hint
        format: Option<String>,
    },

    /// OAuth2 authentication
    #[serde(rename = "oauth2")]
    OAuth2 {
        /// OAuth2 flows configuration
        flows: OAuth2Flows,
    },

    /// Custom authentication
    #[serde(rename = "custom")]
    Custom {
        /// Custom authentication type
        auth_type: String,
        /// Configuration parameters
        parameters: HashMap<String, serde_json::Value>,
    },
}

/// OAuth2 flows configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OAuth2Flows {
    /// Authorization code flow
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorization_code: Option<OAuth2AuthorizationCodeFlow>,

    /// Client credentials flow
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_credentials: Option<OAuth2ClientCredentialsFlow>,

    /// Implicit flow
    #[serde(skip_serializing_if = "Option::is_none")]
    pub implicit: Option<OAuth2ImplicitFlow>,
}

/// OAuth2 authorization code flow
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OAuth2AuthorizationCodeFlow {
    /// Authorization URL
    pub authorization_url: Url,
    /// Token URL
    pub token_url: Url,
    /// Supported scopes
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub scopes: Vec<String>,
}

/// OAuth2 client credentials flow
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OAuth2ClientCredentialsFlow {
    /// Token URL
    pub token_url: Url,
    /// Supported scopes
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub scopes: Vec<String>,
}

/// OAuth2 implicit flow
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OAuth2ImplicitFlow {
    /// Authorization URL
    pub authorization_url: Url,
    /// Supported scopes
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub scopes: Vec<String>,
}

/// Rate limiting configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RateLimit {
    /// Maximum requests per time window
    pub max_requests: u32,

    /// Time window in seconds
    pub window_seconds: u32,

    /// Strategy for handling rate limits
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strategy: Option<RateLimitStrategy>,
}

/// Rate limit strategy
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RateLimitStrategy {
    /// Reject requests when limit is exceeded
    Reject,

    /// Queue requests when limit is exceeded
    Queue,

    /// Return cached responses when limit is exceeded
    Cache,
}

impl AgentCard {
    /// Create a minimal agent card
    pub fn new<S: Into<String>>(id: AgentId, name: S, url: Url) -> Self {
        Self {
            id,
            name: name.into(),
            description: None,
            version: None,
            url,
            protocols: Vec::new(),
            capabilities: Vec::new(),
            skills: Vec::new(),
            authentication: None,
            rate_limits: None,
            metadata: HashMap::new(),
        }
    }

    /// Add a protocol support
    pub fn with_protocol(mut self, protocol: ProtocolSupport) -> Self {
        self.protocols.push(protocol);
        self
    }

    /// Add a capability
    pub fn with_capability(mut self, capability: AgentCapability) -> Self {
        self.capabilities.push(capability);
        self
    }

    /// Add a skill
    pub fn with_skill(mut self, skill: AgentSkill) -> Self {
        self.skills.push(skill);
        self
    }

    /// Set authentication requirement
    pub fn with_authentication(mut self, auth: AuthenticationRequirement) -> Self {
        self.authentication = Some(auth);
        self
    }

    /// Set rate limiting
    pub fn with_rate_limit(mut self, limit: RateLimit) -> Self {
        self.rate_limits = Some(limit);
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

    /// Check if the agent supports a specific protocol
    pub fn supports_protocol(&self, name: &str, version: &str) -> bool {
        self.protocols.iter().any(|p| p.name == name && p.version == version)
    }

    /// Get all supported protocol endpoints
    pub fn protocol_endpoints(&self) -> Vec<&Url> {
        self.protocols.iter().map(|p| &p.endpoint).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minimal_agent_card() {
        let id = AgentId::new("agent1".to_string()).unwrap();
        let url = Url::parse("https://agent.example.com").unwrap();
        let card = AgentCard::new(id, "Test Agent", url);

        assert_eq!(card.name, "Test Agent");
        assert_eq!(card.protocols.len(), 0);
    }

    #[test]
    fn test_agent_card_with_protocol() {
        let id = AgentId::new("agent1".to_string()).unwrap();
        let url = Url::parse("https://agent.example.com").unwrap();
        let endpoint = Url::parse("https://agent.example.com/a2a").unwrap();

        let card = AgentCard::new(id, "Test Agent", url)
            .with_protocol(ProtocolSupport {
                name: "a2a-http".to_string(),
                version: "1.0".to_string(),
                endpoint,
                config: None,
            });

        assert_eq!(card.protocols.len(), 1);
        assert!(card.supports_protocol("a2a-http", "1.0"));
    }
}